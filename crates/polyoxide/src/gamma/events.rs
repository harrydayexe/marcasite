//! Events: `/events`, `/events/{id}`, `/events/slug/{slug}`, `/events/{id}/tags`,
//! `/events/{id}/tweet-count`, `/events/{id}/comments/count`, `/events/keyset`,
//! `/events/pagination`, `/events/results` and `/events/creators`.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{
    Query, Result, ValidationError,
    pagination::{CursorPage, cursor_stream, offset_stream},
    serde_util,
    types::EventId,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    Category, Chat, Collection, CommentCount, GammaClient, ImageOptimization, Market, Pagination,
    Series, SeriesId, Tag, TagId, Template,
    util::{Lookup, PageParams, rfc3339, setters, validate_keyset_limit},
};

polyoxide_core::string_id! {
    /// An event creator id.
    pub struct EventCreatorId;
}

/// An event: a group of related markets (`components/schemas/Event`).
///
/// Every field is optional because the spec marks none as required. Fields the spec types
/// as `number` are [`Decimal`]s.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Event {
    /// Event id.
    pub id: Option<EventId>,
    /// Ticker.
    pub ticker: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Resolution source.
    pub resolution_source: Option<String>,
    /// Start date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub start_date: Option<DateTime<Utc>>,
    /// Creation date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub creation_date: Option<DateTime<Utc>>,
    /// End date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub end_date: Option<DateTime<Utc>>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Whether the event is active.
    pub active: Option<bool>,
    /// Whether the event is closed.
    pub closed: Option<bool>,
    /// Whether the event is archived.
    pub archived: Option<bool>,
    /// Whether the event is flagged as new.
    pub new: Option<bool>,
    /// Whether the event is featured.
    pub featured: Option<bool>,
    /// Whether the event is restricted.
    pub restricted: Option<bool>,
    /// Liquidity.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub liquidity: Option<Decimal>,
    /// Volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume: Option<Decimal>,
    /// Open interest.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub open_interest: Option<Decimal>,
    /// Sort order of the event's markets.
    pub sort_by: Option<String>,
    /// Category.
    pub category: Option<String>,
    /// Subcategory.
    pub subcategory: Option<String>,
    /// Whether the event is a template.
    pub is_template: Option<bool>,
    /// Template variables. The spec types this as a plain string.
    pub template_variables: Option<String>,
    /// Publication time (wire name `published_at`). The spec types this as a plain string
    /// with no format.
    #[serde(rename = "published_at")]
    pub published_at: Option<String>,
    /// Creator. The spec types this as a string on events.
    pub created_by: Option<String>,
    /// Last updater. The spec types this as a string on events.
    pub updated_by: Option<String>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// Competitiveness score.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub competitive: Option<Decimal>,
    /// 24-hour volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_24hr: Option<Decimal>,
    /// 1-week volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1wk: Option<Decimal>,
    /// 1-month volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1mo: Option<Decimal>,
    /// 1-year volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_1yr: Option<Decimal>,
    /// Featured image URL.
    pub featured_image: Option<String>,
    /// Disqus thread.
    pub disqus_thread: Option<String>,
    /// Parent event. The spec types this as a plain string.
    pub parent_event: Option<String>,
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// CLOB liquidity.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub liquidity_clob: Option<Decimal>,
    /// Whether the event uses negative risk.
    pub neg_risk: Option<bool>,
    /// Negative-risk market id (wire name `negRiskMarketID`).
    #[serde(rename = "negRiskMarketID")]
    pub neg_risk_market_id: Option<String>,
    /// Negative-risk fee in basis points.
    pub neg_risk_fee_bips: Option<i64>,
    /// Number of comments.
    pub comment_count: Option<i64>,
    /// Optimized image metadata.
    pub image_optimized: Option<ImageOptimization>,
    /// Optimized icon metadata.
    pub icon_optimized: Option<ImageOptimization>,
    /// Optimized featured image metadata.
    pub featured_image_optimized: Option<ImageOptimization>,
    /// Sub-events.
    pub sub_events: Option<Vec<String>>,
    /// The event's markets.
    pub markets: Option<Vec<Market>>,
    /// The series the event belongs to.
    pub series: Option<Vec<Series>>,
    /// Categories.
    pub categories: Option<Vec<Category>>,
    /// Collections.
    pub collections: Option<Vec<Collection>>,
    /// Tags.
    pub tags: Option<Vec<Tag>>,
    /// Whether this is a "create your own market" event.
    pub cyom: Option<bool>,
    /// Close time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub closed_time: Option<DateTime<Utc>>,
    /// Whether all outcomes are shown.
    pub show_all_outcomes: Option<bool>,
    /// Whether market images are shown.
    pub show_market_images: Option<bool>,
    /// Whether the event resolves automatically.
    pub automatically_resolved: Option<bool>,
    /// Whether negative risk is enabled.
    pub enable_neg_risk: Option<bool>,
    /// Whether the event activates automatically.
    pub automatically_active: Option<bool>,
    /// Event date. The spec types this as a plain string with no format.
    pub event_date: Option<String>,
    /// Start time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub start_time: Option<DateTime<Utc>>,
    /// Event week.
    pub event_week: Option<i64>,
    /// Series slug.
    pub series_slug: Option<String>,
    /// Score (for sports events).
    pub score: Option<String>,
    /// Elapsed time (for sports events).
    pub elapsed: Option<String>,
    /// Period (for sports events).
    pub period: Option<String>,
    /// Whether the event is live.
    pub live: Option<bool>,
    /// Whether the event has ended.
    pub ended: Option<bool>,
    /// When the event finished.
    #[serde(default, with = "serde_util::datetime_option")]
    pub finished_timestamp: Option<DateTime<Utc>>,
    /// GMP chart mode.
    pub gmp_chart_mode: Option<String>,
    /// Event creators.
    pub event_creators: Option<Vec<EventCreator>>,
    /// Number of tweets.
    pub tweet_count: Option<i64>,
    /// Chats (included with `include_chat=true`).
    pub chats: Option<Vec<Chat>>,
    /// Featured order.
    pub featured_order: Option<i64>,
    /// Whether to estimate the value.
    pub estimate_value: Option<bool>,
    /// Whether the value cannot be estimated.
    pub cant_estimate: Option<bool>,
    /// Estimated value. The spec types this as a plain string.
    pub estimated_value: Option<String>,
    /// Templates (included with `include_template=true`).
    pub templates: Option<Vec<Template>>,
    /// Main spreads line (for sports events).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub spreads_main_line: Option<Decimal>,
    /// Main totals line (for sports events).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub totals_main_line: Option<Decimal>,
    /// Carousel map. The spec types this as a plain string.
    pub carousel_map: Option<String>,
    /// Whether deployment is pending.
    pub pending_deployment: Option<bool>,
    /// Whether the event is being deployed.
    pub deploying: Option<bool>,
    /// When deployment started.
    #[serde(default, with = "serde_util::datetime_option")]
    pub deploying_timestamp: Option<DateTime<Utc>>,
    /// When deployment is scheduled.
    #[serde(default, with = "serde_util::datetime_option")]
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    /// Game status (for sports events).
    pub game_status: Option<String>,
}

/// The creator of an event (`components/schemas/EventCreator`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EventCreator {
    /// Creator id.
    pub id: Option<EventCreatorId>,
    /// Creator name.
    pub creator_name: Option<String>,
    /// Creator handle.
    pub creator_handle: Option<String>,
    /// Creator URL.
    pub creator_url: Option<String>,
    /// Creator image URL.
    pub creator_image: Option<String>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// An event's tweet count (`components/schemas/EventTweetCount`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EventTweetCount {
    /// Number of tweets.
    pub tweet_count: Option<i64>,
}

/// One page of [`GammaClient::list_events_paginated`] (`components/schemas/EventsPagination`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventsPage {
    /// The events on this page.
    pub data: Option<Vec<Event>>,
    /// Pagination metadata.
    pub pagination: Option<Pagination>,
}

/// One page of [`GammaClient::list_events_keyset`] (`components/schemas/KeysetEventsResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventsKeysetPage {
    /// The events on this page (empty if none were found).
    pub events: Option<Vec<Event>>,
    /// Cursor for the next page, passed as
    /// [`after_cursor`](ListEventsKeyset::after_cursor). Present only when the page is
    /// full; absent on the last page.
    pub next_cursor: Option<String>,
}

impl GammaClient {
    /// Lists events (offset pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/events/list-events>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let events = gamma
    ///     .list_events()
    ///     .active(true)
    ///     .closed(false)
    ///     .limit(20)
    ///     .send()
    ///     .await?;
    /// for event in events {
    ///     println!("{:?}: {} markets", event.title, event.markets.map_or(0, |m| m.len()));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_events(&self) -> ListEvents {
        ListEvents {
            client: self.clone(),
            params: ListEventsParams::default(),
        }
    }

    /// Lists events with pagination metadata (`hasMore`, `totalResults`).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `listEventsPagination` (no published
    /// doc page).
    pub fn list_events_paginated(&self) -> ListEventsPaginated {
        ListEventsPaginated {
            client: self.clone(),
            params: ListEventsPaginatedParams::default(),
        }
    }

    /// Lists sport events with their results (offset pagination).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `listSportEventsResults` (no
    /// published doc page).
    pub fn list_sport_event_results(&self) -> ListSportEventResults {
        ListSportEventResults {
            client: self.clone(),
            params: PageParams::default(),
        }
    }

    /// Gets an event by id.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the event does not
    /// exist; see [`Error::is_not_found`](crate::Error::is_not_found).
    ///
    /// See <https://docs.polymarket.com/api-reference/events/get-event-by-id>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let event = gamma.get_event("16167").send().await?;
    /// for market in event.markets.unwrap_or_default() {
    ///     println!("{:?}", market.question);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_event(&self, id: impl Into<EventId>) -> GetEvent {
        GetEvent {
            client: self.clone(),
            lookup: Lookup::Id(id.into()),
            params: GetEventParams::default(),
        }
    }

    /// Gets an event by slug.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the event does not
    /// exist.
    ///
    /// See <https://docs.polymarket.com/api-reference/events/get-event-by-slug>.
    pub fn get_event_by_slug(&self, slug: impl Into<String>) -> GetEvent {
        GetEvent {
            client: self.clone(),
            lookup: Lookup::Slug(slug.into()),
            params: GetEventParams::default(),
        }
    }

    /// Gets the tags attached to an event.
    ///
    /// See <https://docs.polymarket.com/api-reference/events/get-event-tags>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing event is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_event_tags(&self, id: impl Into<EventId>) -> Result<Vec<Tag>> {
        let id = id.into();
        self.transport
            .get(&["events", id.as_str(), "tags"])
            .send()
            .await
    }

    /// Gets an event's tweet count.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getEventTweetCount` (no published
    /// doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing event is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_event_tweet_count(&self, id: impl Into<EventId>) -> Result<EventTweetCount> {
        let id = id.into();
        self.transport
            .get(&["events", id.as_str(), "tweet-count"])
            .send()
            .await
    }

    /// Gets the number of comments on an event.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getEventCommentsCount` (no
    /// published doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing event is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_event_comment_count(&self, id: impl Into<EventId>) -> Result<CommentCount> {
        let id = id.into();
        self.transport
            .get(&["events", id.as_str(), "comments", "count"])
            .send()
            .await
    }

    /// Lists event creators (offset pagination).
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `listEventCreators` (no published
    /// doc page).
    pub fn list_event_creators(&self) -> ListEventCreators {
        ListEventCreators {
            client: self.clone(),
            params: ListEventCreatorsParams::default(),
        }
    }

    /// Gets an event creator by id.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getEventCreator` (no published doc
    /// page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing creator is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_event_creator(&self, id: impl Into<EventCreatorId>) -> Result<EventCreator> {
        let id = id.into();
        self.transport
            .get(&["events", "creators", id.as_str()])
            .send()
            .await
    }

    /// Lists events with cursor-based (keyset) pagination, for stable paging through large
    /// result sets.
    ///
    /// [`send`](ListEventsKeyset::send) returns one [`EventsKeysetPage`]; pass its
    /// `next_cursor` to [`after_cursor`](ListEventsKeyset::after_cursor) for the next page,
    /// or use [`into_stream`](ListEventsKeyset::into_stream) to walk every page.
    ///
    /// See <https://docs.polymarket.com/api-reference/events/list-events-keyset-pagination>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let first = gamma.list_events_keyset().limit(50).closed(false).send().await?;
    /// if let Some(cursor) = first.next_cursor {
    ///     let second = gamma
    ///         .list_events_keyset()
    ///         .limit(50)
    ///         .closed(false)
    ///         .after_cursor(cursor)
    ///         .send()
    ///         .await?;
    ///     # let _ = second;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_events_keyset(&self) -> ListEventsKeyset {
        ListEventsKeyset {
            client: self.clone(),
            params: ListEventsKeysetParams::default(),
        }
    }
}

/// Request builder for [`GammaClient::list_events`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEvents {
    client: GammaClient,
    params: ListEventsParams,
}

#[derive(Debug, Clone, Default)]
struct ListEventsParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    id: Vec<EventId>,
    tag_id: Option<TagId>,
    exclude_tag_id: Vec<TagId>,
    slug: Vec<String>,
    tag_slug: Option<String>,
    related_tags: Option<bool>,
    active: Option<bool>,
    archived: Option<bool>,
    featured: Option<bool>,
    cyom: Option<bool>,
    include_chat: Option<bool>,
    include_template: Option<bool>,
    recurrence: Option<String>,
    closed: Option<bool>,
    liquidity_min: Option<Decimal>,
    liquidity_max: Option<Decimal>,
    volume_min: Option<Decimal>,
    volume_max: Option<Decimal>,
    start_date_min: Option<DateTime<Utc>>,
    start_date_max: Option<DateTime<Utc>>,
    end_date_min: Option<DateTime<Utc>>,
    end_date_max: Option<DateTime<Utc>>,
}

impl ListEventsParams {
    fn query(&self, offset: Option<u64>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("offset", offset)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_all("id", &self.id)
            .push_opt("tag_id", self.tag_id.as_ref())
            .push_all("exclude_tag_id", &self.exclude_tag_id)
            .push_all("slug", &self.slug)
            .push_opt("tag_slug", self.tag_slug.as_deref())
            .push_opt("related_tags", self.related_tags)
            .push_opt("active", self.active)
            .push_opt("archived", self.archived)
            .push_opt("featured", self.featured)
            .push_opt("cyom", self.cyom)
            .push_opt("include_chat", self.include_chat)
            .push_opt("include_template", self.include_template)
            .push_opt("recurrence", self.recurrence.as_deref())
            .push_opt("closed", self.closed)
            .push_opt("liquidity_min", self.liquidity_min)
            .push_opt("liquidity_max", self.liquidity_max)
            .push_opt("volume_min", self.volume_min)
            .push_opt("volume_max", self.volume_max)
            .push_opt("start_date_min", self.start_date_min.as_ref().map(rfc3339))
            .push_opt("start_date_max", self.start_date_max.as_ref().map(rfc3339))
            .push_opt("end_date_min", self.end_date_min.as_ref().map(rfc3339))
            .push_opt("end_date_max", self.end_date_max.as_ref().map(rfc3339));
        q
    }
}

impl ListEvents {
    setters! {
        /// Maximum number of events per page.
        limit: u64;
        /// Number of events to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only events with these ids.
        id: many EventId;
        /// Only events with this tag.
        tag_id: into TagId;
        /// Exclude events with these tags.
        exclude_tag_id: many TagId;
        /// Only events with these slugs.
        slug: many String;
        /// Only events with the tag with this slug.
        tag_slug: into String;
        /// Include events with tags related to [`tag_id`](Self::tag_id).
        related_tags: bool;
        /// Only active (`true`) or only inactive (`false`) events.
        active: bool;
        /// Only archived (`true`) or only unarchived (`false`) events.
        archived: bool;
        /// Only featured (`true`) or only non-featured (`false`) events.
        featured: bool;
        /// Only "create your own market" events (`true`) or only other events (`false`).
        cyom: bool;
        /// Include each event's chats.
        include_chat: bool;
        /// Include each event's templates.
        include_template: bool;
        /// Only events with this recurrence.
        recurrence: into String;
        /// Only closed (`true`) or only open (`false`) events.
        closed: bool;
        /// Minimum liquidity.
        liquidity_min: into Decimal;
        /// Maximum liquidity.
        liquidity_max: into Decimal;
        /// Minimum volume.
        volume_min: into Decimal;
        /// Maximum volume.
        volume_max: into Decimal;
        /// Earliest start date.
        start_date_min: DateTime<Utc>;
        /// Latest start date.
        start_date_max: DateTime<Utc>;
        /// Earliest end date.
        end_date_min: DateTime<Utc>;
        /// Latest end date.
        end_date_max: DateTime<Utc>;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Event>> {
        self.client
            .transport
            .get(&["events"])
            .query(self.params.query(offset))
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Event>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::list_events_paginated`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventsPaginated {
    client: GammaClient,
    params: ListEventsPaginatedParams,
}

#[derive(Debug, Clone, Default)]
struct ListEventsPaginatedParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    include_chat: Option<bool>,
    include_template: Option<bool>,
    recurrence: Option<String>,
}

impl ListEventsPaginated {
    setters! {
        /// Maximum number of events per page.
        limit: u64;
        /// Number of events to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Include each event's chats.
        include_chat: bool;
        /// Include each event's templates.
        include_template: bool;
        /// Only events with this recurrence.
        recurrence: into String;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<EventsPage> {
        let p = &self.params;
        let mut q = Query::new();
        q.push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push_opt("include_chat", p.include_chat)
            .push_opt("include_template", p.include_template)
            .push_opt("recurrence", p.recurrence.as_deref());
        self.client
            .transport
            .get(&["events", "pagination"])
            .query(q)
            .send()
            .await
    }

    /// Fetches one page, including its pagination metadata.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<EventsPage> {
        self.fetch(self.params.offset).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends after a page whose `pagination.hasMore` is `false`, at the first
    /// empty page, or at the first page shorter than [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        let exhausted = Arc::new(AtomicBool::new(false));
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            let exhausted = Arc::clone(&exhausted);
            async move {
                if exhausted.load(Ordering::Relaxed) {
                    return Ok(Vec::new());
                }
                let page = request.fetch(Some(offset)).await?;
                if page.pagination.as_ref().and_then(|p| p.has_more) == Some(false) {
                    exhausted.store(true, Ordering::Relaxed);
                }
                Ok(page.data.unwrap_or_default())
            }
        })
    }
}

/// Request builder for [`GammaClient::list_sport_event_results`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSportEventResults {
    client: GammaClient,
    params: PageParams,
}

impl ListSportEventResults {
    setters! {
        /// Maximum number of events per page.
        limit: u64;
        /// Number of events to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Event>> {
        self.client
            .transport
            .get(&["events", "results"])
            .query(self.params.query(offset))
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Event>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::get_event`] and [`GammaClient::get_event_by_slug`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetEvent {
    client: GammaClient,
    lookup: Lookup<EventId>,
    params: GetEventParams,
}

#[derive(Debug, Clone, Default)]
struct GetEventParams {
    include_chat: Option<bool>,
    include_template: Option<bool>,
}

impl GetEvent {
    setters! {
        /// Include the event's chats.
        include_chat: bool;
        /// Include the event's templates.
        include_template: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing event is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn send(self) -> Result<Event> {
        let mut query = Query::new();
        query
            .push_opt("include_chat", self.params.include_chat)
            .push_opt("include_template", self.params.include_template);
        let transport = &self.client.transport;
        let request = match &self.lookup {
            Lookup::Id(id) => transport.get(&["events", id.as_str()]),
            Lookup::Slug(slug) => transport.get(&["events", "slug", slug.as_str()]),
        };
        request.query(query).send().await
    }
}

/// Request builder for [`GammaClient::list_event_creators`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventCreators {
    client: GammaClient,
    params: ListEventCreatorsParams,
}

#[derive(Debug, Clone, Default)]
struct ListEventCreatorsParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    creator_name: Option<String>,
    creator_handle: Option<String>,
}

impl ListEventCreators {
    setters! {
        /// Maximum number of creators per page.
        limit: u64;
        /// Number of creators to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only creators with this name.
        creator_name: into String;
        /// Only creators with this handle.
        creator_handle: into String;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<EventCreator>> {
        let p = &self.params;
        let mut q = Query::new();
        q.push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push_opt("creator_name", p.creator_name.as_deref())
            .push_opt("creator_handle", p.creator_handle.as_deref());
        self.client
            .transport
            .get(&["events", "creators"])
            .query(q)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<EventCreator>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every creator from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page or the first page shorter than
    /// [`limit`](Self::limit) (when set).
    pub fn into_stream(self) -> impl Stream<Item = Result<EventCreator>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        let page_size = self.params.limit;
        offset_stream(start, page_size, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::list_events_keyset`].
///
/// The endpoint rejects `offset`, so there is no setter for it; page with
/// [`after_cursor`](Self::after_cursor) instead.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventsKeyset {
    client: GammaClient,
    params: ListEventsKeysetParams,
}

#[derive(Debug, Clone, Default)]
struct ListEventsKeysetParams {
    limit: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    after_cursor: Option<String>,
    id: Vec<EventId>,
    slug: Vec<String>,
    closed: Option<bool>,
    live: Option<bool>,
    featured: Option<bool>,
    cyom: Option<bool>,
    title_search: Option<String>,
    liquidity_min: Option<Decimal>,
    liquidity_max: Option<Decimal>,
    volume_min: Option<Decimal>,
    volume_max: Option<Decimal>,
    start_date_min: Option<DateTime<Utc>>,
    start_date_max: Option<DateTime<Utc>>,
    end_date_min: Option<DateTime<Utc>>,
    end_date_max: Option<DateTime<Utc>>,
    start_time_min: Option<DateTime<Utc>>,
    start_time_max: Option<DateTime<Utc>>,
    tag_id: Vec<TagId>,
    tag_slug: Option<String>,
    exclude_tag_id: Vec<TagId>,
    related_tags: Option<bool>,
    tag_match: Option<String>,
    series_id: Vec<SeriesId>,
    game_id: Vec<i64>,
    event_date: Option<DateTime<Utc>>,
    event_week: Option<i64>,
    featured_order: Option<bool>,
    recurrence: Option<String>,
    created_by: Vec<String>,
    parent_event_id: Option<EventId>,
    include_children: Option<bool>,
    partner_slug: Option<String>,
    include_chat: Option<bool>,
    include_template: Option<bool>,
    include_best_lines: Option<bool>,
    locale: Option<String>,
}

impl ListEventsKeysetParams {
    fn validate(&self) -> Result<()> {
        validate_keyset_limit(self.limit)?;
        if let Some(overlap) = self.exclude_tag_id.iter().find(|t| self.tag_id.contains(t)) {
            return Err(ValidationError::new(
                "exclude_tag_id",
                format!("cannot overlap with tag_id (both contain {overlap})"),
            )
            .into());
        }
        Ok(())
    }

    fn query(&self, after_cursor: Option<&str>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_opt("after_cursor", after_cursor)
            .push_all("id", &self.id)
            .push_all("slug", &self.slug)
            .push_opt("closed", self.closed)
            .push_opt("live", self.live)
            .push_opt("featured", self.featured)
            .push_opt("cyom", self.cyom)
            .push_opt("title_search", self.title_search.as_deref())
            .push_opt("liquidity_min", self.liquidity_min)
            .push_opt("liquidity_max", self.liquidity_max)
            .push_opt("volume_min", self.volume_min)
            .push_opt("volume_max", self.volume_max)
            .push_opt("start_date_min", self.start_date_min.as_ref().map(rfc3339))
            .push_opt("start_date_max", self.start_date_max.as_ref().map(rfc3339))
            .push_opt("end_date_min", self.end_date_min.as_ref().map(rfc3339))
            .push_opt("end_date_max", self.end_date_max.as_ref().map(rfc3339))
            .push_opt("start_time_min", self.start_time_min.as_ref().map(rfc3339))
            .push_opt("start_time_max", self.start_time_max.as_ref().map(rfc3339))
            .push_all("tag_id", &self.tag_id)
            .push_opt("tag_slug", self.tag_slug.as_deref())
            .push_all("exclude_tag_id", &self.exclude_tag_id)
            .push_opt("related_tags", self.related_tags)
            .push_opt("tag_match", self.tag_match.as_deref())
            .push_all("series_id", &self.series_id)
            .push_all("game_id", &self.game_id)
            .push_opt("event_date", self.event_date.as_ref().map(rfc3339))
            .push_opt("event_week", self.event_week)
            .push_opt("featured_order", self.featured_order)
            .push_opt("recurrence", self.recurrence.as_deref())
            .push_all("created_by", &self.created_by)
            .push_opt("parent_event_id", self.parent_event_id.as_ref())
            .push_opt("include_children", self.include_children)
            .push_opt("partner_slug", self.partner_slug.as_deref())
            .push_opt("include_chat", self.include_chat)
            .push_opt("include_template", self.include_template)
            .push_opt("include_best_lines", self.include_best_lines)
            .push_opt("locale", self.locale.as_deref());
        q
    }
}

impl ListEventsKeyset {
    setters! {
        /// Maximum number of events per page, between 1 and 100 (server default 20).
        /// Values outside that range are rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        limit: u64;
        /// Comma-separated list of JSON field names to order by.
        order: into String;
        /// Sort direction (server default ascending). Only used when
        /// [`order`](Self::order) is set.
        ascending: bool;
        /// Opaque cursor from a previous page's
        /// [`next_cursor`](EventsKeysetPage::next_cursor).
        after_cursor: into String;
        /// Only events with these ids.
        id: many EventId;
        /// Only events with these slugs.
        slug: many String;
        /// Only closed (`true`) or only open (`false`) events.
        closed: bool;
        /// Only live (`true`) or only non-live (`false`) events.
        live: bool;
        /// Only featured (`true`) or only non-featured (`false`) events.
        featured: bool;
        /// Only "create your own market" events (`true`) or only other events (`false`).
        cyom: bool;
        /// Only events whose title matches this search text.
        title_search: into String;
        /// Minimum liquidity.
        liquidity_min: into Decimal;
        /// Maximum liquidity.
        liquidity_max: into Decimal;
        /// Minimum volume.
        volume_min: into Decimal;
        /// Maximum volume.
        volume_max: into Decimal;
        /// Earliest start date.
        start_date_min: DateTime<Utc>;
        /// Latest start date.
        start_date_max: DateTime<Utc>;
        /// Earliest end date.
        end_date_min: DateTime<Utc>;
        /// Latest end date.
        end_date_max: DateTime<Utc>;
        /// Earliest start time.
        start_time_min: DateTime<Utc>;
        /// Latest start time.
        start_time_max: DateTime<Utc>;
        /// Only events with these tags.
        tag_id: many TagId;
        /// Only events with the tag with this slug.
        tag_slug: into String;
        /// Exclude events with these tags. Must not overlap with [`tag_id`](Self::tag_id);
        /// an overlap is rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        exclude_tag_id: many TagId;
        /// Include events with tags related to [`tag_id`](Self::tag_id).
        related_tags: bool;
        /// How to match tags (the spec documents no values).
        tag_match: into String;
        /// Only events in these series.
        series_id: many SeriesId;
        /// Only events for these game ids.
        game_id: many i64;
        /// Only events on this date.
        event_date: DateTime<Utc>;
        /// Only events in this week.
        event_week: i64;
        /// The `featured_order` flag (undocumented beyond its boolean type).
        featured_order: bool;
        /// Only events with this recurrence.
        recurrence: into String;
        /// Only events created by these creators.
        created_by: many String;
        /// Only children of this parent event.
        parent_event_id: into EventId;
        /// Include child events.
        include_children: bool;
        /// Attach external partners matching this partner slug to the events.
        partner_slug: into String;
        /// Include chats (the `Chats` and `Series.Chats` relations).
        include_chat: bool;
        /// Include templates (the `Templates` relation).
        include_template: bool;
        /// Include the `BestLines` relation.
        include_best_lines: bool;
        /// Locale.
        locale: into String;
    }

    async fn fetch(&self, after_cursor: Option<&str>) -> Result<EventsKeysetPage> {
        self.params.validate()?;
        self.client
            .transport
            .get(&["events", "keyset"])
            .query(self.params.query(after_cursor))
            .send()
            .await
    }

    /// Fetches one page, starting at [`after_cursor`](Self::after_cursor) if set.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if [`limit`](Self::limit) is out of
    /// range or [`exclude_tag_id`](Self::exclude_tag_id) overlaps
    /// [`tag_id`](Self::tag_id); otherwise see [`Error`](crate::Error). The server answers
    /// `422` (an [`Error::Api`](crate::Error::Api) whose
    /// [`error_type`](crate::ApiError::error_type) is `"validation error"`) for an invalid
    /// cursor, order field, recurrence or filter.
    pub async fn send(self) -> Result<EventsKeysetPage> {
        self.fetch(self.params.after_cursor.as_deref()).await
    }

    /// Streams every event from [`after_cursor`](Self::after_cursor) (or the beginning)
    /// onwards, fetching pages lazily until a page has no `next_cursor`.
    pub fn into_stream(self) -> impl Stream<Item = Result<Event>> + Send + 'static {
        let start = self.params.after_cursor.clone();
        cursor_stream(start, move |cursor| {
            let request = self.clone();
            async move {
                let page = request.fetch(cursor.as_deref()).await?;
                Ok(CursorPage::new(
                    page.events.unwrap_or_default(),
                    page.next_cursor,
                ))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Event` in
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body).
    #[test]
    fn deserializes_event_wire_names_and_types() {
        let json = r#"{
            "id": "16167",
            "title": "Election",
            "startDate": "2024-01-01T00:00:00Z",
            "liquidity": 1234.5,
            "volume": "99",
            "published_at": "2024-01-01 00:00:00+00",
            "createdBy": "admin",
            "negRiskMarketID": "0xabc",
            "negRiskFeeBips": 10,
            "subEvents": ["a"],
            "markets": [{"id": "1", "liquidity": "5"}],
            "series": [{"id": "2"}],
            "eventCreators": [{"id": "3", "creatorName": "Alice"}],
            "closedTime": "2024-02-01T00:00:00Z",
            "eventDate": "2024-02-01",
            "volume24hr": 1
        }"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, Some(EventId::from("16167")));
        assert_eq!(event.liquidity, Some(Decimal::new(12_345, 1)));
        assert_eq!(event.volume, Some(Decimal::from(99)));
        assert_eq!(
            event.published_at.as_deref(),
            Some("2024-01-01 00:00:00+00")
        );
        assert_eq!(event.created_by.as_deref(), Some("admin"));
        assert_eq!(event.neg_risk_market_id.as_deref(), Some("0xabc"));
        assert_eq!(event.sub_events, Some(vec!["a".to_owned()]));
        let markets = event.markets.unwrap();
        assert_eq!(markets[0].liquidity, Some(Decimal::from(5)));
        assert_eq!(event.series.unwrap()[0].id, Some(SeriesId::from("2")));
        assert_eq!(
            event.event_creators.unwrap()[0].creator_name.as_deref(),
            Some("Alice")
        );
        assert!(event.closed_time.is_some());
        assert_eq!(event.volume_24hr, Some(Decimal::from(1)));
    }

    #[test]
    fn deserializes_pages() {
        let page: EventsPage = serde_json::from_str(
            r#"{"data":[{"id":"1"}],"pagination":{"hasMore":false,"totalResults":1}}"#,
        )
        .unwrap();
        assert_eq!(page.data.unwrap().len(), 1);
        assert_eq!(page.pagination.unwrap().has_more, Some(false));

        let keyset: EventsKeysetPage =
            serde_json::from_str(r#"{"events":[],"next_cursor":"c2"}"#).unwrap();
        assert_eq!(keyset.events, Some(vec![]));
        assert_eq!(keyset.next_cursor.as_deref(), Some("c2"));
    }

    #[test]
    fn keyset_rejects_overlapping_tags() {
        let params = ListEventsKeysetParams {
            tag_id: vec![TagId::from("1"), TagId::from("2")],
            exclude_tag_id: vec![TagId::from("2")],
            ..ListEventsKeysetParams::default()
        };
        let err = params.validate().unwrap_err();
        let crate::Error::Validation(v) = &err else {
            panic!("expected a validation error, got {err:?}")
        };
        assert_eq!(v.parameter(), "exclude_tag_id");

        let ok = ListEventsKeysetParams {
            tag_id: vec![TagId::from("1")],
            exclude_tag_id: vec![TagId::from("2")],
            limit: Some(100),
            ..ListEventsKeysetParams::default()
        };
        assert!(ok.validate().is_ok());
    }
}
