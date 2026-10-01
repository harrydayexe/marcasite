//! Types shared by several Data API v2 endpoints: the response envelope, pagination,
//! identifiers, common enums, the combo-leg rows and the request validation helpers.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{
    Error, Query, Result, Service, ValidationError,
    pagination::{CursorPage, cursor_stream},
    serde_util,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::types::{ConditionId, EventId, MarketId, TokenId};

use super::DataClient;

/// The `outcome_index` sentinel meaning "the outcome could not be labeled".
///
/// See the *Units and Sentinels* section of
/// <https://docs.polymarket.com/api-reference/data-api/overview>.
pub const UNLABELED_OUTCOME_INDEX: i32 = 999;

/// The maximum number of distinct values accepted by the comma-separated `condition` and
/// `event_id` list parameters.
pub const MAX_LIST_VALUES: usize = 20;

/// One page of a cursor-paginated Data API v2 listing (the `{ data, pagination }` envelope).
///
/// Follow [`Pagination::next_cursor`] (or use the request's `into_stream()`) to read the
/// next page; the walk is over when it is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// The page's rows (`data` on the wire).
    #[serde(rename = "data")]
    pub items: Vec<T>,
    /// The paging envelope.
    pub pagination: Pagination,
}

impl<T> Page<T> {
    /// The cursor for the next page, or `None` on the last page.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.pagination.next_cursor.as_deref()
    }

    /// `true` if another page exists (exact, never inferred from page fullness).
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.pagination.has_more
    }

    pub(crate) fn into_cursor_page(self) -> CursorPage<T> {
        CursorPage::new(self.items, self.pagination.next_cursor)
    }
}

/// The paging envelope of a Data API v2 page (`components/schemas/Pagination`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Pagination {
    /// Page size this page was served with.
    pub limit: u32,
    /// Running item offset, for numbering rows across pages. Cosmetic: the cursor drives
    /// the actual seek, there is no total, and `offset` is never a request parameter.
    pub offset: u32,
    /// Exact: `true` iff another page exists.
    pub has_more: bool,
    /// Opaque, signed cursor for the next page; `None` on the last page.
    pub next_cursor: Option<String>,
}

/// The `{ "data": T }` envelope of the non-paginated routes.
#[derive(Debug, Deserialize)]
pub(crate) struct Envelope<T> {
    pub(crate) data: T,
}

polyoxide_core::string_enum! {
    /// A sort direction (`sort_direction`).
    pub enum SortDirection {
        /// Ascending.
        Asc => "ASC",
        /// Descending.
        Desc => "DESC",
    }
}

polyoxide_core::string_enum! {
    /// Which unit a `filter_amount` floor is expressed in (`filter_type`).
    pub enum FilterType {
        /// A USDC amount.
        Cash => "CASH",
        /// A number of shares (tokens).
        Tokens => "TOKENS",
    }
}

polyoxide_core::string_enum! {
    /// A board window (`time_period`, and the bucket width `interval` of
    /// `/v2/builders/volume`).
    pub enum TimePeriod {
        /// One day.
        Day => "day",
        /// One week.
        Week => "week",
        /// One month.
        Month => "month",
        /// All time (on `/v2/builders/volume`: yearly buckets).
        All => "all",
    }
}

polyoxide_core::string_enum! {
    /// Stable machine-readable classification of a Data API v2 failure
    /// (`components/schemas/ErrorCode`), carried in the `code` field of every error body.
    ///
    /// See <https://docs.polymarket.com/api-reference/data-api/overview>.
    pub enum ErrorCode {
        /// `400`: invalid query parameters, cursor or selector.
        InvalidRequest => "invalid_request",
        /// Unauthorized.
        Unauthorized => "unauthorized",
        /// `404`: not found.
        NotFound => "not_found",
        /// `405`: method not allowed.
        MethodNotAllowed => "method_not_allowed",
        /// `503`: the request deadline, the datastore statement timeout or the connection
        /// pool's acquire budget was exceeded. Retry after `Retry-After`.
        RequestTimeout => "request_timeout",
        /// `429`: rate limited. Retry after `Retry-After`.
        RateLimited => "rate_limited",
        /// `503`: a serving dependency is unavailable.
        DependencyUnavailable => "dependency_unavailable",
        /// `500`: internal server error.
        Internal => "internal",
    }
}

impl ErrorCode {
    /// The Data API v2 error code of `err`, if it is an API error returned by the Data API
    /// with a `code` field.
    ///
    /// ```
    /// use polyoxide::data::ErrorCode;
    ///
    /// fn should_fix_request(err: &polyoxide::Error) -> bool {
    ///     ErrorCode::from_error(err) == Some(ErrorCode::InvalidRequest)
    /// }
    /// # let _ = should_fix_request;
    /// ```
    #[must_use]
    pub fn from_error(err: &Error) -> Option<Self> {
        let api = err.api_error()?;
        if api.service() != Service::Data {
            return None;
        }
        api.code().map(Self::from)
    }
}

/// One leg of a combo, with its market and event enrichment
/// (`components/schemas/ComboLeg`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboLeg {
    /// Position of the leg within the combo, 0-based.
    pub leg_index: i32,
    /// Outcome token id of the leg.
    pub leg_position_id: TokenId,
    /// On-chain condition id of the leg's market.
    pub leg_condition_id: ConditionId,
    /// Index of the outcome the combo takes on this leg;
    /// [`UNLABELED_OUTCOME_INDEX`] means the outcome could not be labeled.
    pub leg_outcome_index: i32,
    /// Label of the outcome the combo takes on this leg.
    pub leg_outcome_label: String,
    /// Live resolution state of the leg.
    pub leg_status: ComboLegStatus,
    /// Live price of the leg outcome (Gamma marks).
    pub leg_current_price: Decimal,
    /// When the leg's market closed (Gamma `closed_time`); `None` while open.
    #[serde(default, with = "serde_util::datetime_option")]
    pub leg_resolved_at: Option<DateTime<Utc>>,
    /// The leg's market, with its event nested.
    pub market: ComboLegMarket,
}

polyoxide_core::string_enum! {
    /// Live resolution state of a combo leg (`leg_status`).
    pub enum ComboLegStatus {
        /// Unresolved.
        Open => "OPEN",
        /// Resolved in the combo's favour.
        ResolvedWin => "RESOLVED_WIN",
        /// Resolved against the combo.
        ResolvedLoss => "RESOLVED_LOSS",
    }
}

/// A combo leg's market, with its (single) event nested
/// (`components/schemas/ComboLegMarket`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboLegMarket {
    /// Gamma's own market id; not the on-chain condition id (that is the leg's
    /// `leg_condition_id`).
    pub market_id: MarketId,
    /// Market slug; the URL segment on polymarket.com.
    pub slug: String,
    /// Short per-leg label (`group_item_title`), falling back to the question.
    pub title: String,
    /// Label of the leg's outcome on this market.
    pub outcome: String,
    /// Market image URL.
    pub image_url: String,
    /// Market icon URL.
    pub icon_url: String,
    /// Gamma market category (e.g. `sports`).
    pub category: String,
    /// Gamma market subcategory.
    pub subcategory: String,
    /// Reserved; always empty today.
    pub tags: Vec<String>,
    /// Market end date; `None` when Gamma has none (served as `""`).
    #[serde(with = "serde_util::datetime_option")]
    pub end_date: Option<DateTime<Utc>>,
    /// The market's parent event.
    pub event: ComboLegEvent,
    /// The market's full question; `""` when Gamma has none.
    pub question: Option<String>,
    /// Raw short per-leg label, without the fallback `title` applies; `""` when Gamma has
    /// none.
    pub group_item_title: Option<String>,
    /// The market's outcome labels, in outcome-index order; empty when Gamma has none.
    pub outcomes: Option<Vec<String>>,
    /// Granular sports market type (e.g. `totals`); `""` for non-sports markets.
    pub sports_market_type: Option<String>,
    /// The sports line the market is quoted on; `None` when it has none.
    pub line: Option<Decimal>,
}

/// A combo leg market's event (`components/schemas/ComboLegEvent`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboLegEvent {
    /// Gamma event id.
    pub event_id: EventId,
    /// Event slug; the URL segment on polymarket.com.
    pub event_slug: String,
    /// Event title.
    pub event_title: String,
    /// Event image URL.
    pub event_image: String,
}

// ---------------------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------------------

impl DataClient {
    /// Sends a `GET` to a non-paginated route and unwraps the `{ data }` envelope.
    pub(crate) async fn fetch_data<T: DeserializeOwned>(
        &self,
        path: &[&str],
        query: Query,
    ) -> Result<T> {
        let envelope: Envelope<T> = self.transport.get(path).query(query).send().await?;
        Ok(envelope.data)
    }

    /// Sends a `GET` to a paginated route and decodes the `{ data, pagination }` page.
    pub(crate) async fn fetch_page<T: DeserializeOwned>(
        &self,
        path: &[&str],
        query: Query,
    ) -> Result<Page<T>> {
        self.transport.get(path).query(query).send().await
    }
}

/// Lazily walks a paginated route from `start`, building each page's query (with the
/// same filters) through `query`.
pub(crate) fn page_stream<T, F>(
    client: DataClient,
    path: &'static [&'static str],
    start: Option<String>,
    mut query: F,
) -> impl Stream<Item = Result<T>> + Send + 'static
where
    T: DeserializeOwned + Send + 'static,
    F: FnMut(Option<&str>) -> Result<Query> + Send + 'static,
{
    cursor_stream(start, move |cursor| {
        let query = query(cursor.as_deref());
        let client = client.clone();
        async move {
            client
                .fetch_page::<T>(path, query?)
                .await
                .map(Page::into_cursor_page)
        }
    })
}

/// Fails if `limit` exceeds the documented maximum.
pub(crate) fn check_limit(limit: Option<u32>, max: u32) -> Result<()> {
    match limit {
        Some(limit) if limit > max => {
            Err(ValidationError::new("limit", format!("must be at most {max}, got {limit}")).into())
        }
        _ => Ok(()),
    }
}

/// The number of distinct values in a list parameter.
pub(crate) fn distinct<S: AsRef<str>>(values: &[S]) -> usize {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<BTreeSet<_>>()
        .len()
}

/// Fails if a comma-separated list parameter has more than [`MAX_LIST_VALUES`] distinct
/// values, or contains an empty value (which would corrupt the comma-separated list).
pub(crate) fn check_list<S: AsRef<str>>(parameter: &'static str, values: &[S]) -> Result<()> {
    if values.iter().any(|v| v.as_ref().trim().is_empty()) {
        return Err(ValidationError::new(parameter, "values must not be empty").into());
    }
    let count = distinct(values);
    if count > MAX_LIST_VALUES {
        return Err(ValidationError::new(
            parameter,
            format!("at most {MAX_LIST_VALUES} distinct values are accepted, got {count}"),
        )
        .into());
    }
    Ok(())
}

/// As [`check_list`], and also fails if the list is empty.
pub(crate) fn check_required_list<S: AsRef<str>>(
    parameter: &'static str,
    values: &[S],
) -> Result<()> {
    if values.is_empty() {
        return Err(ValidationError::new(parameter, "at least one value is required").into());
    }
    check_list(parameter, values)
}

/// Collects list-parameter values into the given id type.
pub(crate) fn collect_ids<I, T>(values: I) -> Vec<T>
where
    I: IntoIterator,
    I::Item: Into<T>,
{
    values.into_iter().map(Into::into).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pagination object from the example response in
    /// `docs/api-reference/data-api/overview.md` ("Make a First Request").
    #[test]
    fn deserializes_overview_pagination() {
        let json = r#"{
            "limit": 1,
            "offset": 0,
            "has_more": true,
            "next_cursor": "eyJkYXRhIjp7InR5cGUiOiJwb3NpdGlvbnMi…"
        }"#;
        let pagination: Pagination = serde_json::from_str(json).unwrap();
        assert_eq!(pagination.limit, 1);
        assert_eq!(pagination.offset, 0);
        assert!(pagination.has_more);
        assert_eq!(
            pagination.next_cursor.as_deref(),
            Some("eyJkYXRhIjp7InR5cGUiOiJwb3NpdGlvbnMi…")
        );

        let last: Pagination =
            serde_json::from_str(r#"{"limit":1,"offset":3,"has_more":false,"next_cursor":null}"#)
                .unwrap();
        assert_eq!(last.next_cursor, None);
        // `next_cursor` is optional in the schema; the other three fields are required.
        assert!(serde_json::from_str::<Pagination>(r#"{"limit":1,"offset":0}"#).is_err());
    }

    #[test]
    fn page_exposes_cursor() {
        let page: Page<u8> = serde_json::from_str(
            r#"{"data":[1,2],"pagination":{"limit":2,"offset":0,"has_more":true,"next_cursor":"c"}}"#,
        )
        .unwrap();
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.next_cursor(), Some("c"));
        assert!(page.has_more());
        let cursor_page = page.into_cursor_page();
        assert_eq!(cursor_page.next_cursor.as_deref(), Some("c"));
    }

    /// Values from `components/schemas/ErrorCode` in `docs/specs/data-v2-openapi.json`.
    #[test]
    fn error_codes() {
        for (wire, code) in [
            ("invalid_request", ErrorCode::InvalidRequest),
            ("unauthorized", ErrorCode::Unauthorized),
            ("not_found", ErrorCode::NotFound),
            ("method_not_allowed", ErrorCode::MethodNotAllowed),
            ("request_timeout", ErrorCode::RequestTimeout),
            ("rate_limited", ErrorCode::RateLimited),
            ("dependency_unavailable", ErrorCode::DependencyUnavailable),
            ("internal", ErrorCode::Internal),
        ] {
            assert_eq!(ErrorCode::from(wire), code);
            assert_eq!(code.as_str(), wire);
        }
        assert!(ErrorCode::from("brand_new").is_unknown());
        let validation = Error::from(ValidationError::new("limit", "too big"));
        assert_eq!(ErrorCode::from_error(&validation), None);
    }

    #[test]
    fn list_validation() {
        let ids: Vec<String> = (0..21).map(|i| format!("0x{i}")).collect();
        assert!(check_list("condition", &ids).is_err());
        assert!(check_list("condition", ids.get(..20).unwrap()).is_ok());
        // Duplicates count once.
        let dupes = vec!["0x1"; 30];
        assert!(check_list("condition", &dupes).is_ok());
        assert!(check_list("condition", &["0x1", " "]).is_err());
        assert!(check_required_list::<&str>("condition", &[]).is_err());
        assert!(check_limit(Some(1001), 1000).is_err());
        assert!(check_limit(Some(1000), 1000).is_ok());
        assert!(check_limit(None, 1000).is_ok());
    }
}
