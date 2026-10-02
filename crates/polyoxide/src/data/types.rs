//! Types shared by several Data API v2 endpoints: the response envelope, pagination,
//! identifiers, common enums, the combo-leg rows and the request validation helpers.

use std::collections::HashSet;

use crate::Paginated;
use chrono::{DateTime, Utc};
use polyoxide_core::{
    Error, Query, Result, Service, ValidationError,
    pagination::{CursorPage, cursor_stream},
    serde_util, validate,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::types::{Address, ConditionId, EventId, MarketId, TokenId};

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
/// Read the rows with [`items`](Self::items) (or [`into_items`](Self::into_items)) and pass
/// [`next_cursor`](Self::next_cursor) to the request's `cursor(..)` setter for the next
/// page, or use the request's `into_stream()` to walk every page. The walk is over when
/// `next_cursor()` is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// The page's rows.
    pub data: Vec<T>,
    /// The paging envelope, as served.
    pub pagination: Pagination,
    /// The `x-trace-id` header of the response that carried this page (not part of the
    /// JSON body).
    #[serde(skip)]
    trace_id: Option<String>,
}

impl<T> Page<T> {
    /// The page's rows.
    #[must_use]
    pub fn items(&self) -> &[T] {
        &self.data
    }

    /// Consumes the page and returns its rows.
    #[must_use]
    pub fn into_items(self) -> Vec<T> {
        self.data
    }

    /// The cursor for the next page, or `None` when there is no next page.
    ///
    /// This is [`Pagination::next_cursor`], except that it is also `None` when the server
    /// reports [`has_more`](Pagination::has_more) `false` or serves an empty cursor, so
    /// that a contradictory page can never restart a walk.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.pagination
            .next_cursor
            .as_deref()
            .filter(|cursor| self.pagination.has_more && !cursor.is_empty())
    }

    /// `true` if another page exists, as reported by the server (`pagination.has_more`,
    /// documented as exact: never inferred from page fullness).
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.pagination.has_more
    }

    /// The trace id the server sent with this page in its `x-trace-id` header, if any.
    ///
    /// The API echoes one on every response; quote it when reporting wrong-looking data.
    /// It is `None` for a page that was not received from the API (e.g. deserialized from
    /// JSON by the caller).
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    pub(crate) fn into_cursor_page(self) -> CursorPage<T> {
        let next_cursor = self.next_cursor().map(str::to_owned);
        CursorPage::new(self.data, next_cursor)
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
    /// Opaque, signed cursor for the next page; `None` on the last page. Prefer
    /// [`Page::next_cursor`], which is also `None` when `has_more` is `false`.
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
    /// (`components/schemas/ErrorCode`), carried in the `code` field of the error body.
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
        /// pool's acquire budget was exceeded.
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
    /// `None` for other errors, and for a Data API error body without `code` (such as the
    /// `{ "error": "..." }` body shown in the overview), whose message is still available
    /// from [`ApiError::message`](crate::ApiError::message).
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
    ///
    /// Live, the two routes that carry combo legs disagree on its format for the same leg:
    /// `/v2/activity/combos` serves the market's bytes32 condition id (`0x` plus 64 hex
    /// digits), while `/v2/positions/combos` serves a 62-digit, `0x01`/`0x02`-prefixed
    /// value (e.g. `0x0104db2bc4f21eef1caf06186806e548800000000000000000000000000000`)
    /// that is not the market's condition id. The same leg has the same
    /// [`leg_position_id`](Self::leg_position_id) on both. The value is kept as served;
    /// do not use it as a `condition` filter on the single-market routes.
    pub leg_condition_id: ConditionId,
    /// Index of the outcome the combo takes on this leg;
    /// [`UNLABELED_OUTCOME_INDEX`] means the outcome could not be labeled.
    pub leg_outcome_index: i32,
    /// Label of the outcome the combo takes on this leg.
    pub leg_outcome_label: String,
    /// Live resolution state of the leg.
    pub leg_status: ComboLegStatus,
    /// Live price of the leg outcome (Gamma marks).
    #[serde(with = "serde_util::decimal_number")]
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
///
/// The docs say the display-metadata fields `question`, `group_item_title`, `outcomes`,
/// `sports_market_type` and `line` default when absent, so that cached payloads written
/// before they existed still deserialize, while the query always serves them.
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
    /// Market end date; `None` when Gamma has none (served as `""`, and serialized back
    /// as `""`).
    #[serde(with = "datetime_or_empty")]
    pub end_date: Option<DateTime<Utc>>,
    /// The market's parent event.
    pub event: ComboLegEvent,
    /// The market's full question; `""` when Gamma has none. `None` when absent from the
    /// payload (an older cached payload).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Raw short per-leg label, without the fallback `title` applies; `""` when Gamma has
    /// none. `None` when absent from the payload (an older cached payload).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_item_title: Option<String>,
    /// The market's outcome labels, in outcome-index order; empty when Gamma has none.
    /// `None` when absent from the payload (an older cached payload).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<Vec<String>>,
    /// Granular sports market type (e.g. `totals`); `""` for non-sports markets. `None`
    /// when absent from the payload (an older cached payload).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sports_market_type: Option<String>,
    /// The sports line the market is quoted on; `None` when it has none (served as
    /// `null`) or when absent from an older cached payload.
    #[serde(default, with = "serde_util::decimal_number_option")]
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
// Serde helpers for `""` sentinels
// ---------------------------------------------------------------------------------------

/// An `Option<T>` that the API serves as `""` when absent: `""` and `null` deserialize as
/// `None`, and `None` serializes back as `""`, as on the wire.
pub(crate) mod empty_string_or {
    use serde::{Deserializer, Serialize, Serializer, de::DeserializeOwned};

    pub(crate) fn serialize<T: Serialize, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_str(""),
        }
    }

    pub(crate) fn deserialize<'de, T: DeserializeOwned, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        polyoxide_core::serde_util::empty_string_as_none(deserializer)
    }
}

/// An RFC 3339 date-time string that the API serves as `""` when absent: `""` and `null`
/// deserialize as `None`, and `None` serializes back as `""`.
pub(crate) mod datetime_or_empty {
    use chrono::{DateTime, Utc};
    use polyoxide_core::serde_util;
    use serde::{Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serde_util::datetime::serialize(value, serializer),
            None => serializer.serialize_str(""),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        serde_util::datetime_option::deserialize(deserializer)
    }
}

/// An optional epoch-seconds timestamp for which the API serves `0` when there is none:
/// `0`, `null` and a missing key all deserialize as `None`, and `None` serializes back as
/// `0`, as on the wire. Use with `#[serde(default, with = "...")]`.
pub(crate) mod seconds_or_zero {
    use chrono::{DateTime, Utc};
    use polyoxide_core::serde_util::timestamp_seconds_option;
    use serde::{Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => timestamp_seconds_option::serialize(&Some(*value), serializer),
            None => serializer.serialize_i64(0),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        Ok(timestamp_seconds_option::deserialize(deserializer)?.filter(|t| t.timestamp() != 0))
    }
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

    /// Sends a `GET` to a paginated route and decodes the `{ data, pagination }` page,
    /// keeping the response's `x-trace-id`.
    pub(crate) async fn fetch_page<T: DeserializeOwned>(
        &self,
        path: &[&str],
        query: Query,
    ) -> Result<Page<T>> {
        let response = self.transport.get(path).query(query).send_raw().await?;
        let mut page: Page<T> = response.json()?;
        page.trace_id = response.trace_id().map(str::to_owned);
        Ok(page)
    }
}

/// Lazily walks a paginated route from `start`, building each page's query (with the
/// same filters) through `query`.
///
/// The walk ends when a page has no next cursor (see [`Page::next_cursor`]), or when the
/// server hands back a cursor that was already requested during this walk (a cycle that
/// would otherwise never end).
pub(crate) fn page_stream<T, F>(
    client: DataClient,
    path: &'static [&'static str],
    start: Option<String>,
    mut query: F,
) -> Paginated<T>
where
    T: DeserializeOwned + Send + 'static,
    F: FnMut(Option<&str>) -> Result<Query> + Send + 'static,
{
    let mut requested: HashSet<String> = HashSet::new();
    cursor_stream(start, move |cursor| {
        let repeated = cursor
            .as_ref()
            .is_some_and(|cursor| !requested.insert(cursor.clone()));
        let query = (!repeated).then(|| query(cursor.as_deref()));
        let client = client.clone();
        async move {
            match query {
                None => Ok(CursorPage::new(Vec::new(), None)),
                Some(query) => client
                    .fetch_page::<T>(path, query?)
                    .await
                    .map(Page::into_cursor_page),
            }
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

/// A check applied to every value of a list parameter.
pub(crate) type ValueCheck = fn(&'static str, &str) -> std::result::Result<(), ValidationError>;

/// Accepts any value (beyond the generic list checks of [`distinct_values`]).
pub(crate) fn any_value(_: &'static str, _: &str) -> std::result::Result<(), ValidationError> {
    Ok(())
}

/// A condition id: `0x` followed by 64 hex digits, the format the overview documents for
/// the `condition` key of the single-market routes.
pub(crate) fn condition_id(
    parameter: &'static str,
    value: &str,
) -> std::result::Result<(), ValidationError> {
    validate::bytes32(parameter, value)
}

/// A combo condition id: `0x` followed by one to 64 hex digits.
///
/// The overview documents combo condition ids as `0x03`-prefixed, but live they are `0x`
/// plus **62** hex digits (31 bytes, e.g.
/// `0x037cb523f88f4c6ef6a31c33f8a2e72be70000000000000000000000000000`), and the server
/// answers `400` to a 64-digit id (`invalid combo condition id`). The check is deliberately
/// looser than live (any 1 to 64 digits) so that a change in the id length on Polymarket's
/// side does not turn into a client-side rejection; the server has the last word. See
/// `SPEC_DEVIATIONS.md`.
pub(crate) fn combo_condition_id(
    parameter: &'static str,
    value: &str,
) -> std::result::Result<(), ValidationError> {
    let valid = value.strip_prefix("0x").is_some_and(|hex| {
        (1..=COMBO_CONDITION_MAX_HEX_DIGITS).contains(&hex.len())
            && hex.bytes().all(|b| b.is_ascii_hexdigit())
    });
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            parameter,
            format!(
                "must be a combo condition id: `0x` followed by 1 to \
                 {COMBO_CONDITION_MAX_HEX_DIGITS} hex digits (live ids have 62), got {value:?}"
            ),
        ))
    }
}

/// The most hex digits a combo condition id may have before it cannot be a combo id at
/// all (a bytes32).
const COMBO_CONDITION_MAX_HEX_DIGITS: usize = 64;

/// An integer id: one or more ASCII digits.
pub(crate) fn integer_id(
    parameter: &'static str,
    value: &str,
) -> std::result::Result<(), ValidationError> {
    if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new(
            parameter,
            format!("must be an integer id (ASCII digits only), got {value:?}"),
        ))
    }
}

/// A positive integer id: ASCII digits, not zero.
pub(crate) fn positive_integer_id(
    parameter: &'static str,
    value: &str,
) -> std::result::Result<(), ValidationError> {
    integer_id(parameter, value)?;
    if value.bytes().all(|b| b == b'0') {
        return Err(ValidationError::new(
            parameter,
            format!("must be a positive integer id, got {value:?}"),
        ));
    }
    Ok(())
}

/// Checks a comma-separated list parameter and returns its distinct values, in the order
/// they were first given, ready to be sent.
///
/// Every value must be non-blank, must not contain a comma (which would split it into
/// several values on the wire) and must pass `check`. At most [`MAX_LIST_VALUES`]
/// distinct values are accepted; duplicates are dropped before counting and sending.
pub(crate) fn distinct_values<'a, S: AsRef<str>>(
    parameter: &'static str,
    values: &'a [S],
    check: ValueCheck,
) -> Result<Vec<&'a str>> {
    let mut seen = HashSet::new();
    let mut distinct = Vec::new();
    for value in values.iter().map(AsRef::as_ref) {
        if value.trim().is_empty() {
            return Err(ValidationError::new(parameter, "values must not be empty").into());
        }
        if value.contains(',') {
            return Err(ValidationError::new(
                parameter,
                format!("a value must not contain a comma, got {value:?}"),
            )
            .into());
        }
        check(parameter, value)?;
        if seen.insert(value) {
            distinct.push(value);
        }
    }
    if distinct.len() > MAX_LIST_VALUES {
        return Err(ValidationError::new(
            parameter,
            format!(
                "at most {MAX_LIST_VALUES} distinct values are accepted, got {}",
                distinct.len()
            ),
        )
        .into());
    }
    Ok(distinct)
}

/// As [`distinct_values`], and also fails if the list is empty.
pub(crate) fn required_values<'a, S: AsRef<str>>(
    parameter: &'static str,
    values: &'a [S],
    check: ValueCheck,
) -> Result<Vec<&'a str>> {
    if values.is_empty() {
        return Err(ValidationError::new(parameter, "at least one value is required").into());
    }
    distinct_values(parameter, values, check)
}

/// Fails if the `user` wallet is blank. An empty `user` is a `400` on most routes, and on
/// `/v2/positions` and `/v2/leaderboard` it would silently select a different anchor or
/// arm.
pub(crate) fn check_user(user: &Address) -> Result<()> {
    if user.as_str().trim().is_empty() {
        return Err(ValidationError::new("user", "must not be empty").into());
    }
    Ok(())
}

/// Fails if the `user` wallet is not an EVM address (`0x` followed by 40 hex digits), on
/// the routes that document that format.
pub(crate) fn check_wallet(user: &Address) -> Result<()> {
    validate::evm_address("user", user.as_str())?;
    Ok(())
}

/// The epoch seconds of a time bound; fails for a time before the Unix epoch, which no
/// Data API window accepts as a meaningful bound.
pub(crate) fn epoch_seconds(
    parameter: &'static str,
    bound: Option<DateTime<Utc>>,
) -> Result<Option<i64>> {
    match bound.map(|t| t.timestamp()) {
        Some(seconds) if seconds < 0 => Err(ValidationError::new(
            parameter,
            format!("must not be before the Unix epoch, got {seconds}"),
        )
        .into()),
        seconds => Ok(seconds),
    }
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
pub(crate) mod test_ids {
    //! Well-formed identifiers for unit tests.

    /// A wallet address (`0x` + 40 hex digits).
    pub(crate) const WALLET: &str = "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748";
    /// A condition id (`0x` + 64 hex digits).
    pub(crate) const CONDITION: &str =
        "0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79";
    /// A second condition id.
    pub(crate) const CONDITION_2: &str =
        "0x1111111111111111111111111111111111111111111111111111111111111111";
    /// A combo condition id as served live: `0x` plus 62 hex digits (captured 2026-10-02
    /// from `GET /v2/positions/combos`).
    pub(crate) const COMBO_CONDITION: &str =
        "0x033c72a79df1dfd46683b15b5c0ce78ef50000000000000000000000000000";
}

#[cfg(test)]
mod tests {
    use super::test_ids::{CONDITION, CONDITION_2};
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
        let absent: Pagination =
            serde_json::from_str(r#"{"limit":1,"offset":3,"has_more":false}"#).unwrap();
        assert_eq!(absent.next_cursor, None);
        assert!(serde_json::from_str::<Pagination>(r#"{"limit":1,"offset":0}"#).is_err());
    }

    #[test]
    fn page_accessors() {
        let page: Page<u8> = serde_json::from_str(
            r#"{"data":[1,2],"pagination":{"limit":2,"offset":0,"has_more":true,"next_cursor":"c"}}"#,
        )
        .unwrap();
        assert_eq!(page.items(), &[1, 2]);
        assert_eq!(page.next_cursor(), Some("c"));
        assert!(page.has_more());
        assert_eq!(page.trace_id(), None);
        assert_eq!(page.clone().into_items(), vec![1, 2]);
        let cursor_page = page.into_cursor_page();
        assert_eq!(cursor_page.next_cursor.as_deref(), Some("c"));

        // The trace id is not part of the body, in either direction.
        let value = serde_json::to_value(serde_json::from_str::<Page<u8>>(
            r#"{"data":[],"pagination":{"limit":2,"offset":0,"has_more":false,"next_cursor":null}}"#,
        )
        .unwrap())
        .unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "data": [],
                "pagination": {"limit": 2, "offset": 0, "has_more": false, "next_cursor": null}
            })
        );
    }

    /// `has_more` is exact: a page that says there is no more data ends the walk even if
    /// it carries a cursor, and an empty cursor is no cursor.
    #[test]
    fn next_cursor_honours_has_more() {
        let contradictory: Page<u8> = serde_json::from_str(
            r#"{"data":[1],"pagination":{"limit":1,"offset":0,"has_more":false,"next_cursor":"c"}}"#,
        )
        .unwrap();
        assert_eq!(contradictory.next_cursor(), None);
        assert_eq!(contradictory.into_cursor_page().next_cursor, None);

        let empty: Page<u8> = serde_json::from_str(
            r#"{"data":[1],"pagination":{"limit":1,"offset":0,"has_more":true,"next_cursor":""}}"#,
        )
        .unwrap();
        assert_eq!(empty.next_cursor(), None);
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
        let ids: Vec<String> = (0..21).map(|i| format!("0x{i:064x}")).collect();
        assert!(distinct_values("condition", &ids, condition_id).is_err());
        assert_eq!(
            distinct_values("condition", ids.get(..20).unwrap(), condition_id)
                .unwrap()
                .len(),
            20
        );
        // Duplicates are dropped (keeping the first-seen order) before counting.
        let dupes = [CONDITION, CONDITION_2, CONDITION].repeat(10);
        assert_eq!(
            distinct_values("condition", &dupes, condition_id).unwrap(),
            vec![CONDITION, CONDITION_2]
        );
        for bad in ["", " ", "1,2", "0x12"] {
            let err = distinct_values("condition", &[bad], condition_id).unwrap_err();
            assert!(
                matches!(&err, Error::Validation(v) if v.parameter() == "condition"),
                "{bad:?}: {err}"
            );
        }
        // A value with a comma is never split silently, whatever the format check.
        assert!(distinct_values("event_id", &["1,2"], any_value).is_err());
        assert!(required_values::<&str>("condition", &[], any_value).is_err());
        assert!(check_limit(Some(1001), 1000).is_err());
        assert!(check_limit(Some(1000), 1000).is_ok());
        assert!(check_limit(None, 1000).is_ok());
    }

    #[test]
    fn id_checks() {
        assert!(integer_id("event_id", "12345").is_ok());
        assert!(integer_id("event_id", "0").is_ok());
        for bad in ["", "-1", "1.5", "abc", " 1", "１"] {
            assert!(integer_id("event_id", bad).is_err(), "{bad:?}");
        }
        assert!(positive_integer_id("event_id", "7").is_ok());
        assert!(positive_integer_id("event_id", "0").is_err());
        assert!(positive_integer_id("event_id", "000").is_err());
        assert!(condition_id("condition", CONDITION).is_ok());
        assert!(condition_id("condition", "GLOBAL").is_err());
    }

    #[test]
    fn user_and_time_checks() {
        assert!(check_user(&Address::from(" ")).is_err());
        assert!(check_user(&Address::from("0xabc")).is_ok());
        assert!(check_wallet(&Address::from("0xabc")).is_err());
        assert!(check_wallet(&Address::from(super::test_ids::WALLET)).is_ok());
        assert_eq!(epoch_seconds("start", None).unwrap(), None);
        assert_eq!(
            epoch_seconds("start", Some(DateTime::UNIX_EPOCH)).unwrap(),
            Some(0)
        );
        let before = DateTime::from_timestamp(-1, 0);
        let err = epoch_seconds("start", before).unwrap_err();
        assert!(matches!(&err, Error::Validation(v) if v.parameter() == "start"));
    }

    /// `""` sentinels serialize back to `""`, as on the wire.
    #[test]
    fn empty_string_sentinels_round_trip() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Wire {
            #[serde(with = "empty_string_or")]
            side: Option<SortDirection>,
            #[serde(with = "datetime_or_empty")]
            at: Option<DateTime<Utc>>,
        }
        let empty: Wire = serde_json::from_str(r#"{"side":"","at":""}"#).unwrap();
        assert_eq!(
            empty,
            Wire {
                side: None,
                at: None
            }
        );
        assert_eq!(
            serde_json::to_string(&empty).unwrap(),
            r#"{"side":"","at":""}"#
        );
        let null: Wire = serde_json::from_str(r#"{"side":null,"at":null}"#).unwrap();
        assert_eq!(null, empty);
        let set: Wire =
            serde_json::from_str(r#"{"side":"ASC","at":"2026-08-19T10:00:00Z"}"#).unwrap();
        assert_eq!(
            serde_json::to_string(&set).unwrap(),
            r#"{"side":"ASC","at":"2026-08-19T10:00:00Z"}"#
        );
        // Both are required on the wire.
        assert!(serde_json::from_str::<Wire>(r#"{"side":""}"#).is_err());
    }
}
