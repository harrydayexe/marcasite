//! Events: `/events`, `/events/{id}`, `/events/slug/{slug}`, `/events/{id}/tags`,
//! `/events/{id}/tweet-count`, `/events/{id}/comments/count`, `/events/keyset`,
//! `/events/pagination`, `/events/results` and `/events/creators`.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::Paginated;
use chrono::{DateTime, Utc};
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
    util::{
        Lookup, PageParams, canonical_integer, check_integer_id, check_integer_ids, rfc3339,
        setters, validate_keyset_limit,
    },
};

polyoxide_core::string_id! {
    /// An event creator id.
    pub struct EventCreatorId;
}

/// An event: a group of related markets (`components/schemas/Event`).
///
/// Every field is optional because the spec marks none as required. Fields the spec types
/// as `number` are [`Decimal`]s and serialize back as JSON numbers.
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
    /// The `sortBy` value (a string; the spec documents no values).
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
    /// The `cyom` flag (documented only as a boolean).
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
    /// Score. The spec types this as a plain string.
    pub score: Option<String>,
    /// Elapsed time. The spec types this as a plain string.
    pub elapsed: Option<String>,
    /// Period. The spec types this as a plain string.
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
    /// Chats. The keyset listing documents them as included only with
    /// `include_chat=true`.
    pub chats: Option<Vec<Chat>>,
    /// Featured order.
    pub featured_order: Option<i64>,
    /// The `estimateValue` flag (documented only as a boolean).
    pub estimate_value: Option<bool>,
    /// The `cantEstimate` flag (documented only as a boolean).
    pub cant_estimate: Option<bool>,
    /// Estimated value. The spec types this as a plain string.
    pub estimated_value: Option<String>,
    /// Templates. The keyset listing documents them as included only with
    /// `include_template=true`.
    pub templates: Option<Vec<Template>>,
    /// Main spreads line.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub spreads_main_line: Option<Decimal>,
    /// Main totals line.
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
    /// Game status. The spec types this as a plain string.
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
///
/// The fields keep their wire names; [`items`](Self::items),
/// [`into_items`](Self::into_items) and [`has_more`](Self::has_more) give a uniform view.
/// The endpoint is offset-paginated, so there is no cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventsPage {
    /// The events on this page.
    pub data: Option<Vec<Event>>,
    /// Pagination metadata.
    pub pagination: Option<Pagination>,
}

impl EventsPage {
    /// The events on this page (empty if the page carries none).
    #[must_use]
    pub fn items(&self) -> &[Event] {
        self.data.as_deref().unwrap_or_default()
    }

    /// Consumes the page and returns its events.
    #[must_use]
    pub fn into_items(self) -> Vec<Event> {
        self.data.unwrap_or_default()
    }

    /// The page's `pagination.hasMore`, or `None` if the server did not send it.
    #[must_use]
    pub fn has_more(&self) -> Option<bool> {
        self.pagination.as_ref().and_then(|p| p.has_more)
    }
}

/// One page of [`GammaClient::list_events_keyset`] (`components/schemas/KeysetEventsResponse`).
///
/// The fields keep their wire names; [`items`](Self::items),
/// [`into_items`](Self::into_items) and [`next_cursor()`](Self::next_cursor()) give the
/// same view as every other page type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EventsKeysetPage {
    /// The events on this page (documented as an empty array if none were found).
    pub events: Option<Vec<Event>>,
    /// Cursor for the next page, passed to [`cursor`](ListEventsKeyset::cursor). The spec
    /// documents it as present only when the number of returned events equals the
    /// effective limit, and omitted on the last page.
    pub next_cursor: Option<String>,
}

impl EventsKeysetPage {
    /// The events on this page (empty if the page carries none).
    #[must_use]
    pub fn items(&self) -> &[Event] {
        self.events.as_deref().unwrap_or_default()
    }

    /// Consumes the page and returns its events.
    #[must_use]
    pub fn into_items(self) -> Vec<Event> {
        self.events.unwrap_or_default()
    }

    /// The cursor for the next page, or `None` on the last page (an absent or empty
    /// `next_cursor`).
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor
            .as_deref()
            .filter(|cursor| !cursor.is_empty())
    }
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

    /// Lists events with pagination metadata (`hasMore`, `totalResults`; offset
    /// pagination).
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the event does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_event_tags(&self, id: impl Into<EventId>) -> Result<Vec<Tag>> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the event does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_event_tweet_count(&self, id: impl Into<EventId>) -> Result<EventTweetCount> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the event does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_event_comment_count(&self, id: impl Into<EventId>) -> Result<CommentCount> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
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
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `id`) if `id` is not an
    ///   integer (one or more ASCII digits), checked before sending;
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the creator does not exist;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn get_event_creator(&self, id: impl Into<EventCreatorId>) -> Result<EventCreator> {
        let id = id.into();
        check_integer_id("id", id.as_str())?;
        self.transport
            .get(&["events", "creators", id.as_str()])
            .send()
            .await
    }

    /// Lists events with cursor-based (keyset) pagination, for stable paging through large
    /// result sets.
    ///
    /// [`send`](ListEventsKeyset::send) returns one [`EventsKeysetPage`]; pass its
    /// [`next_cursor()`](EventsKeysetPage::next_cursor()) to
    /// [`cursor`](ListEventsKeyset::cursor) for the next page, or use
    /// [`into_stream`](ListEventsKeyset::into_stream) to walk every page.
    ///
    /// The spec's response description mentions relations that the documented `Event` and
    /// `Market` schemas do not contain: `BestLines` (see
    /// [`include_best_lines`](ListEventsKeyset::include_best_lines)), `external_partners`
    /// (see [`partner_slug`](ListEventsKeyset::partner_slug)), `Teams`, and the nested
    /// markets' `clob_rewards`. That data is not modelled, so it is dropped when decoding.
    /// The description also spells the markets' fee schedule `fee_schedule`, while the
    /// schema spells it `feeSchedule` ([`Market::fee_schedule`]).
    ///
    /// See <https://docs.polymarket.com/api-reference/events/list-events-keyset-pagination>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let first = gamma.list_events_keyset().limit(50).closed(false).send().await?;
    /// if let Some(cursor) = first.next_cursor() {
    ///     let second = gamma
    ///         .list_events_keyset()
    ///         .limit(50)
    ///         .closed(false)
    ///         .cursor(cursor)
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
    limit: Option<u32>,
    offset: Option<u32>,
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
    fn validate(&self) -> Result<()> {
        check_integer_ids("id", &self.id)?;
        if let Some(tag_id) = &self.tag_id {
            check_integer_id("tag_id", tag_id.as_str())?;
        }
        check_integer_ids("exclude_tag_id", &self.exclude_tag_id)
    }

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
        /// Maximum number of events per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of events to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// Filter by event ids (`id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        ids => id: many EventId;
        /// Filter by tag id (`tag_id`). The spec types it as an integer, so an id that is not
        /// one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        tag_id: into TagId;
        /// Tag ids to exclude (`exclude_tag_id`, repeated). The spec types them as
        /// integers, so an id that is not one or more ASCII digits is rejected before
        /// sending with [`Error::Validation`](crate::Error::Validation).
        exclude_tag_ids => exclude_tag_id: many TagId;
        /// Filter by slugs (`slug`, repeated).
        slugs => slug: many String;
        /// Filter by tag slug (`tag_slug`).
        tag_slug: into String;
        /// The `related_tags` flag (documented only as a boolean).
        related_tags: bool;
        /// The `active` filter (documented only as a boolean).
        active: bool;
        /// The `archived` filter (documented only as a boolean).
        archived: bool;
        /// The `featured` filter (documented only as a boolean).
        featured: bool;
        /// The `cyom` filter (documented only as a boolean).
        cyom: bool;
        /// The `include_chat` flag (documented only as a boolean on this endpoint).
        include_chat: bool;
        /// The `include_template` flag (documented only as a boolean on this endpoint).
        include_template: bool;
        /// The `recurrence` filter (a string; the spec documents no values).
        recurrence: into String;
        /// The `closed` filter (documented only as a boolean).
        closed: bool;
        /// Minimum liquidity (`liquidity_min`).
        liquidity_min: into Decimal;
        /// Maximum liquidity (`liquidity_max`).
        liquidity_max: into Decimal;
        /// Minimum volume (`volume_min`).
        volume_min: into Decimal;
        /// Maximum volume (`volume_max`).
        volume_max: into Decimal;
        /// Earliest start date (`start_date_min`).
        start_date_min: DateTime<Utc>;
        /// Latest start date (`start_date_max`).
        start_date_max: DateTime<Utc>;
        /// Earliest end date (`end_date_min`).
        end_date_min: DateTime<Utc>;
        /// Latest end date (`end_date_max`).
        end_date_max: DateTime<Utc>;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Event>> {
        self.params.validate()?;
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
    /// - [`Error::Validation`](crate::Error::Validation) if an [`ids`](Self::ids) or
    ///   [`exclude_tag_ids`](Self::exclude_tag_ids) entry or [`tag_id`](Self::tag_id) is
    ///   not an integer, checked before sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Event>> {
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error
    /// (the errors of [`send`](Self::send)). A page shorter than [`limit`](Self::limit)
    /// does not end it, because the docs give no maximum `limit` and the server may return
    /// fewer events, so the last request returns an empty page.
    pub fn into_stream(self) -> Paginated<Event> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
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
    limit: Option<u32>,
    offset: Option<u32>,
    order: Option<String>,
    ascending: Option<bool>,
    include_chat: Option<bool>,
    include_template: Option<bool>,
    recurrence: Option<String>,
}

impl ListEventsPaginated {
    setters! {
        /// Maximum number of events per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of events to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// The `include_chat` flag (documented only as a boolean on this endpoint).
        include_chat: bool;
        /// The `include_template` flag (documented only as a boolean on this endpoint).
        include_template: bool;
        /// The `recurrence` filter (a string; the spec documents no values).
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
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends after a page whose `pagination.hasMore` is `false`, at the first
    /// empty page, or right after yielding the first error. A page shorter than
    /// [`limit`](Self::limit) does not end it while `hasMore` is not `false`, because the
    /// server may return fewer events than requested.
    pub fn into_stream(self) -> Paginated<Event> {
        let start = self.params.offset.map_or(0, u64::from);
        let exhausted = Arc::new(AtomicBool::new(false));
        offset_stream(start, move |offset| {
            let request = self.clone();
            let exhausted = Arc::clone(&exhausted);
            async move {
                if exhausted.load(Ordering::Relaxed) {
                    return Ok(Vec::new());
                }
                let page = request.fetch(Some(offset)).await?;
                if page.has_more() == Some(false) {
                    exhausted.store(true, Ordering::Relaxed);
                }
                Ok(page.into_items())
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
        /// Maximum number of events per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of events to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
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
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every event from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error.
    /// A page shorter than [`limit`](Self::limit) does not end it, because the docs give no
    /// maximum `limit` and the server may return fewer events, so the last request returns
    /// an empty page.
    pub fn into_stream(self) -> Paginated<Event> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
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
        /// The `include_chat` flag (documented only as a boolean on this endpoint).
        include_chat: bool;
        /// The `include_template` flag (documented only as a boolean on this endpoint).
        include_template: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if the id
    ///   is not an integer (parameter `id`) or the slug is empty, `.` or `..` (parameter
    ///   `slug`);
    /// - [`Error::Api`](crate::Error::Api) with status `404` if the event does not exist
    ///   (see [`Error::is_not_found`](crate::Error::is_not_found));
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Event> {
        let segments = self.lookup.path("events", &[])?;
        let mut query = Query::new();
        query
            .push_opt("include_chat", self.params.include_chat)
            .push_opt("include_template", self.params.include_template);
        self.client
            .transport
            .get(&segments)
            .query(query)
            .send()
            .await
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
    limit: Option<u32>,
    offset: Option<u32>,
    order: Option<String>,
    ascending: Option<bool>,
    creator_name: Option<String>,
    creator_handle: Option<String>,
}

impl ListEventCreators {
    setters! {
        /// Maximum number of creators per page (`limit`; the docs give a minimum of `0` and
        /// no maximum).
        limit: u32;
        /// Number of creators to skip (`offset`).
        offset: u32;
        /// Comma-separated list of fields to order by (`order`).
        order: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// Filter by creator name (`creator_name`).
        creator_name: into String;
        /// Filter by creator handle (`creator_handle`).
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
        self.fetch(self.params.offset.map(u64::from)).await
    }

    /// Streams every creator from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page, or right after yielding the first error.
    /// A page shorter than [`limit`](Self::limit) does not end it, because the docs give no
    /// maximum `limit` and the server may return fewer creators, so the last request
    /// returns an empty page.
    pub fn into_stream(self) -> Paginated<EventCreator> {
        let start = self.params.offset.map_or(0, u64::from);
        offset_stream(start, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::list_events_keyset`].
///
/// The endpoint rejects `offset`, so there is no setter for it; page with
/// [`cursor`](Self::cursor) instead.
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListEventsKeyset {
    client: GammaClient,
    params: ListEventsKeysetParams,
}

#[derive(Debug, Clone, Default)]
struct ListEventsKeysetParams {
    limit: Option<u32>,
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
        check_integer_ids("id", &self.id)?;
        check_integer_ids("tag_id", &self.tag_id)?;
        check_integer_ids("exclude_tag_id", &self.exclude_tag_id)?;
        check_integer_ids("series_id", &self.series_id)?;
        if let Some(parent) = &self.parent_event_id {
            check_integer_id("parent_event_id", parent.as_str())?;
        }
        // Documented: "Cannot overlap with tag_id". Compare the integers, not the spellings.
        let overlap = self.exclude_tag_id.iter().find(|excluded| {
            let excluded = canonical_integer(excluded.as_str());
            self.tag_id
                .iter()
                .any(|tag| canonical_integer(tag.as_str()) == excluded)
        });
        if let Some(overlap) = overlap {
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
        /// Maximum number of events per page (`limit`), between 1 and 100 (server default
        /// 20). Values outside that range are rejected client-side with
        /// [`Error::Validation`](crate::Error::Validation).
        limit: u32;
        /// Comma-separated list of JSON field names to order by (`order`).
        order: into String;
        /// Sort direction (`ascending`, server default `true`). Only used when
        /// [`order`](Self::order) is set.
        ascending: bool;
        /// Opaque cursor from a previous page's
        /// [`next_cursor()`](EventsKeysetPage::next_cursor()), sent as `after_cursor`.
        cursor => after_cursor: into String;
        /// Filter by event ids (`id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        ids => id: many EventId;
        /// Filter by slugs (`slug`, repeated).
        slugs => slug: many String;
        /// The `closed` filter (documented only as a boolean).
        closed: bool;
        /// The `live` filter (documented only as a boolean).
        live: bool;
        /// The `featured` filter (documented only as a boolean).
        featured: bool;
        /// The `cyom` filter (documented only as a boolean).
        cyom: bool;
        /// The `title_search` filter (a string; the docs do not describe the matching).
        title_search: into String;
        /// Minimum liquidity (`liquidity_min`).
        liquidity_min: into Decimal;
        /// Maximum liquidity (`liquidity_max`).
        liquidity_max: into Decimal;
        /// Minimum volume (`volume_min`).
        volume_min: into Decimal;
        /// Maximum volume (`volume_max`).
        volume_max: into Decimal;
        /// Earliest start date (`start_date_min`).
        start_date_min: DateTime<Utc>;
        /// Latest start date (`start_date_max`).
        start_date_max: DateTime<Utc>;
        /// Earliest end date (`end_date_min`).
        end_date_min: DateTime<Utc>;
        /// Latest end date (`end_date_max`).
        end_date_max: DateTime<Utc>;
        /// Earliest start time (`start_time_min`).
        start_time_min: DateTime<Utc>;
        /// Latest start time (`start_time_max`).
        start_time_max: DateTime<Utc>;
        /// Filter by tag ids (`tag_id`, repeated). The spec types them as integers, so an id
        /// that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        tag_ids => tag_id: many TagId;
        /// Filter by tag slug (`tag_slug`).
        tag_slug: into String;
        /// Tag ids to exclude (`exclude_tag_id`, repeated). Documented as "Cannot overlap
        /// with tag_id": an overlap with [`tag_ids`](Self::tag_ids) (compared as integers),
        /// or an id that is not one or more ASCII digits, is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        exclude_tag_ids => exclude_tag_id: many TagId;
        /// The `related_tags` flag (documented only as a boolean).
        related_tags: bool;
        /// The `tag_match` parameter (a string; the spec documents no values).
        tag_match: into String;
        /// Filter by series ids (`series_id`, repeated). The spec types them as integers, so
        /// an id that is not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        series_ids => series_id: many SeriesId;
        /// Filter by game ids (`game_id`, repeated; integers on this endpoint).
        game_ids => game_id: many i64;
        /// The `event_date` filter.
        event_date: DateTime<Utc>;
        /// The `event_week` filter (an integer).
        event_week: i64;
        /// The `featured_order` flag (documented only as a boolean).
        featured_order: bool;
        /// The `recurrence` filter (a string; the spec documents no values, and the server
        /// answers `422` for an invalid one).
        recurrence: into String;
        /// The `created_by` filter (repeated strings).
        created_by: many String;
        /// The `parent_event_id` filter. The spec types it as an integer, so an id that is
        /// not one or more ASCII digits is rejected before sending with
        /// [`Error::Validation`](crate::Error::Validation).
        parent_event_id: into EventId;
        /// The `include_children` flag (documented only as a boolean).
        include_children: bool;
        /// The `partner_slug` parameter, documented as: "When set, external_partners are
        /// attached to matching events". The documented `Event` schema has no
        /// `external_partners` property, so that data is not modelled and is dropped when
        /// decoding.
        partner_slug: into String;
        /// When `true`, includes the `Chats` and `Series.Chats` relations (`include_chat`),
        /// modelled as [`Event::chats`] and [`Series::chats`].
        include_chat: bool;
        /// When `true`, includes the `Templates` relation (`include_template`), modelled as
        /// [`Event::templates`].
        include_template: bool;
        /// When `true`, includes the `BestLines` relation (`include_best_lines`). The
        /// documented `Event` schema has no property for it, so that data is not modelled
        /// and is dropped when decoding.
        include_best_lines: bool;
        /// The `locale` parameter (a string; the spec documents no values).
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

    /// Fetches one page, starting at [`cursor`](Self::cursor) if set.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation), checked before sending, if
    ///   [`limit`](Self::limit) is out of range, an integer-typed id is not one or more
    ///   ASCII digits, or [`exclude_tag_ids`](Self::exclude_tag_ids) overlaps
    ///   [`tag_ids`](Self::tag_ids);
    /// - [`Error::Api`](crate::Error::Api) with status `422` (whose
    ///   [`error_type`](crate::ApiError::error_type) is `"validation error"`) for an
    ///   invalid cursor, order field, recurrence or filter;
    /// - [`Error::Api`](crate::Error::Api) with status `500` (`"internal error"`) for a
    ///   server-side failure;
    /// - [`Error::Api`](crate::Error::Api) with status `503` and
    ///   [`error_type`](crate::ApiError::error_type) `"service unavailable"`, which the
    ///   spec documents as "keyset pagination is not configured". That is a server
    ///   configuration state rather than a transient failure, so retrying is unlikely to
    ///   help; [`GammaClient::list_events`] and [`GammaClient::list_events_paginated`] are
    ///   the offset-paginated alternatives;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<EventsKeysetPage> {
        self.fetch(self.params.after_cursor.as_deref()).await
    }

    /// Streams every event from [`cursor`](Self::cursor) (or the beginning) onwards,
    /// fetching pages lazily until a page has no (or an empty) `next_cursor`.
    ///
    /// The stream ends right after yielding the first error (the errors of
    /// [`send`](Self::send)).
    pub fn into_stream(self) -> Paginated<Event> {
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

    /// The parameter named by a validation error, or `""` for any other outcome.
    fn validation_parameter(result: Result<()>) -> String {
        match result {
            Err(crate::Error::Validation(v)) => v.parameter().to_owned(),
            _ => String::new(),
        }
    }

    /// Field names and types from `components/schemas/Event` in
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body): `liquidity`,
    /// `volume` and `volume24hr` are numbers, `createdBy` and `published_at` strings.
    #[test]
    fn deserializes_event_wire_names_and_types() {
        let json = r#"{
            "id": "16167",
            "title": "Election",
            "startDate": "2024-01-01T00:00:00Z",
            "liquidity": 1234.5,
            "volume": 99,
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
        let markets = event.markets.as_ref().unwrap();
        assert_eq!(markets[0].liquidity, Some(Decimal::from(5)));
        assert_eq!(
            event.series.as_ref().unwrap()[0].id,
            Some(SeriesId::from("2"))
        );
        assert_eq!(
            event.event_creators.as_ref().unwrap()[0]
                .creator_name
                .as_deref(),
            Some("Alice")
        );
        assert!(event.closed_time.is_some());
        assert_eq!(event.volume_24hr, Some(Decimal::from(1)));

        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["volume"], serde_json::json!(99));
        assert_eq!(value["liquidity"], serde_json::json!(1234.5));
        assert_eq!(value["markets"][0]["liquidity"], serde_json::json!("5"));
    }

    /// Lenient decoding beyond the spec: a number-typed field sent as a numeric string is
    /// accepted and serializes back as a number.
    #[test]
    fn number_fields_accept_numeric_strings() {
        let event: Event = serde_json::from_str(r#"{"volume":"99"}"#).unwrap();
        assert_eq!(event.volume, Some(Decimal::from(99)));
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["volume"], serde_json::json!(99));
    }

    #[test]
    fn deserializes_pages_and_accessors() {
        let page: EventsPage = serde_json::from_str(
            r#"{"data":[{"id":"1"}],"pagination":{"hasMore":false,"totalResults":1}}"#,
        )
        .unwrap();
        assert_eq!(page.items().len(), 1);
        assert_eq!(page.has_more(), Some(false));
        assert_eq!(page.pagination.as_ref().unwrap().total_results, Some(1));
        assert_eq!(page.into_items()[0].id, Some(EventId::from("1")));
        let bare: EventsPage = serde_json::from_str("{}").unwrap();
        assert!(bare.items().is_empty());
        assert_eq!(bare.has_more(), None);

        let keyset: EventsKeysetPage =
            serde_json::from_str(r#"{"events":[],"next_cursor":"c2"}"#).unwrap();
        assert_eq!(keyset.events, Some(vec![]));
        assert!(keyset.items().is_empty());
        assert_eq!(keyset.next_cursor(), Some("c2"));
        let last: EventsKeysetPage =
            serde_json::from_str(r#"{"events":[{"id":"1"}],"next_cursor":""}"#).unwrap();
        assert_eq!(last.next_cursor(), None);
        assert_eq!(last.into_items().len(), 1);
    }

    #[test]
    fn keyset_rejects_overlapping_tags() {
        let params = ListEventsKeysetParams {
            tag_id: vec![TagId::from("1"), TagId::from("2")],
            exclude_tag_id: vec![TagId::from("2")],
            ..ListEventsKeysetParams::default()
        };
        assert_eq!(validation_parameter(params.validate()), "exclude_tag_id");

        // The ids are integers: "02" and "2" overlap.
        let params = ListEventsKeysetParams {
            tag_id: vec![TagId::from("2")],
            exclude_tag_id: vec![TagId::from("02")],
            ..ListEventsKeysetParams::default()
        };
        assert_eq!(validation_parameter(params.validate()), "exclude_tag_id");

        let ok = ListEventsKeysetParams {
            tag_id: vec![TagId::from("1")],
            exclude_tag_id: vec![TagId::from("2"), TagId::from("10")],
            limit: Some(100),
            ..ListEventsKeysetParams::default()
        };
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn keyset_validates_integer_typed_filters() {
        let cases = [
            (
                ListEventsKeysetParams {
                    id: vec![EventId::from("1"), EventId::from("one")],
                    ..ListEventsKeysetParams::default()
                },
                "id",
            ),
            (
                ListEventsKeysetParams {
                    tag_id: vec![TagId::from("")],
                    ..ListEventsKeysetParams::default()
                },
                "tag_id",
            ),
            (
                ListEventsKeysetParams {
                    exclude_tag_id: vec![TagId::from("-3")],
                    ..ListEventsKeysetParams::default()
                },
                "exclude_tag_id",
            ),
            (
                ListEventsKeysetParams {
                    series_id: vec![SeriesId::from("nba")],
                    ..ListEventsKeysetParams::default()
                },
                "series_id",
            ),
            (
                ListEventsKeysetParams {
                    parent_event_id: Some(EventId::from("1.5")),
                    ..ListEventsKeysetParams::default()
                },
                "parent_event_id",
            ),
            (
                ListEventsKeysetParams {
                    limit: Some(0),
                    ..ListEventsKeysetParams::default()
                },
                "limit",
            ),
        ];
        for (params, parameter) in cases {
            assert_eq!(validation_parameter(params.validate()), parameter);
        }
    }

    #[test]
    fn offset_listing_validates_integer_typed_filters() {
        let ok = ListEventsParams {
            id: vec![EventId::from("1")],
            tag_id: Some(TagId::from("2")),
            exclude_tag_id: vec![TagId::from("3")],
            ..ListEventsParams::default()
        };
        assert!(ok.validate().is_ok());
        let cases = [
            (
                ListEventsParams {
                    id: vec![EventId::from("x")],
                    ..ListEventsParams::default()
                },
                "id",
            ),
            (
                ListEventsParams {
                    tag_id: Some(TagId::from("politics")),
                    ..ListEventsParams::default()
                },
                "tag_id",
            ),
            (
                ListEventsParams {
                    exclude_tag_id: vec![TagId::from(" 4")],
                    ..ListEventsParams::default()
                },
                "exclude_tag_id",
            ),
        ];
        for (params, parameter) in cases {
            assert_eq!(validation_parameter(params.validate()), parameter);
        }
    }
}
