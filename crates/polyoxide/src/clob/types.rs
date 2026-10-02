//! Types shared by several CLOB endpoints: pagination pages, batch request items, id
//! newtypes, serde helpers and client-side validation helpers.

use crate::Paginated;
use polyoxide_core::{
    Result, ValidationError,
    pagination::{CursorPage, cursor_stream},
    types::{Side, TokenId},
};
use serde::{Deserialize, Serialize};

/// The `next_cursor` value the CLOB returns on the last page of a cursor-paginated listing.
///
/// Documented for the rewards and builder-trades listings ("A next_cursor value of `"LTE="`
/// indicates the last page"). The page types in this module treat it, and an empty cursor,
/// as "no next page": their `next_cursor()` methods return `None` for it, and a listing's
/// `into_stream()` started at it yields nothing.
pub const END_CURSOR: &str = "LTE=";

polyoxide_core::string_id! {
    /// A builder code: `0x` followed by 64 hex characters, e.g.
    /// `"0x0000000000000000000000000000000000000000000000000000000000000001"`.
    ///
    /// Requests validate the documented pattern `^0x[a-fA-F0-9]{64}$` before sending.
    pub struct BuilderCode;
}

polyoxide_core::string_id! {
    /// A CLOB trade id, e.g. `"trade-123"`.
    pub struct TradeId;
}

polyoxide_core::string_id! {
    /// A CLOB order id, which is the order hash (e.g. a builder trade's `takerOrderHash`).
    pub struct OrderId;
}

/// One page of a CLOB cursor-paginated listing whose envelope fields are all required
/// (rewards listings and builder trades).
///
/// Mirrors `PaginatedCurrentReward`, `PaginatedMarketReward`, `PaginatedMultiMarketInfo`
/// and `BuilderTradesResponse` in the CLOB OpenAPI spec. The public fields hold the wire
/// values; [`items`](Self::items) and [`next_cursor()`](Self::next_cursor()) are the
/// service-independent accessors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// Maximum number of items per page.
    pub limit: u32,
    /// Number of items in this page.
    pub count: u32,
    /// Cursor for the next page, exactly as sent by the server (wire name `next_cursor`).
    /// [`END_CURSOR`] (`"LTE="`) marks the last page; prefer the
    /// [`next_cursor()`](Self::next_cursor()) method, which maps it to `None`.
    pub next_cursor: String,
    /// The items on this page.
    pub data: Vec<T>,
}

impl<T> Page<T> {
    /// The items on this page.
    #[must_use]
    pub fn items(&self) -> &[T] {
        &self.data
    }

    /// Consumes the page and returns its items.
    #[must_use]
    pub fn into_items(self) -> Vec<T> {
        self.data
    }

    /// The cursor to request the next page with (the listing's `cursor` setter), or `None`
    /// on the last page: when the server sent [`END_CURSOR`] or an empty cursor.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        normalize_cursor(Some(&self.next_cursor))
    }

    /// `true` if this is the last page (see [`next_cursor()`](Self::next_cursor())).
    #[must_use]
    pub fn is_last_page(&self) -> bool {
        self.next_cursor().is_none()
    }
}

/// One page of the CLOB market listings (`/simplified-markets`, `/sampling-markets`,
/// `/sampling-simplified-markets`).
///
/// Mirrors `PaginatedSimplifiedMarkets` and `PaginatedMarkets` in the CLOB OpenAPI spec,
/// which mark no field as required, so every field is optional. The public fields hold the
/// wire values; [`items`](Self::items) and [`next_cursor()`](Self::next_cursor()) are the
/// service-independent accessors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketsPage<T> {
    /// Maximum number of items per page.
    pub limit: Option<u32>,
    /// Cursor for the next page, exactly as sent by the server (wire name `next_cursor`);
    /// prefer the [`next_cursor()`](Self::next_cursor()) method, which maps the
    /// [`END_CURSOR`] sentinel and an empty cursor to `None`.
    pub next_cursor: Option<String>,
    /// Number of items in this page.
    pub count: Option<u32>,
    /// The items on this page (`None` if the server omitted the field).
    pub data: Option<Vec<T>>,
}

impl<T> MarketsPage<T> {
    /// The items on this page (empty if the server omitted them).
    #[must_use]
    pub fn items(&self) -> &[T] {
        self.data.as_deref().unwrap_or_default()
    }

    /// Consumes the page and returns its items (empty if the server omitted them).
    #[must_use]
    pub fn into_items(self) -> Vec<T> {
        self.data.unwrap_or_default()
    }

    /// The cursor to request the next page with (the listing's `cursor` setter), or `None`
    /// on the last page: when the server sent no cursor, an empty cursor or
    /// [`END_CURSOR`].
    ///
    /// The spec documents no end sentinel for these listings; the CLOB-wide `"LTE="`
    /// sentinel documented for its other cursor listings is assumed.
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        normalize_cursor(self.next_cursor.as_deref())
    }

    /// `true` if this is the last page (see [`next_cursor()`](Self::next_cursor())).
    #[must_use]
    pub fn is_last_page(&self) -> bool {
        self.next_cursor().is_none()
    }
}

/// An item of the request body of the batch market-data endpoints
/// (`components/schemas/BookRequest`): a token id and an optional side.
///
/// Every `impl Into<TokenId>` value (`&str`, `String`, [`TokenId`]) converts into a
/// `BookRequest` without a side, and a `(token id, Side)` pair (owned or borrowed) into one
/// with a side, so token ids and pairs can be passed directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookRequest {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Order side. Optional; not used for midpoint calculation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

impl BookRequest {
    /// Creates a request item for `token_id` without a side.
    #[must_use]
    pub fn new(token_id: impl Into<TokenId>) -> Self {
        Self {
            token_id: token_id.into(),
            side: None,
        }
    }

    /// Sets the side.
    #[must_use]
    pub fn with_side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }
}

impl From<&BookRequest> for BookRequest {
    fn from(request: &BookRequest) -> Self {
        request.clone()
    }
}

impl From<TokenId> for BookRequest {
    fn from(token_id: TokenId) -> Self {
        Self::new(token_id)
    }
}

impl From<&TokenId> for BookRequest {
    fn from(token_id: &TokenId) -> Self {
        Self::new(token_id.clone())
    }
}

impl From<&str> for BookRequest {
    fn from(token_id: &str) -> Self {
        Self::new(token_id)
    }
}

impl From<String> for BookRequest {
    fn from(token_id: String) -> Self {
        Self::new(token_id)
    }
}

impl From<&String> for BookRequest {
    fn from(token_id: &String) -> Self {
        Self::new(token_id.as_str())
    }
}

impl<T: Into<TokenId>> From<(T, Side)> for BookRequest {
    fn from((token_id, side): (T, Side)) -> Self {
        Self::new(token_id).with_side(side)
    }
}

impl<T: Clone + Into<TokenId>> From<&(T, Side)> for BookRequest {
    fn from((token_id, side): &(T, Side)) -> Self {
        Self::new(token_id.clone()).with_side(side.clone())
    }
}

/// `true` if `cursor` marks the end of a listing: [`END_CURSOR`] or an empty string.
pub(crate) fn is_end_cursor(cursor: &str) -> bool {
    cursor.is_empty() || cursor == END_CURSOR
}

/// Maps a raw `next_cursor` to the cursor of the next page (`None` at the end).
pub(crate) fn normalize_cursor(raw: Option<&str>) -> Option<&str> {
    raw.filter(|cursor| !is_end_cursor(cursor))
}

/// Lazily walks a CLOB cursor listing, starting at `start` (`None` for the first page).
///
/// `fetch(cursor)` fetches the page at `cursor` and returns its items and raw next cursor;
/// the walk stops when that cursor marks the end (see [`is_end_cursor`]). A walk started at
/// an end cursor (e.g. a persisted `"LTE="`) yields nothing and sends no request.
pub(crate) fn clob_cursor_stream<T, F, Fut>(start: Option<String>, fetch: F) -> Paginated<T>
where
    T: Send + 'static,
    F: Fn(Option<String>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(Vec<T>, Option<String>)>> + Send + 'static,
{
    cursor_stream(start, move |cursor: Option<String>| {
        let page = match cursor.as_deref() {
            Some(cursor) if is_end_cursor(cursor) => None,
            _ => Some(fetch(cursor)),
        };
        async move {
            let Some(page) = page else {
                return Ok(CursorPage::new(Vec::new(), None));
            };
            let (items, next) = page.await?;
            let next = next.filter(|cursor| !is_end_cursor(cursor));
            Ok(CursorPage::new(items, next))
        }
    })
}

/// Streams every item of a [`Page`]-based cursor listing.
///
/// `fetch(request, cursor)` fetches the page at `cursor` (`None` for the first page) using a
/// clone of `request`; walking starts at `start` and stops at the [`END_CURSOR`] sentinel.
pub(crate) fn page_stream<R, T, F, Fut>(request: R, start: Option<String>, fetch: F) -> Paginated<T>
where
    R: Clone + Send + 'static,
    T: Send + 'static,
    F: Fn(R, Option<String>) -> Fut + Send + Copy + 'static,
    Fut: Future<Output = Result<Page<T>>> + Send + 'static,
{
    clob_cursor_stream(start, move |cursor| {
        let page = fetch(request.clone(), cursor);
        async move {
            let page = page.await?;
            Ok((page.data, Some(page.next_cursor)))
        }
    })
}

/// Fails if a required list parameter is empty.
pub(crate) fn require_non_empty<T>(parameter: &'static str, items: &[T]) -> Result<()> {
    if items.is_empty() {
        return Err(ValidationError::new(parameter, "at least one value is required").into());
    }
    Ok(())
}

/// Fails if a list parameter has more than `max` items.
pub(crate) fn require_at_most<T>(parameter: &'static str, items: &[T], max: usize) -> Result<()> {
    if items.len() > max {
        return Err(ValidationError::new(
            parameter,
            format!("at most {max} values are allowed, got {}", items.len()),
        )
        .into());
    }
    Ok(())
}

/// Fails if a required id (a path or query parameter) is empty.
pub(crate) fn require_id(parameter: &'static str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(ValidationError::new(parameter, "must not be empty").into());
    }
    Ok(())
}

/// Fails if a value of a list parameter is empty.
pub(crate) fn require_list_values<'a>(
    parameter: &'static str,
    values: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    for value in values {
        if value.is_empty() {
            return Err(ValidationError::new(parameter, "must not contain an empty value").into());
        }
    }
    Ok(())
}

/// A side the API sends as an empty string when there is none (e.g. a token without
/// trades): `""` deserializes to `None`, and `None` serializes back to `""`.
pub(crate) mod side_or_empty {
    use polyoxide_core::{serde_util, types::Side};
    use serde::{Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<Side>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.as_ref().map_or("", Side::as_str))
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Side>, D::Error> {
        serde_util::empty_string_as_none(deserializer)
    }
}

/// Unix seconds sent as a decimal string (e.g. `"1700000000"`); serializes back to a
/// string.
pub(crate) mod timestamp_seconds_string {
    use chrono::{DateTime, Utc};
    use polyoxide_core::serde_util;
    use serde::{Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&value.timestamp())
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        serde_util::timestamp_seconds::deserialize(deserializer)
    }
}

/// Unix milliseconds sent as a decimal string (e.g. `"1790934258308"`); serializes back to a
/// string.
pub(crate) mod timestamp_millis_string {
    use chrono::{DateTime, Utc};
    use polyoxide_core::serde_util;
    use serde::{Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&value.timestamp_millis())
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        serde_util::timestamp_millis::deserialize(deserializer)
    }
}

/// Helpers for the unit tests of the CLOB modules.
#[cfg(test)]
pub(crate) mod test_util {
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::Value;

    /// Normalizes JSON for comparison: removes `null` object members (a missing optional
    /// field and a `null` one both decode to `None`, which serializes as `null`) and writes
    /// integral floats as integers (`3.0` and `3` are the same JSON number).
    fn normalize(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .filter(|(_, v)| !v.is_null())
                    .map(|(k, v)| (k, normalize(v)))
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.into_iter().map(normalize).collect()),
            Value::Number(n) => match n.as_f64() {
                Some(f) if n.is_f64() && f.fract() == 0.0 && f.abs() < 1e15 => {
                    Value::from(f as i64)
                }
                _ => Value::Number(n),
            },
            other => other,
        }
    }

    /// Parses `json` as `T` and asserts that serializing the result gives back the same
    /// JSON (same JSON types, e.g. numbers stay numbers and numeric strings stay strings),
    /// up to `null` versus missing optional fields.
    pub(crate) fn round_trip<T: DeserializeOwned + Serialize>(json: &str) -> T {
        let parsed: T = serde_json::from_str(json).unwrap();
        let wire: Value = serde_json::from_str(json).unwrap();
        let again = serde_json::to_value(&parsed).unwrap();
        assert_eq!(normalize(again), normalize(wire));
        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyoxide_core::Error;

    fn parameter(result: Result<()>) -> String {
        match result {
            Err(Error::Validation(v)) => v.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn page_maps_end_sentinel() {
        // Envelope from the `/rewards/markets/current` example in docs/specs/clob-openapi.yaml.
        let page: Page<serde_json::Value> =
            serde_json::from_str(r#"{"limit":500,"count":0,"next_cursor":"LTE=","data":[]}"#)
                .unwrap();
        assert_eq!(page.next_cursor, "LTE=");
        assert_eq!(page.next_cursor(), None);
        assert!(page.is_last_page());
        assert!(page.items().is_empty());

        // `next_cursor: MzAw` from the `/builder/trades` example.
        let page: Page<u8> =
            serde_json::from_str(r#"{"limit":300,"count":2,"next_cursor":"MzAw","data":[1,2]}"#)
                .unwrap();
        assert_eq!(page.next_cursor(), Some("MzAw"));
        assert!(!page.is_last_page());
        assert_eq!(page.items(), [1, 2]);
        assert_eq!(page.into_items(), vec![1, 2]);

        // An empty cursor also ends the listing.
        let page: Page<u8> =
            serde_json::from_str(r#"{"limit":300,"count":0,"next_cursor":"","data":[]}"#).unwrap();
        assert_eq!(page.next_cursor(), None);
    }

    #[test]
    fn page_requires_envelope_fields() {
        let err = serde_json::from_str::<Page<serde_json::Value>>(r#"{"data":[]}"#).unwrap_err();
        assert!(err.to_string().contains("missing field"), "{err}");
    }

    #[test]
    fn markets_page_fields_are_optional() {
        let page: MarketsPage<u8> = serde_json::from_str("{}").unwrap();
        assert_eq!(page.data, None);
        assert!(page.items().is_empty());
        assert!(page.is_last_page());
        assert!(page.into_items().is_empty());

        let page: MarketsPage<u8> =
            serde_json::from_str(r#"{"next_cursor":"MTAw","data":[1]}"#).unwrap();
        assert_eq!(page.next_cursor(), Some("MTAw"));
        assert_eq!(page.items(), [1]);

        for end in ["LTE=", ""] {
            let page = MarketsPage::<u8> {
                limit: None,
                next_cursor: Some(end.to_owned()),
                count: None,
                data: None,
            };
            assert_eq!(page.next_cursor(), None, "{end:?}");
        }
    }

    #[test]
    fn book_request_serialization() {
        // Request body examples of `POST /books` and `POST /prices` in
        // docs/specs/clob-openapi.yaml.
        let body = vec![
            BookRequest::from("0xabc123def456..."),
            BookRequest::new("0xdef456abc123...").with_side(Side::Sell),
        ];
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"[{"token_id":"0xabc123def456..."},{"token_id":"0xdef456abc123...","side":"SELL"}]"#
        );
    }

    #[test]
    fn book_request_from_pairs() {
        let owned = BookRequest::from(("1", Side::Buy));
        assert_eq!(owned, BookRequest::new("1").with_side(Side::Buy));
        let pair = (TokenId::from("2"), Side::Sell);
        let borrowed = BookRequest::from(&pair);
        assert_eq!(borrowed.token_id, pair.0);
        assert_eq!(borrowed.side, Some(Side::Sell));
        assert_eq!(BookRequest::from(&borrowed), borrowed);
    }

    #[test]
    fn end_cursors() {
        assert!(is_end_cursor(END_CURSOR));
        assert!(is_end_cursor(""));
        assert!(!is_end_cursor("MTAw"));
        assert_eq!(normalize_cursor(None), None);
        assert_eq!(normalize_cursor(Some("LTE=")), None);
        assert_eq!(normalize_cursor(Some("MTAw")), Some("MTAw"));
    }

    #[test]
    fn validation_helpers() {
        assert_eq!(
            parameter(require_non_empty::<u8>("token_ids", &[])),
            "token_ids"
        );
        assert!(require_non_empty("token_ids", &[1]).is_ok());
        assert!(require_at_most("token_ids", &[1, 2], 2).is_ok());
        assert_eq!(
            parameter(require_at_most("token_ids", &[1, 2, 3], 2)),
            "token_ids"
        );
        assert_eq!(parameter(require_id("token_id", "")), "token_id");
        assert!(require_id("token_id", "1").is_ok());
        assert!(require_list_values("token_ids", ["1", "2"]).is_ok());
        assert!(require_list_values("token_ids", ["1,2"]).is_ok());
        assert_eq!(
            parameter(require_list_values("token_ids", ["1", ""])),
            "token_ids"
        );
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct WithSide {
        #[serde(with = "side_or_empty")]
        side: Option<Side>,
    }

    #[test]
    fn side_or_empty_round_trips() {
        for (wire, side) in [
            (r#"{"side":"BUY"}"#, Some(Side::Buy)),
            (r#"{"side":""}"#, None),
            (r#"{"side":"HOLD"}"#, Some(Side::Unknown("HOLD".to_owned()))),
        ] {
            let parsed: WithSide = serde_json::from_str(wire).unwrap();
            assert_eq!(parsed.side, side, "{wire}");
            assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
        }
        // `null` is accepted as "no side" too and written back as the documented `""`.
        let parsed: WithSide = serde_json::from_str(r#"{"side":null}"#).unwrap();
        assert_eq!(parsed.side, None);
        // The field stays required.
        assert!(serde_json::from_str::<WithSide>("{}").is_err());
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct WithTime {
        #[serde(with = "timestamp_seconds_string")]
        time: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct WithMillis {
        #[serde(with = "timestamp_millis_string")]
        time: chrono::DateTime<chrono::Utc>,
    }

    #[test]
    fn timestamp_millis_string_round_trips() {
        let parsed: WithMillis = serde_json::from_str(r#"{"time":"1790934258308"}"#).unwrap();
        assert_eq!(parsed.time.timestamp_millis(), 1_790_934_258_308);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            r#"{"time":"1790934258308"}"#
        );
        let parsed: WithMillis = serde_json::from_str(r#"{"time":1790934258308}"#).unwrap();
        assert_eq!(parsed.time.timestamp_millis(), 1_790_934_258_308);
        assert!(serde_json::from_str::<WithMillis>(r#"{"time":"soon"}"#).is_err());
    }

    #[test]
    fn timestamp_seconds_string_round_trips() {
        let parsed: WithTime = serde_json::from_str(r#"{"time":"1700000000"}"#).unwrap();
        assert_eq!(parsed.time.timestamp(), 1_700_000_000);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            r#"{"time":"1700000000"}"#
        );
        // A JSON integer is accepted as well.
        let parsed: WithTime = serde_json::from_str(r#"{"time":1700000000}"#).unwrap();
        assert_eq!(parsed.time.timestamp(), 1_700_000_000);
        assert!(serde_json::from_str::<WithTime>(r#"{"time":"soon"}"#).is_err());
    }
}
