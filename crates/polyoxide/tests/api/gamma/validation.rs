//! Client-side checks made before any request is sent: integer-typed ids (paths and
//! filters) must be ASCII digits, slugs and addresses in paths must be usable segments.

use futures_util::TryStreamExt as _;
use polyoxide::{Error, Result, gamma::GammaClient};

use super::requests;
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
