//! Price history: `GET /prices-history` and `POST /batch-prices-history`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use polyoxide_core::{Query, Result, serde_util, types::TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{ClobClient, types::require_at_most};

/// Maximum number of markets per [`ClobClient::get_batch_prices_history`] request.
pub const MAX_BATCH_PRICES_HISTORY_MARKETS: usize = 20;

polyoxide_core::string_enum! {
    /// Time interval for price-history aggregation (the `interval` parameter).
    ///
    /// The spec lists the values without describing them; the duration names below follow
    /// their documented order (`max`, `all`, `1m`, `1w`, `1d`, `6h`, `1h`).
    pub enum PriceHistoryInterval {
        /// `max`.
        Max => "max",
        /// `all`.
        All => "all",
        /// `1m`: one month (listed between `all` and `1w`).
        OneMonth => "1m",
        /// `1w`: one week.
        OneWeek => "1w",
        /// `1d`: one day.
        OneDay => "1d",
        /// `6h`: six hours.
        SixHours => "6h",
        /// `1h`: one hour.
        OneHour => "1h",
    }
}

/// One point of a price history (`components/schemas/MarketPrice`).
///
/// The spec marks neither field as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PricePoint {
    /// Time of the point (wire name `t`).
    ///
    /// The spec types it as a `uint32` without stating the unit; it is decoded as Unix
    /// seconds, the only Unix-timestamp unit a `uint32` can hold for current dates and the
    /// unit documented for the `start_ts` / `end_ts` filters.
    #[serde(rename = "t", default, with = "serde_util::timestamp_seconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Price at that time (wire name `p`).
    #[serde(rename = "p")]
    pub price: Option<Decimal>,
}

/// The price history of one market (`components/schemas/PricesHistoryResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PricesHistory {
    /// The price points.
    pub history: Option<Vec<PricePoint>>,
}

/// The price histories of several markets
/// (`components/schemas/BatchPricesHistoryResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BatchPricesHistory {
    /// Map of market asset id (token id) to its price points.
    pub history: Option<HashMap<TokenId, Vec<PricePoint>>>,
}

/// The request body of `POST /batch-prices-history`
/// (`components/schemas/BatchPricesHistoryRequest`).
#[derive(Debug, Serialize)]
struct BatchPricesHistoryRequest<'a> {
    markets: &'a [TokenId],
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::timestamp_seconds_option"
    )]
    start_ts: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_util::timestamp_seconds_option"
    )]
    end_ts: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<&'a PriceHistoryInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fidelity: Option<u32>,
}

impl ClobClient {
    /// Gets the price history of a market (`GET /prices-history`).
    ///
    /// `market` is the token id (asset id) to query.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-prices-history>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::clob::PriceHistoryInterval;
    ///
    /// let clob = polyoxide::clob::ClobClient::new()?;
    /// let history = clob
    ///     .get_prices_history(
    ///         "71321045679252212594626385532706912750332728571942532289631379312455583992563",
    ///     )
    ///     .interval(PriceHistoryInterval::OneDay)
    ///     .fidelity(60)
    ///     .send()
    ///     .await?;
    /// for point in history.history.unwrap_or_default() {
    ///     println!("{:?}: {:?}", point.timestamp, point.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_prices_history(&self, market: impl Into<TokenId>) -> GetPricesHistory {
        GetPricesHistory {
            client: self.clone(),
            market: market.into(),
            start_ts: None,
            end_ts: None,
            interval: None,
            fidelity: None,
        }
    }

    /// Gets the price histories of up to [`MAX_BATCH_PRICES_HISTORY_MARKETS`] markets
    /// (`POST /batch-prices-history`).
    ///
    /// `markets` are token ids (asset ids).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-batch-prices-history>.
    pub fn get_batch_prices_history(
        &self,
        markets: impl IntoIterator<Item = impl Into<TokenId>>,
    ) -> GetBatchPricesHistory {
        GetBatchPricesHistory {
            client: self.clone(),
            markets: markets.into_iter().map(Into::into).collect(),
            start_ts: None,
            end_ts: None,
            interval: None,
            fidelity: None,
        }
    }
}

/// Request builder for [`ClobClient::get_prices_history`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetPricesHistory {
    client: ClobClient,
    market: TokenId,
    start_ts: Option<DateTime<Utc>>,
    end_ts: Option<DateTime<Utc>>,
    interval: Option<PriceHistoryInterval>,
    fidelity: Option<u32>,
}

impl GetPricesHistory {
    /// Only points after this time (`startTs`, sent as Unix seconds).
    pub fn start_ts(mut self, start: DateTime<Utc>) -> Self {
        self.start_ts = Some(start);
        self
    }

    /// Only points before this time (`endTs`, sent as Unix seconds).
    pub fn end_ts(mut self, end: DateTime<Utc>) -> Self {
        self.end_ts = Some(end);
        self
    }

    /// Time interval for data aggregation.
    pub fn interval(mut self, interval: PriceHistoryInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Accuracy of the data in minutes (server default: 1 minute).
    pub fn fidelity(mut self, minutes: u32) -> Self {
        self.fidelity = Some(minutes);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error). Invalid parameters are an
    /// [`Error::Api`](crate::Error::Api) with status `400`.
    pub async fn send(self) -> Result<PricesHistory> {
        let mut query = Query::new();
        query
            .push("market", &self.market)
            .push_opt("startTs", self.start_ts.map(|t| t.timestamp()))
            .push_opt("endTs", self.end_ts.map(|t| t.timestamp()))
            .push_opt("interval", self.interval.as_ref())
            .push_opt("fidelity", self.fidelity);
        self.client
            .transport
            .get(&["prices-history"])
            .query(query)
            .send()
            .await
    }
}

/// Request builder for [`ClobClient::get_batch_prices_history`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetBatchPricesHistory {
    client: ClobClient,
    markets: Vec<TokenId>,
    start_ts: Option<DateTime<Utc>>,
    end_ts: Option<DateTime<Utc>>,
    interval: Option<PriceHistoryInterval>,
    fidelity: Option<u32>,
}

impl GetBatchPricesHistory {
    /// Only points after this time (`start_ts`, sent as Unix seconds).
    pub fn start_ts(mut self, start: DateTime<Utc>) -> Self {
        self.start_ts = Some(start);
        self
    }

    /// Only points before this time (`end_ts`, sent as Unix seconds).
    pub fn end_ts(mut self, end: DateTime<Utc>) -> Self {
        self.end_ts = Some(end);
        self
    }

    /// Time interval for data aggregation.
    pub fn interval(mut self, interval: PriceHistoryInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Accuracy of the data in minutes (server default: 1 minute).
    pub fn fidelity(mut self, minutes: u32) -> Self {
        self.fidelity = Some(minutes);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if there are more than
    /// [`MAX_BATCH_PRICES_HISTORY_MARKETS`] markets; otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<BatchPricesHistory> {
        require_at_most("markets", &self.markets, MAX_BATCH_PRICES_HISTORY_MARKETS)?;
        let body = BatchPricesHistoryRequest {
            markets: &self.markets,
            start_ts: self.start_ts,
            end_ts: self.end_ts,
            interval: self.interval.as_ref(),
            fidelity: self.fidelity,
        };
        self.client
            .transport
            .post(&["batch-prices-history"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    /// Field names and types from `components/schemas/PricesHistoryResponse` /
    /// `MarketPrice` in docs/specs/clob-openapi.yaml (the spec has no example).
    #[test]
    fn deserializes_prices_history() {
        let json = r#"{"history":[{"t":1700000000,"p":0.45},{"t":1700003600,"p":0.5}]}"#;
        let history: PricesHistory = serde_json::from_str(json).unwrap();
        let points = history.history.unwrap();
        assert_eq!(
            points[0].timestamp,
            Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap())
        );
        assert_eq!(points[1].price, Some("0.5".parse().unwrap()));

        let empty: PricesHistory = serde_json::from_str(r#"{"history":[]}"#).unwrap();
        assert_eq!(empty.history, Some(vec![]));
    }

    /// Field names and types from `components/schemas/BatchPricesHistoryResponse` in
    /// docs/specs/clob-openapi.yaml (the spec has no example).
    #[test]
    fn deserializes_batch_prices_history() {
        let json = r#"{"history":{"123":[{"t":1700000000,"p":0.45}],"456":[]}}"#;
        let batch: BatchPricesHistory = serde_json::from_str(json).unwrap();
        let history = batch.history.unwrap();
        assert_eq!(history[&TokenId::from("123")].len(), 1);
        assert!(history[&TokenId::from("456")].is_empty());
    }

    /// Field names from `components/schemas/BatchPricesHistoryRequest`.
    #[test]
    fn serializes_batch_request() {
        let markets = [TokenId::from("123"), TokenId::from("456")];
        let interval = PriceHistoryInterval::OneDay;
        let body = BatchPricesHistoryRequest {
            markets: &markets,
            start_ts: Some(Utc.timestamp_opt(1_700_000_000, 0).unwrap()),
            end_ts: None,
            interval: Some(&interval),
            fidelity: Some(60),
        };
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"{"markets":["123","456"],"start_ts":1700000000,"interval":"1d","fidelity":60}"#
        );
    }

    #[test]
    fn interval_wire_values() {
        assert_eq!(PriceHistoryInterval::OneMonth.as_str(), "1m");
        assert_eq!(
            serde_json::from_str::<PriceHistoryInterval>(r#""6h""#).unwrap(),
            PriceHistoryInterval::SixHours
        );
        assert!(
            serde_json::from_str::<PriceHistoryInterval>(r#""5m""#)
                .unwrap()
                .is_unknown()
        );
    }
}
