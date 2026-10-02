//! Builder trades: `GET /builder/trades`.

use crate::Paginated;
use chrono::{DateTime, Utc};
use polyoxide_core::{
    Query, Result, ValidationError, serde_util,
    types::{Address, ConditionId, Side, TokenId},
    validate,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    ClobClient,
    types::{
        BuilderCode, OrderId, Page, TradeId, page_stream, require_id, timestamp_seconds_string,
    },
};

/// A trade attributed to a builder code (`components/schemas/BuilderTrade`).
///
/// Amounts (`size`, `size_usdc`, `fee`, `fee_usdc`, `builder_fee`) are decimal units, not
/// the micro-units the spec examples show (live check, 2026-10-02): a trade of 5 shares at
/// `0.01` has `size` `"5"` and `sizeUsdc` `"0.05"`. See `SPEC_DEVIATIONS.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderTrade {
    /// Trade id.
    pub id: TradeId,
    /// Trade type, e.g. `"TAKER"` or `"MAKER"` (the spec documents no fixed set of values).
    pub trade_type: String,
    /// Hash of the taker order.
    pub taker_order_hash: OrderId,
    /// The spec's builder-code field. Live it is always an empty string; the code is in
    /// [`builder_code`](Self::builder_code) instead.
    pub builder: BuilderCode,
    /// Market (condition id).
    pub market: ConditionId,
    /// Asset id (token id).
    pub asset_id: TokenId,
    /// Trade side.
    pub side: Side,
    /// Trade size in shares, as a decimal (e.g. `"5"`; the spec's example shows
    /// micro-units, live sends decimal units).
    pub size: Decimal,
    /// Trade size in USDC, as a decimal (e.g. `"0.05"`; the spec's example shows
    /// micro-units, live sends decimal units).
    pub size_usdc: Decimal,
    /// Trade price.
    pub price: Decimal,
    /// Trade status, e.g. `"TRADE_STATUS_CONFIRMED"` (the spec documents no fixed set of
    /// values).
    pub status: String,
    /// Market outcome label, e.g. `"Yes"` or `"Up"`.
    pub outcome: String,
    /// Outcome index.
    pub outcome_index: i64,
    /// Owner UUID.
    pub owner: String,
    /// Maker address.
    pub maker: Address,
    /// Transaction hash.
    pub transaction_hash: String,
    /// Match time (a Unix timestamp in seconds, sent as a numeric string, e.g.
    /// `"1700000000"`; serializes back to a string).
    #[serde(with = "timestamp_seconds_string")]
    pub match_time: DateTime<Utc>,
    /// Bucket index.
    pub bucket_index: i64,
    /// Fee amount, as a decimal (live sends decimal units; the spec's example shows
    /// micro-units).
    pub fee: Decimal,
    /// Fee amount in USDC, as a decimal (live sends decimal units; the spec's example shows
    /// micro-units).
    pub fee_usdc: Decimal,
    /// Fee amount attributed to the builder, as a decimal. Undocumented; observed live
    /// (wire name `builderFee`).
    pub builder_fee: Option<Decimal>,
    /// Builder code the trade is attributed to. Undocumented; observed live (wire name
    /// `builderCode`), where it carries the code that the documented
    /// [`builder`](Self::builder) field leaves empty.
    pub builder_code: Option<BuilderCode>,
    /// Error message, if any (wire name `err_msg`).
    #[serde(rename = "err_msg")]
    pub err_msg: Option<String>,
    /// Creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl ClobClient {
    /// Lists trades attributed to a builder code (`GET /builder/trades`, cursor pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/trade/get-builder-trades>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::chrono::{Duration, Utc};
    ///
    /// let clob = polyoxide::clob::ClobClient::new()?;
    /// let page = clob
    ///     .list_builder_trades("0x0000000000000000000000000000000000000000000000000000000000000001")
    ///     .after(Utc::now() - Duration::days(1))
    ///     .send()
    ///     .await?;
    /// for trade in page.items() {
    ///     println!("{} {} {} @ {}", trade.match_time, trade.side, trade.size, trade.price);
    /// }
    /// if let Some(cursor) = page.next_cursor() {
    ///     println!("more trades after cursor {cursor}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_builder_trades(&self, builder_code: impl Into<BuilderCode>) -> ListBuilderTrades {
        ListBuilderTrades {
            client: self.clone(),
            builder_code: builder_code.into(),
            id: None,
            market: None,
            asset_id: None,
            before: None,
            after: None,
            cursor: None,
        }
    }
}

/// Request builder for [`ClobClient::list_builder_trades`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBuilderTrades {
    client: ClobClient,
    builder_code: BuilderCode,
    id: Option<TradeId>,
    market: Option<ConditionId>,
    asset_id: Option<TokenId>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
    cursor: Option<String>,
}

/// Formats a filter timestamp as the documented `^\d+$` Unix timestamp.
fn unix_seconds(parameter: &'static str, time: DateTime<Utc>) -> Result<u64> {
    u64::try_from(time.timestamp()).map_err(|_| {
        ValidationError::new(parameter, "must not be before 1970-01-01T00:00:00Z").into()
    })
}

impl ListBuilderTrades {
    /// Only the trade with this id.
    pub fn id(mut self, id: impl Into<TradeId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Only trades in this market (condition id).
    pub fn market(mut self, market: impl Into<ConditionId>) -> Self {
        self.market = Some(market.into());
        self
    }

    /// Only trades of this asset id (token id).
    pub fn asset_id(mut self, asset_id: impl Into<TokenId>) -> Self {
        self.asset_id = Some(asset_id.into());
        self
    }

    /// Only trades before this time (sent as Unix seconds).
    pub fn before(mut self, before: DateTime<Utc>) -> Self {
        self.before = Some(before);
        self
    }

    /// Only trades after this time (sent as Unix seconds).
    pub fn after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }

    /// Cursor of the page to fetch (the `next_cursor` query parameter), from a previous
    /// page's [`next_cursor()`](Page::next_cursor()). Omit for the first page.
    ///
    /// [`END_CURSOR`](super::END_CURSOR) (or an empty cursor) means there are no more
    /// pages: [`into_stream`](Self::into_stream) then yields nothing, while
    /// [`send`](Self::send) still sends it as given.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    async fn fetch(self, cursor: Option<String>) -> Result<Page<BuilderTrade>> {
        validate::bytes32("builder_code", self.builder_code.as_str())?;
        if let Some(market) = &self.market {
            validate::bytes32("market", market.as_str())?;
        }
        if let Some(id) = &self.id {
            require_id("id", id.as_str())?;
        }
        if let Some(asset_id) = &self.asset_id {
            require_id("asset_id", asset_id.as_str())?;
        }
        let before = self
            .before
            .map(|time| unix_seconds("before", time))
            .transpose()?;
        let after = self
            .after
            .map(|time| unix_seconds("after", time))
            .transpose()?;
        let mut query = Query::new();
        query
            .push("builder_code", &self.builder_code)
            .push_opt("id", self.id.as_ref())
            .push_opt("market", self.market.as_ref())
            .push_opt("asset_id", self.asset_id.as_ref())
            .push_opt("before", before)
            .push_opt("after", after)
            .push_opt("next_cursor", cursor);
        self.client
            .transport
            .get(&["builder", "trades"])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if the builder code or market does not
    /// match the documented pattern (`0x` followed by 64 hex characters), the trade id or
    /// asset id is set but empty, or a time filter is before the Unix epoch. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn send(self) -> Result<Page<BuilderTrade>> {
        let cursor = self.cursor.clone();
        self.fetch(cursor).await
    }

    /// Streams every trade from the configured cursor onwards, fetching pages lazily until
    /// the last page (`next_cursor` `"LTE="`). The stream ends after yielding the first
    /// error (e.g. a validation error, before any request is sent).
    pub fn into_stream(self) -> Paginated<BuilderTrade> {
        let start = self.cursor.clone();
        page_stream(self, start, Self::fetch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clob::types::test_util::round_trip;

    /// Trimmed live response of `GET /builder/trades?builder_code=0x00..01` (captured
    /// 2026-10-02, one of 300 items): decimal units, empty `builder`, `builderCode` and
    /// `builderFee` present, `outcome` `"Up"`.
    const EXAMPLE: &str = r#"{
        "limit": 300,
        "next_cursor": "MzAw",
        "count": 1,
        "data": [{
            "id": "01a0d664-92cd-7a93-91ad-cac08aeef86d",
            "tradeType": "MAKER",
            "takerOrderHash": "0x3e354a3f471f454d6532caa1a8ac447d771af720bf14b330d541881517b7e31e",
            "builder": "",
            "market": "0x481fafed1cba1c490af3ac2b69ce2ca15710ff5e8b1b63f486b48471677b3ee1",
            "assetId": "14819207982932366142943017817551212408227964397023321046800597001588471048639",
            "side": "BUY",
            "size": "5",
            "sizeUsdc": "0.05",
            "price": "0.01",
            "status": "TRADE_STATUS_CONFIRMED",
            "outcome": "Up",
            "outcomeIndex": 0,
            "owner": "ed03f028-74b2-3f73-542a-e78b73279d5b",
            "maker": "0xBF119Cf0f6ff285688857487E669Adad35cF2CE1",
            "transactionHash": "0x664500e429d4b54db4f1441f55ebc584d82a1be0c450bd009cef3043af8e9a6d",
            "matchTime": "1790303310",
            "bucketIndex": 0,
            "fee": "0",
            "feeUsdc": "0",
            "builderFee": "0",
            "builderCode": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "createdAt": "2026-09-25T02:28:30.542213Z",
            "updatedAt": "2026-09-25T02:28:38.460921Z"
        }]
    }"#;

    /// Example response of `GET /builder/trades` in docs/specs/clob-openapi.yaml
    /// (docs/api-reference/trade/get-builder-trades.md): no `builderCode` / `builderFee`,
    /// micro-unit amounts.
    const DOCUMENTED_EXAMPLE: &str = r#"{
        "limit": 300,
        "next_cursor": "MzAw",
        "count": 1,
        "data": [{
            "id": "trade-123",
            "tradeType": "TAKER",
            "takerOrderHash": "0xabcdef1234567890abcdef1234567890abcdef12",
            "builder": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "assetId": "15871154585880608648532107628464183779895785213830018178010423617714102767076",
            "side": "BUY",
            "size": "100000000",
            "sizeUsdc": "50000000",
            "price": "0.5",
            "status": "TRADE_STATUS_CONFIRMED",
            "outcome": "YES",
            "outcomeIndex": 0,
            "owner": "f4f247b7-4ac7-ff29-a152-04fda0a8755a",
            "maker": "0x1234567890123456789012345678901234567890",
            "transactionHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "matchTime": "1700000000",
            "bucketIndex": 0,
            "fee": "300000",
            "feeUsdc": "150000",
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-01T00:00:00Z"
        }]
    }"#;

    #[test]
    fn deserializes_builder_trades_page() {
        let page: Page<BuilderTrade> = round_trip(EXAMPLE);
        assert_eq!(page.limit, 300);
        assert_eq!(page.next_cursor(), Some("MzAw"));
        let trade = &page.data[0];
        assert_eq!(trade.id, "01a0d664-92cd-7a93-91ad-cac08aeef86d");
        assert_eq!(trade.trade_type, "MAKER");
        assert_eq!(trade.builder, "");
        assert_eq!(
            trade.builder_code.as_ref().map(BuilderCode::as_str),
            Some("0x0000000000000000000000000000000000000000000000000000000000000001")
        );
        assert_eq!(trade.builder_fee, Some(Decimal::ZERO));
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.size, "5".parse::<Decimal>().unwrap());
        assert_eq!(trade.size_usdc, "0.05".parse::<Decimal>().unwrap());
        assert_eq!(trade.price, "0.01".parse::<Decimal>().unwrap());
        assert_eq!(trade.outcome, "Up");
        assert_eq!(trade.match_time.timestamp(), 1_790_303_310);
        assert_eq!(trade.err_msg, None);
        assert_eq!(trade.created_at.map(|t| t.timestamp()), Some(1_790_303_310));
    }

    /// The documented example (no `builderCode` / `builderFee`) still decodes.
    #[test]
    fn deserializes_documented_example() {
        let page: Page<BuilderTrade> = round_trip(DOCUMENTED_EXAMPLE);
        let trade = &page.data[0];
        assert_eq!(trade.id, "trade-123");
        assert_eq!(trade.builder_code, None);
        assert_eq!(trade.builder_fee, None);
        assert_eq!(trade.size, "100000000".parse::<Decimal>().unwrap());
        assert_eq!(trade.created_at.map(|t| t.timestamp()), Some(1_704_067_200));
    }

    /// A side outside the documented `BUY`/`SELL` is kept, not rejected.
    #[test]
    fn unknown_side_is_kept() {
        let mut value: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        value["data"][0]["side"] = "HOLD".into();
        let page: Page<BuilderTrade> = serde_json::from_value(value).unwrap();
        assert_eq!(page.data[0].side, Side::Unknown("HOLD".to_owned()));
        assert!(page.data[0].side.is_unknown());
    }

    #[test]
    fn err_msg_uses_snake_case() {
        let mut value: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        value["data"][0]["err_msg"] = "boom".into();
        let page: Page<BuilderTrade> = serde_json::from_value(value).unwrap();
        assert_eq!(page.data[0].err_msg.as_deref(), Some("boom"));
    }

    #[test]
    fn required_fields_are_enforced() {
        let mut value: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        value["data"][0]
            .as_object_mut()
            .unwrap()
            .remove("matchTime");
        assert!(serde_json::from_value::<Page<BuilderTrade>>(value).is_err());
    }
}
