//! Types shared by several CLOB endpoints: pagination pages, batch request items, id
//! newtypes and client-side validation helpers.

use futures_core::Stream;
use polyoxide_core::{
    Result, ValidationError,
    pagination::{CursorPage, cursor_stream},
    types::{Side, TokenId},
};
use serde::{Deserialize, Serialize};

/// The `next_cursor` value the CLOB returns on the last page of a cursor-paginated listing.
///
/// Documented for the rewards and builder-trades listings ("A next_cursor value of `"LTE="`
/// indicates the last page"). The page types in this module treat it as "no next page".
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

polyoxide_core::string_id! {
    /// A Polymarket market id as used by the rewards endpoints (e.g. `"248849"`).
    ///
    /// Not to be confused with a [`ConditionId`](crate::types::ConditionId), which the CLOB
    /// calls `market`.
    pub struct MarketId;
}

polyoxide_core::string_id! {
    /// A Polymarket event id as used by the rewards endpoints (e.g. `"12345"`).
    pub struct EventId;
}

/// One page of a CLOB cursor-paginated listing whose envelope fields are all required
/// (rewards listings and builder trades).
///
/// Mirrors `PaginatedCurrentReward`, `PaginatedMarketReward`, `PaginatedMultiMarketInfo`
/// and `BuilderTradesResponse` in the CLOB OpenAPI spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page<T> {
    /// Maximum number of items per page.
    pub limit: u64,
    /// Number of items in this page.
    pub count: u64,
    /// Cursor for the next page, exactly as sent by the server. [`END_CURSOR`] (`"LTE="`)
    /// marks the last page; prefer [`Page::next_page_cursor`], which maps it to `None`.
    pub next_cursor: String,
    /// The items on this page.
    pub data: Vec<T>,
}

impl<T> Page<T> {
    /// The cursor to request the next page with, or `None` on the last page (when the
    /// server sent [`END_CURSOR`] or an empty cursor).
    #[must_use]
    pub fn next_page_cursor(&self) -> Option<&str> {
        next_page_cursor(Some(&self.next_cursor))
    }

    /// `true` if this is the last page (see [`Page::next_page_cursor`]).
    #[must_use]
    pub fn is_last_page(&self) -> bool {
        self.next_page_cursor().is_none()
    }
}

/// One page of the CLOB market listings (`/simplified-markets`, `/sampling-markets`,
/// `/sampling-simplified-markets`).
///
/// Mirrors `PaginatedSimplifiedMarkets` and `PaginatedMarkets` in the CLOB OpenAPI spec,
/// which mark no field as required, so every field is optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketsPage<T> {
    /// Maximum number of items per page.
    pub limit: Option<u64>,
    /// Cursor for the next page, exactly as sent by the server; prefer
    /// [`MarketsPage::next_page_cursor`], which maps the [`END_CURSOR`] sentinel to `None`.
    pub next_cursor: Option<String>,
    /// Number of items in this page.
    pub count: Option<u64>,
    /// The items on this page.
    pub data: Option<Vec<T>>,
}

impl<T> MarketsPage<T> {
    /// The cursor to request the next page with, or `None` on the last page (when the
    /// server sent no cursor, an empty cursor or [`END_CURSOR`]).
    ///
    /// The spec does not document an end sentinel for these listings; the CLOB-wide
    /// `"LTE="` sentinel documented for its other cursor listings is assumed.
    #[must_use]
    pub fn next_page_cursor(&self) -> Option<&str> {
        next_page_cursor(self.next_cursor.as_deref())
    }

    /// `true` if this is the last page (see [`MarketsPage::next_page_cursor`]).
    #[must_use]
    pub fn is_last_page(&self) -> bool {
        self.next_page_cursor().is_none()
    }
}

/// An item of the request body of the batch market-data endpoints
/// (`components/schemas/BookRequest`): a token id and an optional side.
///
/// Every `impl Into<TokenId>` value (`&str`, `String`, [`TokenId`]) converts into a
/// `BookRequest` without a side, so token ids can be passed directly.
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

/// Maps a raw `next_cursor` to the cursor of the next page (`None` at the end).
pub(crate) fn next_page_cursor(raw: Option<&str>) -> Option<&str> {
    raw.filter(|cursor| !cursor.is_empty() && *cursor != END_CURSOR)
}

/// Streams every item of a [`Page`]-based cursor listing.
///
/// `fetch(request, cursor)` fetches the page at `cursor` (`None` for the first page) using a
/// clone of `request`; walking starts at `start` and stops at the [`END_CURSOR`] sentinel.
pub(crate) fn page_stream<R, T, F, Fut>(
    request: R,
    start: Option<String>,
    fetch: F,
) -> impl Stream<Item = Result<T>> + Send + 'static
where
    R: Clone + Send + 'static,
    T: Send + 'static,
    F: Fn(R, Option<String>) -> Fut + Send + Copy + 'static,
    Fut: Future<Output = Result<Page<T>>> + Send + 'static,
{
    cursor_stream(start, move |cursor| {
        let page = fetch(request.clone(), cursor);
        async move {
            page.await.map(|page| {
                let next = page.next_page_cursor().map(str::to_owned);
                CursorPage::new(page.data, next)
            })
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

/// Fails unless `value` is `0x` followed by exactly `hex_len` hex characters.
pub(crate) fn require_hex(parameter: &'static str, value: &str, hex_len: usize) -> Result<()> {
    let valid = value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == hex_len && hex.bytes().all(|b| b.is_ascii_hexdigit()));
    if !valid {
        return Err(ValidationError::new(
            parameter,
            format!("must be `0x` followed by {hex_len} hex characters, got {value:?}"),
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyoxide_core::Error;

    #[test]
    fn page_maps_end_sentinel() {
        // Envelope from the `/rewards/markets/current` example in docs/specs/clob-openapi.yaml.
        let page: Page<serde_json::Value> =
            serde_json::from_str(r#"{"limit":500,"count":0,"next_cursor":"LTE=","data":[]}"#)
                .unwrap();
        assert_eq!(page.next_cursor, "LTE=");
        assert_eq!(page.next_page_cursor(), None);
        assert!(page.is_last_page());

        // `next_cursor: MzAw` from the `/builder/trades` example.
        let page: Page<serde_json::Value> =
            serde_json::from_str(r#"{"limit":300,"count":0,"next_cursor":"MzAw","data":[]}"#)
                .unwrap();
        assert_eq!(page.next_page_cursor(), Some("MzAw"));
        assert!(!page.is_last_page());
    }

    #[test]
    fn page_requires_envelope_fields() {
        let err = serde_json::from_str::<Page<serde_json::Value>>(r#"{"data":[]}"#).unwrap_err();
        assert!(err.to_string().contains("missing field"), "{err}");
    }

    #[test]
    fn markets_page_fields_are_optional() {
        let page: MarketsPage<serde_json::Value> = serde_json::from_str("{}").unwrap();
        assert_eq!(page.data, None);
        assert!(page.is_last_page());
        let page: MarketsPage<serde_json::Value> =
            serde_json::from_str(r#"{"next_cursor":"MTAw","data":[1]}"#).unwrap();
        assert_eq!(page.next_page_cursor(), Some("MTAw"));
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
    fn validation_helpers() {
        assert!(require_non_empty::<u8>("token_ids", &[]).is_err());
        assert!(require_non_empty("token_ids", &[1]).is_ok());
        assert!(require_at_most("token_ids", &[1, 2], 2).is_ok());
        let err = require_at_most("token_ids", &[1, 2, 3], 2).unwrap_err();
        let Error::Validation(v) = err else {
            panic!("expected a validation error")
        };
        assert_eq!(v.parameter(), "token_ids");

        let ok = format!("0x{}", "aB".repeat(32));
        assert!(require_hex("builder_code", &ok, 64).is_ok());
        assert!(require_hex("builder_code", &ok[2..], 64).is_err());
        assert!(require_hex("builder_code", "0x12", 64).is_err());
        assert!(require_hex("builder_code", &format!("0x{}", "g".repeat(64)), 64).is_err());
    }
}
