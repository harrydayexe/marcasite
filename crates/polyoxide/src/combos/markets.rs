//! Combo markets: `GET /v1/rfq/combo-markets`.

use futures_core::Stream;
use polyoxide_core::{
    Query, Result, ValidationError,
    pagination::{CursorPage, cursor_stream},
    serde_util,
    types::ConditionId,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::CombosClient;

/// The largest page size `GET /v1/rfq/combo-markets` accepts.
const MAX_LIMIT: u32 = 100;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboMarketsPage {
    /// The markets on this page, by volume descending.
    pub markets: Vec<ComboMarket>,
    /// Cursor for the next page, or `None` on the final page. Pass it back unchanged with
    /// [`ListComboMarkets::cursor`].
    pub next_cursor: Option<String>,
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
    /// Number of markets per page, `1..=100` (server default `50`).
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
    /// - [`Error::Validation`](crate::Error::Validation) if the limit is outside `1..=100`
    ///   (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for parameters the server
    ///   rejects.
    /// - Any other [`Error`](crate::Error) for transport, server, rate limiting or decoding
    ///   failures.
    pub async fn send(self) -> Result<ComboMarketsPage> {
        self.fetch(self.cursor.as_deref()).await
    }

    /// Streams every market from the configured cursor (or the start of the catalog)
    /// onwards, following `next_cursor` until it is `null`.
    ///
    /// The stream yields the first error (including the validation errors of
    /// [`send`](Self::send)) and then ends.
    pub fn into_stream(self) -> impl Stream<Item = Result<ComboMarket>> + Send + 'static {
        let start = self.cursor.clone();
        cursor_stream(start, move |cursor| {
            let request = self.clone();
            async move {
                let page = request.fetch(cursor.as_deref()).await?;
                Ok(CursorPage::new(page.markets, page.next_cursor))
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
    fn index_helpers_tolerate_short_arrays() {
        let json = r#"{"id":"1","condition_id":"0x1","position_ids":["7"],"slug":"s",
            "title":"t","outcomes":["Yes"],"outcome_prices":[],"image":"","volume":0,"tags":[]}"#;
        let market: ComboMarket = serde_json::from_str(json).unwrap();
        assert_eq!(market.yes_position_id(), Some(&PositionId::from("7")));
        assert_eq!(market.no_position_id(), None);
        assert_eq!(market.yes_price(), None);
    }
}
