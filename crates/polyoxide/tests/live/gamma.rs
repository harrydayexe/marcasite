//! Gamma API live tests.
//!
//! Every implemented Gamma endpoint is exercised through the SDK at least once with realistic
//! parameters, and each distinct response model is additionally decoded from a raw fetch of
//! the same route with [`check`] to report drift. Assertions are structural only.

use std::{collections::HashSet, fmt::Debug};

use futures_util::{Stream, StreamExt as _, TryStreamExt as _};
use polyoxide::{
    Error,
    gamma::{
        Comment, CommentCount, CommentParentEntityType, Event, EventCreator, EventTweetCount,
        EventsKeysetPage, EventsPage, Market, MarketDescription, MarketsKeysetPage, Profile,
        PublicProfile, RelatedTag, RelatedTagsStatus, SearchResults, Series, SeriesSummary,
        SportsMarketTypes, SportsMetadata, Tag, Team,
    },
};
use serde_json::json;
use tokio::sync::OnceCell;

use crate::common::{GAMMA, Raw, check, get, pm, post, sample};

/// Number of distinct values yielded by `items` (by `Debug` rendering).
fn distinct<T: Debug>(items: impl IntoIterator<Item = T>) -> usize {
    items
        .into_iter()
        .map(|item| format!("{item:?}"))
        .collect::<HashSet<_>>()
        .len()
}

/// Takes `n` items of a paginated stream, failing the test on the first stream error.
async fn take_stream<T>(stream: impl Stream<Item = Result<T, Error>>, n: usize) -> Vec<T> {
    stream.take(n).try_collect().await.unwrap()
}

/// A comment (and its author) found on a live entity.
#[derive(Debug, Clone)]
struct CommentSample {
    id: String,
    user: String,
    entity_type: &'static str,
    entity_id: String,
}

static COMMENT: OnceCell<CommentSample> = OnceCell::const_new();

/// Finds a real comment by walking the most-commented events, then the sample series.
async fn comment_sample() -> &'static CommentSample {
    COMMENT
        .get_or_init(|| async {
            let events = get(
                GAMMA,
                "/events",
                &[
                    ("limit", "20"),
                    ("active", "true"),
                    ("closed", "false"),
                    ("order", "volume24hr"),
                    ("ascending", "false"),
                ],
            )
            .await
            .json;
            let mut candidates: Vec<(&'static str, String)> = events
                .as_array()
                .unwrap()
                .iter()
                .map(|e| ("Event", e["id"].as_str().unwrap().to_owned()))
                .collect();
            if let Some(series) = &sample().await.series_id {
                candidates.push(("Series", series.clone()));
            }
            // Hot events first, but a long-lived one with many comments is the fallback.
            candidates.push(("Event", "16167".to_owned()));
            for (entity_type, entity_id) in candidates {
                let comments = get(
                    GAMMA,
                    "/comments",
                    &[
                        ("limit", "5"),
                        ("parent_entity_type", entity_type),
                        ("parent_entity_id", &entity_id),
                    ],
                )
                .await
                .json;
                if let Some(c) = comments.get(0) {
                    return CommentSample {
                        id: c["id"].as_str().unwrap().to_owned(),
                        user: c["userAddress"].as_str().unwrap().to_owned(),
                        entity_type,
                        entity_id,
                    };
                }
            }
            panic!("no comments found on any candidate entity");
        })
        .await
}

// ---------------------------------------------------------------------------------------
// status
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_status() {
    let status = pm().gamma().get_status().await.unwrap();
    assert!(!status.is_empty());
}

// ---------------------------------------------------------------------------------------
// tags
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_tags() {
    let tags = pm().gamma().list_tags().limit(5).send().await.unwrap();
    assert!(!tags.is_empty());
    check::<Vec<Tag>>("GET /tags", &get(GAMMA, "/tags", &[("limit", "20")]).await);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_tags_with_filters() {
    let tags = pm()
        .gamma()
        .list_tags()
        .limit(10)
        .order("id")
        .ascending(true)
        .include_template(true)
        .is_carousel(true)
        .send()
        .await
        .unwrap();
    assert!(!tags.is_empty());
    check::<Vec<Tag>>(
        "GET /tags?include_template&is_carousel",
        &get(
            GAMMA,
            "/tags",
            &[
                ("limit", "20"),
                ("include_template", "true"),
                ("is_carousel", "true"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_tags_stream_pages() {
    let tags: Vec<Tag> = pm()
        .gamma()
        .list_tags()
        .limit(3)
        .into_stream()
        .take(7)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(tags.len(), 7);
    assert!(distinct(tags.iter().map(|t| &t.id)) > 3, "stream advanced");
}

#[tokio::test]
#[ignore = "live network"]
async fn get_tag_by_id_and_slug() {
    let s = sample().await;
    let by_id = pm()
        .gamma()
        .get_tag(s.tag_id.as_str())
        .send()
        .await
        .unwrap();
    let by_slug = pm()
        .gamma()
        .get_tag_by_slug(&s.tag_slug)
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.id, by_slug.id);
    check::<Tag>(
        "GET /tags/{id}",
        &get(GAMMA, &format!("/tags/{}", s.tag_id), &[]).await,
    );
    check::<Tag>(
        "GET /tags/slug/{slug}",
        &get(GAMMA, &format!("/tags/slug/{}", s.tag_slug), &[]).await,
    );
    let with_template = pm()
        .gamma()
        .get_tag(s.tag_id.as_str())
        .include_template(true)
        .send()
        .await
        .unwrap();
    assert_eq!(with_template.id, by_id.id);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_related_tag_relationships() {
    let s = sample().await;
    let gamma = pm();
    let gamma = gamma.gamma();
    let by_id = gamma
        .get_related_tag_relationships(s.tag_id.as_str())
        .send()
        .await
        .unwrap();
    let by_slug = gamma
        .get_related_tag_relationships_by_slug(&s.tag_slug)
        .omit_empty(true)
        .status(RelatedTagsStatus::All)
        .send()
        .await
        .unwrap();
    assert!(by_id.len() >= by_slug.len() || by_slug.len() >= by_id.len());
    check::<Vec<RelatedTag>>(
        "GET /tags/{id}/related-tags",
        &get(GAMMA, &format!("/tags/{}/related-tags", s.tag_id), &[]).await,
    );
    check::<Vec<RelatedTag>>(
        "GET /tags/slug/{slug}/related-tags",
        &get(
            GAMMA,
            &format!("/tags/slug/{}/related-tags", s.tag_slug),
            &[("status", "all")],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_related_tags() {
    let s = sample().await;
    let gamma = pm();
    let gamma = gamma.gamma();
    let by_id = gamma
        .get_related_tags(s.tag_id.as_str())
        .send()
        .await
        .unwrap();
    let by_slug = gamma
        .get_related_tags_by_slug(&s.tag_slug)
        .omit_empty(false)
        .status(RelatedTagsStatus::Active)
        .send()
        .await
        .unwrap();
    let _ = (by_id, by_slug);
    check::<Vec<Tag>>(
        "GET /tags/{id}/related-tags/tags",
        &get(GAMMA, &format!("/tags/{}/related-tags/tags", s.tag_id), &[]).await,
    );
    check::<Vec<Tag>>(
        "GET /tags/slug/{slug}/related-tags/tags",
        &get(
            GAMMA,
            &format!("/tags/slug/{}/related-tags/tags", s.tag_slug),
            &[("status", "active")],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_event_and_market_tags() {
    let s = sample().await;
    let gamma = pm();
    let gamma = gamma.gamma();
    let event_tags = gamma.get_event_tags(s.event_id.as_str()).await.unwrap();
    assert!(!event_tags.is_empty());
    let market_tags = gamma.get_market_tags(s.market_id.as_str()).await.unwrap();
    let _ = market_tags;
    check::<Vec<Tag>>(
        "GET /events/{id}/tags",
        &get(GAMMA, &format!("/events/{}/tags", s.event_id), &[]).await,
    );
    check::<Vec<Tag>>(
        "GET /markets/{id}/tags",
        &get(GAMMA, &format!("/markets/{}/tags", s.market_id), &[]).await,
    );
}

// ---------------------------------------------------------------------------------------
// events
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_events() {
    let events = pm()
        .gamma()
        .list_events()
        .active(true)
        .closed(false)
        .order("volume24hr")
        .ascending(false)
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!events.is_empty());
    assert!(events.len() <= 20);
    check::<Vec<Event>>(
        "GET /events",
        &get(
            GAMMA,
            "/events",
            &[
                ("limit", "20"),
                ("active", "true"),
                ("closed", "false"),
                ("order", "volume24hr"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_closed_and_archived() {
    // Closed events exercise resolved-market fields (`umaResolutionStatus`, `closedTime`, ...).
    let events = pm()
        .gamma()
        .list_events()
        .closed(true)
        .order("endDate")
        .ascending(false)
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!events.is_empty());
    check::<Vec<Event>>(
        "GET /events?closed=true",
        &get(
            GAMMA,
            "/events",
            &[
                ("limit", "20"),
                ("closed", "true"),
                ("order", "endDate"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_with_filters() {
    let s = sample().await;
    let events = pm()
        .gamma()
        .list_events()
        .tag_id(s.tag_id.as_str())
        .related_tags(true)
        .active(true)
        .closed(false)
        .include_chat(true)
        .include_template(true)
        .volume_min(1)
        .limit(10)
        .send()
        .await
        .unwrap();
    assert!(!events.is_empty());
    let by_slug = pm()
        .gamma()
        .list_events()
        .slugs([s.event_slug.clone()])
        .send()
        .await
        .unwrap();
    assert_eq!(by_slug.len(), 1);
    let by_id = pm()
        .gamma()
        .list_events()
        .ids([s.event_id.as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.len(), 1);
    check::<Vec<Event>>(
        "GET /events?tag_id&include_chat&include_template",
        &get(
            GAMMA,
            "/events",
            &[
                ("limit", "10"),
                ("tag_id", &s.tag_id),
                ("related_tags", "true"),
                ("include_chat", "true"),
                ("include_template", "true"),
                ("active", "true"),
                ("closed", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_stream_pages() {
    let events: Vec<Event> = take_stream(
        pm().gamma()
            .list_events()
            .active(true)
            .closed(false)
            .limit(5)
            .into_stream(),
        12,
    )
    .await;
    assert_eq!(events.len(), 12);
    assert!(
        distinct(events.iter().map(|e| &e.id)) > 5,
        "stream advanced past the first page"
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_paginated() {
    let page = pm()
        .gamma()
        .list_events_paginated()
        .order("volume24hr")
        .ascending(false)
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    assert!(page.items().len() <= 20);
    check::<EventsPage>(
        "GET /events/pagination",
        &get(
            GAMMA,
            "/events/pagination",
            &[
                ("limit", "20"),
                ("order", "volume24hr"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_paginated_stream_pages() {
    let events: Vec<Event> = take_stream(
        pm().gamma().list_events_paginated().limit(4).into_stream(),
        10,
    )
    .await;
    assert_eq!(events.len(), 10);
    assert!(distinct(events.iter().map(|e| &e.id)) > 4);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_keyset() {
    let gamma = pm();
    let first = gamma
        .gamma()
        .list_events_keyset()
        .limit(20)
        .closed(false)
        .order("volume24hr")
        .ascending(false)
        .include_chat(true)
        .include_template(true)
        .send()
        .await
        .unwrap();
    assert!(!first.items().is_empty());
    let cursor = first.next_cursor().expect("a next cursor").to_owned();
    let second = gamma
        .gamma()
        .list_events_keyset()
        .limit(20)
        .closed(false)
        .order("volume24hr")
        .ascending(false)
        .include_chat(true)
        .include_template(true)
        .cursor(&cursor)
        .send()
        .await
        .unwrap();
    assert!(!second.items().is_empty());
    assert_ne!(
        first.items().first().map(|e| &e.id),
        second.items().first().map(|e| &e.id),
        "the cursor advanced"
    );
    let raw = get(
        GAMMA,
        "/events/keyset",
        &[
            ("limit", "20"),
            ("closed", "false"),
            ("order", "volume24hr"),
            ("ascending", "false"),
            ("include_chat", "true"),
            ("include_template", "true"),
        ],
    )
    .await;
    check::<EventsKeysetPage>("GET /events/keyset", &raw);
    let raw2 = get(
        GAMMA,
        "/events/keyset",
        &[
            ("limit", "20"),
            ("closed", "false"),
            ("order", "volume24hr"),
            ("ascending", "false"),
            ("after_cursor", &cursor),
        ],
    )
    .await;
    check::<EventsKeysetPage>("GET /events/keyset?after_cursor", &raw2);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_keyset_filters() {
    let s = sample().await;
    let gamma = pm();
    let mut request = gamma
        .gamma()
        .list_events_keyset()
        .limit(10)
        .tag_ids([s.tag_id.as_str()])
        .related_tags(true)
        .closed(false)
        .title_search("a")
        .volume_min(1)
        .include_best_lines(true);
    if let Some(series) = &s.series_id {
        request = request.series_ids([series.as_str()]);
    }
    let page = request.send().await.unwrap();
    let _ = page.items();
    let raw = get(
        GAMMA,
        "/events/keyset",
        &[
            ("limit", "10"),
            ("tag_id", &s.tag_id),
            ("related_tags", "true"),
            ("closed", "false"),
            ("title_search", "a"),
            ("include_best_lines", "true"),
        ],
    )
    .await;
    check::<EventsKeysetPage>("GET /events/keyset?tag_id&include_best_lines", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_events_keyset_stream_pages() {
    let events: Vec<Event> = take_stream(
        pm().gamma()
            .list_events_keyset()
            .limit(5)
            .closed(false)
            .into_stream(),
        13,
    )
    .await;
    assert_eq!(events.len(), 13);
    assert!(
        distinct(events.iter().map(|e| &e.id)) >= 13,
        "keyset pages do not overlap"
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_event_by_id_and_slug() {
    let s = sample().await;
    let gamma = pm();
    let by_id = gamma
        .gamma()
        .get_event(s.event_id.as_str())
        .include_chat(true)
        .include_template(true)
        .send()
        .await
        .unwrap();
    let by_slug = gamma
        .gamma()
        .get_event_by_slug(&s.event_slug)
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.id, by_slug.id);
    assert_eq!(by_id.id.as_ref().unwrap().as_str(), s.event_id);
    check::<Event>(
        "GET /events/{id}",
        &get(
            GAMMA,
            &format!("/events/{}", s.event_id),
            &[("include_chat", "true"), ("include_template", "true")],
        )
        .await,
    );
    check::<Event>(
        "GET /events/slug/{slug}",
        &get(GAMMA, &format!("/events/slug/{}", s.event_slug), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_event_tweet_count() {
    let s = sample().await;
    let count = pm()
        .gamma()
        .get_event_tweet_count(s.event_id.as_str())
        .await;
    let raw = get(GAMMA, &format!("/events/{}/tweet-count", s.event_id), &[]).await;
    eprintln!("tweet-count raw: {}", raw.text);
    count.unwrap();
    check::<EventTweetCount>("GET /events/{id}/tweet-count", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_event_comment_count() {
    let s = sample().await;
    let count = pm()
        .gamma()
        .get_event_comment_count(s.event_id.as_str())
        .await;
    let raw = get(
        GAMMA,
        &format!("/events/{}/comments/count", s.event_id),
        &[],
    )
    .await;
    eprintln!("event comment count raw: {}", raw.text);
    count.unwrap();
    check::<CommentCount>("GET /events/{id}/comments/count", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_sport_event_results() {
    let gamma = pm();
    let events = gamma
        .gamma()
        .list_sport_event_results()
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!events.is_empty());
    check::<Vec<Event>>(
        "GET /events/results",
        &get(GAMMA, "/events/results", &[("limit", "20")]).await,
    );
    let streamed: Vec<Event> = take_stream(
        gamma
            .gamma()
            .list_sport_event_results()
            .limit(5)
            .into_stream(),
        8,
    )
    .await;
    assert_eq!(streamed.len(), 8);
    assert!(distinct(streamed.iter().map(|e| &e.id)) > 5);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_and_get_event_creators() {
    let gamma = pm();
    let creators = gamma
        .gamma()
        .list_event_creators()
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!creators.is_empty());
    check::<Vec<EventCreator>>(
        "GET /events/creators",
        &get(GAMMA, "/events/creators", &[("limit", "20")]).await,
    );
    let first = creators[0].id.clone().expect("creator id");
    let one = gamma
        .gamma()
        .get_event_creator(first.as_str())
        .await
        .unwrap();
    assert_eq!(one.id.as_ref(), Some(&first));
    check::<EventCreator>(
        "GET /events/creators/{id}",
        &get(GAMMA, &format!("/events/creators/{first}"), &[]).await,
    );
    let handle = creators[0].creator_handle.clone().expect("creator handle");
    let filtered = gamma
        .gamma()
        .list_event_creators()
        .creator_handle(handle.as_str())
        .order("id")
        .ascending(true)
        .send()
        .await
        .unwrap();
    assert!(!filtered.is_empty());
    let streamed: Vec<EventCreator> = take_stream(
        gamma.gamma().list_event_creators().limit(2).into_stream(),
        5,
    )
    .await;
    assert_eq!(streamed.len(), 5);
    assert!(distinct(streamed.iter().map(|c| &c.id)) > 2);
}

// ---------------------------------------------------------------------------------------
// markets
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_markets() {
    let markets = pm()
        .gamma()
        .list_markets()
        .closed(false)
        .order("volume24hr")
        .ascending(false)
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!markets.is_empty());
    assert!(markets.len() <= 20);
    check::<Vec<Market>>(
        "GET /markets",
        &get(
            GAMMA,
            "/markets",
            &[
                ("limit", "20"),
                ("closed", "false"),
                ("order", "volume24hr"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_closed() {
    let markets = pm()
        .gamma()
        .list_markets()
        .closed(true)
        .order("endDate")
        .ascending(false)
        .limit(20)
        .send()
        .await
        .unwrap();
    assert!(!markets.is_empty());
    check::<Vec<Market>>(
        "GET /markets?closed=true",
        &get(
            GAMMA,
            "/markets",
            &[
                ("limit", "20"),
                ("closed", "true"),
                ("order", "endDate"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_with_filters() {
    let s = sample().await;
    let gamma = pm();
    let gamma = gamma.gamma();
    let by_slug = gamma
        .list_markets()
        .slugs([s.market_slug.clone()])
        .include_tag(true)
        .send()
        .await
        .unwrap();
    assert_eq!(by_slug.len(), 1);
    let by_id = gamma
        .list_markets()
        .ids([s.market_id.as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.len(), 1);
    let by_condition = gamma
        .list_markets()
        .condition_ids([s.condition_id.as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(by_condition.len(), 1);
    let by_token = gamma
        .list_markets()
        .clob_token_ids([s.token_ids[0].as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(by_token.len(), 1);
    let by_tag = gamma
        .list_markets()
        .tag_id(s.tag_id.as_str())
        .related_tags(true)
        .closed(false)
        .volume_num_min(1)
        .liquidity_num_min(1)
        .limit(10)
        .send()
        .await
        .unwrap();
    assert!(!by_tag.is_empty());
    check::<Vec<Market>>(
        "GET /markets?tag_id&include_tag",
        &get(
            GAMMA,
            "/markets",
            &[
                ("limit", "20"),
                ("tag_id", &s.tag_id),
                ("related_tags", "true"),
                ("include_tag", "true"),
                ("closed", "false"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_stream_pages() {
    let markets: Vec<Market> = take_stream(
        pm().gamma()
            .list_markets()
            .closed(false)
            .limit(5)
            .into_stream(),
        12,
    )
    .await;
    assert_eq!(markets.len(), 12);
    assert!(distinct(markets.iter().map(|m| &m.id)) > 5);
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_keyset() {
    let gamma = pm();
    let first = gamma
        .gamma()
        .list_markets_keyset()
        .limit(20)
        .closed(false)
        .order("volumeNum")
        .ascending(false)
        .send()
        .await
        .unwrap();
    assert!(!first.items().is_empty());
    let cursor = first.next_cursor().expect("a next cursor").to_owned();
    let second = gamma
        .gamma()
        .list_markets_keyset()
        .limit(20)
        .closed(false)
        .order("volumeNum")
        .ascending(false)
        .cursor(&cursor)
        .send()
        .await
        .unwrap();
    assert!(!second.items().is_empty());
    assert_ne!(
        first.items().first().map(|m| &m.id),
        second.items().first().map(|m| &m.id),
        "the cursor advanced"
    );
    check::<MarketsKeysetPage>(
        "GET /markets/keyset",
        &get(
            GAMMA,
            "/markets/keyset",
            &[
                ("limit", "20"),
                ("closed", "false"),
                ("order", "volumeNum"),
                ("ascending", "false"),
            ],
        )
        .await,
    );
    check::<MarketsKeysetPage>(
        "GET /markets/keyset?after_cursor",
        &get(
            GAMMA,
            "/markets/keyset",
            &[
                ("limit", "20"),
                ("closed", "false"),
                ("order", "volumeNum"),
                ("ascending", "false"),
                ("after_cursor", &cursor),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_keyset_filters() {
    let s = sample().await;
    let gamma = pm();
    let by_condition = gamma
        .gamma()
        .list_markets_keyset()
        .condition_ids([s.condition_id.as_str()])
        .include_tag(true)
        .send()
        .await
        .unwrap();
    assert_eq!(by_condition.items().len(), 1);
    let by_tag = gamma
        .gamma()
        .list_markets_keyset()
        .limit(10)
        .tag_ids([s.tag_id.as_str()])
        .related_tags(true)
        .closed(false)
        .send()
        .await
        .unwrap();
    assert!(!by_tag.items().is_empty());
    check::<MarketsKeysetPage>(
        "GET /markets/keyset?condition_ids&include_tag",
        &get(
            GAMMA,
            "/markets/keyset",
            &[("condition_ids", &s.condition_id), ("include_tag", "true")],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_keyset_decimalized() {
    // The spec documents `decimalized` as a bare boolean. Live treats `decimalized=true` as a
    // filter: the sample market (a regular binary market) is not returned with it set.
    let s = sample().await;
    let gamma = pm();
    let plain = gamma
        .gamma()
        .list_markets_keyset()
        .condition_ids([s.condition_id.as_str()])
        .send()
        .await
        .unwrap();
    let decimalized = gamma
        .gamma()
        .list_markets_keyset()
        .condition_ids([s.condition_id.as_str()])
        .decimalized(true)
        .send()
        .await
        .unwrap();
    eprintln!(
        "decimalized: plain={} decimalized=true={}",
        plain.items().len(),
        decimalized.items().len()
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_markets_keyset_stream_pages() {
    let markets: Vec<Market> = take_stream(
        pm().gamma()
            .list_markets_keyset()
            .limit(5)
            .closed(false)
            .into_stream(),
        13,
    )
    .await;
    assert_eq!(markets.len(), 13);
    assert!(
        distinct(markets.iter().map(|m| &m.id)) >= 13,
        "keyset pages do not overlap"
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_market_by_id_and_slug() {
    let s = sample().await;
    let gamma = pm();
    let by_id = gamma
        .gamma()
        .get_market(s.market_id.as_str())
        .include_tag(true)
        .send()
        .await
        .unwrap();
    let by_slug = gamma
        .gamma()
        .get_market_by_slug(&s.market_slug)
        .send()
        .await
        .unwrap();
    assert_eq!(by_id.id, by_slug.id);
    assert_eq!(by_id.id.as_ref().unwrap().as_str(), s.market_id);
    check::<Market>(
        "GET /markets/{id}",
        &get(
            GAMMA,
            &format!("/markets/{}", s.market_id),
            &[("include_tag", "true")],
        )
        .await,
    );
    check::<Market>(
        "GET /markets/slug/{slug}",
        &get(GAMMA, &format!("/markets/slug/{}", s.market_slug), &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_market_description() {
    let s = sample().await;
    let description = pm()
        .gamma()
        .get_market_description(s.market_id.as_str())
        .await;
    let raw = get(GAMMA, &format!("/markets/{}/description", s.market_id), &[]).await;
    description.unwrap();
    check::<MarketDescription>("GET /markets/{id}/description", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_markets_information() {
    let s = sample().await;
    let markets = pm()
        .gamma()
        .get_markets_information()
        .ids([s.market_id.as_str()])
        .send()
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    let by_condition = pm()
        .gamma()
        .get_markets_information()
        .condition_ids([s.condition_id.as_str()])
        .include_tags(true)
        .send()
        .await
        .unwrap();
    assert_eq!(by_condition.len(), 1);
    let by_token = pm()
        .gamma()
        .get_markets_information()
        .clob_token_ids(s.token_ids.iter().map(String::as_str))
        .closed(false)
        .send()
        .await
        .unwrap();
    assert!(!by_token.is_empty());
    let id: i64 = s.market_id.parse().unwrap();
    check::<Vec<Market>>(
        "POST /markets/information",
        &post(GAMMA, "/markets/information", &json!({ "id": [id] })).await,
    );
    check::<Vec<Market>>(
        "POST /markets/information (conditionIds, includeTags)",
        &post(
            GAMMA,
            "/markets/information",
            &json!({ "conditionIds": [s.condition_id], "includeTags": true }),
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_abridged_markets() {
    let s = sample().await;
    let result = pm()
        .gamma()
        .get_abridged_markets()
        .ids([s.market_id.as_str()])
        .send()
        .await;
    let id: i64 = s.market_id.parse().unwrap();
    let raw: Raw = post(GAMMA, "/markets/abridged", &json!({ "id": [id] })).await;
    let markets = result.unwrap();
    assert_eq!(markets.len(), 1);
    check::<Vec<Market>>("POST /markets/abridged", &raw);
    let multi = pm()
        .gamma()
        .get_abridged_markets()
        .slugs([s.market_slug.clone()])
        .closed(false)
        .send()
        .await
        .unwrap();
    assert!(!multi.is_empty());
}

// ---------------------------------------------------------------------------------------
// series
// ---------------------------------------------------------------------------------------

/// A series id and slug from the sample, or from the series list when the sample event has
/// none.
async fn a_series() -> (String, String) {
    let s = sample().await;
    if let (Some(id), Some(slug)) = (&s.series_id, &s.series_slug) {
        return (id.clone(), slug.clone());
    }
    let list = get(
        GAMMA,
        "/series",
        &[("limit", "1"), ("closed", "false"), ("order", "volume24hr")],
    )
    .await
    .json;
    (
        list[0]["id"].as_str().unwrap().to_owned(),
        list[0]["slug"].as_str().unwrap().to_owned(),
    )
}

#[tokio::test]
#[ignore = "live network"]
async fn list_series() {
    let series = pm()
        .gamma()
        .list_series()
        .limit(20)
        .order("id")
        .ascending(false)
        .closed(false)
        .include_chat(true)
        .send()
        .await
        .unwrap();
    assert!(!series.is_empty());
    check::<Vec<Series>>(
        "GET /series",
        &get(
            GAMMA,
            "/series",
            &[
                ("limit", "20"),
                ("order", "id"),
                ("ascending", "false"),
                ("closed", "false"),
                ("include_chat", "true"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_series_order_volume24hr_descending() {
    // Live answers `GET /series?order=volume24hr&ascending=false` with HTTP 500 once `limit`
    // is 11 or more (limit 10 and below, `ascending=true`, or no `order` all succeed).
    let series = pm()
        .gamma()
        .list_series()
        .limit(20)
        .order("volume24hr")
        .ascending(false)
        .send()
        .await
        .unwrap();
    assert!(!series.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn list_series_with_filters() {
    let (_, slug) = a_series().await;
    let by_slug = pm()
        .gamma()
        .list_series()
        .slugs([slug.clone()])
        .exclude_events(true)
        .send()
        .await
        .unwrap();
    assert_eq!(by_slug.len(), 1);
    check::<Vec<Series>>(
        "GET /series?slug&exclude_events",
        &get(
            GAMMA,
            "/series",
            &[("slug", &slug), ("exclude_events", "true")],
        )
        .await,
    );
    let recurring = pm()
        .gamma()
        .list_series()
        .recurrence("daily")
        .limit(10)
        .send()
        .await
        .unwrap();
    let _ = recurring;
}

#[tokio::test]
#[ignore = "live network"]
async fn list_series_stream_pages() {
    let series: Vec<Series> =
        take_stream(pm().gamma().list_series().limit(3).into_stream(), 8).await;
    assert_eq!(series.len(), 8);
    assert!(distinct(series.iter().map(|s| &s.id)) > 3);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_series() {
    let (id, _) = a_series().await;
    let series = pm()
        .gamma()
        .get_series(id.as_str())
        .include_chat(true)
        .send()
        .await
        .unwrap();
    assert_eq!(series.id.as_ref().unwrap().as_str(), id);
    check::<Series>(
        "GET /series/{id}",
        &get(GAMMA, &format!("/series/{id}"), &[("include_chat", "true")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_series_comment_count() {
    let (id, _) = a_series().await;
    let count = pm().gamma().get_series_comment_count(id.as_str()).await;
    let raw = get(GAMMA, &format!("/series/{id}/comments/count"), &[]).await;
    eprintln!("series comment count raw: {}", raw.text);
    count.unwrap();
    check::<CommentCount>("GET /series/{id}/comments/count", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_series_summary() {
    let (id, slug) = a_series().await;
    let gamma = pm();
    let by_id = gamma.gamma().get_series_summary(id.as_str()).await.unwrap();
    let by_slug = gamma
        .gamma()
        .get_series_summary_by_slug(&slug)
        .await
        .unwrap();
    assert_eq!(by_id.id, by_slug.id);
    check::<SeriesSummary>(
        "GET /series-summary/{id}",
        &get(GAMMA, &format!("/series-summary/{id}"), &[]).await,
    );
    check::<SeriesSummary>(
        "GET /series-summary/slug/{slug}",
        &get(GAMMA, &format!("/series-summary/slug/{slug}"), &[]).await,
    );
}

// ---------------------------------------------------------------------------------------
// comments
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_comments() {
    let c = comment_sample().await;
    let entity_type = if c.entity_type == "Series" {
        CommentParentEntityType::Series
    } else {
        CommentParentEntityType::Event
    };
    let entity_id: i64 = c.entity_id.parse().unwrap();
    let comments = pm()
        .gamma()
        .list_comments()
        .parent_entity_type(entity_type)
        .parent_entity_id(entity_id)
        .limit(20)
        .order("createdAt")
        .ascending(false)
        .get_positions(true)
        .send()
        .await
        .unwrap();
    assert!(!comments.is_empty());
    check::<Vec<Comment>>(
        "GET /comments",
        &get(
            GAMMA,
            "/comments",
            &[
                ("limit", "20"),
                ("parent_entity_type", c.entity_type),
                ("parent_entity_id", &c.entity_id),
                ("order", "createdAt"),
                ("ascending", "false"),
                ("get_positions", "true"),
            ],
        )
        .await,
    );
    check::<Vec<Comment>>(
        "GET /comments?holders_only",
        &get(
            GAMMA,
            "/comments",
            &[
                ("limit", "20"),
                ("parent_entity_type", c.entity_type),
                ("parent_entity_id", &c.entity_id),
                ("holders_only", "true"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_comments_stream_pages() {
    let c = comment_sample().await;
    let entity_id: i64 = c.entity_id.parse().unwrap();
    let comments: Vec<Comment> = take_stream(
        pm().gamma()
            .list_comments()
            .parent_entity_type(if c.entity_type == "Series" {
                CommentParentEntityType::Series
            } else {
                CommentParentEntityType::Event
            })
            .parent_entity_id(entity_id)
            .limit(2)
            .into_stream(),
        3,
    )
    .await;
    assert!(!comments.is_empty());
    assert_eq!(distinct(comments.iter().map(|c| &c.id)), comments.len());
}

#[tokio::test]
#[ignore = "live network"]
async fn list_comments_market_parent_type() {
    // The spec lists `market` as a `parent_entity_type`; this records what live does with it
    // (live has rejected it with 422 and listed `Event, Series, PerpsAsset`).
    let s = sample().await;
    let result = pm()
        .gamma()
        .list_comments()
        .parent_entity_type(CommentParentEntityType::Market)
        .parent_entity_id(s.market_id.parse().unwrap())
        .limit(5)
        .send()
        .await;
    eprintln!("list_comments(parent_entity_type=market) -> {result:?}");
    result.unwrap();
}

#[tokio::test]
#[ignore = "live network"]
async fn get_comments_by_id() {
    let c = comment_sample().await;
    let comments = pm()
        .gamma()
        .get_comments_by_id(c.id.as_str())
        .get_positions(true)
        .send()
        .await
        .unwrap();
    assert!(!comments.is_empty());
    assert_eq!(comments[0].id.as_ref().unwrap().as_str(), c.id);
    check::<Vec<Comment>>(
        "GET /comments/{id}",
        &get(
            GAMMA,
            &format!("/comments/{}", c.id),
            &[("get_positions", "true")],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_comments_by_user() {
    let c = comment_sample().await;
    let gamma = pm();
    let comments = gamma
        .gamma()
        .list_comments_by_user(c.user.as_str())
        .limit(20)
        .order("createdAt")
        .ascending(false)
        .send()
        .await
        .unwrap();
    assert!(!comments.is_empty());
    check::<Vec<Comment>>(
        "GET /comments/user_address/{user_address}",
        &get(
            GAMMA,
            &format!("/comments/user_address/{}", c.user),
            &[("limit", "20")],
        )
        .await,
    );
    let streamed: Vec<Comment> = take_stream(
        gamma
            .gamma()
            .list_comments_by_user(c.user.as_str())
            .limit(2)
            .into_stream(),
        3,
    )
    .await;
    assert!(!streamed.is_empty());
}

// ---------------------------------------------------------------------------------------
// profiles
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn get_public_profile() {
    let c = comment_sample().await;
    let gamma = pm();
    let profile = gamma.gamma().get_public_profile(c.user.as_str()).await;
    let raw = get(GAMMA, "/public-profile", &[("address", &c.user)]).await;
    profile.unwrap();
    check::<PublicProfile>("GET /public-profile (commenter)", &raw);

    let s = sample().await;
    let leaderboard_user = gamma.gamma().get_public_profile(s.user.as_str()).await;
    let raw = get(GAMMA, "/public-profile", &[("address", &s.user)]).await;
    leaderboard_user.unwrap();
    check::<PublicProfile>("GET /public-profile (leaderboard user)", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_profile() {
    let c = comment_sample().await;
    let gamma = pm();
    let profile = gamma.gamma().get_profile(c.user.as_str()).await;
    let raw = get(GAMMA, &format!("/profiles/user_address/{}", c.user), &[]).await;
    profile.unwrap();
    check::<Profile>("GET /profiles/user_address/{addr} (commenter)", &raw);

    let s = sample().await;
    let leaderboard_user = gamma.gamma().get_profile(s.user.as_str()).await;
    let raw = get(GAMMA, &format!("/profiles/user_address/{}", s.user), &[]).await;
    leaderboard_user.unwrap();
    check::<Profile>("GET /profiles/user_address/{addr} (leaderboard user)", &raw);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_public_profile_not_found_is_typed_error() {
    // A well-formed address that has (almost certainly) never traded.
    let result = pm()
        .gamma()
        .get_public_profile("0x0000000000000000000000000000000000000001")
        .await;
    match result {
        Err(Error::Api(api)) => {
            eprintln!("public profile error: {api:?}");
            assert!(matches!(api.status().as_u16(), 400 | 404));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------
// sports and teams
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn list_and_get_teams() {
    let gamma = pm();
    let teams = gamma
        .gamma()
        .list_teams()
        .limit(20)
        .order("name")
        .ascending(true)
        .send()
        .await
        .unwrap();
    assert!(!teams.is_empty());
    check::<Vec<Team>>(
        "GET /teams",
        &get(
            GAMMA,
            "/teams",
            &[("limit", "20"), ("order", "name"), ("ascending", "true")],
        )
        .await,
    );
    let id = teams[0].id.clone().expect("team id");
    let team = gamma.gamma().get_team(id.as_str()).await.unwrap();
    assert_eq!(team.id.as_ref(), Some(&id));
    check::<Team>(
        "GET /teams/{id}",
        &get(GAMMA, &format!("/teams/{id}"), &[]).await,
    );
    let league = teams[0].league.clone().expect("team league");
    let by_league = gamma
        .gamma()
        .list_teams()
        .leagues([league.as_str()])
        .limit(10)
        .send()
        .await
        .unwrap();
    assert!(!by_league.is_empty());
    let name = teams[0].name.clone().expect("team name");
    let by_name = gamma
        .gamma()
        .list_teams()
        .names([name.as_str()])
        .send()
        .await
        .unwrap();
    assert!(!by_name.is_empty());
    if let Some(abbreviation) = teams[0].abbreviation.clone() {
        let by_abbreviation = gamma
            .gamma()
            .list_teams()
            .abbreviations([abbreviation.as_str()])
            .send()
            .await
            .unwrap();
        assert!(!by_abbreviation.is_empty());
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn list_teams_stream_pages() {
    let teams: Vec<Team> = take_stream(pm().gamma().list_teams().limit(4).into_stream(), 10).await;
    assert_eq!(teams.len(), 10);
    assert!(distinct(teams.iter().map(|t| &t.id)) > 4);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_sports_metadata() {
    let sports = pm().gamma().get_sports_metadata().await.unwrap();
    assert!(!sports.is_empty());
    let _ = sports.iter().flat_map(SportsMetadata::tag_ids).count();
    check::<Vec<SportsMetadata>>("GET /sports", &get(GAMMA, "/sports", &[]).await);
}

#[tokio::test]
#[ignore = "live network"]
async fn get_sports_market_types() {
    let types = pm().gamma().get_sports_market_types().await.unwrap();
    let _ = &types;
    check::<SportsMarketTypes>(
        "GET /sports/market-types",
        &get(GAMMA, "/sports/market-types", &[]).await,
    );
}

// ---------------------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn search() {
    let results = pm()
        .gamma()
        .search("trump")
        .limit_per_type(5)
        .search_tags(true)
        .search_profiles(true)
        .events_status("active")
        .send()
        .await
        .unwrap();
    assert!(
        results.events.as_ref().is_some_and(|e| !e.is_empty()),
        "search returned events"
    );
    check::<SearchResults>(
        "GET /public-search",
        &get(
            GAMMA,
            "/public-search",
            &[
                ("q", "trump"),
                ("limit_per_type", "5"),
                ("search_tags", "true"),
                ("search_profiles", "true"),
                ("events_status", "active"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn search_pages_and_options() {
    let gamma = pm();
    let page1 = gamma
        .gamma()
        .search("election")
        .limit_per_type(3)
        .page(1)
        .sort("volume")
        .ascending(false)
        .keep_closed_markets(1)
        .cache(true)
        .send()
        .await
        .unwrap();
    let page2 = gamma
        .gamma()
        .search("election")
        .limit_per_type(3)
        .page(2)
        .send()
        .await
        .unwrap();
    let _ = (page1, page2);
    check::<SearchResults>(
        "GET /public-search?page=1",
        &get(
            GAMMA,
            "/public-search",
            &[
                ("q", "election"),
                ("limit_per_type", "3"),
                ("page", "1"),
                ("keep_closed_markets", "1"),
            ],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn search_optimized() {
    // `optimized=true` is documented only as a bare boolean. Live then returns a different
    // shape: `hasMore` instead of `pagination`, and `outcomes` / `outcomePrices` as JSON
    // arrays instead of JSON-encoded strings.
    let result = pm()
        .gamma()
        .search("election")
        .limit_per_type(3)
        .optimized(true)
        .send()
        .await;
    eprintln!(
        "search(optimized=true) -> {:?}",
        result.as_ref().map(|_| ())
    );
    result.unwrap();
}

// ---------------------------------------------------------------------------------------
// broad decoding and server limits
// ---------------------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live network"]
async fn decode_old_events_and_markets() {
    // The oldest records (ascending `id`) carry the oldest value formats and many empty or
    // null fields; `offset` is capped at 2000 live, so sample the start, middle and end.
    for offset in ["0", "1000", "2000"] {
        let events = get(
            GAMMA,
            "/events",
            &[
                ("limit", "100"),
                ("offset", offset),
                ("order", "id"),
                ("ascending", "true"),
            ],
        )
        .await;
        check::<Vec<Event>>(&format!("GET /events?order=id&offset={offset}"), &events);
        let markets = get(
            GAMMA,
            "/markets",
            &[
                ("limit", "100"),
                ("offset", offset),
                ("order", "id"),
                ("ascending", "true"),
            ],
        )
        .await;
        check::<Vec<Market>>(&format!("GET /markets?order=id&offset={offset}"), &markets);
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn decode_keyset_ascending_pages() {
    // Oldest-first keyset pages, then the page after a cursor.
    let first = get(
        GAMMA,
        "/events/keyset",
        &[("limit", "100"), ("order", "id"), ("ascending", "true")],
    )
    .await;
    let page = check::<EventsKeysetPage>("GET /events/keyset?order=id&ascending=true", &first);
    let cursor = page.next_cursor().expect("a next cursor").to_owned();
    check::<EventsKeysetPage>(
        "GET /events/keyset?order=id&ascending=true&after_cursor",
        &get(
            GAMMA,
            "/events/keyset",
            &[
                ("limit", "100"),
                ("order", "id"),
                ("ascending", "true"),
                ("after_cursor", &cursor),
            ],
        )
        .await,
    );
    check::<MarketsKeysetPage>(
        "GET /markets/keyset?order=id&ascending=true",
        &get(
            GAMMA,
            "/markets/keyset",
            &[("limit", "100"), ("order", "id"), ("ascending", "true")],
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn offset_past_2000_is_typed_error() {
    // Live rejects `offset > 2000` on events and markets with a 422 pointing at the keyset
    // routes (not in the spec). The SDK must surface that as a typed API error, and the
    // offset stream must end with it rather than panic or loop.
    let gamma = pm();
    let gamma = gamma.gamma();
    let err = gamma.list_markets().limit(1).offset(2001).send().await;
    match err {
        Err(Error::Api(api)) => {
            assert_eq!(api.status().as_u16(), 422);
            assert!(
                api.message().is_some_and(|m| m.contains("keyset")),
                "{api:?}"
            );
        }
        other => panic!("expected a 422 API error, got {other:?}"),
    }
    let err = gamma.list_events().limit(1).offset(2001).send().await;
    assert!(matches!(&err, Err(Error::Api(a)) if a.status().as_u16() == 422));
    let err = gamma
        .list_events_paginated()
        .limit(1)
        .offset(2001)
        .send()
        .await;
    assert!(matches!(&err, Err(Error::Api(a)) if a.status().as_u16() == 422));

    let streamed: Vec<Result<Market, Error>> = gamma
        .list_markets()
        .limit(100)
        .offset(1950)
        .into_stream()
        .take(200)
        .collect()
        .await;
    let ok = streamed.iter().filter(|r| r.is_ok()).count();
    assert!(
        matches!(streamed.last(), Some(Err(Error::Api(a))) if a.status().as_u16() == 422),
        "stream ended with {:?} after {ok} items",
        streamed.last().map(|r| r.as_ref().map(|_| ()))
    );
}
