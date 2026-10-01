//! Gamma API client (`https://gamma-api.polymarket.com`).
//!
//! Covers events, markets, tags, series, comments, sports, search and public profiles.
//! Start from [`GammaClient`]; every endpoint is a method on it.
//!
//! | Topic | Methods |
//! |---|---|
//! | Markets | [`list_markets`](GammaClient::list_markets), [`list_markets_keyset`](GammaClient::list_markets_keyset), [`get_market`](GammaClient::get_market), [`get_market_by_slug`](GammaClient::get_market_by_slug), [`get_market_tags`](GammaClient::get_market_tags), [`get_market_description`](GammaClient::get_market_description), [`get_markets_information`](GammaClient::get_markets_information), [`get_abridged_markets`](GammaClient::get_abridged_markets) |
//! | Events | [`list_events`](GammaClient::list_events), [`list_events_keyset`](GammaClient::list_events_keyset), [`list_events_paginated`](GammaClient::list_events_paginated), [`list_sport_event_results`](GammaClient::list_sport_event_results), [`get_event`](GammaClient::get_event), [`get_event_by_slug`](GammaClient::get_event_by_slug), [`get_event_tags`](GammaClient::get_event_tags), [`get_event_tweet_count`](GammaClient::get_event_tweet_count), [`get_event_comment_count`](GammaClient::get_event_comment_count), [`list_event_creators`](GammaClient::list_event_creators), [`get_event_creator`](GammaClient::get_event_creator) |
//! | Tags | [`list_tags`](GammaClient::list_tags), [`get_tag`](GammaClient::get_tag), [`get_tag_by_slug`](GammaClient::get_tag_by_slug), [`get_related_tags`](GammaClient::get_related_tags), [`get_related_tags_by_slug`](GammaClient::get_related_tags_by_slug), [`get_related_tag_relationships`](GammaClient::get_related_tag_relationships), [`get_related_tag_relationships_by_slug`](GammaClient::get_related_tag_relationships_by_slug) |
//! | Series | [`list_series`](GammaClient::list_series), [`get_series`](GammaClient::get_series), [`get_series_comment_count`](GammaClient::get_series_comment_count), [`get_series_summary`](GammaClient::get_series_summary), [`get_series_summary_by_slug`](GammaClient::get_series_summary_by_slug) |
//! | Comments | [`list_comments`](GammaClient::list_comments), [`get_comments_by_id`](GammaClient::get_comments_by_id), [`list_comments_by_user`](GammaClient::list_comments_by_user) |
//! | Sports | [`list_teams`](GammaClient::list_teams), [`get_team`](GammaClient::get_team), [`get_sports_metadata`](GammaClient::get_sports_metadata), [`get_sports_market_types`](GammaClient::get_sports_market_types) |
//! | Search | [`search`](GammaClient::search) |
//! | Profiles | [`get_public_profile`](GammaClient::get_public_profile), [`get_profile`](GammaClient::get_profile) |
//! | Status | [`status`](GammaClient::status) |
//!
//! The Gamma spec marks no response field as required, so every field of the response
//! types is an `Option`. Error responses carry a `type` and an `error` message, exposed as
//! [`ApiError::error_type`](crate::ApiError::error_type) and
//! [`ApiError::message`](crate::ApiError::message).
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
    Comment, CommentCount, CommentId, CommentParentEntityType, CommentPosition, CommentProfile,
    GetCommentsById, ListComments, ListCommentsByUser, Reaction,
};
pub use events::{
    Event, EventCreator, EventCreatorId, EventTweetCount, EventsKeysetPage, EventsPage, GetEvent,
    ListEventCreators, ListEvents, ListEventsKeyset, ListEventsPaginated, ListSportEventResults,
};
pub use markets::{
    FeeSchedule, GetMarket, GetMarketsInformation, ListMarkets, ListMarketsKeyset, Market,
    MarketDescription, MarketsKeysetPage,
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
