//! Markets: simplified/sampling listings, CLOB market info, market by token, live activity.

use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{Error, clob::END_CURSOR, types::ConditionId};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, body_json, method, path, query_param, query_param_is_missing},
};

use super::{api_error, clob, json, retrying_clob};
use crate::common;

const CONDITION_ID: &str = "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af";

/// A `PaginatedSimplifiedMarkets` page (fields from the schema in
/// docs/polymarket/specs/clob-openapi.yaml; the spec has no example).
fn simplified_page(ids: &[&str], next_cursor: &str) -> String {
    let data: Vec<_> = ids
        .iter()
        .map(|id| {
            json!({
                "condition_id": id,
                "rewards": {"rates": [], "min_size": 10, "max_spread": 99},
                "tokens": [{"token_id": "1", "outcome": "Yes", "price": 0.5, "winner": false}],
                "active": true,
                "closed": false,
                "archived": false,
                "accepting_orders": true
            })
        })
        .collect();
    json!({"limit": 2, "count": ids.len(), "next_cursor": next_cursor, "data": data}).to_string()
}

#[tokio::test]
async fn simplified_markets_first_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/simplified-markets"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&simplified_page(&["0x01", "0x02"], "Mg==")))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_simplified_markets()
        .send()
        .await
        .unwrap();
    assert_eq!(page.count, Some(2));
    assert_eq!(page.next_cursor(), Some("Mg=="));
    assert_eq!(page.items().len(), 2);
    let data = page.into_items();
    assert_eq!(data[1].condition_id, Some(ConditionId::from("0x02")));
    assert_eq!(data[0].accepting_orders, Some(true));
}

/// Items of `GET /simplified-markets` as sent live (captured 2026-10-02, trimmed): a closed
/// market has `rewards.rates: null`, an active one has rates; the page `limit` is 1000 and the
/// cursor is an opaque base64 string.
#[tokio::test]
async fn simplified_markets_live_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/simplified-markets"))
        .respond_with(json(
            r#"{
                "data": [
                    {"condition_id": "0xb62f71aef4c9972ddb75ec977b48156e1cc72e30f0e36b0f12c14de7fd94c777",
                     "rewards": {"rates": null, "min_size": 0, "max_spread": 0},
                     "tokens": [
                        {"token_id": "4764842916462157287738418815193165769775706842468360015637456523413749678364", "outcome": "Yes", "price": 0, "winner": false},
                        {"token_id": "60746995446126444881122884468632478444639071490773182750920065820822712887585", "outcome": "No", "price": 1, "winner": false}],
                     "active": true, "closed": true, "archived": false, "accepting_orders": false},
                    {"condition_id": "0x26ee82bee2493a302d21283cb578f7e2fff2dd15743854f53034d12420863b55",
                     "rewards": {"rates": [{"asset_address": "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", "rewards_daily_rate": 5}], "min_size": 200, "max_spread": 1.5},
                     "tokens": [
                        {"token_id": "11015470973684177829729219287262166995141465048508201953575582100565462316088", "outcome": "Democratic", "price": 0, "winner": false},
                        {"token_id": "65444287174436666395099524416802980027579283433860283898747701594488689243696", "outcome": "Republican", "price": 1, "winner": true}],
                     "active": true, "closed": true, "archived": false, "accepting_orders": false}
                ],
                "next_cursor": "aWQ6MjQ5Mzk2",
                "limit": 1000,
                "count": 2
            }"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_simplified_markets()
        .send()
        .await
        .unwrap();
    assert_eq!(page.limit, Some(1000));
    assert_eq!(page.next_cursor(), Some("aWQ6MjQ5Mzk2"));
    let items = page.items();
    assert_eq!(items[0].rewards.as_ref().unwrap().rates, None);
    assert_eq!(
        items[1].rewards.as_ref().unwrap().rates.as_ref().unwrap()[0].rewards_daily_rate,
        Some(5.into())
    );
    assert_eq!(items[1].tokens.as_ref().unwrap()[1].winner, Some(true));
}

#[tokio::test]
async fn simplified_markets_stream_stops_at_end_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/simplified-markets"))
        .and(query_param_is_missing("next_cursor"))
        .respond_with(json(&simplified_page(&["0x01", "0x02"], "Mg==")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/simplified-markets"))
        .and(query_param("next_cursor", "Mg=="))
        .respond_with(json(&simplified_page(&["0x03"], END_CURSOR)))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = clob(&server)
        .list_simplified_markets()
        .into_stream()
        .map_ok(|m| m.condition_id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids, ["0x01", "0x02", "0x03"].map(ConditionId::from));
}

#[tokio::test]
async fn sampling_simplified_markets_with_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/sampling-simplified-markets"))
        .and(query_param("next_cursor", "MTAw"))
        .respond_with(json(&simplified_page(&["0x01"], END_CURSOR)))
        .expect(1)
        .mount(&server)
        .await;

    let page = clob(&server)
        .list_sampling_simplified_markets()
        .cursor("MTAw")
        .send()
        .await
        .unwrap();
    assert!(page.is_last_page());
    assert_eq!(page.next_cursor(), None);
    // The raw wire value stays available.
    assert_eq!(page.next_cursor.as_deref(), Some(END_CURSOR));
}

#[tokio::test]
async fn stream_resumed_at_end_cursor_sends_nothing() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json(&simplified_page(&["0x01"], END_CURSOR)))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    for cursor in [END_CURSOR, ""] {
        let markets: Vec<_> = client
            .list_sampling_simplified_markets()
            .cursor(cursor)
            .into_stream()
            .try_collect()
            .await
            .unwrap();
        assert!(markets.is_empty(), "{cursor:?}");
    }
}

#[tokio::test]
async fn sampling_markets_stream() {
    let server = common::server().await;
    // Fields from `components/schemas/Market` (the spec has no example).
    let body = json!({
        "limit": 1,
        "count": 1,
        "next_cursor": END_CURSOR,
        "data": [{
            "enable_order_book": true,
            "condition_id": CONDITION_ID,
            "question": "Will Trump win the 2024 Iowa Caucus?",
            "minimum_tick_size": 0.01,
            "end_date_iso": "2024-08-10T00:00:00Z",
            "tokens": [{"token_id": "1", "outcome": "Yes", "price": 0.5}],
            "tags": ["politics"]
        }]
    });
    Mock::given(method("GET"))
        .and(path("/sampling-markets"))
        .respond_with(json(&body.to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let markets: Vec<_> = clob(&server)
        .list_sampling_markets()
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].minimum_tick_size, Some("0.01".parse().unwrap()));
}

#[tokio::test]
async fn stream_yields_error_and_ends() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/sampling-markets"))
        .respond_with(api_error(500, "Internal server error"))
        .expect(1)
        .mount(&server)
        .await;

    let results: Vec<_> = clob(&server)
        .list_sampling_markets()
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let err = results.into_iter().next().unwrap().unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(500));
}

#[tokio::test]
async fn get_clob_market_info() {
    let server = common::server().await;
    // Documented fields from `components/schemas/ClobMarketDetails`, plus the undocumented
    // keys sent live (`c`, `sd`, `ao`, `aot`, `nr`, `cbos`, `v`; captured 2026-10-02).
    Mock::given(method("GET"))
        .and(path(format!("/clob-markets/{CONDITION_ID}")))
        .respond_with(json(&format!(
            r#"{{"gst":null,"r":{{}},"t":[{{"t":"71321045679252212594626385532706912750332728571942532289631379312455583992563","o":"Yes"}}],"c":"{CONDITION_ID}","mos":5,"mts":0.01,"mbf":0,"tbf":0,"rfqe":false,"itode":true,"ao":true,"nr":true,"cbos":true,"aot":"2025-09-18T20:07:36Z","sd":3,"ibce":true,"fd":{{"r":0.02,"e":2,"to":true}},"v":"v1","oas":3}}"#,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let info = clob(&server)
        .get_clob_market_info(CONDITION_ID)
        .await
        .unwrap();
    assert_eq!(info.min_tick_size, Some("0.01".parse().unwrap()));
    assert_eq!(info.taker_order_delay_enabled, Some(true));
    assert_eq!(info.min_order_age_seconds, Some(3));
    assert_eq!(info.condition_id, Some(ConditionId::from(CONDITION_ID)));
    assert_eq!(info.seconds_delay, Some(3));
    assert_eq!(info.accepting_orders, Some(true));
    assert_eq!(
        info.accepting_order_timestamp.map(|t| t.timestamp()),
        Some(1_758_226_056)
    );
    assert_eq!(info.neg_risk, Some(true));
    assert_eq!(info.cbos, Some(true));
    assert_eq!(info.version.as_deref(), Some("v1"));
    assert_eq!(info.tokens.unwrap()[0].outcome.as_deref(), Some("Yes"));
}

#[tokio::test]
async fn get_market_by_token() {
    let server = common::server().await;
    // Examples from `components/schemas/MarketByTokenResponse`.
    Mock::given(method("GET"))
        .and(path(
            "/markets-by-token/71321045679252212594626385532706912750332728571942532289631379312455583992563",
        ))
        .respond_with(json(&json!({
            "condition_id": CONDITION_ID,
            "primary_token_id": "71321045679252212594626385532706912750332728571942532289631379312455583992563",
            "secondary_token_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426"
        }).to_string()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets-by-token/404"))
        .respond_with(api_error(404, "market not found"))
        .expect(1)
        .mount(&server)
        .await;

    let client = clob(&server);
    let market = client
        .get_market_by_token(
            "71321045679252212594626385532706912750332728571942532289631379312455583992563",
        )
        .await
        .unwrap();
    assert_eq!(market.condition_id, CONDITION_ID);
    assert_eq!(
        market.secondary_token_id,
        "52114319501245915516055106046884209969926127482827954674443846427813813222426"
    );
    assert!(
        client
            .get_market_by_token("404")
            .await
            .unwrap_err()
            .is_not_found()
    );
}

/// A `LiveActivityMarket` (fields from the schema; the spec has no example).
fn live_activity(condition_id: &str) -> serde_json::Value {
    json!({
        "condition_id": condition_id,
        "id": 248849,
        "question": "Will Trump win the 2024 Iowa Caucus?",
        "market_slug": "will-trump-win-the-2024-iowa-caucus",
        "event_slug": "2024-us-election",
        "icon": "https://example.com/icon.png",
        "image": "https://example.com/image.png",
        "tags": ["politics"]
    })
}

#[tokio::test]
async fn get_markets_live_activity() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/markets/live-activity"))
        .and(body_json(json!(["0x1234", "0x5678"])))
        .respond_with(json(
            &json!([live_activity("0x1234"), live_activity("0x5678")]).to_string(),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let markets = clob(&server)
        .get_markets_live_activity(["0x1234", "0x5678"])
        .await
        .unwrap();
    assert_eq!(markets.len(), 2);
    assert_eq!(markets[1].condition_id, Some(ConditionId::from("0x5678")));
    assert_eq!(markets[0].id, Some(248_849));
}

#[tokio::test]
async fn get_markets_live_activity_rejects_empty_body() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json("[]"))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    for err in [
        client
            .get_markets_live_activity(Vec::<ConditionId>::new())
            .await
            .unwrap_err(),
        client
            .get_markets_live_activity(["0x1234", ""])
            .await
            .unwrap_err(),
    ] {
        let Error::Validation(v) = &err else {
            panic!("expected Error::Validation, got {err:?}")
        };
        assert_eq!(v.parameter(), "condition_ids");
    }
}

#[tokio::test]
async fn get_markets_live_activity_is_retried() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/markets/live-activity"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/markets/live-activity"))
        .respond_with(json(&json!([live_activity("0x1234")]).to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let markets = retrying_clob(&server)
        .get_markets_live_activity(["0x1234"])
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
}

#[tokio::test]
async fn empty_path_ids_are_rejected_before_sending() {
    let server = common::server().await;
    Mock::given(any())
        .respond_with(json("{}"))
        .expect(0)
        .mount(&server)
        .await;

    let client = clob(&server);
    let parameter = |err: Error| match err {
        Error::Validation(v) => v.parameter().to_owned(),
        other => panic!("expected Error::Validation, got {other:?}"),
    };
    assert_eq!(
        parameter(client.get_clob_market_info("").await.unwrap_err()),
        "condition_id"
    );
    assert_eq!(
        parameter(client.get_market_live_activity("").await.unwrap_err()),
        "condition_id"
    );
    assert_eq!(
        parameter(client.get_market_by_token("").await.unwrap_err()),
        "token_id"
    );
}

#[tokio::test]
async fn get_market_live_activity() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path(format!("/markets/live-activity/{CONDITION_ID}")))
        .respond_with(json(&live_activity(CONDITION_ID).to_string()))
        .expect(1)
        .mount(&server)
        .await;

    let market = clob(&server)
        .get_market_live_activity(CONDITION_ID)
        .await
        .unwrap();
    assert_eq!(market.tags.unwrap(), ["politics"]);
}
