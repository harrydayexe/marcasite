//! CLOB markets: simplified and sampling market listings, CLOB market info, market lookup by
//! token and live-activity summaries.

use crate::Paginated;
use chrono::{DateTime, Utc};
use polyoxide_core::{
    Query, Result, serde_util,
    types::{Address, ConditionId, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{
    ClobClient,
    types::{MarketsPage, clob_cursor_stream, require_id, require_list_values, require_non_empty},
};

/// A reward rate of a market (an item of `Rewards.rates`).
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RewardRate {
    /// Address of the reward asset.
    pub asset_address: Option<Address>,
    /// Daily reward rate (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_daily_rate: Option<Decimal>,
}

/// The liquidity rewards of a market (`components/schemas/Rewards`).
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Rewards {
    /// Reward rates per reward asset.
    pub rates: Option<Vec<RewardRate>>,
    /// Minimum order size to be eligible for rewards (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub min_size: Option<Decimal>,
    /// Maximum spread to be eligible for rewards (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub max_spread: Option<Decimal>,
}

/// An outcome token of a market (`components/schemas/Token`).
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Token {
    /// Token id (asset id).
    pub token_id: Option<TokenId>,
    /// Outcome label (e.g. `"Yes"`).
    pub outcome: Option<String>,
    /// Price of the token (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub price: Option<Decimal>,
    /// Whether this outcome won.
    pub winner: Option<bool>,
}

/// A market in its simplified form (`components/schemas/SimplifiedMarket`), as listed by
/// [`ClobClient::list_simplified_markets`] and
/// [`ClobClient::list_sampling_simplified_markets`].
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimplifiedMarket {
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Liquidity rewards.
    pub rewards: Option<Rewards>,
    /// Outcome tokens.
    pub tokens: Option<Vec<Token>>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// Whether the market is closed.
    pub closed: Option<bool>,
    /// Whether the market is archived.
    pub archived: Option<bool>,
    /// Whether the market accepts orders.
    pub accepting_orders: Option<bool>,
}

/// A CLOB market (`components/schemas/Market`), as listed by
/// [`ClobClient::list_sampling_markets`].
///
/// The spec marks no field as required and documents none of them; descriptions here are
/// limited to what the field names state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Market {
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// Whether the market is closed.
    pub closed: Option<bool>,
    /// Whether the market is archived.
    pub archived: Option<bool>,
    /// Whether the market accepts orders.
    pub accepting_orders: Option<bool>,
    /// Since when the market accepts orders.
    #[serde(default, with = "serde_util::datetime_option")]
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    /// Minimum order size (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub minimum_order_size: Option<Decimal>,
    /// Minimum tick size, the price increment (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub minimum_tick_size: Option<Decimal>,
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Question id, exactly as sent.
    ///
    /// This is the same id as the Gamma market's `questionID` (live check, 2026-10-02), i.e.
    /// a [`QuestionId`](crate::types::QuestionId) value; it stays a plain string because the
    /// spec types it as one.
    pub question_id: Option<String>,
    /// The market question.
    pub question: Option<String>,
    /// Market description.
    pub description: Option<String>,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// End date.
    #[serde(default, with = "serde_util::datetime_option")]
    pub end_date_iso: Option<DateTime<Utc>>,
    /// Game start time (sports markets).
    #[serde(default, with = "serde_util::datetime_option")]
    pub game_start_time: Option<DateTime<Utc>>,
    /// The `seconds_delay` value (integer; the spec does not document it further).
    pub seconds_delay: Option<i64>,
    /// The `fpmm` value (string; the spec does not document it further).
    pub fpmm: Option<String>,
    /// Maker base fee (integer; the spec does not state the unit).
    pub maker_base_fee: Option<i64>,
    /// Taker base fee (integer; the spec does not state the unit).
    pub taker_base_fee: Option<i64>,
    /// Whether notifications are enabled.
    pub notifications_enabled: Option<bool>,
    /// Whether negative risk is enabled for this market.
    pub neg_risk: Option<bool>,
    /// Negative-risk market id (a string; the spec does not document it further).
    pub neg_risk_market_id: Option<String>,
    /// Negative-risk request id (a string; the spec does not document it further).
    pub neg_risk_request_id: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Liquidity rewards.
    pub rewards: Option<Rewards>,
    /// Whether the market is a 50/50 outcome market.
    pub is_50_50_outcome: Option<bool>,
    /// Outcome tokens.
    pub tokens: Option<Vec<Token>>,
    /// Tags.
    pub tags: Option<Vec<String>>,
}

/// A token of a market in [`ClobMarketDetails`] (`components/schemas/ClobToken`).
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ClobToken {
    /// The token id (wire name `t`).
    #[serde(rename = "t")]
    pub token_id: Option<TokenId>,
    /// Outcome label, e.g. `"Yes"` (wire name `o`).
    #[serde(rename = "o")]
    pub outcome: Option<String>,
}

/// Fee curve parameters of a market (`components/schemas/FeeDetails`).
///
/// Every field is optional and nullable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FeeDetails {
    /// Fee rate (wire name `r`; a JSON number on the wire).
    #[serde(rename = "r", default, with = "serde_util::decimal_number_option")]
    pub rate: Option<Decimal>,
    /// Fee curve exponent (wire name `e`; a JSON number on the wire).
    #[serde(rename = "e", default, with = "serde_util::decimal_number_option")]
    pub exponent: Option<Decimal>,
    /// Whether fees apply to takers only (wire name `to`).
    #[serde(rename = "to")]
    pub takers_only: Option<bool>,
}

/// All CLOB-level parameters of a market (`components/schemas/ClobMarketDetails`): tokens,
/// tick size, base fees, rewards, RFQ status and fee details.
///
/// The wire format uses abbreviated field names (noted on each field). The spec marks no
/// field as required. Fields marked "undocumented" are sent by the live API (observed
/// 2026-10-02) but are not in the spec; see `SPEC_DEVIATIONS.md`. Any field may be absent:
/// which ones appear depends on the market (e.g. `mbf`, `tbf` and `fd` only on markets with
/// fees, `gst`/`sd` on sports markets, `ao`/`aot` on markets that have been opened).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ClobMarketDetails {
    /// Game start time for sports markets, or `None` (wire name `gst`).
    #[serde(rename = "gst", default, with = "serde_util::datetime_option")]
    pub game_start_time: Option<DateTime<Utc>>,
    /// Rewards configuration (wire name `r`).
    ///
    /// The spec describes it as an object with arbitrary properties
    /// (`components/schemas/ClobRewards`), so it is kept as raw JSON. This deliberately
    /// exposes [`serde_json`](crate::serde_json) 1.x in the public API (see the crate docs
    /// on dependencies in the public API); a typed representation would replace it if the
    /// docs ever describe the object's properties.
    #[serde(rename = "r")]
    pub rewards: Option<serde_json::Map<String, serde_json::Value>>,
    /// Tokens of this market (wire name `t`).
    #[serde(rename = "t")]
    pub tokens: Option<Vec<ClobToken>>,
    /// Minimum order size (wire name `mos`; a JSON number on the wire).
    #[serde(rename = "mos", default, with = "serde_util::decimal_number_option")]
    pub min_order_size: Option<Decimal>,
    /// Minimum tick size, the price increment (wire name `mts`; a JSON number on the wire).
    #[serde(rename = "mts", default, with = "serde_util::decimal_number_option")]
    pub min_tick_size: Option<Decimal>,
    /// Maker base fee in basis points (wire name `mbf`).
    #[serde(rename = "mbf")]
    pub maker_base_fee: Option<i64>,
    /// Taker base fee in basis points (wire name `tbf`).
    #[serde(rename = "tbf")]
    pub taker_base_fee: Option<i64>,
    /// Whether RFQ (request for quote) is enabled (wire name `rfqe`).
    #[serde(rename = "rfqe")]
    pub rfq_enabled: Option<bool>,
    /// Whether the taker order delay is enabled (wire name `itode`): marketable orders are
    /// then held for the 250 ms taker-delay window before processing. The server omits the
    /// field when `false`.
    #[serde(rename = "itode", default, skip_serializing_if = "Option::is_none")]
    pub taker_order_delay_enabled: Option<bool>,
    /// Whether the Blockaid check is enabled (wire name `ibce`).
    #[serde(rename = "ibce")]
    pub blockaid_check_enabled: Option<bool>,
    /// Fee curve parameters (wire name `fd`).
    #[serde(rename = "fd")]
    pub fee_details: Option<FeeDetails>,
    /// Minimum order age in seconds (wire name `oas`).
    #[serde(rename = "oas")]
    pub min_order_age_seconds: Option<i64>,
    /// Condition id of the market (wire name `c`). Undocumented; observed live.
    #[serde(rename = "c")]
    pub condition_id: Option<ConditionId>,
    /// Delay in seconds applied to marketable orders (wire name `sd`). Undocumented; observed
    /// live as `1` or `3` on sports markets and omitted when the delay is 0. It always equals
    /// the [`Market::seconds_delay`] of the market listings (live check, 2026-10-02); what
    /// the delay applies to is not documented.
    #[serde(rename = "sd")]
    pub seconds_delay: Option<u64>,
    /// Whether the market accepts orders (wire name `ao`). Undocumented; observed live.
    #[serde(rename = "ao")]
    pub accepting_orders: Option<bool>,
    /// Since when the market accepts orders (wire name `aot`, an RFC 3339 date-time).
    /// Undocumented; observed live.
    #[serde(rename = "aot", default, with = "serde_util::datetime_option")]
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    /// Whether negative risk is enabled for this market (wire name `nr`). Undocumented;
    /// observed live, and sent only for neg-risk markets.
    #[serde(rename = "nr")]
    pub neg_risk: Option<bool>,
    /// The `cbos` flag (wire name `cbos`). Undocumented; observed live as a boolean on every
    /// market. Its meaning is unknown, so the field keeps the wire name.
    pub cbos: Option<bool>,
    /// Version tag of the market (wire name `v`). Undocumented; observed live as `"v1"` on
    /// every market.
    #[serde(rename = "v")]
    pub version: Option<String>,
}

/// The parent market of a token (`components/schemas/MarketByTokenResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketByToken {
    /// Condition id of the market containing the token.
    pub condition_id: ConditionId,
    /// The primary token id.
    ///
    /// The spec calls it "the primary (Yes) token", but live it is not reliably the first or
    /// "Yes" token of the market: it is one of the market's two tokens, chosen independently
    /// of outcome order (see `SPEC_DEVIATIONS.md`). Look the outcome up with
    /// [`ClobClient::get_clob_market_info`] if it matters.
    pub primary_token_id: TokenId,
    /// The secondary token id (the market's other token; see
    /// [`primary_token_id`](Self::primary_token_id)).
    pub secondary_token_id: TokenId,
}

/// Minimal market information for live-activity widgets
/// (`components/schemas/LiveActivityMarket`).
///
/// The spec marks no field as required.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LiveActivityMarket {
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Market id (a JSON integer).
    ///
    /// This is the same id as the Gamma market's `id` (live check, 2026-10-02), i.e. the
    /// integer behind a [`MarketId`](crate::types::MarketId); it stays an integer because
    /// the CLOB sends a JSON number (Gamma sends it as a string).
    pub id: Option<i64>,
    /// The market question.
    pub question: Option<String>,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// URL slug of the parent event.
    pub event_slug: Option<String>,
    /// URL slug of the series, if any.
    pub series_slug: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Tag slugs of the market.
    pub tags: Option<Vec<String>>,
}

impl ClobClient {
    /// Lists markets in simplified form (`GET /simplified-markets`, cursor pagination).
    ///
    /// Live pages hold up to 1000 markets (all states, including closed ones, whose
    /// `rewards.rates` is `null`), and the cursors are opaque base64 strings. See
    /// <https://docs.polymarket.com/api-reference/markets/get-simplified-markets>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::{StreamExt as _, TryStreamExt as _};
    ///
    /// let clob = polyoxide::clob::ClobClient::new()?;
    ///
    /// // One page, with the cursor of the next one.
    /// let page = clob.list_simplified_markets().send().await?;
    /// println!("{} markets, next page: {:?}", page.items().len(), page.next_cursor());
    ///
    /// // Or every market, fetching pages lazily.
    /// let first_1000: Vec<_> = clob
    ///     .list_simplified_markets()
    ///     .into_stream()
    ///     .take(1000)
    ///     .try_collect()
    ///     .await?;
    /// # let _ = first_1000;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_simplified_markets(&self) -> ListSimplifiedMarkets {
        ListSimplifiedMarkets {
            client: self.clone(),
            cursor: None,
        }
    }

    /// Lists sampling markets (`GET /sampling-markets`, cursor pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-sampling-markets>.
    pub fn list_sampling_markets(&self) -> ListSamplingMarkets {
        ListSamplingMarkets {
            client: self.clone(),
            cursor: None,
        }
    }

    /// Lists sampling markets in simplified form (`GET /sampling-simplified-markets`, cursor
    /// pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-sampling-simplified-markets>.
    pub fn list_sampling_simplified_markets(&self) -> ListSamplingSimplifiedMarkets {
        ListSamplingSimplifiedMarkets {
            client: self.clone(),
            cursor: None,
        }
    }

    /// Gets all CLOB-level parameters of a market (`GET /clob-markets/{condition_id}`).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-clob-market-info>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `condition_id` is empty. An invalid
    /// condition id is an [`Error::Api`](crate::Error::Api) with status `400`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_clob_market_info(
        &self,
        condition_id: impl Into<ConditionId>,
    ) -> Result<ClobMarketDetails> {
        let condition_id = condition_id.into();
        require_id("condition_id", condition_id.as_str())?;
        self.transport
            .get(&["clob-markets", condition_id.as_str()])
            .send()
            .await
    }

    /// Gets the parent market of a token (`GET /markets-by-token/{token_id}`).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-market-by-token>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty (which the
    /// server documents as `400`). An unknown token is an [`Error::Api`](crate::Error::Api)
    /// with status `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_market_by_token(&self, token_id: impl Into<TokenId>) -> Result<MarketByToken> {
        let token_id = token_id.into();
        require_id("token_id", token_id.as_str())?;
        self.transport
            .get(&["markets-by-token", token_id.as_str()])
            .send()
            .await
    }

    /// Gets live-activity summaries of several markets (`POST /markets/live-activity`).
    ///
    /// Documented only in the CLOB OpenAPI spec (operation `getMarketsLiveActivity`); there
    /// is no reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `condition_ids`) if
    /// `condition_ids` is empty (the server rejects an empty body with `400`) or a condition
    /// id is empty. Unknown markets are an [`Error::Api`](crate::Error::Api) with status
    /// `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_markets_live_activity(
        &self,
        condition_ids: impl IntoIterator<Item = impl Into<ConditionId>>,
    ) -> Result<Vec<LiveActivityMarket>> {
        let condition_ids: Vec<ConditionId> = condition_ids.into_iter().map(Into::into).collect();
        require_non_empty("condition_ids", &condition_ids)?;
        require_list_values(
            "condition_ids",
            condition_ids.iter().map(ConditionId::as_str),
        )?;
        self.transport
            .post(&["markets", "live-activity"])
            .json(&condition_ids)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the live-activity summary of a market
    /// (`GET /markets/live-activity/{condition_id}`).
    ///
    /// Documented only in the CLOB OpenAPI spec (operation `getMarketLiveActivity`); there is
    /// no reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `condition_id` is empty. An invalid
    /// condition id is an [`Error::Api`](crate::Error::Api) with status `400`, an unknown
    /// market one with status `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_market_live_activity(
        &self,
        condition_id: impl Into<ConditionId>,
    ) -> Result<LiveActivityMarket> {
        let condition_id = condition_id.into();
        require_id("condition_id", condition_id.as_str())?;
        self.transport
            .get(&["markets", "live-activity", condition_id.as_str()])
            .send()
            .await
    }
}

/// Fetches one page of a market listing.
async fn fetch_markets_page<T: DeserializeOwned>(
    client: &ClobClient,
    path: &'static str,
    cursor: Option<&str>,
) -> Result<MarketsPage<T>> {
    let mut query = Query::new();
    query.push_opt("next_cursor", cursor);
    client.transport.get(&[path]).query(query).send().await
}

/// Streams every item of a market listing, starting at `start`.
fn markets_stream<T>(client: ClobClient, path: &'static str, start: Option<String>) -> Paginated<T>
where
    T: DeserializeOwned + Send + 'static,
{
    clob_cursor_stream(start, move |cursor: Option<String>| {
        let client = client.clone();
        async move {
            let page: MarketsPage<T> = fetch_markets_page(&client, path, cursor.as_deref()).await?;
            Ok((page.data.unwrap_or_default(), page.next_cursor))
        }
    })
}

/// Generates the request builder of a CLOB market listing.
macro_rules! markets_listing {
    ($(#[$meta:meta])* $name:ident, $path:literal, $item:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
        pub struct $name {
            client: ClobClient,
            cursor: Option<String>,
        }

        impl $name {
            /// Cursor of the page to fetch (the `next_cursor` query parameter), from a
            /// previous page's [`next_cursor()`](MarketsPage::next_cursor()). Omit for the
            /// first page.
            ///
            /// [`END_CURSOR`](super::END_CURSOR) (or an empty cursor) means there are no
            /// more pages: [`into_stream`](Self::into_stream) then yields nothing, while
            /// [`send`](Self::send) still sends it as given.
            pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
                self.cursor = Some(cursor.into());
                self
            }

            /// Fetches one page.
            ///
            /// # Errors
            ///
            /// An invalid request (e.g. an invalid cursor) is an
            /// [`Error::Api`](crate::Error::Api) with status `400`. See
            /// [`Error`](crate::Error) for the other cases.
            pub async fn send(self) -> Result<MarketsPage<$item>> {
                fetch_markets_page(&self.client, $path, self.cursor.as_deref()).await
            }

            /// Streams every market from the configured cursor onwards, fetching pages
            /// lazily until the last page. The stream ends after yielding the first error.
            pub fn into_stream(self) -> Paginated<$item> {
                markets_stream(self.client, $path, self.cursor)
            }
        }
    };
}

markets_listing!(
    /// Request builder for [`ClobClient::list_simplified_markets`].
    ListSimplifiedMarkets,
    "simplified-markets",
    SimplifiedMarket
);

markets_listing!(
    /// Request builder for [`ClobClient::list_sampling_markets`].
    ListSamplingMarkets,
    "sampling-markets",
    Market
);

markets_listing!(
    /// Request builder for [`ClobClient::list_sampling_simplified_markets`].
    ListSamplingSimplifiedMarkets,
    "sampling-simplified-markets",
    SimplifiedMarket
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clob::types::test_util::round_trip;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    /// Field names and types from `components/schemas/PaginatedSimplifiedMarkets` /
    /// `SimplifiedMarket` in docs/specs/clob-openapi.yaml (the spec has no example); token
    /// ids from the `/rewards/markets/{condition_id}` example.
    #[test]
    fn deserializes_simplified_markets_page() {
        let json = r#"{
            "limit": 1,
            "next_cursor": "MQ==",
            "count": 1,
            "data": [{
                "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
                "rewards": {
                    "rates": [{"asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "rewards_daily_rate": 2}],
                    "min_size": 10,
                    "max_spread": 99
                },
                "tokens": [
                    {"token_id": "1343197538147866997676250008839231694243646439454152539053893078719042421992", "outcome": "YES", "price": 0.8, "winner": false},
                    {"token_id": "16678291189211314787145083999015737376658799626183230671758641503291735614088", "outcome": "NO", "price": 0.2}
                ],
                "active": true,
                "closed": false,
                "archived": false,
                "accepting_orders": true
            }]
        }"#;
        let page: MarketsPage<SimplifiedMarket> = round_trip(json);
        assert_eq!(page.next_cursor(), Some("MQ=="));
        let market = &page.items()[0];
        let rewards = market.rewards.as_ref().unwrap();
        assert_eq!(rewards.max_spread, Some(d("99")));
        assert_eq!(
            rewards.rates.as_ref().unwrap()[0].rewards_daily_rate,
            Some(d("2"))
        );
        let tokens = market.tokens.as_ref().unwrap();
        assert_eq!(tokens[0].price, Some(d("0.8")));
        assert_eq!(tokens[1].winner, None);
        assert_eq!(market.accepting_orders, Some(true));
    }

    /// Field names and types from `components/schemas/Market` in docs/specs/clob-openapi.yaml
    /// (the spec has no example).
    #[test]
    fn deserializes_market() {
        let json = r#"{
            "enable_order_book": true,
            "active": true,
            "closed": false,
            "archived": false,
            "accepting_orders": true,
            "accepting_order_timestamp": "2024-01-01T00:00:00Z",
            "minimum_order_size": 5,
            "minimum_tick_size": 0.01,
            "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "question_id": "0x01",
            "question": "Will Trump win the 2024 Iowa Caucus?",
            "description": "",
            "market_slug": "will-trump-win-the-2024-iowa-caucus",
            "end_date_iso": "2024-08-10T00:00:00Z",
            "game_start_time": null,
            "seconds_delay": 0,
            "fpmm": "",
            "maker_base_fee": 0,
            "taker_base_fee": 0,
            "notifications_enabled": true,
            "neg_risk": false,
            "neg_risk_market_id": "",
            "neg_risk_request_id": "",
            "icon": "https://example.com/icon.png",
            "image": "https://example.com/image.png",
            "rewards": {"rates": null, "min_size": 10, "max_spread": 99},
            "is_50_50_outcome": false,
            "tokens": [],
            "tags": ["politics"]
        }"#;
        let market: Market = round_trip(json);
        assert_eq!(
            market
                .accepting_order_timestamp
                .map(|t| t.timestamp())
                .unwrap(),
            1_704_067_200
        );
        assert_eq!(market.minimum_tick_size, Some(d("0.01")));
        assert_eq!(market.game_start_time, None);
        assert_eq!(market.is_50_50_outcome, Some(false));
        assert_eq!(market.tags.as_deref(), Some(&["politics".to_owned()][..]));

        let empty: Market = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.condition_id, None);
    }

    /// Field names, types and examples from `components/schemas/ClobMarketDetails` in
    /// docs/specs/clob-openapi.yaml (docs/api-reference/markets/get-clob-market-info.md).
    #[test]
    fn deserializes_clob_market_details() {
        let json = r#"{
            "gst": null,
            "r": {"min_size": 10},
            "t": [
                {"t": "71321045679252212594626385532706912750332728571942532289631379312455583992563", "o": "Yes"},
                {"t": "52114319501245915516055106046884209969926127482827954674443846427813813222426", "o": "No"}
            ],
            "mos": 5,
            "mts": 0.01,
            "mbf": 0,
            "tbf": 0,
            "rfqe": true,
            "ibce": false,
            "fd": {"r": 0.02, "e": 2, "to": true},
            "oas": 0
        }"#;
        let details: ClobMarketDetails = round_trip(json);
        assert_eq!(details.game_start_time, None);
        assert!(details.rewards.unwrap().contains_key("min_size"));
        let tokens = details.tokens.unwrap();
        assert_eq!(tokens[1].outcome.as_deref(), Some("No"));
        assert_eq!(details.min_order_size, Some(d("5")));
        assert_eq!(details.min_tick_size, Some(d("0.01")));
        assert_eq!(details.rfq_enabled, Some(true));
        // `itode` is omitted when false, and stays omitted when serialized.
        assert_eq!(details.taker_order_delay_enabled, None);
        let fees = details.fee_details.unwrap();
        assert_eq!(fees.rate, Some(d("0.02")));
        assert_eq!(fees.exponent, Some(d("2")));
        assert_eq!(fees.takers_only, Some(true));

        let enabled: ClobMarketDetails = serde_json::from_str(r#"{"itode":true}"#).unwrap();
        assert_eq!(enabled.taker_order_delay_enabled, Some(true));

        let with_gst: ClobMarketDetails =
            serde_json::from_str(r#"{"gst":"2024-05-01T12:00:00Z","fd":{"r":null}}"#).unwrap();
        assert_eq!(
            with_gst.game_start_time.map(|t| t.timestamp()),
            Some(1_714_564_800)
        );
        assert_eq!(with_gst.fee_details.unwrap().rate, None);
    }

    /// Live response of `GET /clob-markets/0x81a5..1a3a` (captured 2026-10-02): the undocumented
    /// keys `c`, `ao`, `aot`, `nr`, `cbos` and `v` are present, `rfqe` is absent, and the
    /// rewards object `r` has abbreviated keys.
    #[test]
    fn deserializes_live_clob_market_details() {
        let json = r#"{
            "r": {"mi": 50, "ma": 4.5, "moas": 4},
            "t": [
                {"t": "52634616068523389389514492087655237014427439869589807217055529923225131895030", "o": "Yes"},
                {"t": "106302272146511626715366732538958019243031587527887799373406690681902311718700", "o": "No"}
            ],
            "c": "0x81a537b379a35e4e17c286d3b37394e94bd74c1779bbe9a13670eb991b201a3a",
            "mos": 5,
            "mts": 0.001,
            "mbf": 1000,
            "tbf": 1000,
            "ao": true,
            "nr": true,
            "cbos": true,
            "aot": "2025-09-18T20:07:36Z",
            "ibce": true,
            "fd": {"r": 0.04, "e": 1, "to": true},
            "v": "v1"
        }"#;
        let details: ClobMarketDetails = round_trip(json);
        assert_eq!(
            details.condition_id.as_ref().map(ConditionId::as_str),
            Some("0x81a537b379a35e4e17c286d3b37394e94bd74c1779bbe9a13670eb991b201a3a")
        );
        assert_eq!(details.accepting_orders, Some(true));
        assert_eq!(
            details.accepting_order_timestamp.map(|t| t.timestamp()),
            Some(1_758_226_056)
        );
        assert_eq!(details.neg_risk, Some(true));
        assert_eq!(details.cbos, Some(true));
        assert_eq!(details.version.as_deref(), Some("v1"));
        assert_eq!(details.seconds_delay, None);
        assert_eq!(details.min_tick_size, Some(d("0.001")));
        assert_eq!(details.maker_base_fee, Some(1000));
        assert_eq!(details.rfq_enabled, None);
        assert!(details.rewards.unwrap().contains_key("moas"));

        // `sd` appears on markets with a taker delay, `gst` on sports markets.
        let sports: ClobMarketDetails =
            serde_json::from_str(r#"{"gst":"2026-03-13T14:30:00Z","sd":3}"#).unwrap();
        assert_eq!(sports.seconds_delay, Some(3));
        assert!(sports.game_start_time.is_some());
    }

    /// Examples from `components/schemas/MarketByTokenResponse` in
    /// docs/specs/clob-openapi.yaml.
    #[test]
    fn deserializes_market_by_token() {
        let json = r#"{
            "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "primary_token_id": "71321045679252212594626385532706912750332728571942532289631379312455583992563",
            "secondary_token_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426"
        }"#;
        let market: MarketByToken = round_trip(json);
        assert_eq!(
            market.primary_token_id,
            "71321045679252212594626385532706912750332728571942532289631379312455583992563"
        );
        assert!(serde_json::from_str::<MarketByToken>(r#"{"condition_id":"0x1"}"#).is_err());
    }

    /// Field names and types from `components/schemas/LiveActivityMarket` in
    /// docs/specs/clob-openapi.yaml (the spec has no example).
    #[test]
    fn deserializes_live_activity_market() {
        let json = r#"{
            "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "id": 248849,
            "question": "Will Trump win the 2024 Iowa Caucus?",
            "market_slug": "will-trump-win-the-2024-iowa-caucus",
            "event_slug": "2024-us-election",
            "series_slug": null,
            "icon": "https://example.com/icon.png",
            "image": "https://example.com/image.png",
            "tags": ["politics", "elections"]
        }"#;
        let market: LiveActivityMarket = round_trip(json);
        assert_eq!(market.id, Some(248_849));
        assert_eq!(market.series_slug, None);
        assert_eq!(market.tags.unwrap().len(), 2);
    }
}
