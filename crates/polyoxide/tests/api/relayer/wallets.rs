//! `GET /deployed`.

use polyoxide::{Error, relayer::WalletType};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use crate::common;

/// Example wallet address from `docs/specs/relayer-openapi.yaml`.
const WALLET: &str = "0x6d8c4e9aDF5748Af82Dabe2C6225207770d6B4fa";

#[tokio::test]
async fn check_deployed_omits_type_by_default() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/deployed"))
        .and(query_param("address", WALLET))
        .and(query_param_is_missing("type"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"deployed":true}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = common::polymarket(&server)
        .relayer()
        .check_deployed(WALLET)
        .send()
        .await
        .unwrap();
    assert_eq!(status.deployed, Some(true));
}

#[tokio::test]
async fn check_deployed_sends_wallet_type() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/deployed"))
        .and(query_param("address", WALLET))
        .and(query_param("type", "WALLET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"deployed":false}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = common::polymarket(&server)
        .relayer()
        .check_deployed(WALLET)
        .wallet_type(WalletType::Wallet)
        .send()
        .await
        .unwrap();
    assert_eq!(status.deployed, Some(false));
}

#[tokio::test]
async fn server_side_rejection_is_typed() {
    let server = common::server().await;
    // Documented `400` example.
    Mock::given(method("GET"))
        .and(path("/deployed"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid address"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .check_deployed(WALLET)
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.message(), Some("invalid address"));
}

#[tokio::test]
async fn malformed_address_is_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .check_deployed("0xnot-an-address")
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}
