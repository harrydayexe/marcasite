//! Market endpoints: `/v2/holders`, `/v2/oi`, `/v2/live-volume`, `/v2/prices-history`,
//! `/v2/resolutions`.

use chrono::DateTime;
use futures_util::TryStreamExt as _;
use polyoxide::{
    Decimal, Error,
    data::{PriceHistoryInterval, ResolutionSelector, ResolutionStatus},
    types::TokenId,
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::{
    fixtures::{self, CONDITION, CONDITION_2, TOKEN, WALLET, WALLET_2},
    received_queries,
};
use crate::common;

#[tokio::test]
async fn list_holders_stream_resends_condition() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/holders"))
        .and(query_param("condition", CONDITION))
        .and(query_param("min_balance", "1"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::holder_group("1", WALLET),
                fixtures::holder_group("2", WALLET),
            ],
            Some("window-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/holders"))
        .and(query_param("condition", CONDITION))
        .and(query_param("cursor", "window-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![fixtures::holder_group("1", WALLET_2)],
            None,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let groups: Vec<_> = common::polymarket(&server)
        .data()
        .list_holders([CONDITION])
        .min_balance(Decimal::ONE)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    let tokens: Vec<_> = groups.iter().map(|g| g.token_id.clone()).collect();
    assert_eq!(
        tokens,
        vec![TokenId::from("1"), TokenId::from("2"), TokenId::from("1")]
    );
    assert_eq!(groups[0].holders[0].amount.to_string(), "1500.25");
    assert_eq!(groups[0].holders[0].avg_price, None);
}

#[tokio::test]
async fn list_holders_with_pnl_requires_single_condition() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let err = common::polymarket(&server)
        .data()
        .list_holders([CONDITION, CONDITION_2])
        .include_pnl(true)
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "condition"),
        "{err}"
    );
}

#[tokio::test]
async fn get_open_interest_global_and_per_market() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/oi"))
        .and(query_param_is_missing("condition"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::envelope(
            json!([{"condition_id": "GLOBAL", "value": 98765432.1}]),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/oi"))
        .and(query_param("condition", CONDITION))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::envelope(
            json!([{"condition_id": CONDITION, "value": 0.0}]),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let global = data.get_open_interest().send().await.unwrap();
    assert!(global[0].is_global());
    let market = data
        .get_open_interest()
        .conditions([CONDITION])
        .send()
        .await
        .unwrap();
    assert!(!market[0].is_global());
    assert_eq!(market[0].value, Decimal::ZERO);
}

#[tokio::test]
async fn get_live_volume_sends_event_ids_csv() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/live-volume"))
        .and(query_param("event_id", "20,10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "taker_volume_total": 150.5,
                "conditions": [
                    {"condition_id": CONDITION, "taker_volume": 100.25},
                    {"condition_id": CONDITION_2, "taker_volume": 50.25}
                ]
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let volume = data.get_live_volume(["20", "10"]).await.unwrap();
    assert_eq!(volume.taker_volume_total.to_string(), "150.5");
    assert_eq!(volume.conditions.len(), 2);

    // Required, at most 20 distinct values: validated before sending.
    assert!(matches!(
        data.get_live_volume(Vec::<String>::new()).await,
        Err(Error::Validation(_))
    ));
    let too_many: Vec<String> = (1..=21).map(|i| i.to_string()).collect();
    assert!(matches!(
        data.get_live_volume(too_many).await,
        Err(Error::Validation(_))
    ));
    // The server silently drops unparseable ids; the client rejects them instead.
    assert!(matches!(
        data.get_live_volume(["20", "1O"]).await,
        Err(Error::Validation(v)) if v.parameter() == "event_id"
    ));
}

#[tokio::test]
async fn list_prices_history_sends_window_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/prices-history"))
        .and(query_param("token_id", TOKEN))
        .and(query_param("interval", "max"))
        .and(query_param("bucket_seconds", "43200"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                json!({"timestamp": 1787097600, "price": 0.51, "resolution_seconds": 43200}),
                json!({"timestamp": 1787133611, "price": 0.515, "resolution_seconds": 0}),
            ],
            None,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_prices_history(TOKEN)
        .interval(PriceHistoryInterval::Max)
        .bucket_seconds(43_200)
        .limit(2)
        .send()
        .await
        .unwrap();
    assert_eq!(page.items()[0].resolution_seconds, 43_200);
    assert_eq!(page.items()[1].price.to_string(), "0.515");
    assert_eq!(page.items()[1].timestamp.timestamp(), 1_787_133_611);
}

#[tokio::test]
async fn list_prices_history_range_and_as_of() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/prices-history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(Vec::new(), None)))
        .expect(2)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let start = DateTime::from_timestamp(1_787_000_000, 0).unwrap();
    let end = DateTime::from_timestamp(1_787_100_000, 0).unwrap();
    let page = data
        .list_prices_history(TOKEN)
        .start(start)
        .end(end)
        .send()
        .await
        .unwrap();
    assert!(page.items().is_empty());
    data.list_prices_history(TOKEN)
        .as_of(end)
        .send()
        .await
        .unwrap();
    // Mixing window forms never reaches the server.
    assert!(matches!(
        data.list_prices_history(TOKEN)
            .as_of(end)
            .interval(PriceHistoryInterval::OneDay)
            .send()
            .await,
        Err(Error::Validation(_))
    ));
    assert_eq!(
        received_queries(&server).await,
        vec![
            format!("token_id={TOKEN}&start=1787000000&end=1787100000"),
            format!("token_id={TOKEN}&as_of=1787100000"),
        ]
    );
}

#[tokio::test]
async fn get_resolutions_by_question_and_conditions() {
    let server = common::server().await;
    let question = "0x1111111111111111111111111111111111111111111111111111111111111111";
    Mock::given(method("GET"))
        .and(path("/v2/resolutions"))
        .and(query_param("question_id", question))
        .and(query_param_is_missing("condition"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!([{
                "question_id": question,
                "status": "disputed",
                "extended_review": false,
                "was_disputed": true,
                "new_version_q": false,
                "transaction_hash": "0xfeed",
                "log_index": "7",
                "last_update_timestamp": "1787133600",
                "proposed_price": "1000000000000000000",
                "reproposed_price": "0"
            }]))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/resolutions"))
        .and(query_param(
            "condition",
            format!("{CONDITION},{CONDITION_2}"),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!([]))))
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let rows = data
        .get_resolutions(ResolutionSelector::question(question))
        .await
        .unwrap();
    assert_eq!(rows[0].status, ResolutionStatus::Disputed);
    assert!(rows[0].was_disputed);
    assert_eq!(rows[0].condition_id, None);
    assert_eq!(
        rows[0].last_update_time().map(|t| t.timestamp()),
        Some(1_787_133_600)
    );
    let misses = data
        .get_resolutions(ResolutionSelector::conditions([CONDITION, CONDITION_2]))
        .await
        .unwrap();
    assert!(misses.is_empty());
}

#[tokio::test]
async fn list_holders_with_pnl_validates_limit_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let err = common::polymarket(&server)
        .data()
        .list_holders([CONDITION])
        .include_pnl(true)
        .limit(101)
        .send()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Validation(v) if v.parameter() == "limit"),
        "{err}"
    );
}

#[tokio::test]
async fn list_prices_history_stream_resends_window_and_ends_with_terminal_point() {
    let server = common::server().await;
    let start = DateTime::from_timestamp(1_787_000_000, 0).unwrap();
    let end = DateTime::from_timestamp(1_787_100_000, 0).unwrap();
    Mock::given(method("GET"))
        .and(path("/v2/prices-history"))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::price_point(1_787_000_000, 0.5, 1800),
                fixtures::price_point(1_787_001_800, 0.51, 1800),
            ],
            Some("ph-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/prices-history"))
        .and(query_param("cursor", "ph-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::price_point(1_787_003_600, 0.52, 1800),
                // The terminal point: a real tick (`resolution_seconds` 0) on the last page.
                fixtures::price_point(1_787_004_123, 0.525, 0),
            ],
            None,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let points: Vec<_> = common::polymarket(&server)
        .data()
        .list_prices_history(TOKEN)
        .start(start)
        .end(end)
        .bucket_seconds(1800)
        .limit(2)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(points.len(), 4);
    let terminal = points.last().unwrap();
    assert_eq!(terminal.resolution_seconds, 0);
    assert_eq!(terminal.price.to_string(), "0.525");
    let window =
        format!("token_id={TOKEN}&start=1787000000&end=1787100000&bucket_seconds=1800&limit=2");
    assert_eq!(
        received_queries(&server).await,
        vec![window.clone(), format!("{window}&cursor=ph-2")]
    );
}

#[tokio::test]
async fn get_resolutions_by_events() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/resolutions"))
        .and(query_param("event_id", "12345,678"))
        .and(query_param_is_missing("condition"))
        .and(query_param_is_missing("question_id"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!([{
                "condition_id": CONDITION,
                "question_id": "0x2222222222222222222222222222222222222222222222222222222222222222",
                "status": "resolved",
                "extended_review": false,
                "was_disputed": false,
                "new_version_q": false,
                "transaction_hash": "",
                "log_index": "",
                "last_update_timestamp": "2026-08-19T10:00:00Z",
                "market_type": "BINARY",
                "payouts": [0, 1000000],
                "resolved_at": "2026-08-19T10:00:00Z",
                "resolved_block": 75000000
            }]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let rows = data
        .get_resolutions(ResolutionSelector::events(["12345", "678", "12345"]))
        .await
        .unwrap();
    assert_eq!(rows[0].status, ResolutionStatus::Resolved);
    assert_eq!(rows[0].payouts.as_deref(), Some(&[0, 1_000_000][..]));
    // Malformed selectors never reach the server.
    assert!(matches!(
        data.get_resolutions(ResolutionSelector::events(["0"])).await,
        Err(Error::Validation(v)) if v.parameter() == "event_id"
    ));
    assert!(matches!(
        data.get_resolutions(ResolutionSelector::question("0xabc")).await,
        Err(Error::Validation(v)) if v.parameter() == "question_id"
    ));
}
