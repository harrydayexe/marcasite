//! Market data: order books, prices, midpoints, spreads, last trade prices, fee rates, tick
//! sizes and the neg-risk flag.
//!
//! The method naming scheme for endpoints with several documented forms (`get_<things>`,
//! `_by_path`) is described in the [`clob`](crate::clob) module docs.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use marcasite_core::{
    Query, Result, ValidationError, serde_util,
    types::{ConditionId, Side, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    ClobClient,
    types::{
        BookRequest, require_at_most, require_id, require_list_values, require_non_empty,
        side_or_empty, timestamp_millis_string,
    },
};

/// Maximum number of token ids per last-trade-prices request
/// ([`ClobClient::get_last_trade_prices`]).
pub const MAX_LAST_TRADE_PRICES_TOKEN_IDS: usize = 500;

/// Name of the token-id list in validation errors (the request body of the batch routes).
const TOKEN_IDS: &str = "token_ids";

/// A price level of an order book (`components/schemas/OrderSummary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderSummary {
    /// Price of the level (a numeric string on the wire).
    pub price: Decimal,
    /// Total size resting at this price (a numeric string on the wire).
    pub size: Decimal,
}

/// An order book snapshot for one token (`components/schemas/OrderBookSummary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderBookSummary {
    /// Market condition id.
    pub market: ConditionId,
    /// Token id (asset id).
    pub asset_id: TokenId,
    /// Timestamp of the snapshot.
    ///
    /// Sent as a numeric string of Unix **milliseconds** (e.g. `"1790934258308"`; the REST
    /// spec does not document the unit, live confirms it) and serialized back to a string.
    #[serde(with = "timestamp_millis_string")]
    pub timestamp: DateTime<Utc>,
    /// Hash of the order book summary, exactly as sent.
    ///
    /// Live this is 40 hex characters without a `0x` prefix (the spec only has a placeholder
    /// example), so it is kept as a plain string.
    pub hash: String,
    /// Bids, sorted by price descending.
    pub bids: Vec<OrderSummary>,
    /// Asks, sorted by price ascending.
    pub asks: Vec<OrderSummary>,
    /// Minimum order size (a numeric string on the wire).
    pub min_order_size: Decimal,
    /// Minimum price increment (tick size; a numeric string on the wire).
    pub tick_size: Decimal,
    /// Whether negative risk is enabled for this market.
    pub neg_risk: bool,
    /// Last trade price (a numeric string on the wire).
    pub last_trade_price: Decimal,
}

/// The midpoint price of a token: the average of the best bid and best ask
/// (`GET /midpoint` response).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Midpoint {
    /// Midpoint price (a numeric string on the wire).
    ///
    /// The wire name is `mid`; the spec documents `mid_price`, which live never sends (see
    /// `SPEC_DEVIATIONS.md`). `mid_price` is still accepted when reading.
    #[serde(alias = "mid_price")]
    pub mid: Decimal,
}

/// The spread of a token: best ask minus best bid (`GET /spread` response).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Spread {
    /// The spread (a numeric string on the wire).
    pub spread: Decimal,
}

/// The best price for a token and side (`GET /price` response): the best bid for
/// [`Side::Buy`], the best ask for [`Side::Sell`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Price {
    /// Market price (a numeric string on the wire; the spec documents a JSON number, which
    /// is accepted as well).
    pub price: Decimal,
}

/// The last trade of a token (`GET /last-trade-price` response).
///
/// If the token has no trades, the server returns a price of `0.5` and an empty side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastTradePrice {
    /// Last trade price (`0.5` if there were no trades; a numeric string on the wire).
    pub price: Decimal,
    /// Last trade side; `None` when the server sends the documented empty string (no
    /// trades). `None` serializes back to `""`.
    #[serde(with = "side_or_empty")]
    pub side: Option<Side>,
}

/// The last trade of one token in a batch (`GET`/`POST /last-trades-prices` response
/// items).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TokenLastTradePrice {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Last trade price (a numeric string on the wire).
    pub price: Decimal,
    /// Last trade side.
    ///
    /// The spec documents only `BUY` / `SELL` here. `GET /last-trade-price` documents an
    /// empty side for a token without trades; the same value is mapped to `None` here, as
    /// in [`LastTradePrice::side`], and `None` serializes back to `""`.
    #[serde(with = "side_or_empty")]
    pub side: Option<Side>,
}

/// The base fee rate of a token (`components/schemas/FeeRate`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FeeRate {
    /// Base fee in basis points.
    pub base_fee: i64,
}

/// The minimum tick size of a token (`components/schemas/TickSize`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickSize {
    /// Minimum tick size (price increment; a JSON number on the wire).
    #[serde(with = "serde_util::decimal_number")]
    pub minimum_tick_size: Decimal,
}

/// The negative-risk flag of a token's market (`components/schemas/NegRisk`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NegRisk {
    /// Whether negative risk is enabled for this market.
    pub neg_risk: bool,
}

/// Collects the items of a batch request body, failing if there are none or a token id is
/// empty.
fn body_requests(
    requests: impl IntoIterator<Item = impl Into<BookRequest>>,
) -> Result<Vec<BookRequest>> {
    let requests: Vec<BookRequest> = requests.into_iter().map(Into::into).collect();
    require_non_empty(TOKEN_IDS, &requests)?;
    require_list_values(TOKEN_IDS, requests.iter().map(|r| r.token_id.as_str()))?;
    Ok(requests)
}

/// Fails if a price request has no side.
fn require_sides(requests: &[BookRequest]) -> Result<()> {
    if requests.iter().any(|request| request.side.is_none()) {
        return Err(ValidationError::new(
            "side",
            "every request needs a side (each request must include both token_id and side)",
        )
        .into());
    }
    Ok(())
}

/// Builds the `token_id` query of the single-token endpoints, failing on an empty id.
fn token_id_query(token_id: &TokenId) -> Result<Query> {
    require_id("token_id", token_id.as_str())?;
    let mut query = Query::new();
    query.push("token_id", token_id);
    Ok(query)
}

impl ClobClient {
    /// Gets the order book of a token (`GET /book`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-order-book>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// let clob = marcasite::clob::ClobClient::new()?;
    /// let book = clob
    ///     .get_order_book(
    ///         "71321045679252212594626385532706912750332728571942532289631379312455583992563",
    ///     )
    ///     .await?;
    /// if let Some(best_bid) = book.bids.first() {
    ///     println!("best bid {} x {}", best_bid.price, best_bid.size);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. A token
    /// without an order book is an [`Error::Api`](crate::Error::Api) with status `404`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_order_book(&self, token_id: impl Into<TokenId>) -> Result<OrderBookSummary> {
        let query = token_id_query(&token_id.into())?;
        self.transport.get(&["book"]).query(query).send().await
    }

    /// Gets the order books of several tokens (`POST /books`).
    ///
    /// Accepts token ids directly or [`BookRequest`]s.
    ///
    /// The `GET /books?token_ids=...` form that the OpenAPI spec lists (`getBooksGet`) always
    /// answers `400 Invalid payload` live, so it is not implemented (see
    /// `SPEC_DEVIATIONS.md`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-order-books-request-body>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `token_ids`) if `requests`
    /// is empty (the request body is required) or a token id is empty. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_order_books(
        &self,
        requests: impl IntoIterator<Item = impl Into<BookRequest>>,
    ) -> Result<Vec<OrderBookSummary>> {
        let body = body_requests(requests)?;
        self.transport
            .post(&["books"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the best price of a token for a side (`GET /price`): the best bid for
    /// [`Side::Buy`], the best ask for [`Side::Sell`].
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-market-price>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use marcasite::types::Side;
    ///
    /// let clob = marcasite::clob::ClobClient::new()?;
    /// let best_bid = clob
    ///     .get_price(
    ///         "71321045679252212594626385532706912750332728571942532289631379312455583992563",
    ///         Side::Buy,
    ///     )
    ///     .await?;
    /// println!("best bid: {}", best_bid.price);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. A token
    /// without an order book is an [`Error::Api`](crate::Error::Api) with status `404`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_price(&self, token_id: impl Into<TokenId>, side: Side) -> Result<Price> {
        let mut query = token_id_query(&token_id.into())?;
        query.push("side", side);
        self.transport.get(&["price"]).query(query).send().await
    }

    /// Gets the best prices of several `(token id, side)` pairs (`POST /prices`).
    ///
    /// Accepts `(token id, Side)` pairs (owned or borrowed) or [`BookRequest`]s with a side.
    /// Returns a map of token id to a map of side to price (the prices are numeric strings on
    /// the wire).
    ///
    /// Every request needs a side, as the docs say. Live, an item without a side is accepted
    /// but silently priced as `SELL` only, so this is rejected client-side instead. The
    /// `GET /prices?token_ids=...&sides=...` form always answers `400 Invalid payload` live,
    /// so it is not implemented (see `SPEC_DEVIATIONS.md`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-market-prices-request-body>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `requests` is empty or a token id
    /// is empty (parameter `token_ids`), or if a request has no side (parameter `side`; the
    /// docs require "both token_id and side"). See [`Error`](crate::Error) for the other
    /// cases.
    pub async fn get_prices(
        &self,
        requests: impl IntoIterator<Item = impl Into<BookRequest>>,
    ) -> Result<HashMap<TokenId, HashMap<Side, Decimal>>> {
        let body = body_requests(requests)?;
        require_sides(&body)?;
        self.transport
            .post(&["prices"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the midpoint price of a token (`GET /midpoint`).
    ///
    /// See <https://docs.polymarket.com/api-reference/data/get-midpoint-price>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. A token
    /// without an order book is an [`Error::Api`](crate::Error::Api) with status `404`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_midpoint(&self, token_id: impl Into<TokenId>) -> Result<Midpoint> {
        let query = token_id_query(&token_id.into())?;
        self.transport.get(&["midpoint"]).query(query).send().await
    }

    /// Gets the midpoint prices of several tokens (`POST /midpoints`). Returns a map of
    /// token id to midpoint.
    ///
    /// Accepts token ids directly or [`BookRequest`]s (the side is not used). The
    /// `GET /midpoints?token_ids=...` form always answers `400 Invalid payload` live, so it
    /// is not implemented (see `SPEC_DEVIATIONS.md`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-midpoint-prices-request-body>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `token_ids`) if `requests`
    /// is empty (the request body is required) or a token id is empty. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_midpoints(
        &self,
        requests: impl IntoIterator<Item = impl Into<BookRequest>>,
    ) -> Result<HashMap<TokenId, Decimal>> {
        let body = body_requests(requests)?;
        self.transport
            .post(&["midpoints"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the spread of a token (`GET /spread`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-spread>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. A token
    /// without an order book is an [`Error::Api`](crate::Error::Api) with status `404`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_spread(&self, token_id: impl Into<TokenId>) -> Result<Spread> {
        let query = token_id_query(&token_id.into())?;
        self.transport.get(&["spread"]).query(query).send().await
    }

    /// Gets the spreads of several tokens (`POST /spreads`, the only form).
    /// Returns a map of token id to spread.
    ///
    /// Accepts token ids directly or [`BookRequest`]s.
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-spreads>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `token_ids`) if `requests`
    /// is empty (the request body is required) or a token id is empty. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_spreads(
        &self,
        requests: impl IntoIterator<Item = impl Into<BookRequest>>,
    ) -> Result<HashMap<TokenId, Decimal>> {
        let body = body_requests(requests)?;
        self.transport
            .post(&["spreads"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the last trade price and side of a token (`GET /last-trade-price`).
    ///
    /// A token without trades yields a price of `0.5` and no side.
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-last-trade-price>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_last_trade_price(
        &self,
        token_id: impl Into<TokenId>,
    ) -> Result<LastTradePrice> {
        let query = token_id_query(&token_id.into())?;
        self.transport
            .get(&["last-trade-price"])
            .query(query)
            .send()
            .await
    }

    /// Gets the last trade prices of up to [`MAX_LAST_TRADE_PRICES_TOKEN_IDS`] tokens
    /// (`POST /last-trades-prices`).
    ///
    /// Accepts token ids directly or [`BookRequest`]s. An empty side from the server is
    /// mapped to a `None` [`TokenLastTradePrice::side`]. The
    /// `GET /last-trades-prices?token_ids=...` form always answers `400 Invalid payload`
    /// live, so it is not implemented (see `SPEC_DEVIATIONS.md`).
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-last-trade-prices-request-body>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `token_ids`) if `requests`
    /// is empty (the request body is required), has more than
    /// [`MAX_LAST_TRADE_PRICES_TOKEN_IDS`] entries, or a token id is empty. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn get_last_trade_prices(
        &self,
        requests: impl IntoIterator<Item = impl Into<BookRequest>>,
    ) -> Result<Vec<TokenLastTradePrice>> {
        let body = body_requests(requests)?;
        require_at_most(TOKEN_IDS, &body, MAX_LAST_TRADE_PRICES_TOKEN_IDS)?;
        self.transport
            .post(&["last-trades-prices"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }

    /// Gets the base fee rate of a token with a query parameter (`GET /fee-rate`).
    ///
    /// The spec marks the `token_id` query parameter as optional, so it is a builder setter
    /// here. The endpoint is documented as returning the fee rate "for a specific token ID",
    /// and the docs do not say what the server returns when `token_id` is omitted.
    /// [`ClobClient::get_fee_rate_by_path`] takes the token id as a required argument.
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-fee-rate>.
    pub fn get_fee_rate(&self) -> GetFeeRate {
        GetFeeRate {
            client: self.clone(),
            token_id: None,
        }
    }

    /// Gets the base fee rate of a token with a path parameter
    /// (`GET /fee-rate/{token_id}`). The query-parameter form is
    /// [`ClobClient::get_fee_rate`].
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-fee-rate-by-path-parameter>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. An invalid
    /// token id is an [`Error::Api`](crate::Error::Api) with status `400`, an unknown market
    /// one with status `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_fee_rate_by_path(&self, token_id: impl Into<TokenId>) -> Result<FeeRate> {
        let token_id = token_id.into();
        require_id("token_id", token_id.as_str())?;
        self.transport
            .get(&["fee-rate", token_id.as_str()])
            .send()
            .await
    }

    /// Gets the minimum tick size of a token with a query parameter (`GET /tick-size`).
    ///
    /// The spec marks the `token_id` query parameter as optional, so it is a builder setter
    /// here. The endpoint is documented as returning the tick size "for a specific token
    /// ID", and the docs do not say what the server returns when `token_id` is omitted.
    /// [`ClobClient::get_tick_size_by_path`] takes the token id as a required argument.
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-tick-size>.
    pub fn get_tick_size(&self) -> GetTickSize {
        GetTickSize {
            client: self.clone(),
            token_id: None,
        }
    }

    /// Gets the minimum tick size of a token with a path parameter
    /// (`GET /tick-size/{token_id}`). The query-parameter form is
    /// [`ClobClient::get_tick_size`].
    ///
    /// See <https://docs.polymarket.com/api-reference/market-data/get-tick-size-by-path-parameter>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. An invalid
    /// token id is an [`Error::Api`](crate::Error::Api) with status `400`, an unknown market
    /// one with status `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_tick_size_by_path(&self, token_id: impl Into<TokenId>) -> Result<TickSize> {
        let token_id = token_id.into();
        require_id("token_id", token_id.as_str())?;
        self.transport
            .get(&["tick-size", token_id.as_str()])
            .send()
            .await
    }

    /// Gets the negative-risk flag of a token's market with a query parameter
    /// (`GET /neg-risk`).
    ///
    /// The spec marks the `token_id` query parameter as optional, so it is a builder setter
    /// here. The endpoint is documented as returning the flag "for a specific token ID", and
    /// the docs do not say what the server returns when `token_id` is omitted.
    /// [`ClobClient::get_neg_risk_by_path`] takes the token id as a required argument.
    ///
    /// Documented only in the CLOB OpenAPI spec (operation `getNegRisk`); there is no
    /// reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.
    pub fn get_neg_risk(&self) -> GetNegRisk {
        GetNegRisk {
            client: self.clone(),
            token_id: None,
        }
    }

    /// Gets the negative-risk flag of a token's market with a path parameter
    /// (`GET /neg-risk/{token_id}`). The query-parameter form is
    /// [`ClobClient::get_neg_risk`].
    ///
    /// Documented only in the CLOB OpenAPI spec (operation `getNegRiskByPath`); there is no
    /// reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `token_id` is empty. An invalid
    /// token id is an [`Error::Api`](crate::Error::Api) with status `400`, an unknown market
    /// one with status `404`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_neg_risk_by_path(&self, token_id: impl Into<TokenId>) -> Result<NegRisk> {
        let token_id = token_id.into();
        require_id("token_id", token_id.as_str())?;
        self.transport
            .get(&["neg-risk", token_id.as_str()])
            .send()
            .await
    }
}

/// Builds the optional `token_id` query of `/fee-rate`, `/tick-size` and `/neg-risk`,
/// failing on an empty id.
fn optional_token_id_query(token_id: Option<&TokenId>) -> Result<Query> {
    match token_id {
        Some(token_id) => token_id_query(token_id),
        None => Ok(Query::new()),
    }
}

/// Request builder for [`ClobClient::get_fee_rate`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetFeeRate {
    client: ClobClient,
    token_id: Option<TokenId>,
}

impl GetFeeRate {
    /// Token id (asset id), the `token_id` query parameter.
    pub fn token_id(mut self, token_id: impl Into<TokenId>) -> Self {
        self.token_id = Some(token_id.into());
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if the token id is set but empty. An
    /// invalid token id is an [`Error::Api`](crate::Error::Api) with status `400`, an
    /// unknown market one with status `404`. See [`Error`](crate::Error) for the other
    /// cases.
    pub async fn send(self) -> Result<FeeRate> {
        let query = optional_token_id_query(self.token_id.as_ref())?;
        self.client
            .transport
            .get(&["fee-rate"])
            .query(query)
            .send()
            .await
    }
}

/// Request builder for [`ClobClient::get_tick_size`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetTickSize {
    client: ClobClient,
    token_id: Option<TokenId>,
}

impl GetTickSize {
    /// Token id (asset id), the `token_id` query parameter.
    pub fn token_id(mut self, token_id: impl Into<TokenId>) -> Self {
        self.token_id = Some(token_id.into());
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if the token id is set but empty. An
    /// invalid token id is an [`Error::Api`](crate::Error::Api) with status `400`, an
    /// unknown market one with status `404`. See [`Error`](crate::Error) for the other
    /// cases.
    pub async fn send(self) -> Result<TickSize> {
        let query = optional_token_id_query(self.token_id.as_ref())?;
        self.client
            .transport
            .get(&["tick-size"])
            .query(query)
            .send()
            .await
    }
}

/// Request builder for [`ClobClient::get_neg_risk`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetNegRisk {
    client: ClobClient,
    token_id: Option<TokenId>,
}

impl GetNegRisk {
    /// Token id (asset id), the `token_id` query parameter.
    pub fn token_id(mut self, token_id: impl Into<TokenId>) -> Self {
        self.token_id = Some(token_id.into());
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if the token id is set but empty. An
    /// invalid token id is an [`Error::Api`](crate::Error::Api) with status `400`, an
    /// unknown market one with status `404`. See [`Error`](crate::Error) for the other
    /// cases.
    pub async fn send(self) -> Result<NegRisk> {
        let query = optional_token_id_query(self.token_id.as_ref())?;
        self.client
            .transport
            .get(&["neg-risk"])
            .query(query)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clob::types::test_util::round_trip;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    /// Trimmed live response of `GET /book?token_id=...` (captured 2026-10-02): `timestamp` is
    /// Unix milliseconds, `hash` is 40 hex characters without `0x`, `market` 64 hex.
    #[test]
    fn deserializes_order_book() {
        let json = r#"{
            "market": "0x81a537b379a35e4e17c286d3b37394e94bd74c1779bbe9a13670eb991b201a3a",
            "asset_id": "52634616068523389389514492087655237014427439869589807217055529923225131895030",
            "timestamp": "1790934258308",
            "hash": "f3e9b337b4a1184b1f35282e0a0d0ed5e3b7c07e",
            "bids": [],
            "asks": [{"price": "0.999", "size": "10057923.36"}, {"price": "0.998", "size": "512673.29"}],
            "min_order_size": "5",
            "tick_size": "0.001",
            "neg_risk": true,
            "last_trade_price": "0.001"
        }"#;
        let book: OrderBookSummary = round_trip(json);
        assert_eq!(
            book.asset_id,
            "52634616068523389389514492087655237014427439869589807217055529923225131895030"
        );
        assert_eq!(book.timestamp.timestamp_millis(), 1_790_934_258_308);
        assert_eq!(book.hash.len(), 40);
        assert!(book.bids.is_empty());
        assert_eq!(book.asks.len(), 2);
        assert_eq!(book.asks[0].price, d("0.999"));
        assert_eq!(book.asks[0].size, d("10057923.36"));
        assert_eq!(book.tick_size, d("0.001"));
        assert!(book.neg_risk);
        assert_eq!(book.last_trade_price, d("0.001"));
    }

    #[test]
    fn order_book_requires_documented_fields() {
        let err = serde_json::from_str::<OrderBookSummary>(r#"{"market":"0x1"}"#).unwrap_err();
        assert!(err.to_string().contains("missing field"), "{err}");
    }

    /// Live responses of `GET /midpoint`, `GET /spread`, `GET /price`, `GET /fee-rate`,
    /// `GET /tick-size` and `GET /neg-risk` (captured 2026-10-02). `/price` sends a numeric
    /// string (the spec documents a number, which is accepted too) and `/tick-size` a number.
    #[test]
    fn deserializes_single_values() {
        let mid: Midpoint = round_trip(r#"{"mid":"0.0005"}"#);
        assert_eq!(mid.mid, d("0.0005"));
        // The spec documents `mid_price`; it is accepted when reading, never written.
        let documented: Midpoint = serde_json::from_str(r#"{"mid_price":"0.45"}"#).unwrap();
        assert_eq!(documented.mid, d("0.45"));
        assert_eq!(
            serde_json::to_string(&documented).unwrap(),
            r#"{"mid":"0.45"}"#
        );
        let spread: Spread = round_trip(r#"{"spread":"0.001"}"#);
        assert_eq!(spread.spread, d("0.001"));
        let price: Price = round_trip(r#"{"price":"0.999"}"#);
        assert_eq!(price.price, d("0.999"));
        let price: Price = serde_json::from_str(r#"{"price":0.45}"#).unwrap();
        assert_eq!(price.price, d("0.45"));
        let fee: FeeRate = round_trip(r#"{"base_fee":1000}"#);
        assert_eq!(fee.base_fee, 1000);
        let tick: TickSize = round_trip(r#"{"minimum_tick_size":0.001}"#);
        assert_eq!(tick.minimum_tick_size, d("0.001"));
        let neg: NegRisk = round_trip(r#"{"neg_risk":true}"#);
        assert!(neg.neg_risk);

        // A numeric string is accepted for number-typed fields as well.
        let tick: TickSize = serde_json::from_str(r#"{"minimum_tick_size":"0.01"}"#).unwrap();
        assert_eq!(tick.minimum_tick_size, d("0.01"));
    }

    /// `GET /last-trade-price`: example `{price: '0.45', side: BUY}` and the documented
    /// "no trades" default (`"0.5"`, empty side) in docs/polymarket/specs/clob-openapi.yaml.
    #[test]
    fn deserializes_last_trade_price() {
        let last: LastTradePrice = round_trip(r#"{"price":"0.45","side":"BUY"}"#);
        assert_eq!(last.price, d("0.45"));
        assert_eq!(last.side, Some(Side::Buy));

        let none: LastTradePrice = round_trip(r#"{"price":"0.5","side":""}"#);
        assert_eq!(none.price, d("0.5"));
        assert_eq!(none.side, None);

        // `side` is required by the spec.
        assert!(serde_json::from_str::<LastTradePrice>(r#"{"price":"0.5"}"#).is_err());
    }

    /// Example response of `GET /last-trades-prices` in docs/polymarket/specs/clob-openapi.yaml.
    #[test]
    fn deserializes_last_trade_prices() {
        let json = r#"[
            {"token_id": "0xabc123def456...", "price": "0.45", "side": "BUY"},
            {"token_id": "0xdef456abc123...", "price": "0.52", "side": "SELL"}
        ]"#;
        let prices: Vec<TokenLastTradePrice> = round_trip(json);
        assert_eq!(prices[0].token_id, "0xabc123def456...");
        assert_eq!(prices[1].price, d("0.52"));
        assert_eq!(prices[1].side, Some(Side::Sell));
    }

    /// An empty side (the "no trades" value of `GET /last-trade-price`) maps to `None` in
    /// the batch form too; any other undocumented value is kept as `Side::Unknown`.
    #[test]
    fn last_trade_prices_side_edge_cases() {
        let none: TokenLastTradePrice = round_trip(r#"{"token_id":"1","price":"0.5","side":""}"#);
        assert_eq!(none.side, None);
        let unknown: TokenLastTradePrice =
            round_trip(r#"{"token_id":"1","price":"0.5","side":"HOLD"}"#);
        assert_eq!(unknown.side, Some(Side::Unknown("HOLD".to_owned())));
        assert!(
            serde_json::from_str::<TokenLastTradePrice>(r#"{"token_id":"1","price":"0.5"}"#)
                .is_err()
        );
    }

    /// Example responses of `POST /prices` and `POST /midpoints` in
    /// docs/polymarket/specs/clob-openapi.yaml (numbers), and live (numeric strings, captured
    /// 2026-10-02).
    #[test]
    fn deserializes_price_maps() {
        let json = r#"{"0xabc123def456...":{"BUY":0.45},"0xdef456abc123...":{"SELL":0.52}}"#;
        let prices: HashMap<TokenId, HashMap<Side, Decimal>> = serde_json::from_str(json).unwrap();
        assert_eq!(
            prices[&TokenId::from("0xabc123def456...")][&Side::Buy],
            d("0.45")
        );
        assert_eq!(
            prices[&TokenId::from("0xdef456abc123...")][&Side::Sell],
            d("0.52")
        );

        let json = r#"{"0xabc123def456...":"0.45","0xdef456abc123...":"0.52"}"#;
        let mids: HashMap<TokenId, Decimal> = serde_json::from_str(json).unwrap();
        assert_eq!(mids[&TokenId::from("0xdef456abc123...")], d("0.52"));

        let json = r#"{"1063":{"BUY":"0.999"},"5263":{"SELL":"0.001"}}"#;
        let prices: HashMap<TokenId, HashMap<Side, Decimal>> = serde_json::from_str(json).unwrap();
        assert_eq!(prices[&TokenId::from("1063")][&Side::Buy], d("0.999"));
    }

    #[test]
    fn request_validation() {
        let parameter = |result: Result<Vec<BookRequest>>| match result {
            Err(marcasite_core::Error::Validation(v)) => v.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        };
        assert_eq!(
            parameter(body_requests(Vec::<BookRequest>::new())),
            "token_ids"
        );
        assert_eq!(parameter(body_requests([""])), "token_ids");
        assert!(body_requests(["1,2"]).is_ok());

        let unsided = [BookRequest::new("1")];
        let err = require_sides(&unsided).unwrap_err();
        assert!(matches!(&err, marcasite_core::Error::Validation(v) if v.parameter() == "side"));
        assert!(require_sides(&[BookRequest::from(("1", Side::Buy))]).is_ok());

        assert!(token_id_query(&TokenId::from("")).is_err());
        assert_eq!(
            token_id_query(&TokenId::from("1")).unwrap().get("token_id"),
            Some("1")
        );
        assert!(optional_token_id_query(None).unwrap().is_empty());
    }
}
