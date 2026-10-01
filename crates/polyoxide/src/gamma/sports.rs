//! Sports: `/teams`, `/teams/{id}`, `/sports` and `/sports/market-types`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{Query, Result, pagination::offset_stream, serde_util};
use serde::{Deserialize, Serialize};

use super::{GammaClient, TagId, util::setters};

polyoxide_core::string_id! {
    /// A Gamma team id (an integer on the wire).
    pub struct TeamId;
}

/// A sports team (`components/schemas/Team`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Team {
    /// Team id (an integer on the wire).
    #[serde(default, with = "serde_util::integer_id_option")]
    pub id: Option<TeamId>,
    /// Team name.
    pub name: Option<String>,
    /// League.
    pub league: Option<String>,
    /// Record.
    pub record: Option<String>,
    /// Logo URL.
    pub logo: Option<String>,
    /// Abbreviation.
    pub abbreviation: Option<String>,
    /// Alias.
    pub alias: Option<String>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Configuration of one sport (`components/schemas/SportsMetadata`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SportsMetadata {
    /// The sport identifier or abbreviation.
    pub sport: Option<String>,
    /// URL of the sport's logo or image.
    pub image: Option<String>,
    /// URL of the official resolution source for the sport (e.g. the league website).
    pub resolution: Option<String>,
    /// Preferred ordering for display, typically `"home"` or `"away"`.
    pub ordering: Option<String>,
    /// Comma-separated list of tag ids associated with the sport; see
    /// [`tag_ids`](Self::tag_ids).
    pub tags: Option<String>,
    /// Series identifier linking the sport to a tournament or season series.
    pub series: Option<String>,
}

impl SportsMetadata {
    /// The tag ids in [`tags`](Self::tags), split on commas (empty entries are skipped).
    pub fn tag_ids(&self) -> impl Iterator<Item = TagId> + '_ {
        self.tags
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(TagId::from)
    }
}

/// The valid sports market types (`components/schemas/SportsMarketTypesResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SportsMarketTypes {
    /// Every valid sports market type.
    pub market_types: Option<Vec<String>>,
}

impl GammaClient {
    /// Lists sports teams (offset pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/sports/list-teams>.
    pub fn list_teams(&self) -> ListTeams {
        ListTeams {
            client: self.clone(),
            params: ListTeamsParams::default(),
        }
    }

    /// Gets a sports team by id.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getTeam` (no published doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing team is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_team(&self, id: impl Into<TeamId>) -> Result<Team> {
        let id = id.into();
        self.transport.get(&["teams", id.as_str()]).send().await
    }

    /// Gets the configuration of every sport: images, resolution sources and related tag
    /// and series identifiers.
    ///
    /// See <https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn get_sports_metadata(&self) -> Result<Vec<SportsMetadata>> {
        self.transport.get(&["sports"]).send().await
    }

    /// Gets the valid sports market types (the values of
    /// [`Market::sports_market_type`](super::Market::sports_market_type)).
    ///
    /// See <https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn get_sports_market_types(&self) -> Result<SportsMarketTypes> {
        self.transport.get(&["sports", "market-types"]).send().await
    }
}

/// Request builder for [`GammaClient::list_teams`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTeams {
    client: GammaClient,
    params: ListTeamsParams,
}

#[derive(Debug, Clone, Default)]
struct ListTeamsParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    league: Vec<String>,
    name: Vec<String>,
    abbreviation: Vec<String>,
}

impl ListTeams {
    setters! {
        /// Maximum number of teams per page.
        limit: u64;
        /// Number of teams to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only teams in these leagues.
        league: many String;
        /// Only teams with these names.
        name: many String;
        /// Only teams with these abbreviations.
        abbreviation: many String;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Team>> {
        let p = &self.params;
        let mut q = Query::new();
        q.push_opt("limit", p.limit)
            .push_opt("offset", offset)
            .push_opt("order", p.order.as_deref())
            .push_opt("ascending", p.ascending)
            .push_all("league", &p.league)
            .push_all("name", &p.name)
            .push_all("abbreviation", &p.abbreviation);
        self.client.transport.get(&["teams"]).query(q).send().await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Team>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every team from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page. A page shorter than
    /// [`limit`](Self::limit) does not end it, because the server may cap the page size, so
    /// the last request returns an empty page.
    pub fn into_stream(self) -> impl Stream<Item = Result<Team>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        offset_stream(start, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Team` in
    /// `docs/specs/gamma-openapi.yaml`.
    #[test]
    fn team_id_is_an_integer_on_the_wire() {
        let json = serde_json::json!({
            "id": 42,
            "name": "Lakers",
            "league": "nba",
            "record": "10-2",
            "logo": null,
            "abbreviation": "LAL",
            "alias": null,
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": null
        });
        let team: Team = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(team.id, Some(TeamId::from("42")));
        assert_eq!(team.abbreviation.as_deref(), Some("LAL"));
        assert_eq!(serde_json::to_value(&team).unwrap(), json);
    }

    /// Field descriptions from `components/schemas/SportsMetadata` in
    /// `docs/specs/gamma-openapi.yaml`.
    #[test]
    fn sports_metadata_splits_tag_ids() {
        let json = r#"{
            "sport": "nba",
            "image": "https://example.com/nba.png",
            "resolution": "https://www.nba.com/",
            "ordering": "home",
            "tags": "1,745, 100639,",
            "series": "10345"
        }"#;
        let sport: SportsMetadata = serde_json::from_str(json).unwrap();
        let ids: Vec<_> = sport.tag_ids().collect();
        assert_eq!(
            ids,
            vec![TagId::from("1"), TagId::from("745"), TagId::from("100639")]
        );
        let none: SportsMetadata = serde_json::from_str("{}").unwrap();
        assert_eq!(none.tag_ids().count(), 0);
    }

    #[test]
    fn deserializes_market_types() {
        let types: SportsMarketTypes =
            serde_json::from_str(r#"{"marketTypes":["type-a","type-b"]}"#).unwrap();
        assert_eq!(types.market_types.unwrap().len(), 2);
    }
}
