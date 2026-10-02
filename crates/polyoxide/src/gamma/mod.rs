//! Gamma API client (`https://gamma-api.polymarket.com`).
//!
//! Covers events, markets, tags, series, comments, sports, search and public profiles.
//! Start from [`GammaClient`]; every endpoint is a method on it.
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /status` | [`get_status`](GammaClient::get_status) |
//! | `GET /markets` | [`list_markets`](GammaClient::list_markets) |
//! | `GET /markets/keyset` | [`list_markets_keyset`](GammaClient::list_markets_keyset) |
//! | `GET /markets/{id}` | [`get_market`](GammaClient::get_market) |
//! | `GET /markets/slug/{slug}` | [`get_market_by_slug`](GammaClient::get_market_by_slug) |
//! | `GET /markets/{id}/tags` | [`get_market_tags`](GammaClient::get_market_tags) |
//! | `GET /markets/{id}/description` | [`get_market_description`](GammaClient::get_market_description) |
//! | `POST /markets/information` | [`get_markets_information`](GammaClient::get_markets_information) |
//! | `POST /markets/abridged` | [`get_abridged_markets`](GammaClient::get_abridged_markets) |
//! | `GET /events` | [`list_events`](GammaClient::list_events) |
//! | `GET /events/keyset` | [`list_events_keyset`](GammaClient::list_events_keyset) |
//! | `GET /events/pagination` | [`list_events_paginated`](GammaClient::list_events_paginated) |
//! | `GET /events/results` | [`list_sport_event_results`](GammaClient::list_sport_event_results) |
//! | `GET /events/{id}` | [`get_event`](GammaClient::get_event) |
//! | `GET /events/slug/{slug}` | [`get_event_by_slug`](GammaClient::get_event_by_slug) |
//! | `GET /events/{id}/tags` | [`get_event_tags`](GammaClient::get_event_tags) |
//! | `GET /events/{id}/tweet-count` | [`get_event_tweet_count`](GammaClient::get_event_tweet_count) |
//! | `GET /events/{id}/comments/count` | [`get_event_comment_count`](GammaClient::get_event_comment_count) |
//! | `GET /events/creators` | [`list_event_creators`](GammaClient::list_event_creators) |
//! | `GET /events/creators/{id}` | [`get_event_creator`](GammaClient::get_event_creator) |
//! | `GET /tags` | [`list_tags`](GammaClient::list_tags) |
//! | `GET /tags/{id}` | [`get_tag`](GammaClient::get_tag) |
//! | `GET /tags/slug/{slug}` | [`get_tag_by_slug`](GammaClient::get_tag_by_slug) |
//! | `GET /tags/{id}/related-tags` | [`get_related_tag_relationships`](GammaClient::get_related_tag_relationships) |
//! | `GET /tags/slug/{slug}/related-tags` | [`get_related_tag_relationships_by_slug`](GammaClient::get_related_tag_relationships_by_slug) |
//! | `GET /tags/{id}/related-tags/tags` | [`get_related_tags`](GammaClient::get_related_tags) |
//! | `GET /tags/slug/{slug}/related-tags/tags` | [`get_related_tags_by_slug`](GammaClient::get_related_tags_by_slug) |
//! | `GET /series` | [`list_series`](GammaClient::list_series) |
//! | `GET /series/{id}` | [`get_series`](GammaClient::get_series) |
//! | `GET /series/{id}/comments/count` | [`get_series_comment_count`](GammaClient::get_series_comment_count) |
//! | `GET /series-summary/{id}` | [`get_series_summary`](GammaClient::get_series_summary) |
//! | `GET /series-summary/slug/{slug}` | [`get_series_summary_by_slug`](GammaClient::get_series_summary_by_slug) |
//! | `GET /comments` | [`list_comments`](GammaClient::list_comments) |
//! | `GET /comments/{id}` | [`get_comments_by_id`](GammaClient::get_comments_by_id) |
//! | `GET /comments/user_address/{user_address}` | [`list_comments_by_user`](GammaClient::list_comments_by_user) |
//! | `GET /teams` | [`list_teams`](GammaClient::list_teams) |
//! | `GET /teams/{id}` | [`get_team`](GammaClient::get_team) |
//! | `GET /sports` | [`get_sports_metadata`](GammaClient::get_sports_metadata) |
//! | `GET /sports/market-types` | [`get_sports_market_types`](GammaClient::get_sports_market_types) |
//! | `GET /public-search` | [`search`](GammaClient::search) |
//! | `GET /public-profile` | [`get_public_profile`](GammaClient::get_public_profile) |
//! | `GET /profiles/user_address/{user_address}` | [`get_profile`](GammaClient::get_profile) |
//!
//! # Conventions
//!
//! - **Responses.** The Gamma spec marks no response field as required, so every field of
//!   the response types is an `Option`. Fields the spec types as `number` are
//!   [`Decimal`](crate::Decimal)s that serialize back as JSON numbers; the few amounts the
//!   spec types as `string` (such as [`Market::liquidity`]) are decimals that serialize back
//!   as JSON strings. Date-time fields (`format: date-time`) are parsed leniently: besides
//!   RFC 3339, a value without a UTC offset, or a bare date, is read as UTC.
//! - **Live over docs.** Where the live API differs from the spec, this module follows the
//!   live API: the market lists `outcomes`, `outcomePrices`, `clobTokenIds` and
//!   `umaResolutionStatuses` are typed lists (decoded from the JSON-encoded string the API
//!   sends, or from the real array of the optimized search), `order` takes camelCase field
//!   names, offsets are capped (2000 for markets and events, 200 for comments), and fields the
//!   spec omits are modelled (marked "undocumented; observed live"). Every departure is listed
//!   in `SPEC_DEVIATIONS.md` in the repository root.
//! - **Pagination.** Offset listings return a plain `Vec` from `send()` and walk every page
//!   with `into_stream()`. The keyset listings return a page type with `items()` and
//!   `next_cursor()`; pass the cursor back with `cursor()`.
//! - **Ids.** Path and query ids the spec types as `integer` (market, event, tag, series,
//!   comment, team and creator ids) are id newtypes holding the digits as a string. They are
//!   checked before sending: an id that is not one or more ASCII digits is an
//!   [`Error::Validation`](crate::Error::Validation) naming the parameter. Slugs must be
//!   non-empty and not `.` or `..`.
//! - **Errors.** Where the spec documents an error body (`422`, `500` and `503` of the
//!   keyset listings, `400` and `404` of [`get_public_profile`](GammaClient::get_public_profile)),
//!   it carries a `type` and an `error` field, exposed as
//!   [`ApiError::error_type`](crate::ApiError::error_type) and
//!   [`ApiError::message`](crate::ApiError::message). Other errors, such as the `404` of the
//!   lookups by id or slug, are documented by status only.
//!
//! See <https://docs.polymarket.com/api-reference/predictions/overview>.

mod client;
mod comments;
mod events;
mod markets;
mod profiles;
mod search;
mod series;
mod sports;
mod status;
mod tags;
mod types;
mod util;

pub use client::{GammaClient, GammaClientBuilder};
pub use comments::{
    Comment, CommentCount, CommentId, CommentMedia, CommentParentEntityType, CommentPosition,
    CommentProfile, GetCommentsById, ListComments, ListCommentsByUser, Reaction,
};
pub use events::{
    Event, EventCreator, EventCreatorId, EventTweetCount, EventsKeysetPage, EventsPage, GetEvent,
    ListEventCreators, ListEvents, ListEventsKeyset, ListEventsPaginated, ListSportEventResults,
};
pub use markets::{
    ClobReward, FeeSchedule, GetMarket, GetMarketsInformation, ListMarkets, ListMarketsKeyset,
    Market, MarketDescription, MarketsKeysetPage,
};
pub use profiles::{Profile, PublicProfile, PublicProfileUser};
pub use search::{Search, SearchResults, SearchTag};
pub use series::{GetSeries, ListSeries, Series, SeriesId, SeriesSummary};
pub use sports::{ListTeams, SportsMarketTypes, SportsMetadata, Team, TeamId};
pub use tags::{
    GetRelatedTagRelationships, GetRelatedTags, GetTag, ListTags, RelatedTag, RelatedTagsStatus,
    Tag, TagId,
};
pub use types::{Category, Chat, Collection, ImageOptimization, Pagination, Template};

/// Shared identifiers, re-exported here for discoverability (also available from
/// [`crate::types`]).
pub use crate::types::{EventId, MarketId, QuestionId};
