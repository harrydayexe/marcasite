//! `/series` and `/series-summary` endpoints.

use futures_util::TryStreamExt as _;
use marcasite::gamma::SeriesId;
use serde_json::json;
use wiremock::{
    Mock,
    matchers::{method, path, query_param},
};

use super::{fixture, json, json_value, pairs, query_of};
use crate::common;

#[tokio::test]
async fn list_series_sends_every_filter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/series"))
        .respond_with(json_value(&json!([fixture("Series")])))
        .expect(1)
        .mount(&server)
        .await;

    let series = common::polymarket(&server)
        .gamma()
        .list_series()
        .limit(3)
        .offset(6)
        .order("volume")
        .ascending(false)
        .slugs(["nba", "nfl"])
        .categories_ids([1, 2])
        .categories_labels(["Sports"])
        .closed(false)
        .include_chat(true)
        .recurrence("daily")
        .exclude_events(true)
        .send()
        .await
        .unwrap();

    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "3"),
            ("offset", "6"),
            ("order", "volume"),
            ("ascending", "false"),
            ("slug", "nba"),
            ("slug", "nfl"),
            ("categories_ids", "1"),
            ("categories_ids", "2"),
            ("categories_labels", "Sports"),
            ("closed", "false"),
            ("include_chat", "true"),
            ("recurrence", "daily"),
            ("exclude_events", "true"),
        ])
    );
    assert_eq!(series[0].id, Some(SeriesId::from("id-value")));
    assert_eq!(series[0].template_variables, Some(true));
    assert_eq!(series[0].competitive.as_deref(), Some("competitive-value"));
}

#[tokio::test]
async fn list_series_stream_walks_offsets() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/series"))
        .and(query_param("offset", "0"))
        .respond_with(json(r#"[{"id":"1"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/series"))
        .and(query_param("offset", "1"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let series: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_series()
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(series.len(), 1);
}

#[tokio::test]
async fn get_series_and_comment_count() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/series/10"))
        .and(query_param("include_chat", "true"))
        .respond_with(json(r#"{"id":"10","chats":[{"id":"c","live":true}]}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/series/10/comments/count"))
        .respond_with(json_value(&fixture("Count")))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let series = gamma
        .get_series("10")
        .include_chat(true)
        .send()
        .await
        .unwrap();
    assert_eq!(series.chats.unwrap()[0].live, Some(true));
    let count = gamma.get_series_comment_count("10").await.unwrap();
    assert_eq!(count.count, Some(7));
}

#[tokio::test]
async fn series_summary_by_id_and_slug() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/series-summary/10"))
        .respond_with(json_value(&fixture("SeriesSummary")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/series-summary/slug/nba"))
        .respond_with(json(r#"{"id":"10","slug":"nba","eventWeeks":[1]}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let summary = gamma.get_series_summary("10").await.unwrap();
    assert_eq!(summary.earliest_open_week, Some(7));
    assert_eq!(
        summary.event_dates,
        Some(vec!["a".to_owned(), "b".to_owned()])
    );
    let by_slug = gamma.get_series_summary_by_slug("nba").await.unwrap();
    assert_eq!(by_slug.event_weeks, Some(vec![1]));
}

#[tokio::test]
async fn get_series_reports_404() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/series-summary/slug/missing"))
        .respond_with(wiremock::ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .get_series_summary_by_slug("missing")
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "{err}");
}
