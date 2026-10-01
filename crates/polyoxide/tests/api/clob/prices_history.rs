//! Price history: `GET /prices-history`, `POST /batch-prices-history`.

use chrono::{TimeZone as _, Utc};
use polyoxide::{
    Error,
    clob::{MAX_BATCH_PRICES_HISTORY_MARKETS, PriceHistoryInterval},
    types::TokenId,
};
use serde_json::json;
use wiremock::{
    Mock,
    matchers::{any, body_json, method, path, query_param, query_param_is_missing},
};

use super::{api_error, clob, json};
use crate::common;

#[tokio::test]
async fn get_prices_history_with_all_parameters() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/prices-history"))
        .and(query_param("market", "123"))
        .and(query_param("startTs", "1700000000"))
        .and(query_param("endTs", "1700086400"))
        .and(query_param("interval", "1d"))
        .and(query_param("fidelity", "60"))
        // Fields from `components/schemas/PricesHistoryResponse` (no example in the spec).
        .respond_with(json(
            r#"{"history":[{"t":1700000000,"p":0.45},{"t":1700003600,"p":0.47}]}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let history = clob(&server)
        .get_prices_history("123")
        .start_ts(Utc.timestamp_opt(1_700_000_000, 0).unwrap())
        .end_ts(Utc.timestamp_opt(1_700_086_400, 0).unwrap())
        .interval(PriceHistoryInterval::OneDay)
        .fidelity(60)
        .send()
        .await
        .unwrap();
    let points = history.history.unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(
        points[1].timestamp,
        Some(Utc.timestamp_opt(1_700_003_600, 0).unwrap())
    );
    assert_eq!(points[1].price, Some("0.47".parse().unwrap()));
}

#[tokio::test]
async fn get_prices_history_minimal() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/prices-history"))
        .and(query_param("market", "123"))
        .and(query_param_is_missing("startTs"))
        .and(query_param_is_missing("interval"))
        .respond_with(json(r#"{"history":[]}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/prices-history"))
        .and(query_param("market", "bad"))
        .respond_with(api_error(400, "invalid filters"))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let history = client.get_prices_history("123").send().await.unwrap();
    assert_eq!(history.history, Some(vec![]));
    let err = client.get_prices_history("bad").send().await.unwrap_err();
    assert_eq!(err.api_error().unwrap().message(), Some("invalid filters"));
}

#[tokio::test]
async fn get_batch_prices_history() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/batch-prices-history"))
        .and(body_json(json!({
            "markets": ["123", "456"],
            "end_ts": 1700086400,
            "interval": "max",
            "fidelity": 5
        })))
        // Fields from `components/schemas/BatchPricesHistoryResponse` (no example in the spec).
        .respond_with(json(
            r#"{"history":{"123":[{"t":1700000000,"p":0.45}],"456":[]}}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let batch = clob(&server)
        .get_batch_prices_history(["123", "456"])
        .end_ts(Utc.timestamp_opt(1_700_086_400, 0).unwrap())
        .interval(PriceHistoryInterval::Max)
        .fidelity(5)
        .send()
        .await
        .unwrap();
    let history = batch.history.unwrap();
    assert_eq!(history[&TokenId::from("123")].len(), 1);
    assert!(history[&TokenId::from("456")].is_empty());
}

#[tokio::test]
async fn batch_prices_history_limit_is_enforced() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json("{}"))
        .expect(0)
        .mount(&server)
        .await;

    let markets: Vec<String> = (0..=MAX_BATCH_PRICES_HISTORY_MARKETS)
        .map(|i| i.to_string())
        .collect();
    let err = clob(&server)
        .get_batch_prices_history(&markets)
        .send()
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "markets");
}
