//! Board endpoints: `/v2/leaderboard`, `/v2/biggest-winners`, `/v2/builders/leaderboard`,
//! `/v2/builders/volume`.

use chrono::NaiveDate;
use futures_util::TryStreamExt as _;
use polyoxide::{
    Error,
    data::{BuilderCode, LeaderboardSortBy, TimePeriod, WinKind},
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::{
    fixtures::{self, CONDITION, WALLET},
    received_queries,
};
use crate::common;

#[tokio::test]
async fn get_leaderboard_walks_pages_with_pinned_board() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/leaderboard"))
        .and(query_param_is_missing("cursor"))
        .and(query_param_is_missing("user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![
                fixtures::leaderboard_entry(1),
                fixtures::leaderboard_entry(1),
            ],
            Some("board-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/leaderboard"))
        .and(query_param("cursor", "board-2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixtures::page(vec![fixtures::leaderboard_entry(3)], None)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let ranks: Vec<u32> = common::polymarket(&server)
        .data()
        .get_leaderboard()
        .time_period(TimePeriod::Week)
        .category("sports")
        .sort_by(LeaderboardSortBy::Volume)
        .limit(2)
        .into_stream()
        .map_ok(|e| e.rank)
        .try_collect()
        .await
        .unwrap();
    // Ties share a rank and the next one skips.
    assert_eq!(ranks, vec![1, 1, 3]);
    assert_eq!(
        received_queries(&server).await,
        vec![
            "time_period=week&category=sports&sort_by=VOLUME&limit=2",
            "time_period=week&category=sports&sort_by=VOLUME&limit=2&cursor=board-2",
        ]
    );
}

#[tokio::test]
async fn get_leaderboard_standing_reads_user_arm() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/leaderboard"))
        .and(query_param("user", WALLET))
        .and(query_param("time_period", "all"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "user_id": WALLET,
                "pnl": 1000.5,
                "volume": 20000,
                "user_name": "whale",
                "profile_image": "",
                "x_username": "whale_x",
                "verified": true,
                "rank_pnl": 7,
                "rank_volume": null
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/leaderboard"))
        .and(query_param(
            "user",
            "0x0000000000000000000000000000000000000001",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": null})))
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let standing = data
        .get_leaderboard_standing(WALLET)
        .time_period(TimePeriod::All)
        .send()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(standing.rank_pnl, Some(7));
    assert_eq!(standing.rank_volume, None);
    assert_eq!(
        data.get_leaderboard_standing("0x0000000000000000000000000000000000000001")
            .send()
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn list_biggest_winners_decodes_market_and_combo_rows() {
    let server = common::server().await;
    let row = |rank: u32, kind: &str, event_id: i32| {
        json!({
            "win_rank": rank,
            "kind": kind,
            "user_id": WALLET,
            "pnl": 900,
            "initial_value": 100,
            "final_value": 1000,
            "resolved_at": 1787133600,
            "condition_id": CONDITION,
            "position_id": "123",
            "event_id": event_id,
            "event_slug": if event_id == 0 { "" } else { "e" },
            "event_title": "E",
            "user_name": "",
            "profile_image": ""
        })
    };
    Mock::given(method("GET"))
        .and(path("/v2/biggest-winners"))
        .and(query_param("time_period", "month"))
        .and(query_param("category", "combos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![row(1, "market", 42), row(2, "combo", 0)],
            None,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .list_biggest_winners()
        .time_period(TimePeriod::Month)
        .category("combos")
        .send()
        .await
        .unwrap();
    assert_eq!(page.items[0].kind, WinKind::Market);
    assert_eq!(page.items[0].event_id, 42);
    assert_eq!(page.items[1].kind, WinKind::Combo);
    assert_eq!(page.items[1].event_id, 0);
}

#[tokio::test]
async fn get_builders_leaderboard_decodes_standings() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/builders/leaderboard"))
        .and(query_param("time_period", "day"))
        .and(query_param("limit", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::page(
            vec![json!({
                "rank": 1,
                "builder_name": "acme",
                "builder_code": "acme",
                "profile_image": "",
                "verified": true,
                "volume": 123456.5,
                "active_users": 42
            })],
            Some("b-2"),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .data()
        .get_builders_leaderboard()
        .time_period(TimePeriod::Day)
        .limit(1)
        .send()
        .await
        .unwrap();
    assert_eq!(page.items[0].builder_code, BuilderCode::from("acme"));
    assert_eq!(page.items[0].active_users, 42);
    assert_eq!(page.next_cursor(), Some("b-2"));
}

#[tokio::test]
async fn get_builders_volume_decodes_series_and_validates_limit() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/builders/volume"))
        .and(query_param("interval", "week"))
        .and(query_param("limit", "90"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!([{
                "date": "2026-09-28",
                "rank": 1,
                "builder_name": "acme",
                "builder_code": "acme",
                "profile_image": "",
                "verified": false,
                "volume": 10,
                "active_users": 1
            }]))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let data = common::polymarket(&server).data().clone();
    let series = data
        .get_builders_volume()
        .interval(TimePeriod::Week)
        .limit(90)
        .send()
        .await
        .unwrap();
    assert_eq!(
        series[0].date,
        NaiveDate::from_ymd_opt(2026, 9, 28).unwrap()
    );

    let err = data
        .get_builders_volume()
        .limit(91)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, Error::Validation(v) if v.parameter() == "limit"));
}
