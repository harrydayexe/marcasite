//! `POST /deposit`, `POST /withdraw`.

use std::time::Duration;

use polyoxide::{
    Error, HttpClient, RetryPolicy,
    bridge::{BridgeClient, WithdrawalRequest},
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_json, header, method, path},
};

use crate::common;

/// Example values from `docs/specs/bridge-openapi.yaml`.
const WALLET: &str = "0x56687bf447db6ffa42ffe2204a05edaa20f55839";
const BUILDER_CODE: &str = "0x00000000000000000000000000000000000000000000000000000000abcd1234";

/// `201` example of `POST /withdraw` (the same `DepositResponse` schema as `/deposit`).
const CREATED: &str = r#"{
    "address": {
        "evm": "0x23566f8b2E82aDfCf01846E54899d110e97AC053",
        "svm": "CrvTBvzryYxBHbWu2TiQpcqD5M7Le7iBKzVmEj3f36Jb",
        "btc": "bc1q8eau83qffxcj8ht4hsjdza3lha9r3egfqysj3g"
    },
    "note": "Send funds to these addresses to bridge to your destination chain and token."
}"#;

fn has_no_builder_code(request: &wiremock::Request) -> bool {
    !request.headers.contains_key("x-builder-code")
}

fn documented_withdrawal() -> WithdrawalRequest {
    WithdrawalRequest::new(
        "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
        "1",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    )
}

#[tokio::test]
async fn deposit_posts_address_with_builder_code() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/deposit"))
        .and(header("x-builder-code", BUILDER_CODE))
        .and(body_json(serde_json::json!({ "address": WALLET })))
        .respond_with(ResponseTemplate::new(201).set_body_raw(CREATED, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let created = common::polymarket(&server)
        .bridge()
        .create_deposit_addresses(WALLET)
        .builder_code(BUILDER_CODE)
        .send()
        .await
        .unwrap();
    let address = created.address.unwrap();
    assert_eq!(
        address.evm.as_deref(),
        Some("0x23566f8b2E82aDfCf01846E54899d110e97AC053")
    );
    assert_eq!(address.tron, None);
}

#[tokio::test]
async fn deposit_omits_builder_code_by_default() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/deposit"))
        .and(has_no_builder_code)
        .respond_with(ResponseTemplate::new(201).set_body_raw(CREATED, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let created = common::polymarket(&server)
        .bridge()
        .create_deposit_addresses(WALLET)
        .send()
        .await
        .unwrap();
    assert!(created.note.is_some());
}

#[tokio::test]
async fn invalid_inputs_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let bridge = common::polymarket(&server).bridge().clone();
    let err = bridge
        .create_deposit_addresses(WALLET)
        .builder_code("0xabcd1234")
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "X-Builder-Code"),
        "{err:?}"
    );

    let err = bridge
        .create_deposit_addresses("not-an-address")
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "address"),
        "{err:?}"
    );

    let mut request = documented_withdrawal();
    request.address = "0x123".into();
    let err = bridge
        .create_withdrawal_addresses(request)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}

#[tokio::test]
async fn address_creation_is_never_retried() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/deposit"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/withdraw"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;

    let http = HttpClient::builder()
        .retry_policy(RetryPolicy::new(3).with_initial_backoff(Duration::from_millis(1)))
        .build()
        .unwrap();
    let bridge = BridgeClient::builder()
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap();

    let err = bridge
        .create_deposit_addresses(WALLET)
        .send()
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(503));
    let err = bridge
        .create_withdrawal_addresses(documented_withdrawal())
        .send()
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(503));
}

#[tokio::test]
async fn withdraw_posts_documented_body_and_decodes() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/withdraw"))
        .and(header("x-builder-code", BUILDER_CODE))
        .and(body_json(serde_json::json!({
            "address": "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
            "toChainId": "1",
            "toTokenAddress": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "recipientAddr": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_raw(CREATED, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let created = common::polymarket(&server)
        .bridge()
        .create_withdrawal_addresses(documented_withdrawal())
        .builder_code(BUILDER_CODE)
        .send()
        .await
        .unwrap();
    assert_eq!(
        created.address.unwrap().svm.as_deref(),
        Some("CrvTBvzryYxBHbWu2TiQpcqD5M7Le7iBKzVmEj3f36Jb")
    );
}

#[tokio::test]
async fn withdraw_rejection_is_typed() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/withdraw"))
        .and(has_no_builder_code)
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid toChainId"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .create_withdrawal_addresses(documented_withdrawal())
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.message(), Some("invalid toChainId"));
}
