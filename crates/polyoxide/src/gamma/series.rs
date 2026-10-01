//! Series: `/series`, `/series/{id}`, `/series/{id}/comments/count` and
//! `/series-summary/...`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{Query, Result, pagination::offset_stream, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{Category, Chat, Collection, CommentCount, Event, GammaClient, Tag, util::setters};

polyoxide_core::string_id! {
    /// A Gamma series id (sent as a string in responses; an integer in paths and filters).
    pub struct SeriesId;
}

/// A series: a recurring set of events (`components/schemas/Series`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Series {
    /// Series id.
    pub id: Option<SeriesId>,
    /// Ticker.
    pub ticker: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// Series type.
    pub series_type: Option<String>,
    /// Recurrence.
    pub recurrence: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Layout.
    pub layout: Option<String>,
    /// Whether the series is active.
    pub active: Option<bool>,
    /// Whether the series is closed.
    pub closed: Option<bool>,
    /// Whether the series is archived.
    pub archived: Option<bool>,
    /// Whether the series is flagged as new.
    pub new: Option<bool>,
    /// Whether the series is featured.
    pub featured: Option<bool>,
    /// Whether the series is restricted.
    pub restricted: Option<bool>,
    /// Whether the series is a template.
    pub is_template: Option<bool>,
    /// Template variables. The spec types this as a boolean on series (a string on events
    /// and collections).
    pub template_variables: Option<bool>,
    /// Publication time. The spec types this as a plain string with no format.
    pub published_at: Option<String>,
    /// Creator.
    pub created_by: Option<String>,
    /// Last updater.
    pub updated_by: Option<String>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Whether comments are enabled.
    pub comments_enabled: Option<bool>,
    /// Competitiveness. The spec types this as a string on series (a number on events and
    /// markets), so it is kept exactly as sent.
    pub competitive: Option<String>,
    /// 24-hour volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_24hr: Option<Decimal>,
    /// Volume.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume: Option<Decimal>,
    /// Liquidity.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub liquidity: Option<Decimal>,
    /// Start date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub start_date: Option<DateTime<Utc>>,
    /// Pyth token id (wire name `pythTokenID`).
    #[serde(rename = "pythTokenID")]
    pub pyth_token_id: Option<String>,
    /// CoinGecko asset name.
    pub cg_asset_name: Option<String>,
    /// Score.
    pub score: Option<i64>,
    /// The series' events.
    pub events: Option<Vec<Event>>,
    /// Collections.
    pub collections: Option<Vec<Collection>>,
    /// Categories.
    pub categories: Option<Vec<Category>>,
    /// Tags.
    pub tags: Option<Vec<Tag>>,
    /// Number of comments.
    pub comment_count: Option<i64>,
    /// Chats (included with `include_chat=true`).
    pub chats: Option<Vec<Chat>>,
}

/// A summary of a series' event dates (`components/schemas/SeriesSummary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SeriesSummary {
    /// Series id.
    pub id: Option<SeriesId>,
    /// Title.
    pub title: Option<String>,
    /// URL slug.
    pub slug: Option<String>,
    /// Event dates. The spec types these as plain strings with no format.
    pub event_dates: Option<Vec<String>>,
    /// Event weeks.
    pub event_weeks: Option<Vec<i64>>,
    /// Earliest week with an open event (wire name `earliest_open_week`).
    #[serde(rename = "earliest_open_week")]
    pub earliest_open_week: Option<i64>,
    /// Earliest date with an open event (wire name `earliest_open_date`). The spec types
    /// this as a plain string with no format.
    #[serde(rename = "earliest_open_date")]
    pub earliest_open_date: Option<String>,
}

impl GammaClient {
    /// Lists series (offset pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/series/list-series>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let gamma = polyoxide::gamma::GammaClient::new()?;
    /// let series = gamma.list_series().closed(false).limit(10).send().await?;
    /// # let _ = series;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_series(&self) -> ListSeries {
        ListSeries {
            client: self.clone(),
            params: ListSeriesParams::default(),
        }
    }

    /// Gets a series by id.
    ///
    /// Fails with [`Error::Api`](crate::Error::Api) (status `404`) if the series does not
    /// exist; see [`Error::is_not_found`](crate::Error::is_not_found).
    ///
    /// See <https://docs.polymarket.com/api-reference/series/get-series-by-id>.
    pub fn get_series(&self, id: impl Into<SeriesId>) -> GetSeries {
        GetSeries {
            client: self.clone(),
            id: id.into(),
            params: GetSeriesParams::default(),
        }
    }

    /// Gets the number of comments on a series.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getSeriesCommentsCount` (no
    /// published doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing series is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_series_comment_count(&self, id: impl Into<SeriesId>) -> Result<CommentCount> {
        let id = id.into();
        self.transport
            .get(&["series", id.as_str(), "comments", "count"])
            .send()
            .await
    }

    /// Gets a series summary by series id.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getSeriesSummaryById` (no published
    /// doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing series is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_series_summary(&self, id: impl Into<SeriesId>) -> Result<SeriesSummary> {
        let id = id.into();
        self.transport
            .get(&["series-summary", id.as_str()])
            .send()
            .await
    }

    /// Gets a series summary by series slug.
    ///
    /// See `docs/specs/gamma-openapi.yaml`, operationId `getSeriesSummaryBySlug` (no
    /// published doc page).
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing series is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn get_series_summary_by_slug(&self, slug: impl AsRef<str>) -> Result<SeriesSummary> {
        self.transport
            .get(&["series-summary", "slug", slug.as_ref()])
            .send()
            .await
    }
}

/// Request builder for [`GammaClient::list_series`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSeries {
    client: GammaClient,
    params: ListSeriesParams,
}

#[derive(Debug, Clone, Default)]
struct ListSeriesParams {
    limit: Option<u64>,
    offset: Option<u64>,
    order: Option<String>,
    ascending: Option<bool>,
    slug: Vec<String>,
    categories_ids: Vec<i64>,
    categories_labels: Vec<String>,
    closed: Option<bool>,
    include_chat: Option<bool>,
    recurrence: Option<String>,
    exclude_events: Option<bool>,
}

impl ListSeriesParams {
    fn query(&self, offset: Option<u64>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("offset", offset)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending)
            .push_all("slug", &self.slug)
            .push_all("categories_ids", &self.categories_ids)
            .push_all("categories_labels", &self.categories_labels)
            .push_opt("closed", self.closed)
            .push_opt("include_chat", self.include_chat)
            .push_opt("recurrence", self.recurrence.as_deref())
            .push_opt("exclude_events", self.exclude_events);
        q
    }
}

impl ListSeries {
    setters! {
        /// Maximum number of series per page.
        limit: u64;
        /// Number of series to skip.
        offset: u64;
        /// Comma-separated list of fields to order by.
        order: into String;
        /// Sort ascending (`true`) or descending (`false`).
        ascending: bool;
        /// Only series with these slugs.
        slug: many String;
        /// Only series in these category ids.
        categories_ids: many i64;
        /// Only series in these category labels.
        categories_labels: many String;
        /// Only closed (`true`) or only open (`false`) series.
        closed: bool;
        /// Include each series' chats.
        include_chat: bool;
        /// Only series with this recurrence.
        recurrence: into String;
        /// Leave out each series' events.
        exclude_events: bool;
    }

    async fn fetch(&self, offset: Option<u64>) -> Result<Vec<Series>> {
        self.client
            .transport
            .get(&["series"])
            .query(self.params.query(offset))
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<Series>> {
        self.fetch(self.params.offset).await
    }

    /// Streams every series from the configured offset onwards, fetching pages lazily.
    ///
    /// The stream ends at the first empty page. A page shorter than
    /// [`limit`](Self::limit) does not end it, because the server may cap the page size, so
    /// the last request returns an empty page.
    pub fn into_stream(self) -> impl Stream<Item = Result<Series>> + Send + 'static {
        let start = self.params.offset.unwrap_or(0);
        offset_stream(start, move |offset| {
            let request = self.clone();
            async move { request.fetch(Some(offset)).await }
        })
    }
}

/// Request builder for [`GammaClient::get_series`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetSeries {
    client: GammaClient,
    id: SeriesId,
    params: GetSeriesParams,
}

#[derive(Debug, Clone, Default)]
struct GetSeriesParams {
    include_chat: Option<bool>,
}

impl GetSeries {
    setters! {
        /// Include the series' chats.
        include_chat: bool;
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing series is an [`Error::Api`](crate::Error::Api)
    /// with status `404`.
    pub async fn send(self) -> Result<Series> {
        let mut query = Query::new();
        query.push_opt("include_chat", self.params.include_chat);
        self.client
            .transport
            .get(&["series", self.id.as_str()])
            .query(query)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Series` and `SeriesSummary` in
    /// `docs/specs/gamma-openapi.yaml` (the docs publish no example body).
    #[test]
    fn deserializes_series_wire_names_and_types() {
        let json = r#"{
            "id": "10",
            "slug": "nba",
            "templateVariables": true,
            "competitive": "0.9",
            "volume24hr": 1000.5,
            "pythTokenID": "0xpyth",
            "events": [{"id": "1"}],
            "commentCount": 4
        }"#;
        let series: Series = serde_json::from_str(json).unwrap();
        assert_eq!(series.id, Some(SeriesId::from("10")));
        assert_eq!(series.template_variables, Some(true));
        assert_eq!(series.competitive.as_deref(), Some("0.9"));
        assert_eq!(series.volume_24hr, Some(Decimal::new(10_005, 1)));
        assert_eq!(series.pyth_token_id.as_deref(), Some("0xpyth"));
        assert_eq!(series.events.unwrap().len(), 1);

        let summary: SeriesSummary = serde_json::from_str(
            r#"{"id":"10","eventDates":["2024-01-01"],"eventWeeks":[1,2],
                "earliest_open_week":2,"earliest_open_date":"2024-01-08"}"#,
        )
        .unwrap();
        assert_eq!(summary.event_weeks, Some(vec![1, 2]));
        assert_eq!(summary.earliest_open_week, Some(2));
        assert_eq!(summary.earliest_open_date.as_deref(), Some("2024-01-08"));
    }
}
