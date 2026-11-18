//! Geoblock integration tests (`GET /api/geoblock`).

use marcasite::Error;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::common;

/// Response example in `docs/polymarket/api-reference/geoblock.md`.
const EXAMPLE: &str = r#"{"blocked":true,"ip":"203.0.113.42","country":"US","region":"NY"}"#;

#[tokio::test]
async fn get_status_decodes_response() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/api/geoblock"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let status = common::polymarket(&server)
        .geoblock()
        .get_status()
        .await
        .unwrap();
    assert!(status.blocked);
    assert_eq!(status.ip.to_string(), "203.0.113.42");
    assert_eq!(status.country, "US");
    assert_eq!(status.region, "NY");
}

#[tokio::test]
async fn server_error_is_typed() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/api/geoblock"))
        .respond_with(
            ResponseTemplate::new(500).set_body_raw(r#"{"error":"boom"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .geoblock()
        .get_status()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 500);
    assert_eq!(api.message(), Some("boom"));
}

#[tokio::test]
async fn rate_limit_is_typed_with_retry_after() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/api/geoblock"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "7")
                .set_body_raw(r#"{"error":"too many requests"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .geoblock()
        .get_status()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(7)));
}

#[tokio::test]
async fn malformed_json_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/api/geoblock"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("<html>", "application/json"))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .geoblock()
        .get_status()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
}

#[tokio::test]
async fn wrong_type_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/api/geoblock"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"blocked":"yes","ip":"203.0.113.42","country":"US","region":"NY"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .geoblock()
        .get_status()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "blocked");
}
