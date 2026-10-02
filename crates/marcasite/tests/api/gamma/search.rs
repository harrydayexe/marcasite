//! `/public-search`.

use marcasite::gamma::TagId;
use wiremock::{
    Mock,
    matchers::{method, path},
};

use super::{fixture, json, json_value, pairs, query_of};
use crate::common;

#[tokio::test]
async fn search_sends_every_parameter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/public-search"))
        .respond_with(json_value(&fixture("Search")))
        .expect(1)
        .mount(&server)
        .await;

    let results = common::polymarket(&server)
        .gamma()
        .search("us election")
        .cache(false)
        .events_status("active")
        .limit_per_type(5)
        .page(2)
        .events_tags(["politics", "us"])
        .keep_closed_markets(0)
        .sort("volume")
        .ascending(false)
        .search_tags(true)
        .search_profiles(true)
        .recurrence("daily")
        .exclude_tag_ids(["1", "2"])
        .optimized(true)
        .send()
        .await
        .unwrap();

    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("q", "us election"),
            ("cache", "false"),
            ("events_status", "active"),
            ("limit_per_type", "5"),
            ("page", "2"),
            ("events_tag", "politics"),
            ("events_tag", "us"),
            ("keep_closed_markets", "0"),
            ("sort", "volume"),
            ("ascending", "false"),
            ("search_tags", "true"),
            ("search_profiles", "true"),
            ("recurrence", "daily"),
            ("exclude_tag_id", "1"),
            ("exclude_tag_id", "2"),
            ("optimized", "true"),
        ])
    );
    assert_eq!(results.events.unwrap().len(), 1);
    let tag = &results.tags.unwrap()[0];
    assert_eq!(tag.id, Some(TagId::from("id-value")));
    assert_eq!(tag.event_count, Some(7));
    assert_eq!(
        results.profiles.unwrap()[0].pseudonym.as_deref(),
        Some("pseudonym-value")
    );
    assert_eq!(results.pagination.unwrap().has_more, Some(true));
}

/// `optimized=true` answers with `hasMore` instead of `pagination` and real JSON arrays for
/// the markets' `outcomes` / `outcomePrices` (fixture captured from the live API).
#[tokio::test]
async fn optimized_search_decodes_the_live_shape() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/public-search"))
        .respond_with(json_value(&super::live_fixture("SearchOptimized")))
        .expect(1)
        .mount(&server)
        .await;

    let results = common::polymarket(&server)
        .gamma()
        .search("election")
        .limit_per_type(10)
        .optimized(true)
        .send()
        .await
        .unwrap();
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("q", "election"),
            ("limit_per_type", "10"),
            ("optimized", "true")
        ])
    );
    assert_eq!(results.has_more, Some(true));
    assert_eq!(results.pagination, None);
    let market = &results.events.unwrap()[0].markets.as_ref().unwrap()[0].clone();
    assert_eq!(
        market.outcomes,
        Some(vec!["Yes".to_owned(), "No".to_owned()])
    );
    assert_eq!(market.outcome_prices.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn search_with_only_the_required_query() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/public-search"))
        .respond_with(json(r#"{"events":null,"tags":null,"profiles":null}"#))
        .expect(1)
        .mount(&server)
        .await;

    let results = common::polymarket(&server)
        .gamma()
        .search("btc")
        .send()
        .await
        .unwrap();
    assert_eq!(results.events, None);
    assert_eq!(results.pagination, None);
    assert_eq!(query_of(&server, 0).await, pairs(&[("q", "btc")]));
}
