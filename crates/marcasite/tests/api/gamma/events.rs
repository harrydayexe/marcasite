//! `/events` endpoints.

use futures_util::TryStreamExt as _;
use marcasite::{
    Decimal, Error,
    chrono::{TimeZone as _, Utc},
    gamma::{EventCreatorId, EventId, SeriesId},
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param},
};

use super::{
    fixture, internal_error, json, json_value, no_retry_gamma, pairs, query_of, requests,
    validation_error,
};
use crate::common;

#[tokio::test]
async fn list_events_sends_every_filter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(json_value(&json!([fixture("Event")])))
        .expect(1)
        .mount(&server)
        .await;

    let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let events = common::polymarket(&server)
        .gamma()
        .list_events()
        .limit(10)
        .offset(20)
        .order("volume")
        .ascending(true)
        .ids(["1", "2"])
        .tag_id("3")
        .exclude_tag_ids(["4", "5"])
        .slugs(["s"])
        .tag_slug("politics")
        .related_tags(true)
        .active(true)
        .archived(false)
        .featured(true)
        .cyom(false)
        .include_chat(true)
        .include_template(false)
        .recurrence("daily")
        .closed(false)
        .liquidity_min(1)
        .liquidity_max(2)
        .volume_min(Decimal::new(15, 1))
        .volume_max(4)
        .start_date_min(at)
        .start_date_max(at)
        .end_date_min(at)
        .end_date_max(at)
        .send()
        .await
        .unwrap();

    let ts = "2024-01-02T03:04:05Z";
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "10"),
            ("offset", "20"),
            ("order", "volume"),
            ("ascending", "true"),
            ("id", "1"),
            ("id", "2"),
            ("tag_id", "3"),
            ("exclude_tag_id", "4"),
            ("exclude_tag_id", "5"),
            ("slug", "s"),
            ("tag_slug", "politics"),
            ("related_tags", "true"),
            ("active", "true"),
            ("archived", "false"),
            ("featured", "true"),
            ("cyom", "false"),
            ("include_chat", "true"),
            ("include_template", "false"),
            ("recurrence", "daily"),
            ("closed", "false"),
            ("liquidity_min", "1"),
            ("liquidity_max", "2"),
            ("volume_min", "1.5"),
            ("volume_max", "4"),
            ("start_date_min", ts),
            ("start_date_max", ts),
            ("end_date_min", ts),
            ("end_date_max", ts),
        ])
    );
    let event = &events[0];
    assert_eq!(event.id, Some(EventId::from("id-value")));
    assert_eq!(event.volume, Some(Decimal::new(15, 1)));
    assert_eq!(event.markets.as_ref().unwrap().len(), 1);
    assert_eq!(event.collections.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn list_events_stream_walks_offsets() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .and(query_param("offset", "5"))
        .respond_with(json(r#"[{"id":"1"},{"id":"2"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .and(query_param("offset", "7"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let events: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_events()
        .offset(5)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn list_events_paginated_decodes_page_and_stops_on_has_more_false() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/pagination"))
        .and(query_param("offset", "0"))
        .respond_with(json(
            r#"{"data":[{"id":"1"},{"id":"2"}],"pagination":{"hasMore":true,"totalResults":4}}"#,
        ))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/pagination"))
        .and(query_param("offset", "2"))
        .respond_with(json(
            r#"{"data":[{"id":"3"},{"id":"4"}],"pagination":{"hasMore":false,"totalResults":4}}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let ids: Vec<_> = gamma
        .list_events_paginated()
        .limit(2)
        .order("id")
        .ascending(true)
        .include_chat(false)
        .include_template(true)
        .recurrence("weekly")
        .into_stream()
        .map_ok(|e| e.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(ids.len(), 4);
    // The second page was full, but `hasMore: false` ended the stream without a third call.
    assert_eq!(requests(&server).await.len(), 2);
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "2"),
            ("offset", "0"),
            ("order", "id"),
            ("ascending", "true"),
            ("include_chat", "false"),
            ("include_template", "true"),
            ("recurrence", "weekly"),
        ])
    );

    let page = gamma
        .list_events_paginated()
        .limit(2)
        .offset(0)
        .send()
        .await
        .unwrap();
    let pagination = page.pagination.unwrap();
    assert_eq!(pagination.has_more, Some(true));
    assert_eq!(pagination.total_results, Some(4));
    assert_eq!(page.data.unwrap().len(), 2);
}

#[tokio::test]
async fn list_events_paginated_stream_continues_after_a_short_page_while_has_more() {
    let server = common::server().await;
    // The server serves fewer events than `limit` but says there are more.
    Mock::given(method("GET"))
        .and(path("/events/pagination"))
        .and(query_param("offset", "0"))
        .respond_with(json(
            r#"{"data":[{"id":"1"},{"id":"2"}],"pagination":{"hasMore":true,"totalResults":3}}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/pagination"))
        .and(query_param("offset", "2"))
        .respond_with(json(
            r#"{"data":[{"id":"3"}],"pagination":{"hasMore":false,"totalResults":3}}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_events_paginated()
        .limit(3)
        .into_stream()
        .map_ok(|e| e.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        ids,
        vec![EventId::from("1"), EventId::from("2"), EventId::from("3")]
    );
    assert_eq!(requests(&server).await.len(), 2);
}

#[tokio::test]
async fn list_sport_event_results() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/results"))
        .respond_with(json(r#"[{"id":"1","score":"3-1","ended":true}]"#))
        .expect(1)
        .mount(&server)
        .await;

    let events = common::polymarket(&server)
        .gamma()
        .list_sport_event_results()
        .limit(1)
        .offset(0)
        .order("endDate")
        .ascending(false)
        .send()
        .await
        .unwrap();
    assert_eq!(events[0].score.as_deref(), Some("3-1"));
    assert_eq!(events[0].ended, Some(true));
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "1"),
            ("offset", "0"),
            ("order", "endDate"),
            ("ascending", "false"),
        ])
    );
}

#[tokio::test]
async fn get_event_by_id_and_slug() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/16167"))
        .respond_with(json_value(&fixture("Event")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/slug/us-election"))
        .respond_with(json(r#"{"id":"7","slug":"us-election"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let event = gamma
        .get_event("16167")
        .include_chat(true)
        .include_template(true)
        .send()
        .await
        .unwrap();
    assert_eq!(event.chats.unwrap().len(), 1);
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[("include_chat", "true"), ("include_template", "true")])
    );

    let by_slug = gamma.get_event_by_slug("us-election").send().await.unwrap();
    assert_eq!(by_slug.id, Some(EventId::from("7")));
}

#[tokio::test]
async fn get_event_reports_404() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/slug/missing"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .get_event_by_slug("missing")
        .send()
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "{err}");
    assert!(matches!(err, Error::Api(_)));
}

#[tokio::test]
async fn event_tags_tweet_count_and_comment_count() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/5/tags"))
        .respond_with(json(r#"[{"id":"1","label":"Politics"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/5/tweet-count"))
        .respond_with(json_value(&fixture("EventTweetCount")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/5/comments/count"))
        .respond_with(json(r#"{"count":12}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let tags = gamma.get_event_tags("5").await.unwrap();
    assert_eq!(tags[0].label.as_deref(), Some("Politics"));
    let tweets = gamma.get_event_tweet_count("5").await.unwrap();
    assert_eq!(tweets.tweet_count, Some(7));
    let comments = gamma.get_event_comment_count("5").await.unwrap();
    assert_eq!(comments.count, Some(12));
}

#[tokio::test]
async fn event_creators() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/creators"))
        .respond_with(json_value(&json!([fixture("EventCreator")])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/creators/3"))
        .respond_with(json(r#"{"id":"3","creatorHandle":"alice"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let creators = gamma
        .list_event_creators()
        .limit(5)
        .offset(0)
        .order("creatorName")
        .ascending(true)
        .creator_name("Alice")
        .creator_handle("alice")
        .send()
        .await
        .unwrap();
    assert_eq!(
        creators[0].creator_name.as_deref(),
        Some("creatorName-value")
    );
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "5"),
            ("offset", "0"),
            ("order", "creatorName"),
            ("ascending", "true"),
            ("creator_name", "Alice"),
            ("creator_handle", "alice"),
        ])
    );

    let creator = gamma.get_event_creator("3").await.unwrap();
    assert_eq!(creator.id, Some(EventCreatorId::from("3")));
    assert_eq!(creator.creator_handle.as_deref(), Some("alice"));
}

#[tokio::test]
async fn list_events_keyset_sends_every_filter() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(json_value(&json!({"events": [fixture("Event")]})))
        .expect(1)
        .mount(&server)
        .await;

    let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let page = common::polymarket(&server)
        .gamma()
        .list_events_keyset()
        .limit(1)
        .order("volume")
        .ascending(false)
        .cursor("c")
        .ids(["1"])
        .slugs(["s"])
        .closed(true)
        .live(false)
        .featured(true)
        .cyom(false)
        .title_search("rain")
        .liquidity_min(1)
        .liquidity_max(2)
        .volume_min(3)
        .volume_max(4)
        .start_date_min(at)
        .start_date_max(at)
        .end_date_min(at)
        .end_date_max(at)
        .start_time_min(at)
        .start_time_max(at)
        .tag_ids(["10"])
        .tag_slug("sports")
        .exclude_tag_ids(["11"])
        .related_tags(true)
        .tag_match("all")
        .series_ids([SeriesId::from("12")])
        .game_ids([13, 14])
        .event_date(at)
        .event_week(15)
        .featured_order(true)
        .recurrence("daily")
        .created_by(["me"])
        .parent_event_id("16")
        .include_children(true)
        .partner_slug("partner")
        .include_chat(true)
        .include_template(false)
        .include_best_lines(true)
        .locale("en")
        .send()
        .await
        .unwrap();

    let ts = "2024-01-02T03:04:05Z";
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "1"),
            ("order", "volume"),
            ("ascending", "false"),
            ("after_cursor", "c"),
            ("id", "1"),
            ("slug", "s"),
            ("closed", "true"),
            ("live", "false"),
            ("featured", "true"),
            ("cyom", "false"),
            ("title_search", "rain"),
            ("liquidity_min", "1"),
            ("liquidity_max", "2"),
            ("volume_min", "3"),
            ("volume_max", "4"),
            ("start_date_min", ts),
            ("start_date_max", ts),
            ("end_date_min", ts),
            ("end_date_max", ts),
            ("start_time_min", ts),
            ("start_time_max", ts),
            ("tag_id", "10"),
            ("tag_slug", "sports"),
            ("exclude_tag_id", "11"),
            ("related_tags", "true"),
            ("tag_match", "all"),
            ("series_id", "12"),
            ("game_id", "13"),
            ("game_id", "14"),
            ("event_date", ts),
            ("event_week", "15"),
            ("featured_order", "true"),
            ("recurrence", "daily"),
            ("created_by", "me"),
            ("parent_event_id", "16"),
            ("include_children", "true"),
            ("partner_slug", "partner"),
            ("include_chat", "true"),
            ("include_template", "false"),
            ("include_best_lines", "true"),
            ("locale", "en"),
        ])
    );
    assert_eq!(page.items().len(), 1);
    assert_eq!(page.next_cursor(), None);
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn list_events_keyset_stream_and_validation() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .and(query_param("after_cursor", "c1"))
        .respond_with(json(r#"{"events":[],"next_cursor":"c1"}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(json(r#"{"events":[{"id":"1"}],"next_cursor":"c1"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    // The server repeating the cursor it was given ends the stream instead of looping.
    let events: Vec<_> = gamma
        .list_events_keyset()
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(events.len(), 1);

    let err = gamma
        .list_events_keyset()
        .tag_ids(["1", "2"])
        .exclude_tag_ids(["02"])
        .send()
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "exclude_tag_id");
    assert_eq!(requests(&server).await.len(), 2);
}

#[tokio::test]
async fn list_events_keyset_reports_503() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(503).set_body_raw(
            r#"{"type":"service unavailable","error":"keyset pagination is not configured"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let http = marcasite::HttpClient::builder()
        .retry_policy(marcasite::RetryPolicy::none())
        .build()
        .unwrap();
    let gamma = marcasite::gamma::GammaClient::builder()
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap();
    let err = gamma.list_events_keyset().send().await.unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.status().as_u16(), 503);
    // Documented as "keyset pagination is not configured": a configuration state, told
    // apart from other 503s by its error type.
    assert_eq!(api.error_type(), Some("service unavailable"));
    assert_eq!(api.message(), Some("keyset pagination is not configured"));
}

#[tokio::test]
async fn list_events_keyset_reports_documented_422_and_500() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .and(query_param("after_cursor", "bogus"))
        .respond_with(validation_error())
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(internal_error())
        .mount(&server)
        .await;

    let gamma = no_retry_gamma(&server);
    let err = gamma
        .list_events_keyset()
        .cursor("bogus")
        .send()
        .await
        .unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.status().as_u16(), 422);
    assert_eq!(api.error_type(), Some("validation error"));

    let err = gamma.list_events_keyset().send().await.unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.status().as_u16(), 500);
    assert_eq!(api.error_type(), Some("internal error"));
    assert_eq!(api.message(), Some("cannot get the information"));
}

#[tokio::test]
async fn list_events_keyset_pages_manually_with_next_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .and(query_param("after_cursor", "c1"))
        .respond_with(json(r#"{"events":[{"id":"2"}],"next_cursor":""}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(json(r#"{"events":[{"id":"1"}],"next_cursor":"c1"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let mut ids = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let mut request = gamma.list_events_keyset();
        if let Some(cursor) = cursor.take() {
            request = request.cursor(cursor);
        }
        let page = request.send().await.unwrap();
        cursor = page.next_cursor().map(str::to_owned);
        ids.extend(page.into_items().into_iter().map(|e| e.id.unwrap()));
        if cursor.is_none() {
            break;
        }
    }
    // An empty `next_cursor` is the end, like an absent one.
    assert_eq!(ids, vec![EventId::from("1"), EventId::from("2")]);
}
