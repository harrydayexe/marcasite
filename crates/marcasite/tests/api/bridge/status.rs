//! `GET /status/{address}`.

use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{Error, bridge::TransactionStatus};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use crate::common;

/// Example bridge address from `docs/specs/bridge-openapi.yaml`.
const BRIDGE_ADDRESS: &str = "EXoZue2avJae1d45B3fVw2unhkrtToSYQqHtHgfZ2cbE";

/// `200` example of `GET /status/{address}` in `docs/specs/bridge-openapi.yaml`.
const EXAMPLE: &str = r#"{
    "transactions": [
        {
            "fromChainId": "1151111081099710",
            "fromTokenAddress": "11111111111111111111111111111111",
            "fromAmountBaseUnit": "13566635",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
            "status": "DEPOSIT_DETECTED"
        },
        {
            "fromChainId": "1151111081099710",
            "fromTokenAddress": "11111111111111111111111111111111",
            "fromAmountBaseUnit": "13400000",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
            "createdTimeMs": 1757646914535,
            "status": "PROCESSING"
        },
        {
            "fromChainId": "1151111081099710",
            "fromTokenAddress": "11111111111111111111111111111111",
            "fromAmountBaseUnit": "13500152",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
            "txHash": "3atr19NAiNCYt24RHM1WnzZp47RXskpTDzspJoCBBaMFwUB8fk37hFkxz35P5UEnnmWz21rb2t5wJ8pq3EE2XnxU",
            "createdTimeMs": 1757531217339,
            "status": "COMPLETED"
        }
    ],
    "nextCursor": "eyJsYXN0SWQiOiI0MiJ9"
}"#;

#[tokio::test]
async fn get_status_sends_limit_and_decodes_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .and(query_param("limit", "100"))
        .and(query_param_is_missing("cursor"))
        .and(query_param_is_missing("paginate"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .limit(100)
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor.as_deref(), Some("eyJsYXN0SWQiOiI0MiJ9"));
    let statuses: Vec<_> = page
        .transactions
        .iter()
        .map(|t| t.status.clone().unwrap())
        .collect();
    assert_eq!(
        statuses,
        [
            TransactionStatus::DepositDetected,
            TransactionStatus::Processing,
            TransactionStatus::Completed
        ]
    );
    assert_eq!(
        page.transactions[1]
            .created_time
            .map(|t| t.timestamp_millis()),
        Some(1_757_646_914_535)
    );
}

#[tokio::test]
async fn sends_cursor_and_paginate() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .and(query_param("cursor", "eyJsYXN0SWQiOiI0MiJ9"))
        .and(query_param("paginate", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"transactions":[],"nextCursor":null}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .cursor("eyJsYXN0SWQiOiI0MiJ9")
        .paginate()
        .send()
        .await
        .unwrap();
    assert!(page.transactions.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn stream_follows_next_cursor_until_null() {
    let server = common::server().await;
    // A cursor with `+`, `/` and `=`, which the docs say must be URL-encoded.
    let cursor = "ab+c/d==";
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"{{"transactions":[{{"status":"COMPLETED","txHash":"a"}},{{"status":"FAILED"}}],"nextCursor":"{cursor}"}}"#
            ),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .and(query_param("cursor", cursor))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"transactions":[{"status":"SUBMITTED"}],"nextCursor":null}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let statuses: Vec<_> = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .into_stream()
        .map_ok(|t| t.status.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        statuses,
        [
            TransactionStatus::Completed,
            TransactionStatus::Failed,
            TransactionStatus::Submitted
        ]
    );

    let requests = server.received_requests().await.unwrap();
    let second = requests[1].url.query().unwrap();
    assert!(second.contains("cursor=ab%2Bc%2Fd%3D%3D"), "{second}");
}

#[tokio::test]
async fn unknown_status_does_not_break_decoding() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"transactions":[{"status":"REFUNDED"}],"nextCursor":null}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .send()
        .await
        .unwrap();
    assert_eq!(
        page.transactions[0].status,
        Some(TransactionStatus::Unknown("REFUNDED".to_owned()))
    );
}

#[tokio::test]
async fn stale_cursor_is_a_400() {
    let server = common::server().await;
    // Documented `400` example (`rejectedUpstream`).
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid request"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let results: Vec<_> = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .cursor("stale")
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let err = results.into_iter().next().unwrap().unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(400));
    assert_eq!(err.api_error().unwrap().message(), Some("invalid request"));
}

/// Live (2026-10-02): an address that is not a bridge address (e.g. a plain wallet) gets
/// HTTP 500 `{"error":"cannot get transaction status"}`. A server bug, surfaced as a typed
/// API error with status 500 (see `SPEC_DEVIATIONS.md`).
#[tokio::test]
async fn non_bridge_address_is_a_typed_500() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/status/0xcb1822859cef82cd2eb4e6276c7916e692995130"))
        .respond_with(ResponseTemplate::new(500).set_body_raw(
            r#"{"error":"cannot get transaction status"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .list_transactions("0xcb1822859cef82cd2eb4e6276c7916e692995130")
        .send()
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(500));
    assert_eq!(
        err.api_error().unwrap().message(),
        Some("cannot get transaction status")
    );
}

#[tokio::test]
async fn invalid_parameters_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let bridge = common::polymarket(&server).bridge().clone();
    for limit in [0, 101] {
        let err = bridge
            .list_transactions(BRIDGE_ADDRESS)
            .limit(limit)
            .send()
            .await
            .unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "limit"),
            "{err:?}"
        );
    }
    for address in ["", ".", ".."] {
        let err = bridge.list_transactions(address).send().await.unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "address"),
            "{err:?}"
        );
    }

    let results: Vec<_> = bridge
        .list_transactions(BRIDGE_ADDRESS)
        .limit(0)
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(Error::Validation(_))));
}

#[tokio::test]
async fn missing_transactions_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"nextCursor":null}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
}

#[tokio::test]
async fn missing_next_cursor_is_a_decode_error_not_the_end() {
    let server = common::server().await;
    // `nextCursor` is required (but nullable): a page without it must not end the walk
    // silently.
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"transactions":[{"status":"COMPLETED"}]}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let results: Vec<_> = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let Err(Error::Decode(decode)) = &results[0] else {
        panic!("expected a decode error, got {results:?}");
    };
    assert!(decode.to_string().contains("nextCursor"), "{decode}");
}

#[tokio::test]
async fn page_accessors_normalize_the_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/status/{BRIDGE_ADDRESS}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .bridge()
        .list_transactions(BRIDGE_ADDRESS)
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor(), Some("eyJsYXN0SWQiOiI0MiJ9"));
    assert_eq!(page.items().len(), 3);
    assert_eq!(
        page.into_items()[2].status,
        Some(TransactionStatus::Completed)
    );
}
