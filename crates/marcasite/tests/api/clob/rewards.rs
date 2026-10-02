//! Rewards configurations: current, per market, multi-market.

use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{
    Decimal, Error, StatusCode,
    clob::{END_CURSOR, MAX_REWARDS_MARKETS_PAGE_SIZE, RewardsMarketsOrderBy, SortDirection},
    types::ConditionId,
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path, query_param, query_param_is_missing},
};

use super::{api_error, clob, json};
use crate::common;

const CONDITION_ID: &str = "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af";

/// A `PaginatedCurrentReward` page based on the `GET /rewards/markets/current` example in
/// docs/specs/clob-openapi.yaml.
fn current_page(condition_ids: &[&str], next_cursor: &str) -> String {
    let data: Vec<_> = condition_ids
        .iter()
        .map(|id| {
            json!({
                "condition_id": id,
                "rewards_max_spread": 99,
                "rewards_min_size": 10,
                "rewards_config": [{
                    "id": 0,
                    "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78",
                    "start_date": "2024-03-01",
                    "end_date": "2500-12-31",
                    "rate_per_day": 2,
                    "total_rewards": 92
                }],
                "sponsored_daily_rate": 0.5,
                "sponsors_count": 2,
                "native_daily_rate": 2.5,
                "total_daily_rate": 3.0
            })
        })
        .collect();
    json!({"limit": 500, "count": condition_ids.len(), "next_cursor": next_cursor, "data": data})
        .to_string()
}

#[tokio::test]
async fn current_rewards_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/current"))
        .and(query_param("sponsored", "true"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&current_page(&[CONDITION_ID], "LTE=")))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_current_rewards()
        .sponsored(true)
        .send()
        .await
        .unwrap();
    assert_eq!(page.limit, 500);
    assert!(page.is_last_page());
    assert_eq!(page.data[0].total_daily_rate, Some(Decimal::new(3, 0)));
}

#[tokio::test]
async fn current_rewards_stream_walks_cursors_until_lte() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/current"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&current_page(&["0x01", "0x02"], "NTAw")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/current"))
        .and(query_param("next_cursor", "NTAw"))
        .respond_with(json(&current_page(&["0x03"], "LTE=")))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = clob(&server)
        .list_current_rewards()
        .into_stream()
        .map_ok(|r| r.condition_id)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids, ["0x01", "0x02", "0x03"].map(ConditionId::from));
}

#[tokio::test]
async fn invalid_cursor_is_an_api_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/current"))
        .and(query_param("next_cursor", "nope"))
        .respond_with(api_error(400, "Invalid next_cursor"))
        .expect(1)
        .mount(&server)
        .await;

    let err = clob(&server)
        .list_current_rewards()
        .cursor("nope")
        .send()
        .await
        .unwrap_err();
    assert_eq!(
        err.api_error().unwrap().message(),
        Some("Invalid next_cursor")
    );
}

#[tokio::test]
async fn raw_rewards_for_market() {
    let server = common::server().await;
    // Example response of `GET /rewards/markets/{condition_id}` in
    // docs/specs/clob-openapi.yaml.
    let body = json!({
        "limit": 100,
        "count": 1,
        "next_cursor": "LTE=",
        "data": [{
            "condition_id": CONDITION_ID,
            "question": "Will Trump win the 2024 Iowa Caucus?",
            "market_slug": "will-trump-win-the-2024-iowa-caucus",
            "event_slug": "will-trump-win-the-2024-iowa-caucus",
            "image": "https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png",
            "rewards_max_spread": 99,
            "rewards_min_size": 10,
            "market_competitiveness": 0.42,
            "tokens": [
                {"token_id": "1343197538147866997676250008839231694243646439454152539053893078719042421992", "outcome": "YES", "price": 0.8},
                {"token_id": "16678291189211314787145083999015737376658799626183230671758641503291735614088", "outcome": "NO", "price": 0.2}
            ],
            "rewards_config": [
                {"id": 1, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 0.25, "total_rewards": 0, "total_days": 174161}
            ]
        }]
    });
    Mock::given(method("GET"))
        .and(path(format!("/rewards/markets/{CONDITION_ID}")))
        .and(query_param("sponsored", "false"))
        .and(query_param("next_cursor", "MTAw"))
        .respond_with(json(&body.to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_raw_rewards_for_market(CONDITION_ID)
        .sponsored(false)
        .cursor("MTAw")
        .send()
        .await
        .unwrap();
    assert!(page.is_last_page());
    let market = &page.items()[0];
    assert_eq!(market.tokens.len(), 2);
    assert_eq!(
        market.rewards_config.as_ref().unwrap()[0].total_days,
        Some(174_161)
    );
}

#[tokio::test]
async fn raw_rewards_for_market_stream() {
    let server = common::server().await;
    let body = json!({
        "limit": 100,
        "count": 1,
        "next_cursor": "LTE=",
        "data": [{"condition_id": CONDITION_ID, "question": "Q?", "tokens": []}]
    });
    Mock::given(method("GET"))
        .and(path(format!("/rewards/markets/{CONDITION_ID}")))
        .respond_with(json(&body.to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let markets: Vec<_> = clob(&server)
        .list_raw_rewards_for_market(CONDITION_ID)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].question, "Q?");
}

/// Example response of `GET /rewards/markets/multi` in docs/specs/clob-openapi.yaml.
const MULTI: &str = r#"{
    "limit": 50,
    "count": 1,
    "next_cursor": "NQ==",
    "data": [{
        "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
        "event_id": "12345",
        "event_slug": "2024-us-election",
        "created_at": "2024-05-01T12:00:00Z",
        "group_item_title": "",
        "image": "https://example.com/image.png",
        "market_competitiveness": 0.42,
        "market_id": "248849",
        "market_slug": "will-trump-win-the-2024-iowa-caucus",
        "one_day_price_change": 0.03,
        "question": "Will Trump win the 2024 Iowa Caucus?",
        "rewards_max_spread": 99,
        "rewards_min_size": 10,
        "spread": 0.12,
        "end_date": "2025-05-01 12:00:00+00",
        "tokens": [
            {"token_id": "1343197538147866997676250008839231694243646439454152539053893078719042421992", "outcome": "YES", "price": 0.8},
            {"token_id": "16678291189211314787145083999015737376658799626183230671758641503291735614088", "outcome": "NO", "price": 0.2}
        ],
        "volume_24hr": 12345.67,
        "rewards_config": [
            {"id": 7, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 2, "total_rewards": 92}
        ]
    }]
}"#;

#[tokio::test]
async fn markets_with_rewards_sends_every_filter() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/multi"))
        .and(query_param("q", "trump"))
        .and(query_param("tag_slug", "sports"))
        .and(query_param("tag_slug", "politics"))
        .and(query_param("event_id", "100"))
        .and(query_param("event_id", "200"))
        .and(query_param("event_title", "election"))
        .and(query_param("order_by", "volume_24hr"))
        .and(query_param("position", "DESC"))
        .and(query_param("min_volume_24hr", "1000"))
        .and(query_param("max_volume_24hr", "50000.5"))
        .and(query_param("min_spread", "0.01"))
        .and(query_param("max_spread", "0.2"))
        .and(query_param("min_price", "0.1"))
        .and(query_param("max_price", "0.9"))
        .and(query_param("next_cursor", "MA=="))
        .and(query_param("page_size", "500"))
        .respond_with(json(MULTI))
        .expect(1)
        .mount(&server)
        .await;

    let d = |s: &str| -> Decimal { s.parse().unwrap() };
    let page = clob(&server)
        .list_markets_with_rewards()
        .q("trump")
        .tag_slugs(["sports", "politics"])
        .event_ids(["100", "200"])
        .event_title("election")
        .order_by(RewardsMarketsOrderBy::Volume24hr)
        .position(SortDirection::Desc)
        .min_volume_24hr(d("1000"))
        .max_volume_24hr(d("50000.5"))
        .min_spread(d("0.01"))
        .max_spread(d("0.2"))
        .min_price(d("0.1"))
        .max_price(d("0.9"))
        .cursor("MA==")
        .page_size(MAX_REWARDS_MARKETS_PAGE_SIZE)
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor(), Some("NQ=="));
    let market = &page.items()[0];
    assert_eq!(market.market_id, "248849");
    assert_eq!(market.spread, Some(d("0.12")));
}

#[tokio::test]
async fn markets_with_rewards_page_size_limit() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(MULTI))
        .expect(0)
        .mount(&server)
        .await;

    let err = clob(&server)
        .list_markets_with_rewards()
        .page_size(MAX_REWARDS_MARKETS_PAGE_SIZE + 1)
        .send()
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "page_size");
}

/// A `PaginatedMultiMarketInfo` page with one market per id, based on the
/// `GET /rewards/markets/multi` example in docs/specs/clob-openapi.yaml.
fn multi_page(market_ids: &[&str], next_cursor: &str) -> String {
    let mut page: serde_json::Value = serde_json::from_str(MULTI).unwrap();
    let template = page["data"][0].clone();
    let data: Vec<_> = market_ids
        .iter()
        .map(|id| {
            let mut market = template.clone();
            market["market_id"] = (*id).into();
            market
        })
        .collect();
    page["count"] = data.len().into();
    page["data"] = data.into();
    page["next_cursor"] = next_cursor.into();
    page.to_string()
}

#[tokio::test]
async fn markets_with_rewards_stream_walks_cursors_until_lte() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/multi"))
        .and(query_param("tag_slug", "politics"))
        .and(query_param("page_size", "2"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&multi_page(&["1", "2"], "Mg==")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/multi"))
        .and(query_param("tag_slug", "politics"))
        .and(query_param("page_size", "2"))
        .and(query_param("next_cursor", "Mg=="))
        .respond_with(json(&multi_page(&["3"], END_CURSOR)))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<String> = clob(&server)
        .list_markets_with_rewards()
        .tag_slugs(["politics"])
        .page_size(2)
        .into_stream()
        .map_ok(|m| m.market_id.into_inner())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids, ["1", "2", "3"]);
}

#[tokio::test]
async fn markets_with_rewards_stream_stops_at_first_error() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/multi"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&multi_page(&["1"], "Mg==")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/multi"))
        .and(query_param("next_cursor", "Mg=="))
        .respond_with(api_error(400, "Invalid next_cursor"))
        .expect(1)
        .mount(&server)
        .await;

    let results: Vec<_> = clob(&server)
        .list_markets_with_rewards()
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    let err = results[1].as_ref().unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::BAD_REQUEST));
}

#[tokio::test]
async fn reward_streams_resumed_at_end_cursor_send_nothing() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(MULTI))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let markets: Vec<_> = client
        .list_markets_with_rewards()
        .cursor(END_CURSOR)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert!(markets.is_empty());
    let rewards: Vec<_> = client
        .list_current_rewards()
        .cursor(END_CURSOR)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert!(rewards.is_empty());
}

#[tokio::test]
async fn raw_rewards_rejects_ids_that_are_not_condition_ids() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(MULTI))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    // An empty id is documented as `400 Invalid market`; `current` and `multi` would reach
    // the other `/rewards/markets/...` endpoints.
    for id in ["", "current", "multi"] {
        let err = client
            .list_raw_rewards_for_market(id)
            .send()
            .await
            .unwrap_err();
        let Error::Validation(v) = &err else {
            panic!("expected Error::Validation for {id:?}, got {err:?}")
        };
        assert_eq!(v.parameter(), "condition_id");
    }
}

/// `429` with the documented `retry_after_seconds` field and a `Retry-After` header: the
/// header wins.
#[tokio::test]
async fn rate_limited_listing() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/rewards/markets/current"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "7")
                .set_body_json(json!({
                    "error": "rate limited",
                    "code": "TOO_MANY_REQUESTS",
                    "retry_after_seconds": 3
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = clob(&server)
        .list_current_rewards()
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(7)));
    assert_eq!(err.api_error().unwrap().code(), Some("TOO_MANY_REQUESTS"));
}
