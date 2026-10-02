//! Market data: books, prices, midpoints, spreads, last trade prices, fee rate, tick size,
//! neg risk.

use std::time::Duration;

use marcasite::{
    Decimal, Error, StatusCode,
    clob::{BookRequest, MAX_LAST_TRADE_PRICES_TOKEN_IDS},
    types::{Side, TokenId},
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_json, method, path, query_param, query_param_is_missing},
};

use super::{api_error, clob, json, retrying_clob};
use crate::common;

/// The parameter named by a validation error.
fn invalid_parameter(err: Error) -> String {
    match err {
        Error::Validation(v) => v.parameter().to_owned(),
        other => panic!("expected Error::Validation, got {other:?}"),
    }
}

fn d(s: &str) -> Decimal {
    s.parse().unwrap()
}

/// Order book in the live format (shape captured from `GET /book` 2026-10-02: `timestamp` in
/// Unix milliseconds, `hash` 40 hex characters without `0x`, `market` 64 hex), with a
/// two-sided book.
const BOOK: &str = r#"{
    "market": "0x81a537b379a35e4e17c286d3b37394e94bd74c1779bbe9a13670eb991b201a3a",
    "asset_id": "0xabc123def456...",
    "timestamp": "1790934258308",
    "hash": "f3e9b337b4a1184b1f35282e0a0d0ed5e3b7c07e",
    "bids": [{"price": "0.45", "size": "100"}, {"price": "0.44", "size": "200"}],
    "asks": [{"price": "0.46", "size": "150"}, {"price": "0.47", "size": "250"}],
    "min_order_size": "1",
    "tick_size": "0.01",
    "neg_risk": false,
    "last_trade_price": "0.45"
}"#;

#[tokio::test]
async fn get_order_book() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/book"))
        .and(query_param("token_id", "0xabc123def456..."))
        .respond_with(json(BOOK))
        .expect(1)
        .mount(&server)
        .await;

    let book = clob(&server)
        .get_order_book("0xabc123def456...")
        .await
        .unwrap();
    assert_eq!(
        book.market,
        "0x81a537b379a35e4e17c286d3b37394e94bd74c1779bbe9a13670eb991b201a3a"
    );
    assert_eq!(book.timestamp.timestamp_millis(), 1_790_934_258_308);
    assert_eq!(book.bids[0].price, d("0.45"));
    assert_eq!(book.asks.len(), 2);
    assert_eq!(book.min_order_size, d("1"));
}

#[tokio::test]
async fn get_order_book_not_found() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/book"))
        .respond_with(api_error(
            404,
            "No orderbook exists for the requested token id",
        ))
        .mount(&server)
        .await;

    let err = clob(&server).get_order_book("1").await.unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
    let api = err.api_error().unwrap();
    assert_eq!(
        api.message(),
        Some("No orderbook exists for the requested token id")
    );
}

#[tokio::test]
async fn bad_request_carries_error_message() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/price"))
        .respond_with(api_error(400, "Invalid side"))
        .mount(&server)
        .await;

    let err = clob(&server).get_price("1", Side::Buy).await.unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status(), StatusCode::BAD_REQUEST);
    assert_eq!(api.message(), Some("Invalid side"));
}

/// `429` with the documented `ErrorResponse` fields `code` and `retry_after_seconds`
/// (`components/schemas/ErrorResponse` in docs/polymarket/specs/clob-openapi.yaml).
#[tokio::test]
async fn rate_limit_carries_code_and_retry_after() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/midpoint"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": "Too many requests",
            "code": "RATE_LIMITED",
            "retry_after_seconds": 3
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = clob(&server).get_midpoint("1").await.unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.status(), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3)));
    assert!(err.is_retryable());
    let api = err.api_error().unwrap();
    assert_eq!(api.message(), Some("Too many requests"));
    assert_eq!(api.code(), Some("RATE_LIMITED"));
}

#[tokio::test]
async fn malformed_body_is_a_decode_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/book"))
        .respond_with(json(
            r#"{"market":"0x1","asset_id":"1","bids":[{"price":"abc"}]}"#,
        ))
        .mount(&server)
        .await;

    let err = clob(&server).get_order_book("1").await.unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "bids[0].price");
}

#[tokio::test]
async fn empty_required_lists_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(json("[]"))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let none: [&str; 0] = [];
    let no_pairs = Vec::<(TokenId, Side)>::new();
    for err in [
        // Required request bodies.
        client.get_order_books(none).await.unwrap_err(),
        client.get_midpoints(none).await.unwrap_err(),
        client.get_spreads(none).await.unwrap_err(),
        client.get_last_trade_prices(none).await.unwrap_err(),
        client.get_prices(&no_pairs).await.unwrap_err(),
    ] {
        assert_eq!(invalid_parameter(err), "token_ids");
    }
}

#[tokio::test]
async fn malformed_token_ids_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(json("{}"))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let ids = vec!["1", ""];
    let err = client.get_midpoints(ids.clone()).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");
    let err = client.get_order_books(ids.clone()).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");
    let err = client.get_last_trade_prices(ids.clone()).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");
    let err = client.get_spreads(ids).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");
    let err = client.get_prices([("", Side::Buy)]).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");

    // Required single ids, as query or path parameters.
    for err in [
        client.get_order_book("").await.unwrap_err(),
        client.get_price("", Side::Buy).await.unwrap_err(),
        client.get_midpoint("").await.unwrap_err(),
        client.get_spread("").await.unwrap_err(),
        client.get_last_trade_price("").await.unwrap_err(),
        client.get_fee_rate_by_path("").await.unwrap_err(),
        client.get_tick_size_by_path("").await.unwrap_err(),
        client.get_neg_risk_by_path("").await.unwrap_err(),
        client.get_fee_rate().token_id("").send().await.unwrap_err(),
        client
            .get_tick_size()
            .token_id("")
            .send()
            .await
            .unwrap_err(),
        client.get_neg_risk().token_id("").send().await.unwrap_err(),
    ] {
        assert_eq!(invalid_parameter(err), "token_id");
    }
}

#[tokio::test]
async fn prices_require_a_side() {
    let server = common::server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(json("{}"))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let requests = [
        BookRequest::new("1").with_side(Side::Buy),
        BookRequest::new("2"),
    ];
    let err = client.get_prices(&requests).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "side");
}

#[tokio::test]
async fn get_order_books() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/books"))
        .and(body_json(json!([
            {"token_id": "0xabc123def456..."},
            {"token_id": "0xdef456abc123...", "side": "BUY"}
        ])))
        .respond_with(json(&format!("[{BOOK}]")))
        .expect(1)
        .mount(&server)
        .await;

    let books = clob(&server)
        .get_order_books([
            BookRequest::from("0xabc123def456..."),
            BookRequest::new("0xdef456abc123...").with_side(Side::Buy),
        ])
        .await
        .unwrap();
    assert_eq!(books[0].tick_size, d("0.01"));
}

#[tokio::test]
async fn read_only_post_is_retried() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/books"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/books"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let books = retrying_clob(&server).get_order_books(["1"]).await.unwrap();
    assert!(books.is_empty());
}

#[tokio::test]
async fn get_price() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/price"))
        .and(query_param("token_id", "1"))
        .and(query_param("side", "SELL"))
        .respond_with(json(r#"{"price": "0.45"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let price = clob(&server).get_price("1", Side::Sell).await.unwrap();
    assert_eq!(price.price, d("0.45"));
}

/// Response of `POST /prices`: numbers in docs/polymarket/specs/clob-openapi.yaml, numeric strings live
/// (`PRICES_LIVE`, captured 2026-10-02); both decode.
const PRICES: &str = r#"{"0xabc123def456...":{"BUY":0.45},"0xdef456abc123...":{"SELL":0.52}}"#;
const PRICES_LIVE: &str =
    r#"{"0xabc123def456...":{"BUY":"0.45"},"0xdef456abc123...":{"SELL":"0.52"}}"#;

#[tokio::test]
async fn get_prices() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/prices"))
        .and(body_json(json!([
            {"token_id": "0xabc123def456...", "side": "BUY"},
            {"token_id": "0xdef456abc123...", "side": "SELL"}
        ])))
        .respond_with(json(PRICES_LIVE))
        .expect(2)
        .mount(&server)
        .await;

    let prices = clob(&server)
        .get_prices([
            ("0xabc123def456...", Side::Buy),
            ("0xdef456abc123...", Side::Sell),
        ])
        .await
        .unwrap();
    assert_eq!(prices.len(), 2);
    assert_eq!(
        prices[&TokenId::from("0xabc123def456...")][&Side::Buy],
        d("0.45")
    );

    // Borrowed `(TokenId, Side)` pairs work as well as owned ones, and numbers decode too.
    let pairs = vec![
        (TokenId::from("0xabc123def456..."), Side::Buy),
        (TokenId::from("0xdef456abc123..."), Side::Sell),
    ];
    let prices = clob(&server).get_prices(&pairs).await.unwrap();
    assert_eq!(
        prices[&TokenId::from("0xdef456abc123...")][&Side::Sell],
        d("0.52")
    );
    let numbers: std::collections::HashMap<TokenId, std::collections::HashMap<Side, Decimal>> =
        serde_json::from_str(PRICES).unwrap();
    assert_eq!(numbers, prices);
}

#[tokio::test]
async fn get_midpoint() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/midpoint"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"mid": "0.45"}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/midpoint"))
        .and(query_param("token_id", "2"))
        .respond_with(json(r#"{"mid_price": "0.46"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let mid = client.get_midpoint("1").await.unwrap();
    assert_eq!(mid.mid, d("0.45"));
    // The spec's `mid_price` spelling is still accepted.
    let mid = client.get_midpoint("2").await.unwrap();
    assert_eq!(mid.mid, d("0.46"));
}

/// Example response of `POST /midpoints` in docs/polymarket/specs/clob-openapi.yaml (same shape live).
const MIDPOINTS: &str = r#"{"0xabc123def456...":"0.45","0xdef456abc123...":"0.52"}"#;

#[tokio::test]
async fn get_midpoints() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/midpoints"))
        .and(body_json(json!([
            {"token_id": "0xabc123def456..."},
            {"token_id": "0xdef456abc123..."}
        ])))
        .respond_with(json(MIDPOINTS))
        .expect(1)
        .mount(&server)
        .await;

    let token_ids = vec![
        TokenId::from("0xabc123def456..."),
        TokenId::from("0xdef456abc123..."),
    ];
    let mids = clob(&server).get_midpoints(&token_ids).await.unwrap();
    assert_eq!(mids[&token_ids[0]], d("0.45"));
}

#[tokio::test]
async fn get_spread() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/spread"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"spread": "0.02"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let spread = clob(&server).get_spread("1").await.unwrap();
    assert_eq!(spread.spread, d("0.02"));
}

#[tokio::test]
async fn get_spreads() {
    let server = common::server().await;
    // Example response of `POST /spreads` in docs/polymarket/specs/clob-openapi.yaml.
    Mock::given(method("POST"))
        .and(path("/spreads"))
        .and(body_json(json!([
            {"token_id": "0xabc123def456..."},
            {"token_id": "0xdef456abc123..."}
        ])))
        .respond_with(json(
            r#"{"0xabc123def456...":"0.02","0xdef456abc123...":"0.015"}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let spreads = clob(&server)
        .get_spreads(["0xabc123def456...", "0xdef456abc123..."])
        .await
        .unwrap();
    assert_eq!(spreads[&TokenId::from("0xdef456abc123...")], d("0.015"));
}

#[tokio::test]
async fn get_last_trade_price() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/last-trade-price"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"price": "0.45", "side": "BUY"}"#))
        .expect(1)
        .mount(&server)
        .await;
    // Documented default when there are no trades: price "0.5", empty side.
    Mock::given(method("GET"))
        .and(path("/last-trade-price"))
        .and(query_param("token_id", "2"))
        .respond_with(json(r#"{"price": "0.5", "side": ""}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let last = client.get_last_trade_price("1").await.unwrap();
    assert_eq!(last.price, d("0.45"));
    assert_eq!(last.side, Some(Side::Buy));
    let none = client.get_last_trade_price("2").await.unwrap();
    assert_eq!(none.side, None);
}

/// Example response of `POST /last-trades-prices` in docs/polymarket/specs/clob-openapi.yaml.
const LAST_TRADES_PRICES: &str = r#"[
    {"token_id": "0xabc123def456...", "price": "0.45", "side": "BUY"},
    {"token_id": "0xdef456abc123...", "price": "0.52", "side": "SELL"}
]"#;

/// The batch form documents only `BUY`/`SELL`; an empty side (the "no trades" value of
/// `GET /last-trade-price`) maps to `None`, and an unknown value is kept.
#[tokio::test]
async fn last_trade_prices_side_edge_cases() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/last-trades-prices"))
        .respond_with(json(
            r#"[
                {"token_id": "1", "price": "0.5", "side": ""},
                {"token_id": "2", "price": "0.45", "side": "HOLD"}
            ]"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let prices = clob(&server)
        .get_last_trade_prices(["1", "2"])
        .await
        .unwrap();
    assert_eq!(prices[0].side, None);
    assert_eq!(prices[1].side, Some(Side::Unknown("HOLD".to_owned())));
}

#[tokio::test]
async fn get_last_trade_prices() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/last-trades-prices"))
        .and(body_json(json!([
            {"token_id": "0xabc123def456..."},
            {"token_id": "0xdef456abc123..."}
        ])))
        .respond_with(json(LAST_TRADES_PRICES))
        .expect(1)
        .mount(&server)
        .await;

    let prices = clob(&server)
        .get_last_trade_prices(["0xabc123def456...", "0xdef456abc123..."])
        .await
        .unwrap();
    assert_eq!(prices[0].price, d("0.45"));
    assert_eq!(prices[1].token_id, "0xdef456abc123...");
    assert_eq!(prices[1].side, Some(Side::Sell));
}

#[tokio::test]
async fn last_trade_prices_limit_is_enforced() {
    let server = common::server().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(json("[]"))
        .expect(0)
        .mount(&server)
        .await;

    let too_many: Vec<String> = (0..=MAX_LAST_TRADE_PRICES_TOKEN_IDS)
        .map(|i| i.to_string())
        .collect();
    let client = clob(&server);
    let err = client.get_last_trade_prices(&too_many).await.unwrap_err();
    assert_eq!(invalid_parameter(err), "token_ids");
}

#[tokio::test]
async fn last_trade_prices_accepts_the_maximum() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/last-trades-prices"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let max: Vec<String> = (0..MAX_LAST_TRADE_PRICES_TOKEN_IDS)
        .map(|i| i.to_string())
        .collect();
    let prices = clob(&server).get_last_trade_prices(&max).await.unwrap();
    assert!(prices.is_empty());
}

#[tokio::test]
async fn fee_rate_by_query_and_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/fee-rate"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"base_fee": 30}"#))
        .expect(1)
        .mount(&server)
        .await;
    // The docs do not say how the server answers a request without `token_id`; this only
    // checks that the parameter is omitted and that an error response surfaces.
    Mock::given(method("GET"))
        .and(path("/fee-rate"))
        .and(query_param_is_missing("token_id"))
        .respond_with(api_error(400, "Invalid token id"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/fee-rate/2"))
        .respond_with(json(r#"{"base_fee": 0}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let fee = client.get_fee_rate().token_id("1").send().await.unwrap();
    assert_eq!(fee.base_fee, 30);
    let err = client.get_fee_rate().send().await.unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::BAD_REQUEST));
    let fee = client.get_fee_rate_by_path("2").await.unwrap();
    assert_eq!(fee.base_fee, 0);
}

#[tokio::test]
async fn tick_size_by_query_and_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tick-size"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"minimum_tick_size": 0.01}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tick-size/2"))
        .respond_with(json(r#"{"minimum_tick_size": 0.001}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let tick = client.get_tick_size().token_id("1").send().await.unwrap();
    assert_eq!(tick.minimum_tick_size, d("0.01"));
    let tick = client.get_tick_size_by_path("2").await.unwrap();
    assert_eq!(tick.minimum_tick_size, d("0.001"));
}

#[tokio::test]
async fn neg_risk_by_query_and_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/neg-risk"))
        .and(query_param("token_id", "1"))
        .respond_with(json(r#"{"neg_risk": false}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/neg-risk/2"))
        .respond_with(json(r#"{"neg_risk": true}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/neg-risk/3"))
        .respond_with(api_error(404, "market not found"))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    assert!(
        !client
            .get_neg_risk()
            .token_id("1")
            .send()
            .await
            .unwrap()
            .neg_risk
    );
    assert!(client.get_neg_risk_by_path("2").await.unwrap().neg_risk);
    let err = client.get_neg_risk_by_path("3").await.unwrap_err();
    assert!(err.is_not_found(), "{err:?}");
}

#[tokio::test]
async fn path_parameters_are_encoded() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tick-size/a%2Fb"))
        .respond_with(json(r#"{"minimum_tick_size": 0.01}"#))
        .expect(1)
        .mount(&server)
        .await;

    let tick = clob(&server).get_tick_size_by_path("a/b").await.unwrap();
    assert_eq!(tick.minimum_tick_size, d("0.01"));
}
