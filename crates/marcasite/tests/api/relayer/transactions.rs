//! `GET /transaction`.

use marcasite::{
    Error,
    relayer::{TransactionId, TransactionState, TransactionType},
    types::Address,
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param},
};

use crate::common;

const ID: &str = "0190b317-a1d3-7bec-9b91-eeb6dcd3a620";

/// The `200` example of `GET /transaction` in `docs/specs/relayer-openapi.yaml`.
const EXAMPLE: &str = r#"[{
    "transactionID": "0190b317-a1d3-7bec-9b91-eeb6dcd3a620",
    "transactionHash": "0x38cbfbeae8fffa4e2b187ee5978d3ee9cafc53af0363ed90a35b7ea9016535d8",
    "from": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
    "to": "0x2791bca1f2de4661ed88a30c99a7a9449aa84174",
    "proxyAddress": "0x6d8c4e9adf5748af82dabe2c6225207770d6b4fa",
    "data": "0x...",
    "nonce": "60",
    "value": "",
    "signature": "0x01a060c734d7bdf4adde50c4a7e574036b1f8b12890911bdd1c1cfdcd77502381b89fa8a47c36f62a0b9f1cdfee7b260fd8108536db9f6b2089c02637e7de9fc20",
    "state": "STATE_CONFIRMED",
    "type": "SAFE",
    "owner": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
    "metadata": "",
    "createdAt": "2024-07-14T21:13:08.819782Z",
    "updatedAt": "2024-07-14T21:13:46.576639Z"
}]"#;

#[tokio::test]
async fn get_transaction_sends_id_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .and(query_param("id", ID))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let txs = common::polymarket(&server)
        .relayer()
        .get_transaction(ID)
        .await
        .unwrap();
    assert_eq!(txs.len(), 1);
    let tx = &txs[0];
    assert_eq!(tx.transaction_id, Some(TransactionId::from(ID)));
    assert_eq!(tx.state, Some(TransactionState::Confirmed));
    assert_eq!(tx.transaction_type, Some(TransactionType::Safe));
    assert_eq!(
        tx.owner,
        Some(Address::from("0x6e0c80c90ea6c15917308f820eac91ce2724b5b5"))
    );
    assert!(tx.created_at.unwrap() < tx.updated_at.unwrap());
}

#[tokio::test]
async fn unknown_state_does_not_break_decoding() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[{"transactionID":"x","state":"STATE_REPLACED","type":"SMART"}]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let txs = common::polymarket(&server)
        .relayer()
        .get_transaction("x")
        .await
        .unwrap();
    assert_eq!(
        txs[0].state,
        Some(TransactionState::Unknown("STATE_REPLACED".to_owned()))
    );
    assert!(txs[0].transaction_type.as_ref().unwrap().is_unknown());
}

#[tokio::test]
async fn not_found_and_bad_request_are_typed() {
    let server = common::server().await;
    // Documented error bodies (`docs/specs/relayer-openapi.yaml`, `GET /transaction`).
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .and(query_param("id", "missing"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_raw(r#"{"error":"transaction not found"}"#, "application/json"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .and(query_param("id", "bad"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid id"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let relayer = common::polymarket(&server).relayer().clone();
    let err = relayer.get_transaction("missing").await.unwrap_err();
    assert!(err.is_not_found(), "{err}");
    assert_eq!(
        err.api_error().unwrap().message(),
        Some("transaction not found")
    );

    let err = relayer.get_transaction("bad").await.unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.message(), Some("invalid id"));
}

#[tokio::test]
async fn server_error_is_typed() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_raw(r#"{"error":"internal server error"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .get_transaction(ID)
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(500));
    assert_eq!(
        err.api_error().unwrap().message(),
        Some("internal server error")
    );
}

#[tokio::test]
async fn empty_id_is_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .get_transaction("")
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "id");
}

#[tokio::test]
async fn malformed_body_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/transaction"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[{"transactionID":"x","createdAt":"not a date"}]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .relayer()
        .get_transaction("x")
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "[0].createdAt");
}
