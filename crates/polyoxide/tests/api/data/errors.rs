//! Error paths shared by every Data API v2 route: the documented error body
//! (`components/schemas/ErrorResponse`), rate limiting, timeouts, malformed bodies and
//! failures in the middle of a cursor walk.

use std::time::Duration;

use futures_util::StreamExt as _;
use polyoxide::{Error, StatusCode, data::ErrorCode};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::fixtures::{self, WALLET};
use crate::common;

#[tokio::test]
async fn bad_request_exposes_error_body_and_trace_id() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/positions"))
        .respond_with(
            ResponseTemplate::new(400)
                .insert_header("x-trace-id", "trace-400")
                .set_body_json(json!({
                    "error": "malformed condition id '0xzz'",
                    "code": "invalid_request",
                    "retryable": false,
                    "trace_id": "trace-400",
                    "parameter": "condition"
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .list_positions()
        .conditions(["0xzz"])
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status(), StatusCode::BAD_REQUEST);
    assert_eq!(api.message(), Some("malformed condition id '0xzz'"));
    assert_eq!(api.code(), Some("invalid_request"));
    assert_eq!(api.parameter(), Some("condition"));
    assert_eq!(api.retryable(), Some(false));
    assert_eq!(err.trace_id(), Some("trace-400"));
    assert_eq!(ErrorCode::from_error(&err), Some(ErrorCode::InvalidRequest));
    assert!(!err.is_retryable());
}

#[tokio::test]
async fn rate_limit_carries_retry_after_and_trace_id() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "2")
                .insert_header("x-trace-id", "trace-429")
                .set_body_json(json!({
                    "error": "service is at heavy-query capacity",
                    "code": "rate_limited",
                    "retryable": true,
                    "trace_id": "trace-429"
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .list_trades()
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::from_secs(2)));
    assert_eq!(err.trace_id(), Some("trace-429"));
    assert_eq!(ErrorCode::from_error(&err), Some(ErrorCode::RateLimited));
    assert!(err.is_retryable());
}

#[tokio::test]
async fn request_timeout_503_is_retryable_with_retry_after() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/activity"))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("retry-after", "5")
                .insert_header("x-trace-id", "trace-503")
                .set_body_json(json!({
                    "error": "connection pool acquire budget exceeded",
                    "code": "request_timeout",
                    "retryable": true,
                    "trace_id": "trace-503"
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .list_activity(WALLET)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Api(_)), "{err:?}");
    assert_eq!(err.status(), Some(StatusCode::SERVICE_UNAVAILABLE));
    assert_eq!(err.retry_after(), Some(Duration::from_secs(5)));
    assert_eq!(ErrorCode::from_error(&err), Some(ErrorCode::RequestTimeout));
    assert!(err.is_retryable());
}

#[tokio::test]
async fn malformed_row_is_a_decode_error_with_path_and_trace_id() {
    let server = common::server().await;
    let mut row = fixtures::trade("0x1");
    row["size"] = json!("not a number");
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-trace-id", "trace-200")
                .set_body_json(fixtures::page(vec![fixtures::trade("0x0"), row], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .list_trades()
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "data[1].size");
    assert_eq!(err.trace_id(), Some("trace-200"));
}

#[tokio::test]
async fn invalid_json_and_missing_envelope_are_decode_errors() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/value"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{"data": {"#, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/user-volume"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"volume": 1, "volume_usdc": 1, "trade_count": 1})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let err = data.get_portfolio_value(WALLET).send().await.unwrap_err();
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
    // The payload must be wrapped in `data`.
    let err = data.get_user_volume(WALLET).send().await.unwrap_err();
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
}

#[tokio::test]
async fn stream_yields_error_then_ends() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::trade("0x1"), fixtures::trade("0x2")],
            Some("seek-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/trades"))
        .and(query_param("cursor", "seek-2"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "internal",
            "code": "internal",
            "retryable": false,
            "trace_id": "trace-500"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let results: Vec<_> = common::polymarket(&server)
        .data()
        .list_trades()
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok() && results[1].is_ok());
    let err = results[2].as_ref().unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    assert_eq!(ErrorCode::from_error(err), Some(ErrorCode::Internal));
}

#[tokio::test]
async fn unknown_error_code_is_preserved() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/status"))
        .respond_with(ResponseTemplate::new(418).set_body_json(json!({
            "error": "teapot",
            "code": "brand_new_code",
            "retryable": false,
            "trace_id": "t"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .get_status()
        .await
        .unwrap_err();
    assert_eq!(
        ErrorCode::from_error(&err),
        Some(ErrorCode::Unknown("brand_new_code".to_owned()))
    );
}
