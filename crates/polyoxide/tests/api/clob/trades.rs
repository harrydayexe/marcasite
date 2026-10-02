//! Builder trades: `GET /builder/trades`.

use chrono::{TimeZone as _, Utc};
use futures_util::TryStreamExt as _;
use polyoxide::{
    Error,
    clob::{BuilderCode, TradeId},
    types::Side,
};
use serde_json::json;
use wiremock::{
    Mock,
    matchers::{any, method, path, query_param, query_param_is_missing},
};

use super::{clob, json};
use crate::common;

const BUILDER: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";
const MARKET: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

/// A `BuilderTradesResponse` page based on the `GET /builder/trades` example in
/// docs/specs/clob-openapi.yaml.
fn page(ids: &[&str], next_cursor: &str) -> String {
    let data: Vec<_> = ids
        .iter()
        .map(|id| {
            json!({
                "id": id,
                "tradeType": "TAKER",
                "takerOrderHash": "0xabcdef1234567890abcdef1234567890abcdef12",
                "builder": BUILDER,
                "market": MARKET,
                "assetId": "15871154585880608648532107628464183779895785213830018178010423617714102767076",
                "side": "BUY",
                "size": "100000000",
                "sizeUsdc": "50000000",
                "price": "0.5",
                "status": "TRADE_STATUS_CONFIRMED",
                "outcome": "YES",
                "outcomeIndex": 0,
                "owner": "f4f247b7-4ac7-ff29-a152-04fda0a8755a",
                "maker": "0x1234567890123456789012345678901234567890",
                "transactionHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "matchTime": "1700000000",
                "bucketIndex": 0,
                "fee": "300000",
                "feeUsdc": "150000",
                "err_msg": null,
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z"
            })
        })
        .collect();
    json!({"limit": 300, "next_cursor": next_cursor, "count": ids.len(), "data": data}).to_string()
}

#[tokio::test]
async fn builder_trades_sends_filters() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/builder/trades"))
        .and(query_param("builder_code", BUILDER))
        .and(query_param("id", "trade-123"))
        .and(query_param("market", MARKET))
        .and(query_param(
            "asset_id",
            "15871154585880608648532107628464183779895785213830018178010423617714102767076",
        ))
        .and(query_param("before", "1700000000"))
        .and(query_param("after", "1600000000"))
        .and(query_param("next_cursor", "MA=="))
        .respond_with(json(&page(&["trade-123"], "MzAw")))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_builder_trades(BUILDER)
        .id("trade-123")
        .market(MARKET)
        .asset_id("15871154585880608648532107628464183779895785213830018178010423617714102767076")
        .before(Utc.timestamp_opt(1_700_000_000, 0).unwrap())
        .after(Utc.timestamp_opt(1_600_000_000, 0).unwrap())
        .cursor("MA==")
        .send()
        .await
        .unwrap();
    assert_eq!(page.limit, 300);
    assert_eq!(page.next_cursor(), Some("MzAw"));
    let trade = &page.items()[0];
    assert_eq!(trade.id, TradeId::from("trade-123"));
    assert_eq!(trade.builder, BuilderCode::from(BUILDER));
    assert_eq!(trade.side, Side::Buy);
    assert_eq!(trade.match_time.timestamp(), 1_700_000_000);
}

#[tokio::test]
async fn builder_trades_stream_until_lte() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/builder/trades"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&page(&["t1", "t2"], "Mg==")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/builder/trades"))
        .and(query_param("next_cursor", "Mg=="))
        .respond_with(json(&page(&["t3"], "LTE=")))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = clob(&server)
        .list_builder_trades(BUILDER)
        .into_stream()
        .map_ok(|t| t.id)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids, ["t1", "t2", "t3"].map(TradeId::from));
}

#[tokio::test]
async fn invalid_inputs_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(&page(&[], "LTE=")))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let parameter = |err: Error| match err {
        Error::Validation(v) => v.parameter().to_owned(),
        other => panic!("expected Error::Validation, got {other:?}"),
    };

    let err = client.list_builder_trades("0x01").send().await.unwrap_err();
    assert_eq!(parameter(err), "builder_code");

    let err = client
        .list_builder_trades(BUILDER)
        .market("not-a-condition-id")
        .send()
        .await
        .unwrap_err();
    assert_eq!(parameter(err), "market");

    let err = client
        .list_builder_trades(BUILDER)
        .before(Utc.timestamp_opt(-1, 0).unwrap())
        .send()
        .await
        .unwrap_err();
    assert_eq!(parameter(err), "before");

    // The stream yields the validation error once and ends.
    let results: Vec<_> =
        futures_util::StreamExt::collect(client.list_builder_trades("bad").into_stream()).await;
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[tokio::test]
async fn unknown_enum_values_are_kept() {
    let server = common::server().await;
    let mut body: serde_json::Value = serde_json::from_str(&page(&["t1"], "LTE=")).unwrap();
    body["data"][0]["side"] = "HOLD".into();
    Mock::given(method("GET"))
        .and(path("/builder/trades"))
        .respond_with(json(&body.to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_builder_trades(BUILDER)
        .send()
        .await
        .unwrap();
    assert_eq!(page.items()[0].side, Side::Unknown("HOLD".to_owned()));
    assert!(page.is_last_page());
}

#[tokio::test]
async fn empty_filter_ids_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(&page(&[], "LTE=")))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let err = client
        .list_builder_trades(BUILDER)
        .id("")
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "id"),
        "{err:?}"
    );
    let err = client
        .list_builder_trades(BUILDER)
        .asset_id("")
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "asset_id"),
        "{err:?}"
    );
}
