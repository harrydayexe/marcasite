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
    assert!(!book.timestamp.is_empty());
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
    assert!(midpoint.mid_price >= Decimal::ZERO && midpoint.mid_price <= Decimal::ONE);
    check::<Midpoint>(
        "GET /midpoint",
        &get(CLOB, "/midpoint", &[("token_id", &a)]).await,
    );
}

/// `GET /midpoints?token_ids=a,b` (spec operation `getMidpointsGet`).
#[tokio::test]
#[ignore = "live network"]
async fn get_midpoints_query() {
    let (a, b) = tokens().await;
    let midpoints = clob()
        .get_midpoints([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(midpoints.len(), 2);
    let joined = format!("{a},{b}");
    check::<HashMap<TokenId, Decimal>>(
        "GET /midpoints",
        &get(CLOB, "/midpoints", &[("token_ids", &joined)]).await,
    );
}

/// `POST /midpoints` (spec operation `getMidpointsPost`).
#[tokio::test]
#[ignore = "live network"]
async fn get_midpoints_by_body() {
    let (a, b) = tokens().await;
    let midpoints = clob()
        .get_midpoints_by_body([a.as_str(), b.as_str()])
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

/// `GET /last-trades-prices?token_ids=a,b`.
#[tokio::test]
#[ignore = "live network"]
async fn get_last_trade_prices_query() {
    let (a, b) = tokens().await;
    let prices = clob()
        .get_last_trade_prices([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(prices.len(), 2);
    let joined = format!("{a},{b}");
    check::<Vec<TokenLastTradePrice>>(
        "GET /last-trades-prices",
        &get(CLOB, "/last-trades-prices", &[("token_ids", &joined)]).await,
    );
}

/// `POST /last-trades-prices`.
#[tokio::test]
#[ignore = "live network"]
async fn get_last_trade_prices_by_body() {
    let (a, b) = tokens().await;
    let prices = clob()
        .get_last_trade_prices_by_body([a.as_str(), b.as_str()])
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

/// `GET /prices?token_ids=a,b&sides=BUY,SELL`.
#[tokio::test]
#[ignore = "live network"]
async fn get_prices_query() {
    let (a, b) = tokens().await;
    let prices = clob().get_prices(book_requests(&a, &b)).await.unwrap();
    assert_eq!(prices.len(), 2);
    let joined = format!("{a},{b}");
    check::<HashMap<TokenId, HashMap<Side, Decimal>>>(
        "GET /prices",
        &get(
            CLOB,
            "/prices",
            &[("token_ids", &joined), ("sides", "BUY,SELL")],
        )
        .await,
    );
}

/// `POST /prices`.
#[tokio::test]
#[ignore = "live network"]
async fn get_prices_by_body() {
    let (a, b) = tokens().await;
    let prices = clob()
        .get_prices_by_body(book_requests(&a, &b))
        .await
        .unwrap();
    assert_eq!(prices.len(), 2);
    check::<HashMap<TokenId, HashMap<Side, Decimal>>>(
        "POST /prices",
        &post(CLOB, "/prices", &book_requests_body().await).await,
    );
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
    // Open question 14: the unit of the `timestamp` string.
    eprintln!("INFO GET /book timestamp = {}", raw.json["timestamp"]);
    check::<OrderBookSummary>("GET /book", &raw);
}

/// `GET /books?token_ids=a,b`.
#[tokio::test]
#[ignore = "live network"]
async fn get_order_books_query() {
    let s = sample().await;
    let (a, b) = tokens().await;
    let books = clob()
        .get_order_books([a.as_str(), b.as_str()])
        .await
        .unwrap();
    assert_eq!(books.len(), 2);
    assert_book(&books[0], &a, &s.condition_id);
    let joined = format!("{a},{b}");
    check::<Vec<OrderBookSummary>>(
        "GET /books",
        &get(CLOB, "/books", &[("token_ids", &joined)]).await,
    );
}

/// `POST /books`.
#[tokio::test]
#[ignore = "live network"]
async fn get_order_books_by_body() {
    let s = sample().await;
    let (a, b) = tokens().await;
    let books = clob()
        .get_order_books_by_body([a.as_str(), b.as_str()])
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
    check::<Vec<RebatedFees>>("GET /rebates/current", &raw);
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
