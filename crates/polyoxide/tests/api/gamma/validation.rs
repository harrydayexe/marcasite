//! Client-side checks made before any request is sent: integer-typed ids (paths and
//! filters) must be ASCII digits, slugs and addresses in paths must be usable segments.

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{Error, Result, gamma::GammaClient};
use wiremock::{
    Mock,
    matchers::{method, path, query_param},
};

use super::{json, requests};
use crate::common;

/// The parameter named by a validation error, or `""` for any other outcome.
fn parameter<T>(result: Result<T>) -> String {
    match result {
        Err(Error::Validation(v)) => v.parameter().to_owned(),
        _ => String::new(),
    }
}

#[tokio::test]
async fn non_integer_path_ids_are_rejected_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    // Empty, dot segments, words that name other routes, signs and whitespace.
    for bad in [
        "",
        ".",
        "..",
        "keyset",
        "pagination",
        "-1",
        " 1",
        "1.0",
        "abc",
    ] {
        assert_eq!(
            parameter(gamma.get_market(bad).send().await),
            "id",
            "{bad:?}"
        );
        assert_eq!(parameter(gamma.get_market_tags(bad).await), "id");
        assert_eq!(parameter(gamma.get_market_description(bad).await), "id");
        assert_eq!(parameter(gamma.get_event(bad).send().await), "id");
        assert_eq!(parameter(gamma.get_event_tags(bad).await), "id");
        assert_eq!(parameter(gamma.get_event_tweet_count(bad).await), "id");
        assert_eq!(parameter(gamma.get_event_comment_count(bad).await), "id");
        assert_eq!(parameter(gamma.get_event_creator(bad).await), "id");
        assert_eq!(parameter(gamma.get_tag(bad).send().await), "id");
        assert_eq!(parameter(gamma.get_related_tags(bad).send().await), "id");
        assert_eq!(
            parameter(gamma.get_related_tag_relationships(bad).send().await),
            "id"
        );
        assert_eq!(parameter(gamma.get_series(bad).send().await), "id");
        assert_eq!(parameter(gamma.get_series_comment_count(bad).await), "id");
        assert_eq!(parameter(gamma.get_series_summary(bad).await), "id");
        assert_eq!(parameter(gamma.get_comments_by_id(bad).send().await), "id");
        assert_eq!(parameter(gamma.get_team(bad).await), "id");
    }
    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn unusable_slugs_and_addresses_are_rejected_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    for bad in ["", ".", ".."] {
        assert_eq!(
            parameter(gamma.get_market_by_slug(bad).send().await),
            "slug"
        );
        assert_eq!(parameter(gamma.get_event_by_slug(bad).send().await), "slug");
        assert_eq!(parameter(gamma.get_tag_by_slug(bad).send().await), "slug");
        assert_eq!(
            parameter(gamma.get_related_tags_by_slug(bad).send().await),
            "slug"
        );
        assert_eq!(
            parameter(
                gamma
                    .get_related_tag_relationships_by_slug(bad)
                    .send()
                    .await
            ),
            "slug"
        );
        assert_eq!(
            parameter(gamma.get_series_summary_by_slug(bad).await),
            "slug"
        );
        assert_eq!(
            parameter(gamma.list_comments_by_user(bad).send().await),
            "user_address"
        );
        let streamed: Result<Vec<_>> = gamma
            .list_comments_by_user(bad)
            .into_stream()
            .try_collect()
            .await;
        assert_eq!(parameter(streamed), "user_address");
    }
    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn integer_typed_query_filters_are_rejected_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    assert_eq!(
        parameter(gamma.list_markets().ids(["1", "x"]).send().await),
        "id"
    );
    assert_eq!(
        parameter(gamma.list_markets().tag_id("politics").send().await),
        "tag_id"
    );
    assert_eq!(
        parameter(gamma.list_markets_keyset().tag_ids(["1.5"]).send().await),
        "tag_id"
    );
    assert_eq!(parameter(gamma.list_events().ids([""]).send().await), "id");
    assert_eq!(
        parameter(gamma.list_events().exclude_tag_ids(["-2"]).send().await),
        "exclude_tag_id"
    );
    assert_eq!(
        parameter(gamma.list_events_keyset().series_ids(["nba"]).send().await),
        "series_id"
    );
    assert_eq!(
        parameter(gamma.list_events_keyset().parent_event_id("p").send().await),
        "parent_event_id"
    );
    assert_eq!(
        parameter(gamma.search("q").exclude_tag_ids(["a"]).send().await),
        "exclude_tag_id"
    );
    let streamed: Result<Vec<_>> = gamma
        .list_markets()
        .ids(["x"])
        .into_stream()
        .try_collect()
        .await;
    assert_eq!(parameter(streamed), "id");
    assert!(requests(&server).await.is_empty());
}

/// Live rejects `offset` above 2000 on `/markets`, `/events` and `/events/pagination` with a
/// `422` ("offset too large, use /markets/keyset ..."); the SDK refuses it before sending.
#[tokio::test]
async fn offsets_above_2000_are_rejected_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    assert_eq!(
        parameter(gamma.list_markets().offset(2001).send().await),
        "offset"
    );
    assert_eq!(
        parameter(gamma.list_events().offset(2001).send().await),
        "offset"
    );
    assert_eq!(
        parameter(gamma.list_events_paginated().offset(2001).send().await),
        "offset"
    );
    let message = gamma
        .list_markets()
        .offset(5000)
        .send()
        .await
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("5000") && message.contains("list_markets_keyset"),
        "{message}"
    );
    let message = gamma
        .list_events()
        .offset(2001)
        .send()
        .await
        .unwrap_err()
        .to_string();
    assert!(message.contains("list_events_keyset"), "{message}");

    // A stream that starts past the cap yields the validation error and ends.
    let streamed: Result<Vec<_>> = gamma
        .list_markets()
        .offset(2100)
        .into_stream()
        .try_collect()
        .await;
    assert_eq!(parameter(streamed), "offset");
    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn offset_2000_itself_is_sent() {
    let server = common::server().await;
    for route in ["/markets", "/events"] {
        Mock::given(method("GET"))
            .and(path(route))
            .and(query_param("offset", "2000"))
            .respond_with(json("[]"))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/events/pagination"))
        .and(query_param("offset", "2000"))
        .respond_with(json(r#"{"data":[],"pagination":{"hasMore":false}}"#))
        .expect(1)
        .mount(&server)
        .await;
    let gamma = common::polymarket(&server).gamma().clone();
    gamma.list_markets().offset(2000).send().await.unwrap();
    gamma.list_events().offset(2000).send().await.unwrap();
    gamma
        .list_events_paginated()
        .offset(2000)
        .send()
        .await
        .unwrap();
}

/// The offset stream walks up to the last accepted offset (2000), then yields one validation
/// error without sending a request, and ends.
#[tokio::test]
async fn offset_stream_ends_with_a_validation_error_past_2000() {
    let server = common::server().await;
    // 21 pages of 100 (offsets 0, 100, ..., 2000), then offset 2100 is refused client-side.
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(|request: &wiremock::Request| {
            let offset: usize = request
                .url
                .query_pairs()
                .find(|(k, _)| k == "offset")
                .map(|(_, v)| v.parse().unwrap())
                .unwrap();
            let body: Vec<String> = (0..100)
                .map(|i| format!(r#"{{"id":"{}"}}"#, offset + i))
                .collect();
            json(format!("[{}]", body.join(",")))
        })
        .expect(21)
        .mount(&server)
        .await;

    let results: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_markets()
        .limit(100)
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 2100 + 1);
    assert!(results[..2100].iter().all(Result::is_ok));
    let last = results.into_iter().last().unwrap();
    assert_eq!(parameter(last.map(|_| ())), "offset");
    assert_eq!(requests(&server).await.len(), 21);
}

#[test]
fn builders_streams_and_futures_are_send_and_static() {
    fn send_sync_static<T: Send + Sync + 'static>(_: &T) {}
    fn send_static<T: Send + 'static>(_: &T) {}

    let gamma = GammaClient::builder()
        .base_url("http://127.0.0.1:1")
        .build()
        .unwrap();
    let markets = gamma.list_markets();
    send_sync_static(&markets);
    send_static(&markets.clone().send());
    send_sync_static(&markets.into_stream());
    let keyset = gamma.list_events_keyset();
    send_sync_static(&keyset);
    send_static(&keyset.clone().send());
    send_sync_static(&keyset.into_stream());
    send_static(&gamma.get_event("1").send());
    send_static(&gamma.search("q").send());
    send_static(&gamma.get_markets_information().send());
}
