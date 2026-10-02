//! Price history: `GET /prices-history` and `POST /batch-prices-history`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use marcasite_core::{Query, Result, ValidationError, serde_util, types::TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    ClobClient,
    types::{require_at_most, require_id, require_list_values, require_non_empty},
};

/// Maximum number of markets per [`ClobClient::get_batch_prices_history`] request.
pub const MAX_BATCH_PRICES_HISTORY_MARKETS: usize = 20;

/// Minimum `fidelity` (in minutes) the server accepts with [`PriceHistoryInterval::OneMonth`].
pub const MIN_FIDELITY_ONE_MONTH: u32 = 10;

/// Minimum `fidelity` (in minutes) the server accepts with [`PriceHistoryInterval::OneWeek`].
pub const MIN_FIDELITY_ONE_WEEK: u32 = 5;

marcasite_core::string_enum! {
    /// Time interval for price-history aggregation (the `interval` parameter).
    ///
    /// The CLOB spec lists the values (`max`, `all`, `1m`, `1w`, `1d`, `6h`, `1h`) without
    /// describing them. Live checks (2026-10-02) show they are windows ending now:
    ///
    /// - `1h`, `6h`, `1d`, `1w` are one hour, six hours, one day and one week;
    /// - **`1m` is one month** (about 30 days), not one minute;
    /// - `max` and `all` are the same: the whole available history, up to a cap on the number
    ///   of points (about 4,300 observed), so a small `fidelity` only reaches back a few
    ///   weeks (30 days at 10 minutes) while `1440` covers more than a year.
    ///
    /// The server enforces a minimum `fidelity` per window: **10 minutes for `1m`**, 5 for
    /// `1w`, and answers `400` below it, including when `fidelity` is omitted (the default is
    /// 1 minute). The request builders check this client-side
    /// ([`MIN_FIDELITY_ONE_MONTH`], [`MIN_FIDELITY_ONE_WEEK`]). When `start_ts`/`end_ts` are
    /// sent as well, the window of the timestamps is used, but the fidelity minimum of the
    /// interval still applies. See `SPEC_DEVIATIONS.md`.
    pub enum PriceHistoryInterval {
        /// `max`.
        Max => "max",
        /// `all`.
        All => "all",
        /// `1m`: one month (needs `fidelity` of at least 10).
        OneMonth => "1m",
        /// `1w`: one week (needs `fidelity` of at least 5).
        OneWeek => "1w",
        /// `1d`: one day.
        OneDay => "1d",
        /// `6h`: six hours.
        SixHours => "6h",
        /// `1h`: one hour.
        OneHour => "1h",
    }
}

/// Checks the `fidelity` minimum the server enforces for `interval` (see
/// [`PriceHistoryInterval`]).
fn check_fidelity(interval: Option<&PriceHistoryInterval>, fidelity: Option<u32>) -> Result<()> {
    let minimum = match interval {
        Some(PriceHistoryInterval::OneMonth) => MIN_FIDELITY_ONE_MONTH,
        Some(PriceHistoryInterval::OneWeek) => MIN_FIDELITY_ONE_WEEK,
        _ => return Ok(()),
    };
    match fidelity {
        Some(fidelity) if fidelity >= minimum => Ok(()),
        _ => Err(ValidationError::new(
            "fidelity",
            format!(
                "interval {} needs a fidelity of at least {minimum} minutes",
                interval.map_or("", PriceHistoryInterval::as_str)
            ),
        )
        .into()),
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
    /// Price at that time (wire name `p`; a JSON number on the wire).
    #[serde(rename = "p", default, with = "serde_util::decimal_number_option")]
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
    /// `market` is the token id (asset id) to query. The batch form for up to
    /// [`MAX_BATCH_PRICES_HISTORY_MARKETS`] markets is
    /// [`ClobClient::get_batch_prices_history`].
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-prices-history>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use marcasite::clob::PriceHistoryInterval;
    ///
    /// let clob = marcasite::clob::ClobClient::new()?;
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
    /// `markets` are token ids (asset ids). The single-market form is
    /// [`ClobClient::get_prices_history`].
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-batch-prices-history>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use marcasite::clob::PriceHistoryInterval;
    ///
    /// let clob = marcasite::clob::ClobClient::new()?;
    /// let batch = clob
    ///     .get_batch_prices_history(["1", "2"])
    ///     .interval(PriceHistoryInterval::OneWeek)
    ///     .send()
    ///     .await?;
    /// for (token_id, points) in batch.history.unwrap_or_default() {
    ///     println!("{token_id}: {} points", points.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// This endpoint's docs say only "unix timestamp"; live confirms seconds (milliseconds
    /// are rejected as an interval that is too long). The server also appends one point at
    /// the current time even when it lies after `end_ts` (see `SPEC_DEVIATIONS.md`).
    pub fn start_ts(mut self, start: DateTime<Utc>) -> Self {
        self.start_ts = Some(start);
        self
    }

    /// Only points before this time (`endTs`, sent as Unix seconds; see
    /// [`start_ts`](Self::start_ts) on the unit).
    pub fn end_ts(mut self, end: DateTime<Utc>) -> Self {
        self.end_ts = Some(end);
        self
    }

    /// Time interval for data aggregation.
    pub fn interval(mut self, interval: PriceHistoryInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Accuracy of the data in minutes (server default: 1 minute). The `1m` and `1w`
    /// intervals need a minimum (see [`PriceHistoryInterval`]).
    pub fn fidelity(mut self, minutes: u32) -> Self {
        self.fidelity = Some(minutes);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `market` is empty, or (parameter
    /// `fidelity`) if the interval is `1m` or `1w` and the fidelity is missing or below the
    /// server's minimum (10 and 5 minutes). Missing or invalid parameters are an
    /// [`Error::Api`](crate::Error::Api) with status `400`. See [`Error`](crate::Error) for
    /// the other cases.
    pub async fn send(self) -> Result<PricesHistory> {
        require_id("market", self.market.as_str())?;
        check_fidelity(self.interval.as_ref(), self.fidelity)?;
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

    /// Accuracy of the data in minutes (server default: 1 minute). The `1m` and `1w`
    /// intervals need a minimum (see [`PriceHistoryInterval`]).
    pub fn fidelity(mut self, minutes: u32) -> Self {
        self.fidelity = Some(minutes);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) (parameter `markets`) if there are no
    /// markets (the field is required), more than [`MAX_BATCH_PRICES_HISTORY_MARKETS`], or
    /// an empty one; (parameter `fidelity`) if the interval is `1m` or `1w` and the fidelity
    /// is missing or below the server's minimum (10 and 5 minutes). Missing or invalid
    /// parameters are an [`Error::Api`](crate::Error::Api) with status `400`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn send(self) -> Result<BatchPricesHistory> {
        require_non_empty("markets", &self.markets)?;
        check_fidelity(self.interval.as_ref(), self.fidelity)?;
        require_at_most("markets", &self.markets, MAX_BATCH_PRICES_HISTORY_MARKETS)?;
        require_list_values("markets", self.markets.iter().map(TokenId::as_str))?;
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
    use crate::clob::types::test_util::round_trip;

    /// Field names and types from `components/schemas/PricesHistoryResponse` /
    /// `MarketPrice` in docs/specs/clob-openapi.yaml (the spec has no example).
    #[test]
    fn deserializes_prices_history() {
        let json = r#"{"history":[{"t":1700000000,"p":0.45},{"t":1700003600,"p":0.5}]}"#;
        let history: PricesHistory = round_trip(json);
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
        let batch: BatchPricesHistory = round_trip(json);
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
    fn fidelity_minimums_per_interval() {
        let is_fidelity_error = |result: Result<()>| matches!(&result, Err(marcasite_core::Error::Validation(v)) if v.parameter() == "fidelity");
        let month = PriceHistoryInterval::OneMonth;
        let week = PriceHistoryInterval::OneWeek;
        assert!(is_fidelity_error(check_fidelity(Some(&month), None)));
        assert!(is_fidelity_error(check_fidelity(Some(&month), Some(9))));
        assert!(check_fidelity(Some(&month), Some(10)).is_ok());
        assert!(is_fidelity_error(check_fidelity(Some(&week), Some(4))));
        assert!(check_fidelity(Some(&week), Some(5)).is_ok());
        // No minimum for the other intervals, or without an interval.
        assert!(check_fidelity(Some(&PriceHistoryInterval::OneDay), Some(1)).is_ok());
        assert!(check_fidelity(Some(&PriceHistoryInterval::Max), None).is_ok());
        assert!(check_fidelity(None, None).is_ok());
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
