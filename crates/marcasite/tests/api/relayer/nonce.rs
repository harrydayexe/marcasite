//! `GET /nonce`, `GET /relay-payload`.

use marcasite::{Error, relayer::NonceType, types::Address};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param},
};

use crate::common;

/// Example signer address from `docs/specs/relayer-openapi.yaml`.
const SIGNER: &str = "0x77837466dd64fb52ECD00C737F060d0ff5CCB575";

#[tokio::test]
async fn get_nonce_sends_address_and_type() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/nonce"))
        .and(query_param("address", SIGNER))
        .and(query_param("type", "PROXY"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"nonce":"31"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let nonce = common::polymarket(&server)
        .relayer()
        .get_nonce(SIGNER, NonceType::Proxy)
        .await
        .unwrap();
    assert_eq!(nonce.nonce.as_deref(), Some("31"));
}

#[tokio::test]
async fn get_relay_payload_sends_address_and_type() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/relay-payload"))
        .and(query_param("address", SIGNER))
        .and(query_param("type", "SAFE"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"address":"0x4da9395388791c22684e03779c3de10934eb9cfb","nonce":"31"}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let payload = common::polymarket(&server)
        .relayer()
        .get_relay_payload(Address::from(SIGNER), NonceType::Safe)
        .await
        .unwrap();
    assert_eq!(
        payload.address,
        Some(Address::from("0x4da9395388791c22684e03779c3de10934eb9cfb"))
    );
    assert_eq!(payload.nonce.as_deref(), Some("31"));
}

#[tokio::test]
async fn invalid_type_is_reported_by_the_server() {
    let server = common::server().await;
    // Documented `400` example (`invalidType`).
    Mock::given(method("GET"))
        .and(path("/nonce"))
        .and(query_param("type", "EOA"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid type"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .get_nonce(SIGNER, NonceType::from("EOA"))
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(400));
    assert_eq!(err.api_error().unwrap().message(), Some("invalid type"));
}

#[tokio::test]
async fn malformed_address_is_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let relayer = common::polymarket(&server).relayer().clone();
    for bad in ["", "0x1234", "77837466dd64fb52ECD00C737F060d0ff5CCB575"] {
        let err = relayer.get_nonce(bad, NonceType::Proxy).await.unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "address"),
            "{err:?}"
        );
        let err = relayer
            .get_relay_payload(bad, NonceType::Safe)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "{err:?}");
    }
}

#[tokio::test]
async fn wrong_body_shape_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/nonce"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"nonce":31}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .get_nonce(SIGNER, NonceType::Safe)
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "nonce");
}
