//! `GET /supported-assets`.

use marcasite::{Decimal, Error, bridge::ChainId};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::common;

#[tokio::test]
async fn get_supported_assets_decodes() {
    let server = common::server().await;
    // Captured from `GET https://bridge.polymarket.com/supported-assets` on 2026-10-02
    // (trimmed to one asset). The top-level `note` is not in `docs/polymarket/specs/bridge-openapi.yaml`.
    Mock::given(method("GET"))
        .and(path("/supported-assets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"supportedAssets":[{"chainId":"1","chainName":"Ethereum","token":{"name":"TrueUSD","symbol":"TUSD","address":"0x0000000000085d4780B73119b644AE5ecd22b376","decimals":18},"minCheckoutUsd":3}],"note":"These are the currently supported chains and assets for deposits and withdrawals."}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let assets = common::polymarket(&server)
        .bridge()
        .get_supported_assets()
        .await
        .unwrap();
    let [asset] = assets.assets() else {
        panic!("expected one asset, got {assets:?}")
    };
    assert_eq!(asset.chain_id, Some(ChainId::from("1")));
    assert_eq!(asset.min_checkout_usd, Some(Decimal::from(3)));
    assert_eq!(asset.token.as_ref().unwrap().decimals, Some(18));
    assert_eq!(
        assets.note.as_deref(),
        Some("These are the currently supported chains and assets for deposits and withdrawals.")
    );
}

#[tokio::test]
async fn server_error_is_typed() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/supported-assets"))
        .respond_with(
            ResponseTemplate::new(500).set_body_raw(r#"{"error":"boom"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .get_supported_assets()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 500);
    assert_eq!(api.message(), Some("boom"));
}

/// `docs/polymarket/api-reference/rate-limits.md` documents a 50 req / 10 s limit for the Bridge API.
#[tokio::test]
async fn rate_limit_is_typed_with_retry_after() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/supported-assets"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "10")
                .set_body_raw(r#"{"error":"too many requests"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .get_supported_assets()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(10)));
}

#[tokio::test]
async fn wrong_type_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/supported-assets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"supportedAssets":[{"token":{"decimals":"six"}}]}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .get_supported_assets()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "supportedAssets[0].token.decimals");
}
