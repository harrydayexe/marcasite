//! Search: `/public-search`.

use polyoxide_core::{Query, Result};
use serde::{Deserialize, Serialize};

use super::{
    Event, GammaClient, Pagination, Profile, TagId,
    util::{check_integer_ids, setters},
};

/// Results of [`GammaClient::search`] (`components/schemas/Search`).
///
/// The spec calls this schema `Search`; in this crate [`Search`] is the request builder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[doc(alias = "Search")]
pub struct SearchResults {
    /// Matching events.
    pub events: Option<Vec<Event>>,
    /// Matching tags.
    pub tags: Option<Vec<SearchTag>>,
    /// Matching profiles.
    pub profiles: Option<Vec<Profile>>,
    /// Pagination metadata. Sent by the regular search; the
    /// [`optimized`](Search::optimized) search sends [`has_more`](Self::has_more) instead.
    pub pagination: Option<Pagination>,
    /// Whether more results are available (wire name `hasMore`; undocumented, observed live
    /// only with `optimized=true`, where it replaces [`pagination`](Self::pagination)).
    #[serde(rename = "hasMore")]
    pub has_more: Option<bool>,
}

/// A tag matched by [`GammaClient::search`] (`components/schemas/SearchTag`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchTag {
    /// Tag id.
    pub id: Option<TagId>,
    /// Display label.
    pub label: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Number of events with the tag.
    pub event_count: Option<i64>,
}

impl GammaClient {
    /// Searches markets, events and profiles (`GET /public-search`).
    ///
    /// `q` is the (required) search text. The endpoint is paginated with
    /// [`page`](Search::page) and reports `pagination.hasMore` (`hasMore` at the top level
    /// with [`optimized`](Search::optimized)); there is no `into_stream()` because the docs do
    /// not say whether pages are numbered from 0 or 1.
    ///
    /// See <https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let results = gamma
    ///     .search("election")
    ///     .limit_per_type(5)
    ///     .search_profiles(true)
    ///     .send()
    ///     .await?;
    /// for event in results.events.unwrap_or_default() {
    ///     println!("{:?}", event.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search(&self, q: impl Into<String>) -> Search {
        Search {
            client: self.clone(),
            q: q.into(),
            params: SearchParams::default(),
        }
    }
}

/// Request builder for [`GammaClient::search`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct Search {
    client: GammaClient,
    q: String,
    params: SearchParams,
}

#[derive(Debug, Clone, Default)]
struct SearchParams {
    cache: Option<bool>,
    events_status: Option<String>,
    limit_per_type: Option<u32>,
    page: Option<u32>,
    events_tag: Vec<String>,
    keep_closed_markets: Option<i64>,
    sort: Option<String>,
    ascending: Option<bool>,
    search_tags: Option<bool>,
    search_profiles: Option<bool>,
    recurrence: Option<String>,
    exclude_tag_id: Vec<TagId>,
    optimized: Option<bool>,
}

impl Search {
    setters! {
        /// The `cache` flag (documented only as a boolean).
        cache: bool;
        /// The `events_status` filter (a string; the spec documents no values).
        events_status: into String;
        /// The `limit_per_type` parameter (an integer).
        limit_per_type: u32;
        /// The `page` parameter (an integer; the docs do not say whether pages are numbered
        /// from 0 or 1).
        page: u32;
        /// The `events_tag` filter (repeated strings).
        events_tags => events_tag: many String;
        /// The `keep_closed_markets` parameter (an integer; the spec documents no values).
        keep_closed_markets: i64;
        /// The `sort` parameter (a string; the spec documents no values).
        sort: into String;
        /// Sort ascending (`true`) or descending (`false`) (`ascending`).
        ascending: bool;
        /// The `search_tags` flag (documented only as a boolean).
        search_tags: bool;
        /// The `search_profiles` flag (documented only as a boolean).
        search_profiles: bool;
        /// The `recurrence` filter (a string; the spec documents no values).
        recurrence: into String;
        /// Tag ids to exclude (`exclude_tag_id`, repeated). The spec types them as
        /// integers, so an id that is not one or more ASCII digits is rejected before
        /// sending with [`Error::Validation`](crate::Error::Validation).
        exclude_tag_ids => exclude_tag_id: many TagId;
        /// The `optimized` flag (documented only as a boolean).
        ///
        /// With `true` live returns a slimmer result: the events and their markets carry only
        /// a few fields (so most [`Event`] and [`Market`](super::Market) fields are `None`),
        /// the markets' `outcomes` and `outcomePrices` are real JSON arrays instead of
        /// JSON-encoded strings (both decode to the same lists), and the response has
        /// [`has_more`](SearchResults::has_more) instead of
        /// [`pagination`](SearchResults::pagination). See `SPEC_DEVIATIONS.md`.
        optimized: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) (parameter `exclude_tag_id`) if an
    ///   [`exclude_tag_ids`](Self::exclude_tag_ids) entry is not an integer, checked before
    ///   sending;
    /// - otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<SearchResults> {
        let p = &self.params;
        check_integer_ids("exclude_tag_id", &p.exclude_tag_id)?;
        let mut q = Query::new();
        q.push("q", &self.q)
            .push_opt("cache", p.cache)
            .push_opt("events_status", p.events_status.as_deref())
            .push_opt("limit_per_type", p.limit_per_type)
            .push_opt("page", p.page)
            .push_all("events_tag", &p.events_tag)
            .push_opt("keep_closed_markets", p.keep_closed_markets)
            .push_opt("sort", p.sort.as_deref())
            .push_opt("ascending", p.ascending)
            .push_opt("search_tags", p.search_tags)
            .push_opt("search_profiles", p.search_profiles)
            .push_opt("recurrence", p.recurrence.as_deref())
            .push_all("exclude_tag_id", &p.exclude_tag_id)
            .push_opt("optimized", p.optimized);
        self.client
            .transport
            .get(&["public-search"])
            .query(q)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Search` and `SearchTag` in
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body).
    #[test]
    fn deserializes_search_results() {
        let json = r#"{
            "events": [{"id": "1", "title": "Election"}],
            "tags": [{"id": "2", "label": "Politics", "slug": "politics", "event_count": 12}],
            "profiles": null,
            "pagination": {"hasMore": true, "totalResults": 40}
        }"#;
        let results: SearchResults = serde_json::from_str(json).unwrap();
        assert_eq!(results.events.unwrap().len(), 1);
        let tag = &results.tags.as_ref().unwrap()[0];
        assert_eq!(tag.event_count, Some(12));
        assert_eq!(tag.id, Some(TagId::from("2")));
        assert_eq!(results.profiles, None);
        assert_eq!(results.pagination.unwrap().total_results, Some(40));
    }
}
