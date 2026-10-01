//! Search: `/public-search`.

use polyoxide_core::{Query, Result};
use serde::{Deserialize, Serialize};

use super::{Event, GammaClient, Pagination, Profile, TagId, util::setters};

/// Results of [`GammaClient::search`] (`components/schemas/Search`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    /// Matching events.
    pub events: Option<Vec<Event>>,
    /// Matching tags.
    pub tags: Option<Vec<SearchTag>>,
    /// Matching profiles.
    pub profiles: Option<Vec<Profile>>,
    /// Pagination metadata.
    pub pagination: Option<Pagination>,
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
    /// Searches markets, events and profiles.
    ///
    /// `q` is the (required) search text.
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
    limit_per_type: Option<u64>,
    page: Option<u64>,
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
        /// Whether to use the server-side cache.
        cache: bool;
        /// Only events with this status (the spec documents no values).
        events_status: into String;
        /// Maximum number of results per result type.
        limit_per_type: u64;
        /// Page number.
        page: u64;
        /// Only events with these tags.
        events_tag: many String;
        /// The `keep_closed_markets` parameter (an integer; the spec documents no values).
        keep_closed_markets: i64;
        /// Sort field.
        sort: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Also search tags.
        search_tags: bool;
        /// Also search profiles.
        search_profiles: bool;
        /// Only events with this recurrence.
        recurrence: into String;
        /// Exclude events with these tags.
        exclude_tag_id: many TagId;
        /// The `optimized` flag (undocumented beyond its boolean type).
        optimized: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<SearchResults> {
        let p = &self.params;
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
