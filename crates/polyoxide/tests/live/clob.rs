//! CLOB API live tests (public, unauthenticated endpoints).
//!
//! Every implemented endpoint is called once through the SDK, and each distinct response model
//! is also decoded with [`check`] from a raw fetch of the same route, which reports drift.
//! A test that fails shows that the SDK disagrees with the live API; see the report.

use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::clob::{
    BatchPricesHistory, BookRequest, BuilderTrade, ClobClient, ClobMarketDetails, CurrentReward,
    FeeRate, LastTradePrice, LiveActivityMarket, Market, MarketByToken, MarketReward, MarketsPage,
    Midpoint, MultiMarketInfo, NegRisk, OrderBookSummary, Page, PriceHistoryInterval,
    PricesHistory, RebatedFees, RewardsMarketsOrderBy, SimplifiedMarket, SortDirection, Spread,
    TickSize, TokenLastTradePrice,
};
use polyoxide::clob::{Side, TokenId};
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::common::{CLOB, check, get, pm, post, sample};

/// Builder code used by the docs examples (`0x00..01`).
const BUILDER_CODE: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";

fn clob() -> ClobClient {
    pm().clob().clone()
}

/// A raw response of any status (the shared `get` / `post` helpers panic on non-2xx).
struct Response {
    status: u16,
    text: String,
}

async fn raw_get(path: &str, query: &[(&str, &str)]) -> Response {
    let response = reqwest::Client::builder()
        .user_agent("polyoxide-live-tests")
        .build()
        .expect("reqwest client builds")
        .get(format!("{CLOB}{path}"))
        .query(query)
        .send()
        .await
        .unwrap_or_else(|e| panic!("GET {path}: {e}"));
    Response {
        status: response.status().as_u16(),
        text: response.text().await.expect("body reads"),
    }
}

async fn raw_post(path: &str, body: &Value) -> Response {
    let response = reqwest::Client::builder()
        .user_agent("polyoxide-live-tests")
        .build()
        .expect("reqwest client builds")
        .post(format!("{CLOB}{path}"))
        .json(body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {path}: {e}"));
    Response {
        status: response.status().as_u16(),
        text: response.text().await.expect("body reads"),
    }
}

/// The sample market's two token ids, as raw strings.
async fn tokens() -> (String, String) {
    let s = sample().await;
    (s.token_ids[0].clone(), s.token_ids[1].clone())
}

/// A POST body listing both sample tokens, each with a `side` (accepted by every batch route).
async fn book_requests_body() -> Value {
    let (a, b) = tokens().await;
    json!([
        { "token_id": a, "side": "BUY" },
        { "token_id": b, "side": "SELL" },
    ])
}

fn book_requests(a: &str, b: &str) -> Vec<BookRequest> {
    vec![
        BookRequest::new(a).with_side(Side::Buy),
        BookRequest::new(b).with_side(Side::Sell),
    ]
}

/// Asserts a book looks like a book (structure only).
fn assert_book(book: &OrderBookSummary, token: &str, condition_id: &str) {
    assert_eq!(book.asset_id.as_str(), token);
    assert_eq!(book.market.as_str(), condition_id);
    // The timestamp is Unix milliseconds: read as seconds or microseconds it would be far
    // from now.
    assert!(
        (Utc::now() - book.timestamp).num_hours().abs() < 24,
        "book timestamp {}",
        book.timestamp
    );
    assert!(!book.hash.is_empty());
    assert!(book.tick_size > Decimal::ZERO);
}

// ---- time -------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_server_time() {
    let time = clob().get_server_time().await.unwrap();
    let raw = get(CLOB, "/time", &[]).await;
    // `/time` is a bare JSON integer of Unix seconds.
    let seconds = raw.json.as_i64().expect("bare integer");
    assert!((time.timestamp() - seconds).abs() < 60);
    assert!(
        (Utc::now() - time).num_seconds().abs() < 120,
        "clock skew: {time}"
    );
}

// ---- prices -----------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_midpoint() {
    let (a, _) = tokens().await;
    let midpoint = clob().get_midpoint(a.as_str()).await.unwrap();
    assert!(midpoint.mid >= Decimal::ZERO && midpoint.mid <= Decimal::ONE);
    check::<Midpoint>(
        "GET /midpoint",
        &get(CLOB, "/midpoint", &[("token_id", &a)]).await,
    );
}

/// Pins SPEC_DEVIATIONS.md "GET /midpoint field name": the body has `mid`, not the
/// documented `mid_price`.
#[tokio::test]
#[ignore = "live network"]
async fn pin_midpoint_field_is_mid() {
    let (a, _) = tokens().await;
    let raw = get(CLOB, "/midpoint", &[("token_id", &a)]).await;
    let object = raw.json.as_object().expect("an object");
    assert!(object.contains_key("mid"), "{}", raw.text);
    assert!(!object.contains_key("mid_price"), "{}", raw.text);
    assert!(raw.json["mid"].is_string(), "{}", raw.text);
}

/// `POST /midpoints` (spec operation `getMidpointsPost`).
#[tokio::test]
#[ignore = "live network"]
async fn get_midpoints() {
    let (a, b) = tokens().await;
    let midpoints = clob()
        .get_midpoints([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(midpoints.len(), 2);
    let body = json!([{ "token_id": a }, { "token_id": b }]);
    check::<HashMap<TokenId, Decimal>>("POST /midpoints", &post(CLOB, "/midpoints", &body).await);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_spread() {
    let (a, _) = tokens().await;
    let spread = clob().get_spread(a.as_str()).await.unwrap();
    assert!(spread.spread >= Decimal::ZERO);
    check::<Spread>(
        "GET /spread",
        &get(CLOB, "/spread", &[("token_id", &a)]).await,
    );
}

/// `POST /spreads` (the only documented form).
#[tokio::test]
#[ignore = "live network"]
async fn get_spreads() {
    let (a, b) = tokens().await;
    let spreads = clob().get_spreads([a.as_str(), b.as_str()]).await.unwrap();
    assert_eq!(spreads.len(), 2);
    let body = json!([{ "token_id": a }, { "token_id": b }]);
    check::<HashMap<TokenId, Decimal>>("POST /spreads", &post(CLOB, "/spreads", &body).await);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_last_trade_price() {
    let (a, _) = tokens().await;
    let last = clob().get_last_trade_price(a.as_str()).await.unwrap();
    assert!(last.price >= Decimal::ZERO && last.price <= Decimal::ONE);
    check::<LastTradePrice>(
        "GET /last-trade-price",
        &get(CLOB, "/last-trade-price", &[("token_id", &a)]).await,
    );
}

/// `POST /last-trades-prices`.
#[tokio::test]
#[ignore = "live network"]
async fn get_last_trade_prices() {
    let (a, b) = tokens().await;
    let prices = clob()
        .get_last_trade_prices([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(prices.len(), 2);
    let body = json!([{ "token_id": a }, { "token_id": b }]);
    check::<Vec<TokenLastTradePrice>>(
        "POST /last-trades-prices",
        &post(CLOB, "/last-trades-prices", &body).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_price() {
    let (a, _) = tokens().await;
    for side in [Side::Buy, Side::Sell] {
        let price = clob().get_price(a.as_str(), side.clone()).await.unwrap();
        assert!(price.price >= Decimal::ZERO && price.price <= Decimal::ONE);
    }
    check::<polyoxide::clob::Price>(
        "GET /price",
        &get(CLOB, "/price", &[("token_id", &a), ("side", "BUY")]).await,
    );
}

/// `POST /prices`.
#[tokio::test]
#[ignore = "live network"]
async fn get_prices() {
    let (a, b) = tokens().await;
    let prices = clob().get_prices(book_requests(&a, &b)).await.unwrap();
    assert_eq!(prices.len(), 2);
    assert!(prices[&TokenId::from(a.as_str())].contains_key(&Side::Buy));
    assert!(prices[&TokenId::from(b.as_str())].contains_key(&Side::Sell));
    check::<HashMap<TokenId, HashMap<Side, Decimal>>>(
        "POST /prices",
        &post(CLOB, "/prices", &book_requests_body().await).await,
    );
}

/// Pins SPEC_DEVIATIONS.md "Plural GET query forms": `GET /midpoints`, `/last-trades-prices`,
/// `/prices` and `/books` with `token_ids` answer `400 Invalid payload` for every encoding
/// (comma-separated, repeated keys, a single id), while the POST forms work.
#[tokio::test]
#[ignore = "live network"]
async fn pin_plural_get_forms_are_rejected() {
    let (a, b) = tokens().await;
    let joined = format!("{a},{b}");
    let mut unexpected = Vec::new();
    for path in ["/midpoints", "/last-trades-prices", "/prices", "/books"] {
        let sides = path == "/prices";
        let mut variants: Vec<(&str, Vec<(&str, &str)>)> = vec![
            ("csv", vec![("token_ids", joined.as_str())]),
            (
                "repeated",
                vec![("token_ids", a.as_str()), ("token_ids", b.as_str())],
            ),
            ("single", vec![("token_ids", a.as_str())]),
        ];
        if sides {
            for (_, query) in &mut variants {
                query.push(("sides", "BUY,SELL"));
            }
        }
        for (name, query) in variants {
            let response = raw_get(path, &query).await;
            if response.status != 400 || !response.text.contains("Invalid payload") {
                unexpected.push(format!(
                    "GET {path} ({name}): HTTP {} {}",
                    response.status, response.text
                ));
            }
        }
    }
    assert!(
        unexpected.is_empty(),
        "a plural GET form works now; implement it: {unexpected:#?}"
    );
}

/// Pins SPEC_DEVIATIONS.md "POST /prices without a side": an item without `side` is accepted
/// and priced as SELL only.
#[tokio::test]
#[ignore = "live network"]
async fn pin_post_prices_without_side_is_sell_only() {
    let (a, _) = tokens().await;
    let response = raw_post("/prices", &json!([{ "token_id": a }])).await;
    assert_eq!(response.status, 200, "{}", response.text);
    let body: Value = serde_json::from_str(&response.text).unwrap();
    let sides = body[&a].as_object().expect("prices of the token");
    assert_eq!(sides.keys().collect::<Vec<_>>(), ["SELL"], "{body}");
    assert!(body[&a]["SELL"].is_string(), "{body}");
}

/// Pins SPEC_DEVIATIONS.md "Price values are strings": `GET /price` sends a string, the spec
/// documents a number.
#[tokio::test]
#[ignore = "live network"]
async fn pin_price_is_a_string() {
    let (a, _) = tokens().await;
    let raw = get(CLOB, "/price", &[("token_id", &a), ("side", "BUY")]).await;
    assert!(raw.json["price"].is_string(), "{}", raw.text);
}

// ---- market parameters -----------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_fee_rate_query_and_path() {
    let (a, _) = tokens().await;
    let by_query = clob()
        .get_fee_rate()
        .token_id(a.as_str())
        .send()
        .await
        .unwrap();
    let by_path = clob().get_fee_rate_by_path(a.as_str()).await.unwrap();
    assert_eq!(by_query, by_path);
    check::<FeeRate>(
        "GET /fee-rate",
        &get(CLOB, "/fee-rate", &[("token_id", &a)]).await,
    );
    check::<FeeRate>(
        "GET /fee-rate/{token_id}",
        &get(CLOB, &format!("/fee-rate/{a}"), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_tick_size_query_and_path() {
    let (a, _) = tokens().await;
    let by_query = clob()
        .get_tick_size()
        .token_id(a.as_str())
        .send()
        .await
        .unwrap();
    let by_path = clob().get_tick_size_by_path(a.as_str()).await.unwrap();
    assert_eq!(by_query, by_path);
    assert!(by_query.minimum_tick_size > Decimal::ZERO);
    check::<TickSize>(
        "GET /tick-size",
        &get(CLOB, "/tick-size", &[("token_id", &a)]).await,
    );
    check::<TickSize>(
        "GET /tick-size/{token_id}",
        &get(CLOB, &format!("/tick-size/{a}"), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_neg_risk_query_and_path() {
    let (a, _) = tokens().await;
    let by_query = clob()
        .get_neg_risk()
        .token_id(a.as_str())
        .send()
        .await
        .unwrap();
    let by_path = clob().get_neg_risk_by_path(a.as_str()).await.unwrap();
    assert_eq!(by_query, by_path);
    check::<NegRisk>(
        "GET /neg-risk",
        &get(CLOB, "/neg-risk", &[("token_id", &a)]).await,
    );
    check::<NegRisk>(
        "GET /neg-risk/{token_id}",
        &get(CLOB, &format!("/neg-risk/{a}"), &[]).await,
    );
}

// ---- order books -----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_order_book() {
    let s = sample().await;
    let (a, _) = tokens().await;
    let book = clob().get_order_book(a.as_str()).await.unwrap();
    assert_book(&book, &a, &s.condition_id);
    let raw = get(CLOB, "/book", &[("token_id", &a)]).await;
    check::<OrderBookSummary>("GET /book", &raw);
}

/// Pins SPEC_DEVIATIONS.md "GET /book timestamp and hash formats": `timestamp` is a string of
/// Unix milliseconds (13 digits), `hash` is 40 hex characters without `0x`, `market` is 64 hex
/// with `0x`.
#[tokio::test]
#[ignore = "live network"]
async fn pin_book_formats() {
    let (a, _) = tokens().await;
    let raw = get(CLOB, "/book", &[("token_id", &a)]).await;
    let timestamp = raw.json["timestamp"].as_str().expect("a string timestamp");
    assert_eq!(timestamp.len(), 13, "{timestamp}");
    assert!(timestamp.bytes().all(|b| b.is_ascii_digit()));
    let hash = raw.json["hash"].as_str().expect("a hash");
    assert_eq!(hash.len(), 40, "{hash}");
    assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()), "{hash}");
    let market = raw.json["market"].as_str().expect("a market");
    assert_eq!(market.len(), 66, "{market}");
    assert!(market.starts_with("0x"));
}

/// `POST /books`.
#[tokio::test]
#[ignore = "live network"]
async fn get_order_books() {
    let s = sample().await;
    let (a, b) = tokens().await;
    let books = clob()
        .get_order_books([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(books.len(), 2);
    assert_book(&books[0], &a, &s.condition_id);
    let body = json!([{ "token_id": a }, { "token_id": b }]);
    check::<Vec<OrderBookSummary>>("POST /books", &post(CLOB, "/books", &body).await);
}

// ---- market listings -------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_simplified_markets() {
    let page = clob().list_simplified_markets().send().await.unwrap();
    assert!(!page.items().is_empty());
    let raw = get(CLOB, "/simplified-markets", &[]).await;
    eprintln!(
        "INFO /simplified-markets next_cursor = {}",
        raw.json["next_cursor"]
    );
    check::<MarketsPage<SimplifiedMarket>>("GET /simplified-markets", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_simplified_markets_stream_pages() {
    stream_two_pages(
        clob().list_simplified_markets().send().await.unwrap(),
        clob().list_simplified_markets().into_stream(),
    )
    .await;
}

#[tokio::test]
#[ignore = "live network"]
async fn list_sampling_markets() {
    let page = clob().list_sampling_markets().send().await.unwrap();
    assert!(!page.items().is_empty());
    let raw = get(CLOB, "/sampling-markets", &[]).await;
    eprintln!(
        "INFO /sampling-markets next_cursor = {}",
        raw.json["next_cursor"]
    );
    check::<MarketsPage<Market>>("GET /sampling-markets", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_sampling_markets_stream_pages() {
    stream_two_pages(
        clob().list_sampling_markets().send().await.unwrap(),
        clob().list_sampling_markets().into_stream(),
    )
    .await;
}

#[tokio::test]
#[ignore = "live network"]
async fn list_sampling_simplified_markets() {
    let page = clob()
        .list_sampling_simplified_markets()
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    let raw = get(CLOB, "/sampling-simplified-markets", &[]).await;
    eprintln!(
        "INFO /sampling-simplified-markets next_cursor = {}",
        raw.json["next_cursor"]
    );
    check::<MarketsPage<SimplifiedMarket>>("GET /sampling-simplified-markets", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_sampling_simplified_markets_stream_pages() {
    stream_two_pages(
        clob()
            .list_sampling_simplified_markets()
            .send()
            .await
            .unwrap(),
        clob().list_sampling_simplified_markets().into_stream(),
    )
    .await;
}

/// Streams one item past the first page, which forces a second request, and checks that the
/// stream's first page matches `send()`.
async fn stream_two_pages<T: std::fmt::Debug + Send + 'static>(
    first: MarketsPage<T>,
    stream: polyoxide::Paginated<T>,
) {
    let first_len = first.items().len();
    let has_more = first.next_cursor().is_some();
    eprintln!(
        "INFO first page: {first_len} items, next cursor {:?}",
        first.next_cursor()
    );
    let wanted = first_len + 3;
    let items: Vec<T> = stream.take(wanted).try_collect().await.unwrap();
    if has_more {
        assert_eq!(
            items.len(),
            wanted,
            "the stream did not continue past page 1"
        );
    } else {
        assert_eq!(items.len(), first_len);
    }
}

// ---- single-market lookups -------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_clob_market_info() {
    let s = sample().await;
    let info = clob()
        .get_clob_market_info(s.condition_id.as_str())
        .await
        .unwrap();
    eprintln!("INFO clob-markets/{{id}}: {info:?}");
    check::<ClobMarketDetails>(
        "GET /clob-markets/{condition_id}",
        &get(CLOB, &format!("/clob-markets/{}", s.condition_id), &[]).await,
    );
}

/// Pins SPEC_DEVIATIONS.md "ClobMarketDetails undocumented keys": `c`, `cbos` and `v` are on
/// every market, `ao`/`aot` on opened ones, `nr` on neg-risk ones, `sd` (equal to the listing's
/// `seconds_delay`, omitted when 0) on delayed ones; and the SDK models them.
#[tokio::test]
#[ignore = "live network"]
async fn pin_clob_market_details_undocumented_keys() {
    let s = sample().await;
    let raw = get(CLOB, &format!("/clob-markets/{}", s.condition_id), &[]).await;
    let object = raw.json.as_object().expect("an object");
    for key in ["c", "cbos", "v"] {
        assert!(object.contains_key(key), "missing `{key}`: {}", raw.text);
    }
    let info = clob()
        .get_clob_market_info(s.condition_id.as_str())
        .await
        .unwrap();
    assert_eq!(info.condition_id.as_ref().unwrap().as_str(), s.condition_id);
    assert_eq!(info.version.as_deref(), Some("v1"));
    assert!(info.cbos.is_some());

    // `sd` is omitted for the sample market when its delay is 0.
    assert_eq!(object.contains_key("sd"), info.seconds_delay.is_some());

    // Markets that carry the conditional keys, found from the sampling listing: an accepting
    // market (`ao`, `aot`) and a neg-risk one (`nr`).
    let listing = get(CLOB, "/sampling-markets", &[]).await;
    let markets = listing.json["data"].as_array().unwrap();
    let find = |predicate: &dyn Fn(&Value) -> bool| {
        markets
            .iter()
            .find(|m| predicate(m))
            .map(|m| m["condition_id"].as_str().unwrap().to_owned())
    };
    if let Some(id) = find(&|m| m["accepting_orders"] == true) {
        let info = clob().get_clob_market_info(id.as_str()).await.unwrap();
        assert_eq!(info.accepting_orders, Some(true), "{id}");
        assert!(info.accepting_order_timestamp.is_some(), "{id}");
    }
    if let Some(id) = find(&|m| m["neg_risk"] == true) {
        let info = clob().get_clob_market_info(id.as_str()).await.unwrap();
        assert_eq!(info.neg_risk, Some(true), "{id}");
    }

    // A market with a taker delay (`seconds_delay` 1 or 3, sports markets), a few pages in.
    let mut cursor: Option<String> = None;
    for _ in 0..4 {
        let mut query = Vec::new();
        if let Some(cursor) = &cursor {
            query.push(("next_cursor", cursor.as_str()));
        }
        let page = get(CLOB, "/sampling-markets", &query).await;
        if let Some(market) = page.json["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["seconds_delay"].as_u64().is_some_and(|d| d > 0))
        {
            let id = market["condition_id"].as_str().unwrap();
            let info = clob().get_clob_market_info(id).await.unwrap();
            assert_eq!(info.seconds_delay, market["seconds_delay"].as_u64(), "{id}");
            return;
        }
        cursor = page.json["next_cursor"].as_str().map(str::to_owned);
        if cursor.as_deref() == Some("LTE=") {
            break;
        }
    }
    eprintln!("INFO no market with a seconds_delay in the first sampling pages; `sd` not checked");
}

/// Decodes the clob-market info of several different markets (closed, neg-risk, with and
/// without rewards) taken from the simplified listing, to cover optional/nullable fields.
#[tokio::test]
#[ignore = "live network"]
async fn get_clob_market_info_varied_markets() {
    let page = get(CLOB, "/sampling-markets", &[]).await;
    let simplified = get(CLOB, "/simplified-markets", &[]).await;
    let mut ids: Vec<String> = Vec::new();
    for listing in [&page, &simplified] {
        for market in listing.json["data"]
            .as_array()
            .unwrap()
            .iter()
            .step_by(97)
            .take(4)
        {
            ids.push(market["condition_id"].as_str().unwrap().to_owned());
        }
    }
    // A neg-risk market, if the sampling page has one.
    if let Some(market) = page.json["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["neg_risk"] == true)
    {
        ids.push(market["condition_id"].as_str().unwrap().to_owned());
    }
    let mut failures = Vec::new();
    for id in &ids {
        let result = clob().get_clob_market_info(id.as_str()).await;
        if let Err(e) = &result {
            failures.push(format!("{id}: {e}"));
        }
        let raw = get(CLOB, &format!("/clob-markets/{id}"), &[]).await;
        // Drift is reported by `check`; a decode failure panics, so catch it per market.
        let outcome = std::panic::catch_unwind(|| {
            check::<ClobMarketDetails>("GET /clob-markets (varied)", &raw)
        });
        if let Err(e) = outcome {
            let message = e
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "panic".to_owned());
            failures.push(message);
        }
    }
    assert!(failures.is_empty(), "failures: {failures:#?}");
}

#[tokio::test]
#[ignore = "live network"]
async fn get_market_by_token() {
    let s = sample().await;
    let (a, b) = tokens().await;
    let market = clob().get_market_by_token(a.as_str()).await.unwrap();
    assert_eq!(market.condition_id.as_str(), s.condition_id);
    let other = clob().get_market_by_token(b.as_str()).await.unwrap();
    assert_eq!(other.condition_id, market.condition_id);
    check::<MarketByToken>(
        "GET /markets-by-token/{token_id}",
        &get(CLOB, &format!("/markets-by-token/{a}"), &[]).await,
    );
}

/// Pins SPEC_DEVIATIONS.md "markets-by-token primary token": `primary_token_id` and
/// `secondary_token_id` are the market's two tokens, but `primary_token_id` is not always the
/// first ("Yes") token, as the docs say.
#[tokio::test]
#[ignore = "live network"]
async fn pin_market_by_token_primary_is_not_always_yes() {
    let listing = get(CLOB, "/sampling-markets", &[]).await;
    let (mut first, mut second) = (0, 0);
    for market in listing.json["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["tokens"].as_array().is_some_and(|t| t.len() == 2))
        .step_by(37)
        .take(20)
    {
        let tokens = [
            market["tokens"][0]["token_id"].as_str().unwrap(),
            market["tokens"][1]["token_id"].as_str().unwrap(),
        ];
        let found = clob().get_market_by_token(tokens[0]).await.unwrap();
        let primary = found.primary_token_id.as_str();
        let secondary = found.secondary_token_id.as_str();
        assert!(
            (primary == tokens[0] && secondary == tokens[1])
                || (primary == tokens[1] && secondary == tokens[0]),
            "{market}: {found:?}"
        );
        // The same answer from either token of the market.
        let other = clob().get_market_by_token(tokens[1]).await.unwrap();
        assert_eq!(found, other);
        if primary == tokens[0] {
            first += 1;
        } else {
            second += 1;
        }
    }
    assert!(
        first > 0 && second > 0,
        "primary is now always the same token ({first} first, {second} second): the docs may be true"
    );
}

/// `POST /markets/live-activity`.
#[tokio::test]
#[ignore = "live network"]
async fn get_markets_live_activity() {
    let s = sample().await;
    let markets = clob()
        .get_markets_live_activity([s.condition_id.as_str()])
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(
        markets[0].condition_id.as_ref().unwrap().as_str(),
        s.condition_id
    );
    check::<Vec<LiveActivityMarket>>(
        "POST /markets/live-activity",
        &post(CLOB, "/markets/live-activity", &json!([s.condition_id])).await,
    );
}

/// `GET /markets/live-activity/{condition_id}`.
#[tokio::test]
#[ignore = "live network"]
async fn get_market_live_activity() {
    let s = sample().await;
    let market = clob()
        .get_market_live_activity(s.condition_id.as_str())
        .await
        .unwrap();
    assert_eq!(
        market.condition_id.as_ref().unwrap().as_str(),
        s.condition_id
    );
    check::<LiveActivityMarket>(
        "GET /markets/live-activity/{condition_id}",
        &get(
            CLOB,
            &format!("/markets/live-activity/{}", s.condition_id),
            &[],
        )
        .await,
    );
}

// ---- price history ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_prices_history() {
    let (a, _) = tokens().await;
    let history = clob()
        .get_prices_history(a.as_str())
        .interval(PriceHistoryInterval::OneDay)
        .fidelity(60)
        .send()
        .await
        .unwrap();
    let points = history.history.unwrap();
    assert!(!points.is_empty());
    assert!(
        points
            .iter()
            .all(|p| p.timestamp.is_some() && p.price.is_some())
    );
    check::<PricesHistory>(
        "GET /prices-history",
        &get(
            CLOB,
            "/prices-history",
            &[("market", &a), ("interval", "1d"), ("fidelity", "60")],
        )
        .await,
    );
}

/// Every [`PriceHistoryInterval`] variant (open question 16), with a fidelity the server
/// accepts for it. Collects failures so one bad interval does not hide the others.
#[tokio::test]
#[ignore = "live network"]
async fn get_prices_history_every_interval() {
    let (a, _) = tokens().await;
    let mut failures = Vec::new();
    for (interval, wire) in [
        (PriceHistoryInterval::OneHour, "1h"),
        (PriceHistoryInterval::SixHours, "6h"),
        (PriceHistoryInterval::OneDay, "1d"),
        (PriceHistoryInterval::OneWeek, "1w"),
        (PriceHistoryInterval::OneMonth, "1m"),
        (PriceHistoryInterval::Max, "max"),
        (PriceHistoryInterval::All, "all"),
    ] {
        let result = clob()
            .get_prices_history(a.as_str())
            .interval(interval)
            .fidelity(60)
            .send()
            .await;
        match result {
            Ok(history) => {
                let points = history.history.unwrap_or_default();
                let span = match (points.first(), points.last()) {
                    (Some(first), Some(last)) => last
                        .timestamp
                        .zip(first.timestamp)
                        .map(|(l, f)| (l - f).num_hours()),
                    _ => None,
                };
                eprintln!(
                    "INFO interval {wire}: {} points, span {span:?} h",
                    points.len()
                );
            }
            Err(e) => failures.push(format!("interval {wire} fidelity 60: {e}")),
        }
    }
    assert!(failures.is_empty(), "failures: {failures:#?}");
}

/// Pins SPEC_DEVIATIONS.md "prices-history interval and fidelity": `1m` is a month and needs
/// `fidelity >= 10`, `1w` needs `fidelity >= 5`, and `max` behaves as `all`.
#[tokio::test]
#[ignore = "live network"]
async fn pin_prices_history_interval_rules() {
    let (a, _) = tokens().await;
    let status = |interval: &'static str, fidelity: &'static str| {
        let a = a.clone();
        async move {
            raw_get(
                "/prices-history",
                &[
                    ("market", &a),
                    ("interval", interval),
                    ("fidelity", fidelity),
                ],
            )
            .await
        }
    };
    for (interval, fidelity, minimum) in [("1m", "9", "10"), ("1w", "4", "5")] {
        let response = status(interval, fidelity).await;
        assert_eq!(response.status, 400, "{interval}: {}", response.text);
        assert!(
            response.text.contains(&format!("is {minimum}")),
            "{interval}: {}",
            response.text
        );
    }
    assert_eq!(status("1m", "10").await.status, 200);
    assert_eq!(status("1w", "5").await.status, 200);

    // `1m` spans about a month; `max` and `all` return the same series.
    let month = clob()
        .get_prices_history(a.as_str())
        .interval(PriceHistoryInterval::OneMonth)
        .fidelity(60)
        .send()
        .await
        .unwrap()
        .history
        .unwrap_or_default();
    let span_days = match (month.first(), month.last()) {
        (Some(first), Some(last)) => {
            (last.timestamp.unwrap() - first.timestamp.unwrap()).num_days()
        }
        _ => 0,
    };
    assert!(
        (25..=31).contains(&span_days) || month.len() < 24 * 25,
        "1m spans {span_days} days"
    );
    let max = clob()
        .get_prices_history(a.as_str())
        .interval(PriceHistoryInterval::Max)
        .fidelity(60)
        .send()
        .await
        .unwrap()
        .history
        .unwrap_or_default();
    let all = clob()
        .get_prices_history(a.as_str())
        .interval(PriceHistoryInterval::All)
        .fidelity(60)
        .send()
        .await
        .unwrap()
        .history
        .unwrap_or_default();
    assert!(
        max.len().abs_diff(all.len()) <= 2,
        "{} vs {}",
        max.len(),
        all.len()
    );
}

/// Pins SPEC_DEVIATIONS.md "prices-history window": `startTs`/`endTs` are Unix seconds and
/// bound the series, except that the server appends one point at the current time even when
/// `endTs` is in the past.
#[tokio::test]
#[ignore = "live network"]
async fn pin_prices_history_appends_a_now_point() {
    let (a, _) = tokens().await;
    let end = Utc::now() - Duration::days(3);
    let start = end - Duration::days(2);
    let history = clob()
        .get_prices_history(a.as_str())
        .start_ts(start)
        .end_ts(end)
        .fidelity(10)
        .send()
        .await
        .unwrap();
    let points = history.history.unwrap_or_default();
    if points.len() < 2 {
        eprintln!("INFO the sample market has no history in the window; not checked");
        return;
    }
    let (last, rest) = points.split_last().unwrap();
    let outside = |t: DateTime<Utc>| t > end + Duration::hours(1);
    assert!(
        outside(last.timestamp.unwrap()),
        "no trailing point after endTs: the quirk is gone"
    );
    assert!(
        rest.iter().all(|p| !outside(p.timestamp.unwrap())),
        "more than one point after endTs"
    );
    assert!(
        rest.iter()
            .all(|p| p.timestamp.unwrap() >= start - Duration::hours(1)),
        "points before startTs"
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_prices_history_start_end() {
    let (a, _) = tokens().await;
    let end = Utc::now();
    let start = end - Duration::days(2);
    let history = clob()
        .get_prices_history(a.as_str())
        .start_ts(start)
        .end_ts(end)
        .fidelity(60)
        .send()
        .await
        .unwrap();
    let points = history.history.unwrap();
    assert!(!points.is_empty());
    for point in &points {
        let t = point.timestamp.unwrap();
        assert!(
            t >= start - Duration::hours(1) && t <= end + Duration::minutes(5),
            "{t}"
        );
    }
}

/// `POST /batch-prices-history`.
#[tokio::test]
#[ignore = "live network"]
async fn get_batch_prices_history() {
    let (a, b) = tokens().await;
    let batch = clob()
        .get_batch_prices_history([a.as_str(), b.as_str()])
        .interval(PriceHistoryInterval::OneDay)
        .fidelity(60)
        .send()
        .await
        .unwrap();
    let history = batch.history.unwrap();
    assert_eq!(history.len(), 2);
    assert!(history.values().all(|points| !points.is_empty()));
    let body = json!({ "markets": [a, b], "interval": "1d", "fidelity": 60 });
    check::<BatchPricesHistory>(
        "POST /batch-prices-history",
        &post(CLOB, "/batch-prices-history", &body).await,
    );
}

/// The batch endpoint with explicit start/end timestamps instead of an interval.
#[tokio::test]
#[ignore = "live network"]
async fn get_batch_prices_history_start_end() {
    let (a, b) = tokens().await;
    let end = Utc::now();
    let batch = clob()
        .get_batch_prices_history([a.as_str(), b.as_str()])
        .start_ts(end - Duration::days(2))
        .end_ts(end)
        .fidelity(60)
        .send()
        .await
        .unwrap();
    assert_eq!(batch.history.unwrap().len(), 2);
}

// ---- rewards ---------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_current_rewards() {
    let page = clob().list_current_rewards().send().await.unwrap();
    assert!(!page.items().is_empty());
    assert_eq!(page.count as usize, page.items().len());
    let raw = get(CLOB, "/rewards/markets/current", &[]).await;
    eprintln!(
        "INFO /rewards/markets/current: next_cursor {} limit {} count {}",
        raw.json["next_cursor"], raw.json["limit"], raw.json["count"]
    );
    check::<Page<CurrentReward>>("GET /rewards/markets/current", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_current_rewards_sponsored() {
    let page = clob()
        .list_current_rewards()
        .sponsored(true)
        .send()
        .await
        .unwrap();
    eprintln!("INFO sponsored rewards: {} items", page.items().len());
    check::<Page<CurrentReward>>(
        "GET /rewards/markets/current?sponsored=true",
        &get(CLOB, "/rewards/markets/current", &[("sponsored", "true")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_current_rewards_stream_pages() {
    let first = clob().list_current_rewards().send().await.unwrap();
    let wanted = first.items().len() + 3;
    let has_more = first.next_cursor().is_some();
    let items: Vec<CurrentReward> = clob()
        .list_current_rewards()
        .into_stream()
        .take(wanted)
        .try_collect()
        .await
        .unwrap();
    if has_more {
        assert_eq!(items.len(), wanted);
    }
}

/// A market that has a rewards configuration, found from the live listing.
async fn rewards_condition_id() -> String {
    let raw = get(CLOB, "/rewards/markets/current", &[]).await;
    raw.json["data"][0]["condition_id"]
        .as_str()
        .expect("a market with rewards")
        .to_owned()
}

#[tokio::test]
#[ignore = "live network"]
async fn list_raw_rewards_for_market() {
    let condition_id = rewards_condition_id().await;
    let page = clob()
        .list_raw_rewards_for_market(condition_id.as_str())
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    assert_eq!(page.items()[0].condition_id.as_str(), condition_id);
    check::<Page<MarketReward>>(
        "GET /rewards/markets/{condition_id}",
        &get(CLOB, &format!("/rewards/markets/{condition_id}"), &[]).await,
    );
    let streamed: Vec<MarketReward> = clob()
        .list_raw_rewards_for_market(condition_id.as_str())
        .sponsored(true)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    eprintln!(
        "INFO raw rewards (sponsored stream): {} items",
        streamed.len()
    );
}

/// The sample market (an NFL game; likely to have rewards).
#[tokio::test]
#[ignore = "live network"]
async fn list_raw_rewards_for_sample_market() {
    let s = sample().await;
    let page = clob()
        .list_raw_rewards_for_market(s.condition_id.as_str())
        .send()
        .await
        .unwrap();
    eprintln!(
        "INFO raw rewards of the sample market: {} items",
        page.items().len()
    );
    check::<Page<MarketReward>>(
        "GET /rewards/markets/{condition_id} (sample)",
        &get(CLOB, &format!("/rewards/markets/{}", s.condition_id), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_with_rewards() {
    let page = clob()
        .list_markets_with_rewards()
        .page_size(50)
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    let raw = get(CLOB, "/rewards/markets/multi", &[("page_size", "50")]).await;
    eprintln!(
        "INFO /rewards/markets/multi: next_cursor {} limit {} count {}",
        raw.json["next_cursor"], raw.json["limit"], raw.json["count"]
    );
    check::<Page<MultiMarketInfo>>("GET /rewards/markets/multi", &raw);
}

/// Filters, sorting and search on the multi-markets listing.
#[tokio::test]
#[ignore = "live network"]
async fn list_markets_with_rewards_filtered() {
    let page = clob()
        .list_markets_with_rewards()
        .tag_slugs(["sports"])
        .order_by(RewardsMarketsOrderBy::Volume24hr)
        .position(SortDirection::Desc)
        .min_volume_24hr(Decimal::ONE)
        .page_size(20)
        .send()
        .await
        .unwrap();
    eprintln!("INFO filtered multi-markets: {} items", page.items().len());
    let raw = get(
        CLOB,
        "/rewards/markets/multi",
        &[
            ("tag_slugs", "sports"),
            ("order_by", "volume_24hr"),
            ("position", "DESC"),
            ("min_volume_24hr", "1"),
            ("page_size", "20"),
        ],
    )
    .await;
    check::<Page<MultiMarketInfo>>("GET /rewards/markets/multi (filtered)", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_with_rewards_search() {
    let page = clob()
        .list_markets_with_rewards()
        .q("will")
        .page_size(10)
        .send()
        .await
        .unwrap();
    eprintln!("INFO q=will multi-markets: {} items", page.items().len());
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_with_rewards_stream_pages() {
    let items: Vec<MultiMarketInfo> = clob()
        .list_markets_with_rewards()
        .page_size(2)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(items.len(), 5);
}

// ---- rebates ---------------------------------------------------------------------------------

/// Finds `(maker, date)` pairs from recent builder trades, which are the only public source of
/// maker addresses; the maker rebate program pays for some of them.
async fn rebate_candidates() -> Vec<(String, NaiveDate)> {
    let raw = get(CLOB, "/builder/trades", &[("builder_code", BUILDER_CODE)]).await;
    let mut seen = Vec::new();
    for trade in raw.json["data"].as_array().unwrap() {
        let maker = trade["maker"].as_str().unwrap().to_owned();
        let created: DateTime<Utc> = trade["createdAt"].as_str().unwrap().parse().unwrap();
        let pair = (maker, created.date_naive());
        if !seen.contains(&pair) {
            seen.push(pair);
        }
    }
    seen
}

#[tokio::test]
#[ignore = "live network"]
async fn get_current_rebated_fees() {
    let mut found = None;
    for (maker, date) in rebate_candidates().await.into_iter().take(8) {
        let raw = get(
            CLOB,
            "/rebates/current",
            &[("date", &date.to_string()), ("maker_address", &maker)],
        )
        .await;
        if raw.json.as_array().is_some_and(|a| !a.is_empty()) {
            found = Some((maker, date, raw));
            break;
        }
    }
    let (maker, date, raw) = found.expect("no maker with rebates among recent builder trades");
    eprintln!(
        "INFO rebates sample: {}",
        raw.text.chars().take(300).collect::<String>()
    );
    let fees = clob()
        .get_current_rebated_fees(date, maker.as_str())
        .await
        .unwrap();
    assert!(!fees.is_empty());
    assert!(fees.iter().all(|f| f.date == date));
    check::<Vec<RebatedFees>>("GET /rebates/current", &raw);
}

/// Pins SPEC_DEVIATIONS.md "GET /rebates/current": `date` is an RFC 3339 date-time (not a
/// plain date) and a maker without rebates gets the body `null` (not `[]`).
#[tokio::test]
#[ignore = "live network"]
async fn pin_rebates_shape() {
    let none = get(
        CLOB,
        "/rebates/current",
        &[
            ("date", "2026-01-01"),
            (
                "maker_address",
                "0x0000000000000000000000000000000000000001",
            ),
        ],
    )
    .await;
    assert_eq!(none.text.trim(), "null");

    for (maker, date) in rebate_candidates().await.into_iter().take(8) {
        let raw = get(
            CLOB,
            "/rebates/current",
            &[("date", &date.to_string()), ("maker_address", &maker)],
        )
        .await;
        let Some(items) = raw.json.as_array().filter(|a| !a.is_empty()) else {
            continue;
        };
        for item in items {
            let wire = item["date"].as_str().unwrap();
            assert!(
                wire.len() > 10 && wire.contains('T'),
                "date is no longer a date-time: {wire}"
            );
            assert!(wire.starts_with(&date.to_string()), "{wire} vs {date}");
        }
        return;
    }
    panic!("no maker with rebates among recent builder trades");
}

/// A maker without rebates on the date: the live API answers `null` (not `[]`).
#[tokio::test]
#[ignore = "live network"]
async fn get_current_rebated_fees_none() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let maker = "0x0000000000000000000000000000000000000001";
    let raw = get(
        CLOB,
        "/rebates/current",
        &[("date", "2026-01-01"), ("maker_address", maker)],
    )
    .await;
    eprintln!("INFO rebates with none: body {:?}", raw.text);
    let fees = clob().get_current_rebated_fees(date, maker).await.unwrap();
    assert!(fees.is_empty());
}

// ---- builder trades --------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_builder_trades() {
    let page = clob()
        .list_builder_trades(BUILDER_CODE)
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    let raw = get(CLOB, "/builder/trades", &[("builder_code", BUILDER_CODE)]).await;
    eprintln!(
        "INFO /builder/trades: next_cursor {} limit {} count {}",
        raw.json["next_cursor"], raw.json["limit"], raw.json["count"]
    );
    check::<Page<BuilderTrade>>("GET /builder/trades", &raw);
}

/// Pins SPEC_DEVIATIONS.md "BuilderTrade fields": `builder` is empty and the code is in
/// `builderCode`; `builderFee` is sent; amounts are decimal units (`sizeUsdc = size * price`),
/// not micro-units.
#[tokio::test]
#[ignore = "live network"]
async fn pin_builder_trade_fields() {
    let raw = get(CLOB, "/builder/trades", &[("builder_code", BUILDER_CODE)]).await;
    let trades = raw.json["data"].as_array().unwrap();
    assert!(!trades.is_empty());
    for trade in trades {
        assert_eq!(trade["builder"], "", "{trade}");
        assert_eq!(trade["builderCode"], BUILDER_CODE, "{trade}");
        assert!(trade["builderFee"].is_string(), "{trade}");
    }
    let page = clob()
        .list_builder_trades(BUILDER_CODE)
        .send()
        .await
        .unwrap();
    for trade in page.items() {
        assert!(trade.builder_fee.is_some());
        assert_eq!(
            trade.builder_code.as_ref().map(|c| c.as_str()),
            Some(BUILDER_CODE)
        );
        // Decimal units: the USDC size is the share size times the price (to 6 places).
        let expected = (trade.size * trade.price).round_dp(6);
        assert!(
            (trade.size_usdc - expected).abs() <= Decimal::new(1, 5),
            "{trade:?}"
        );
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn list_builder_trades_filters() {
    let raw = get(CLOB, "/builder/trades", &[("builder_code", BUILDER_CODE)]).await;
    let first = &raw.json["data"][0];
    let market = first["market"].as_str().unwrap().to_owned();
    let asset = first["assetId"].as_str().unwrap().to_owned();
    let page = clob()
        .list_builder_trades(BUILDER_CODE)
        .market(market.as_str())
        .asset_id(asset.as_str())
        .after(Utc::now() - Duration::days(60))
        .before(Utc::now())
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    assert!(page.items().iter().all(|t| t.market.as_str() == market));
}

#[tokio::test]
#[ignore = "live network"]
async fn list_builder_trades_stream_pages() {
    let first = clob()
        .list_builder_trades(BUILDER_CODE)
        .send()
        .await
        .unwrap();
    let wanted = first.items().len() + 3;
    let has_more = first.next_cursor().is_some();
    eprintln!(
        "INFO builder trades first page: {} items, more: {has_more}",
        first.items().len()
    );
    let items: Vec<BuilderTrade> = clob()
        .list_builder_trades(BUILDER_CODE)
        .into_stream()
        .take(wanted)
        .try_collect()
        .await
        .unwrap();
    if has_more {
        assert_eq!(items.len(), wanted);
    }
}

/// Pins SPEC_DEVIATIONS.md "`LTE=` end cursor": every cursor-paged route ends with
/// `next_cursor: "LTE="` (a past-the-end cursor gives an empty page ending with it), and `LTE=`
/// itself is rejected as a request cursor.
#[tokio::test]
#[ignore = "live network"]
async fn pin_end_cursor_is_lte() {
    // base64 of "100000000" (an offset cursor) and of "id:999999999" (a market-id cursor).
    let offset = "MTAwMDAwMDAw";
    let id = "aWQ6OTk5OTk5OTk5";
    let cases: [(&str, Vec<(&str, &str)>); 6] = [
        ("/rewards/markets/current", vec![("next_cursor", offset)]),
        ("/rewards/markets/multi", vec![("next_cursor", offset)]),
        (
            "/builder/trades",
            vec![("builder_code", BUILDER_CODE), ("next_cursor", offset)],
        ),
        ("/simplified-markets", vec![("next_cursor", id)]),
        ("/sampling-markets", vec![("next_cursor", id)]),
        ("/sampling-simplified-markets", vec![("next_cursor", id)]),
    ];
    for (path, query) in cases {
        let raw = get(CLOB, path, &query).await;
        assert_eq!(raw.json["next_cursor"], "LTE=", "{path}: {}", raw.text);
        assert_eq!(raw.json["count"], 0, "{path}: {}", raw.text);
    }
    let response = raw_get("/simplified-markets", &[("next_cursor", "LTE=")]).await;
    assert_eq!(response.status, 400, "{}", response.text);
}

// ---- error shapes ----------------------------------------------------------------------------

/// An unknown token id is an API error (documented as `404` for `markets-by-token`).
#[tokio::test]
#[ignore = "live network"]
async fn unknown_token_is_api_error() {
    let bad = "1";
    let error = clob().get_market_by_token(bad).await.unwrap_err();
    eprintln!("INFO markets-by-token unknown token: {error:?}");
    assert!(matches!(error, polyoxide::Error::Api(_)), "{error:?}");
    let error = clob().get_order_book(bad).await.unwrap_err();
    eprintln!("INFO book unknown token: {error:?}");
    assert!(matches!(error, polyoxide::Error::Api(_)), "{error:?}");
}
