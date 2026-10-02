//! `/teams` and `/sports` endpoints.

use futures_util::TryStreamExt as _;
use marcasite::gamma::{SeriesId, TagId, TeamId};
use serde_json::json;
use wiremock::{
    Mock,
    matchers::{method, path, query_param},
};

use super::{fixture, json, json_value, pairs, query_of};
use crate::common;

#[tokio::test]
async fn list_teams_sends_every_filter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/teams"))
        .respond_with(json_value(&json!([fixture("Team")])))
        .expect(1)
        .mount(&server)
        .await;

    let teams = common::polymarket(&server)
        .gamma()
        .list_teams()
        .limit(2)
        .offset(0)
        .order("name")
        .ascending(true)
        .leagues(["nba", "nfl"])
        .names(["Lakers"])
        .abbreviations(["LAL"])
        .send()
        .await
        .unwrap();

    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "2"),
            ("offset", "0"),
            ("order", "name"),
            ("ascending", "true"),
            ("league", "nba"),
            ("league", "nfl"),
            ("name", "Lakers"),
            ("abbreviation", "LAL"),
        ])
    );
    assert_eq!(teams[0].id, Some(TeamId::from("7")));
    assert_eq!(teams[0].league.as_deref(), Some("league-value"));
}

#[tokio::test]
async fn list_teams_stream_and_get_team() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/teams"))
        .and(query_param("offset", "0"))
        .respond_with(json(r#"[{"id":1},{"id":2}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/teams"))
        .and(query_param("offset", "2"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/teams/2"))
        .respond_with(json(r#"{"id":2,"name":"Celtics"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let ids: Vec<_> = gamma
        .list_teams()
        .into_stream()
        .map_ok(|t| t.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids, vec![TeamId::from("1"), TeamId::from("2")]);

    let team = gamma.get_team(ids[1].clone()).await.unwrap();
    assert_eq!(team.name.as_deref(), Some("Celtics"));
}

#[tokio::test]
async fn sports_metadata_and_market_types() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/sports"))
        .respond_with(json_value(&json!([fixture("SportsMetadata")])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sports/market-types"))
        .respond_with(json_value(&fixture("SportsMarketTypesResponse")))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let sports = gamma.get_sports_metadata().await.unwrap();
    assert_eq!(sports[0].sport.as_deref(), Some("sport-value"));
    assert_eq!(sports[0].series, Some(SeriesId::from("series-value")));
    assert_eq!(
        sports[0].tag_ids().collect::<Vec<_>>(),
        vec![TagId::from("tags-value")]
    );
    let types = gamma.get_sports_market_types().await.unwrap();
    assert_eq!(
        types.market_types,
        Some(vec!["a".to_owned(), "b".to_owned()])
    );
}
