//! Combo markets: `GET /v1/rfq/combo-markets`.

use crate::Paginated;
use polyoxide_core::{
    Query, Result, ValidationError,
    pagination::{CursorPage, cursor_stream},
    serde_util,
    types::ConditionId,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::CombosClient;

/// The largest page size the SDK sends for `GET /v1/rfq/combo-markets`.
///
/// The spec says `100`; the live API accepts at least `1000` (and, in practice, well
/// beyond), so the SDK allows `1000`. See `SPEC_DEVIATIONS.md`.
const MAX_LIMIT: u32 = 1000;

polyoxide_core::string_id! {
    /// The id of a market in the combo catalog (`ComboMarket.id`), e.g. `"1897034"`.
    pub struct ComboMarketId;
}

polyoxide_core::string_id! {
    /// A combo position id: a `uint256` as a decimal string. Used as the order's `tokenId`
    /// when quoting the YES or NO side of a combo.
    pub struct PositionId;
}

/// A market that can be used as a combo leg (`components/schemas/ComboMarket`).
///
/// [`position_ids`](Self::position_ids), [`outcomes`](Self::outcomes) and
/// [`outcome_prices`](Self::outcome_prices) correspond by index: `[0]` is YES and `[1]` is
/// NO. Use [`yes_position_id`](Self::yes_position_id), [`no_position_id`](Self::no_position_id),
/// [`yes_price`](Self::yes_price) and [`no_price`](Self::no_price) to read them without
/// indexing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboMarket {
    /// Market id.
    pub id: ComboMarketId,
    /// The market's condition id.
    pub condition_id: ConditionId,
    /// Combo position ids; `[0]` is YES, `[1]` is NO.
    pub position_ids: Vec<PositionId>,
    /// Whether the market is pending. Undocumented; observed live (2026-10-02) on every
    /// market. Optional so that a response without it (as in the spec's example) still
    /// decodes.
    pub pending: Option<bool>,
    /// URL slug.
    pub slug: String,
    /// Market title, e.g. `"Will Mexico win on 2026-06-11?"`.
    pub title: String,
    /// Outcome labels, e.g. `["Yes", "No"]`; `[0]` is YES, `[1]` is NO.
    pub outcomes: Vec<String>,
    /// Outcome prices (strings on the wire, e.g. `["0.685", "0.315"]`); `[0]` is YES,
    /// `[1]` is NO.
    pub outcome_prices: Vec<Decimal>,
    /// Image URL.
    pub image: String,
    /// Market volume (a JSON number on the wire). The catalog is ordered by volume,
    /// descending.
    #[serde(with = "serde_util::decimal_number")]
    pub volume: Decimal,
    /// Tag slugs, e.g. `["sports", "soccer"]`.
    pub tags: Vec<String>,
}

impl ComboMarket {
    /// The YES combo position id (`position_ids[0]`), if present.
    #[must_use]
    pub fn yes_position_id(&self) -> Option<&PositionId> {
        self.position_ids.first()
    }

    /// The NO combo position id (`position_ids[1]`), if present.
    #[must_use]
    pub fn no_position_id(&self) -> Option<&PositionId> {
        self.position_ids.get(1)
    }

    /// The YES price (`outcome_prices[0]`), if present.
    #[must_use]
    pub fn yes_price(&self) -> Option<Decimal> {
        self.outcome_prices.first().copied()
    }

    /// The NO price (`outcome_prices[1]`), if present.
    #[must_use]
    pub fn no_price(&self) -> Option<Decimal> {
        self.outcome_prices.get(1).copied()
    }
}

/// One page of the combo market catalog (`components/schemas/ComboMarketsResponse`),
/// returned by [`ListComboMarkets::send`].
///
/// Both fields are required by the spec, so a response without `markets` or without
/// `next_cursor` fails to decode (with [`Error::Decode`](crate::Error::Decode)) instead of
/// being mistaken for the last page. Every market of the page must decode: one market that
/// does not match the schema (for example an `outcome_prices` entry that is not a decimal
/// number) fails the whole page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboMarketsPage {
    /// The markets on this page, by volume descending.
    pub markets: Vec<ComboMarket>,
    /// Cursor for the next page, as sent: `None` on the final page. Prefer
    /// [`next_cursor()`](Self::next_cursor), which also treats an empty string as the end.
    /// Pass it back unchanged with [`ListComboMarkets::cursor`].
    #[serde(deserialize_with = "Option::deserialize")]
    pub next_cursor: Option<String>,
}

impl ComboMarketsPage {
    /// The markets on this page, by volume descending.
    #[must_use]
    pub fn items(&self) -> &[ComboMarket] {
        &self.markets
    }

    /// The markets on this page, by volume descending, by value.
    #[must_use]
    pub fn into_items(self) -> Vec<ComboMarket> {
        self.markets
    }

    /// The cursor for the next page, to pass to [`ListComboMarkets::cursor`]; `None` on the
    /// final page (a `null` or empty `next_cursor`).
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor
            .as_deref()
            .filter(|cursor| !cursor.is_empty())
    }
}

impl CombosClient {
    /// Lists active markets that can be used as combo legs, ordered by volume descending
    /// (`GET /v1/rfq/combo-markets`, cursor-paginated). Public; no authentication needed.
    ///
    /// Use [`ListComboMarkets::send`] for one page or [`ListComboMarkets::into_stream`] to
    /// walk the whole catalog.
    ///
    /// See <https://docs.polymarket.com/api-reference/combo-markets/get-combo-markets>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::{StreamExt as _, TryStreamExt as _};
    ///
    /// let combos = polyoxide::combos::CombosClient::new()?;
    /// let top: Vec<_> = combos
    ///     .list_combo_markets()
    ///     .limit(100)
    ///     .into_stream()
    ///     .take(250)
    ///     .try_collect()
    ///     .await?;
    /// for market in &top {
    ///     println!("{}: YES {:?}", market.title, market.yes_price());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_combo_markets(&self) -> ListComboMarkets {
        ListComboMarkets {
            client: self.clone(),
            limit: None,
            cursor: None,
            exclude: Vec::new(),
        }
    }
}

/// Request builder for [`CombosClient::list_combo_markets`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboMarkets {
    client: CombosClient,
    limit: Option<u32>,
    cursor: Option<String>,
    exclude: Vec<ConditionId>,
}

impl ListComboMarkets {
    /// Number of markets per page, `1..=1000`.
    ///
    /// The spec documents a default of `50` and a maximum of `100`; the live API returns
    /// `1000` markets per page when no limit is set and accepts limits above `100`
    /// (`0` and non-numeric values get `400`). The SDK allows `1..=1000`. See
    /// `SPEC_DEVIATIONS.md`.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Continue from the `next_cursor` of a previous page.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Condition ids of markets to omit, such as markets already shown (sent as a
    /// comma-separated `exclude` list). Replaces any previously set list.
    pub fn exclude<I>(mut self, condition_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.exclude = condition_ids.into_iter().map(Into::into).collect();
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        if let Some(limit) = self.limit
            && !(1..=MAX_LIMIT).contains(&limit)
        {
            return Err(ValidationError::new(
                "limit",
                format!("must be between 1 and {MAX_LIMIT}, got {limit}"),
            )
            .into());
        }
        let mut query = Query::new();
        query
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_csv("exclude", &self.exclude);
        Ok(query)
    }

    async fn fetch(&self, cursor: Option<&str>) -> Result<ComboMarketsPage> {
        let query = self.query(cursor)?;
        self.client
            .transport
            .get(&["v1", "rfq", "combo-markets"])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if the limit is outside `1..=1000`
    ///   (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for parameters the server
    ///   rejects.
    /// - [`Error::Decode`](crate::Error::Decode) if the response does not match the
    ///   documented schema, including a missing `markets` or `next_cursor` (see
    ///   [`ComboMarketsPage`]).
    /// - Any other [`Error`](crate::Error) for transport, server or rate limiting
    ///   failures.
    pub async fn send(self) -> Result<ComboMarketsPage> {
        self.fetch(self.cursor.as_deref()).await
    }

    /// Streams every market from the configured cursor (or the start of the catalog)
    /// onwards, following `next_cursor` until it is `null` (or empty).
    ///
    /// The stream yields the first error (any error of [`send`](Self::send)) and then
    /// ends.
    pub fn into_stream(self) -> Paginated<ComboMarket> {
        let start = self.cursor.clone();
        cursor_stream(start, move |cursor| {
            let request = self.clone();
            async move {
                let page = request.fetch(cursor.as_deref()).await?;
                let next = page.next_cursor().map(str::to_owned);
                Ok(CursorPage::new(page.into_items(), next))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `200` example of `GET /v1/rfq/combo-markets` in `docs/specs/combos-rfq-openapi.yaml`
    /// (also on `docs/api-reference/combo-markets/get-combo-markets.md`).
    const EXAMPLE: &str = r#"{
        "markets": [
            {
                "id": "1897034",
                "condition_id": "0x4cd7...110ff",
                "position_ids": ["1012585...362880", "1012585...362881"],
                "slug": "fifwc-mex-rsa-2026-06-11-mex",
                "title": "Will Mexico win on 2026-06-11?",
                "outcomes": ["Yes", "No"],
                "outcome_prices": ["0.685", "0.315"],
                "image": "https://...",
                "volume": 330327.7128580074,
                "tags": ["sports", "soccer", "games", "world-cup"]
            }
        ],
        "next_cursor": "Mg"
    }"#;

    #[test]
    fn deserializes_documented_example() {
        let page: ComboMarketsPage = serde_json::from_str(EXAMPLE).unwrap();
        assert_eq!(page.next_cursor.as_deref(), Some("Mg"));
        let market = &page.markets[0];
        assert_eq!(market.id, ComboMarketId::from("1897034"));
        assert_eq!(market.condition_id, ConditionId::from("0x4cd7...110ff"));
        assert_eq!(market.slug, "fifwc-mex-rsa-2026-06-11-mex");
        assert_eq!(market.outcomes, ["Yes", "No"]);
        assert_eq!(market.yes_price(), Some(Decimal::new(685, 3)));
        assert_eq!(market.no_price(), Some(Decimal::new(315, 3)));
        assert_eq!(
            market.yes_position_id(),
            Some(&PositionId::from("1012585...362880"))
        );
        assert_eq!(
            market.no_position_id(),
            Some(&PositionId::from("1012585...362881"))
        );
        assert_eq!(market.volume.to_string(), "330327.7128580074");
        assert_eq!(market.tags.len(), 4);
        // The documented example has no `pending`.
        assert_eq!(market.pending, None);
    }

    /// Captured from `GET https://combos-rfq-api.polymarket.com/v1/rfq/combo-markets`
    /// on 2026-10-02 (trimmed to one market and its tags). Ids are full length and the
    /// market has the undocumented `pending` field; `volume` is a float here (an integer
    /// for some markets).
    const LIVE_CAPTURE: &str = r#"{"markets":[{
        "id":"665374",
        "condition_id":"0x5db999fad322cea2914535aae5517060c3f80ad6d8c0231cde2124a434d16846",
        "position_ids":["798559951534518479645224261511384773234863312866932338530531601041078616064","798559951534518479645224261511384773234863312866932338530531601041078616065"],
        "pending":false,
        "slug":"will-the-us-invade-iran-before-2027",
        "title":"Will the U.S. invade Iran before 2027?",
        "outcomes":["Yes","No"],
        "outcome_prices":["0.145","0.855"],
        "image":"https://polymarket-upload.s3.us-east-2.amazonaws.com/will-the-us-invade-iran-in-2025-0Eh3J0ku_Fbj.jpg",
        "volume":70868404.87693602,
        "tags":["politics","iran","trump"]
    }],"next_cursor":"MTQwNzI0"}"#;

    #[test]
    fn deserializes_live_capture_with_pending() {
        let page: ComboMarketsPage = serde_json::from_str(LIVE_CAPTURE).unwrap();
        let market = &page.markets[0];
        assert_eq!(market.pending, Some(false));
        assert_eq!(market.condition_id.as_str().len(), 66);
        assert_eq!(market.yes_position_id().unwrap().as_str().len(), 75);
        assert_eq!(market.yes_price(), Some(Decimal::new(145, 3)));
        assert_eq!(page.next_cursor(), Some("MTQwNzI0"));
        // `pending` survives a round trip.
        let value = serde_json::to_value(&page).unwrap();
        assert_eq!(value["markets"][0]["pending"], serde_json::json!(false));
    }

    #[test]
    fn volume_may_be_an_integer() {
        let json = r#"{"id":"1","condition_id":"0x1","position_ids":[],"pending":true,"slug":"s",
            "title":"t","outcomes":[],"outcome_prices":[],"image":"","volume":42,"tags":[]}"#;
        let market: ComboMarket = serde_json::from_str(json).unwrap();
        assert_eq!(market.volume, Decimal::from(42));
        assert_eq!(market.pending, Some(true));
    }

    #[test]
    fn roundtrips_wire_format() {
        let page: ComboMarketsPage = serde_json::from_str(EXAMPLE).unwrap();
        let value = serde_json::to_value(&page).unwrap();
        // Prices stay strings and volume stays a number, as on the wire.
        assert_eq!(value["markets"][0]["outcome_prices"][0], "0.685");
        assert_eq!(
            value["markets"][0]["volume"],
            serde_json::json!(330_327.712_858_007_4)
        );
        let again: ComboMarketsPage = serde_json::from_value(value).unwrap();
        assert_eq!(again, page);
    }

    #[test]
    fn required_fields_are_enforced() {
        // `volume` is required by the spec.
        let json = r#"{"markets":[{"id":"1","condition_id":"0x1","position_ids":[],"slug":"s",
            "title":"t","outcomes":[],"outcome_prices":[],"image":"","tags":[]}],
            "next_cursor":null}"#;
        let err = serde_json::from_str::<ComboMarketsPage>(json).unwrap_err();
        assert!(err.to_string().contains("volume"), "{err}");
    }

    #[test]
    fn next_cursor_is_required_but_nullable() {
        // A missing key is not the final page.
        let err = serde_json::from_str::<ComboMarketsPage>(r#"{"markets":[]}"#).unwrap_err();
        assert!(err.to_string().contains("next_cursor"), "{err}");
        let err = serde_json::from_str::<ComboMarketsPage>(r#"{"next_cursor":null}"#).unwrap_err();
        assert!(err.to_string().contains("markets"), "{err}");

        let last: ComboMarketsPage =
            serde_json::from_str(r#"{"markets":[],"next_cursor":null}"#).unwrap();
        assert_eq!(last.next_cursor, None);
        assert_eq!(last.next_cursor(), None);
        let empty: ComboMarketsPage =
            serde_json::from_str(r#"{"markets":[],"next_cursor":""}"#).unwrap();
        assert_eq!(empty.next_cursor.as_deref(), Some(""));
        assert_eq!(empty.next_cursor(), None);
    }

    #[test]
    fn page_accessors() {
        let page: ComboMarketsPage = serde_json::from_str(EXAMPLE).unwrap();
        assert_eq!(page.next_cursor(), Some("Mg"));
        assert_eq!(page.items().len(), 1);
        assert_eq!(page.items()[0].id, ComboMarketId::from("1897034"));
        assert_eq!(page.into_items().len(), 1);
    }

    #[test]
    fn index_helpers_tolerate_short_arrays() {
        let json = r#"{"id":"1","condition_id":"0x1","position_ids":["7"],"slug":"s",
            "title":"t","outcomes":["Yes"],"outcome_prices":[],"image":"","volume":0,"tags":[]}"#;
        let market: ComboMarket = serde_json::from_str(json).unwrap();
        assert_eq!(market.yes_position_id(), Some(&PositionId::from("7")));
        assert_eq!(market.no_position_id(), None);
        assert_eq!(market.yes_price(), None);
    }
}
