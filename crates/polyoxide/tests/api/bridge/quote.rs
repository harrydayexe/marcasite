//! `POST /quote`.

use std::time::Duration;

use polyoxide::{
    Decimal, Error, HttpClient, RetryPolicy,
    bridge::{BridgeClient, QuoteId, QuoteRequest},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path},
};

use crate::common;

/// Request example of `POST /quote` in `docs/specs/bridge-openapi.yaml`.
fn documented_request() -> QuoteRequest {
    QuoteRequest::new(
        "10000000",
        "137",
        "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
        "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
        "137",
        "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
    )
}

/// `200` example of `POST /quote` in `docs/specs/bridge-openapi.yaml`.
const RESPONSE: &str = r#"{
    "estCheckoutTimeMs": 25000,
    "estFeeBreakdown": {
        "appFeeLabel": "Fun.xyz fee",
        "appFeePercent": 0,
        "appFeeUsd": 0,
        "fillCostPercent": 0,
        "fillCostUsd": 0,
        "gasUsd": 0.003854,
        "maxSlippage": 0,
        "minReceived": 14.488305,
        "swapImpact": 0,
        "swapImpactUsd": 0,
        "totalImpact": 0,
        "totalImpactUsd": 0
    },
    "estInputUsd": 14.488305,
    "estOutputUsd": 14.488305,
    "estToTokenBaseUnit": "14491203",
    "quoteId": "0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"
}"#;

fn client_with_one_retry(server: &MockServer) -> BridgeClient {
    let http = HttpClient::builder()
        .retry_policy(RetryPolicy::new(1).with_initial_backoff(Duration::from_millis(1)))
        .build()
        .unwrap();
    BridgeClient::builder()
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap()
}

#[tokio::test]
async fn get_quote_posts_json_body_and_decodes() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/quote"))
        .and(header("content-type", "application/json"))
        .and(body_json(serde_json::json!({
            "fromAmountBaseUnit": "10000000",
            "fromChainId": "137",
            "fromTokenAddress": "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
            "recipientAddress": "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(RESPONSE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let quote = common::polymarket(&server)
        .bridge()
        .get_quote(&documented_request())
        .await
        .unwrap();
    assert_eq!(quote.est_checkout_time_ms, Some(25_000));
    assert_eq!(quote.est_input_usd, Some(Decimal::new(14_488_305, 6)));
    assert_eq!(quote.est_to_token_base_unit.as_deref(), Some("14491203"));
    assert_eq!(
        quote.quote_id,
        Some(QuoteId::from(
            "0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"
        ))
    );
    assert_eq!(
        quote.est_fee_breakdown.unwrap().gas_usd,
        Some(Decimal::new(3_854, 6))
    );
}

#[tokio::test]
async fn missing_field_error_is_typed() {
    let server = common::server().await;
    // One of the documented `400` examples.
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(ResponseTemplate::new(400).set_body_raw(
            r#"{"error":"fromAmountBaseUnit is required"}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let mut request = documented_request();
    request.from_amount_base_unit = String::new();
    let err = common::polymarket(&server)
        .bridge()
        .get_quote(&request)
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.message(), Some("fromAmountBaseUnit is required"));
}

#[tokio::test]
async fn quote_is_retried_because_it_is_read_only() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(RESPONSE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let quote = client_with_one_retry(&server)
        .get_quote(&documented_request())
        .await
        .unwrap();
    assert_eq!(quote.est_checkout_time_ms, Some(25_000));
}

#[tokio::test]
async fn server_error_is_typed() {
    let server = common::server().await;
    // Documented `500` example.
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_raw(r#"{"error":"cannot get quote"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .get_quote(&documented_request())
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(500));
    assert_eq!(err.api_error().unwrap().message(), Some("cannot get quote"));
}

#[tokio::test]
async fn non_numeric_fee_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"{"estFeeBreakdown":{"gasUsd":true}}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .get_quote(&documented_request())
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "estFeeBreakdown.gasUsd");
}
