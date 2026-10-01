//! Feeds: `/v2/trades`, `/v2/activity`, `/v2/activity/combos`.

use chrono::{DateTime, Utc};
use futures_core::Stream;
use polyoxide_core::{Query, Result, ValidationError, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DataClient,
    types::{
        ComboLeg, FilterType, Page, SortDirection, check_limit, check_list, collect_ids,
        page_stream,
    },
};
use crate::types::{Address, ConditionId, EventId, Side, TokenId};

const TRADES: &[&str] = &["v2", "trades"];
const ACTIVITY: &[&str] = &["v2", "activity"];
const COMBO_ACTIVITY: &[&str] = &["v2", "activity", "combos"];

/// Maximum `limit` of the feeds.
const MAX_FEED_LIMIT: u32 = 1000;

/// A trade (`components/schemas/Trade`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Trade {
    /// Proxy wallet the row belongs to.
    pub proxy_wallet: Address,
    /// `BUY` or `SELL`, from this wallet's perspective.
    pub side: Side,
    /// CLOB asset id of the traded outcome token.
    pub token_id: TokenId,
    /// On-chain condition id of the market.
    pub condition_id: ConditionId,
    /// Filled quantity, in shares.
    pub size: Decimal,
    /// Execution price per share, in USDC.
    pub price: Decimal,
    /// Block timestamp of the fill.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Market question title (empty when unenriched).
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event slug.
    pub event_slug: String,
    /// Label of the traded outcome (e.g. `Yes`).
    pub outcome: String,
    /// Index of the traded outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](super::UNLABELED_OUTCOME_INDEX) means it could not be
    /// labeled.
    pub outcome_index: i32,
    /// Profile display name of the wallet.
    pub name: String,
    /// Generated fallback handle for profiles without a display name.
    pub pseudonym: String,
    /// Profile bio text.
    pub bio: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Resized profile image URL, when one exists.
    pub profile_image_optimized: String,
    /// Hash of the settling transaction.
    pub transaction_hash: String,
}

polyoxide_core::string_enum! {
    /// Kind of an activity-feed event (`type`).
    pub enum ActivityType {
        /// A trade.
        Trade => "TRADE",
        /// A split of collateral into outcome tokens.
        Split => "SPLIT",
        /// A merge of outcome tokens back into collateral.
        Merge => "MERGE",
        /// A redemption.
        Redeem => "REDEEM",
        /// A reward.
        Reward => "REWARD",
        /// A conversion.
        Conversion => "CONVERSION",
        /// A user-to-user transfer that is not a trade-settlement leg. Opt-in: only
        /// returned when requested through [`ListActivity::types`].
        Tip => "TIP",
    }
}

polyoxide_core::string_enum! {
    /// Direction of an activity-feed event (`side`).
    pub enum ActivitySide {
        /// A buy (trade rows).
        Buy => "BUY",
        /// A sell (trade rows).
        Sell => "SELL",
        /// Received (tip rows).
        In => "IN",
        /// Sent (tip rows).
        Out => "OUT",
    }
}

polyoxide_core::string_enum! {
    /// Sort key of `/v2/activity` (`sort_by`). Only [`Timestamp`](Self::Timestamp) is
    /// supported (the feed pages by keyset).
    pub enum ActivitySortBy {
        /// `(block_timestamp, sequence_id)` order.
        Timestamp => "TIMESTAMP",
    }
}

/// One activity-feed event: a trade, split, merge, redeem, ... (`components/schemas/Activity`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Activity {
    /// Proxy wallet the row belongs to.
    pub proxy_wallet: Address,
    /// Block timestamp of the action.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub timestamp: DateTime<Utc>,
    /// On-chain condition id of the market.
    pub condition_id: ConditionId,
    /// Kind of event (`type`).
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// Share quantity of the action (for tips, the amount transferred).
    pub size: Decimal,
    /// Cash value of the action, in USDC.
    pub usdc_size: Decimal,
    /// Hash of the settling transaction.
    pub transaction_hash: String,
    /// Price per share in USDC (trades; `0` where no price applies).
    pub price: Decimal,
    /// CLOB asset id of the outcome token the action touched.
    pub token_id: TokenId,
    /// Direction: `BUY`/`SELL` on trade rows, `IN`/`OUT` on tips; `None` where a side does
    /// not apply (served as `""`).
    #[serde(deserialize_with = "serde_util::empty_string_as_none")]
    pub side: Option<ActivitySide>,
    /// Index of the outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](super::UNLABELED_OUTCOME_INDEX) means it could not be
    /// labeled.
    pub outcome_index: i32,
    /// Market question title (empty when unenriched).
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event slug.
    pub event_slug: String,
    /// Label of the outcome (e.g. `Yes`).
    pub outcome: String,
    /// Profile display name of the wallet.
    pub name: String,
    /// Generated fallback handle for profiles without a display name.
    pub pseudonym: String,
    /// Profile bio text.
    pub bio: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Resized profile image URL, when one exists.
    pub profile_image_optimized: String,
    /// Set on combo trade rows; omitted (`None`) on non-combo rows. Combo detail lives on
    /// [`DataClient::list_combo_activity`].
    pub is_combo: Option<bool>,
}

polyoxide_core::string_enum! {
    /// Kind of a combo lifecycle/redemption event (`type`).
    pub enum ComboActivityType {
        /// A split.
        Split => "SPLIT",
        /// A merge.
        Merge => "MERGE",
        /// A conversion.
        Convert => "CONVERT",
        /// A compression.
        Compress => "COMPRESS",
        /// A wrap.
        Wrap => "WRAP",
        /// An unwrap.
        Unwrap => "UNWRAP",
        /// A redemption.
        Redeem => "REDEEM",
    }
}

/// One combo lifecycle/redemption event (`components/schemas/ComboActivity`).
///
/// Ordered by on-chain position `(block_number, log_index)`; `timestamp` is the event's
/// wall-clock time, not the ordering key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboActivity {
    /// `tx_hash-log_index`.
    pub id: String,
    /// Canonical action verb (`type`).
    #[serde(rename = "type")]
    pub activity_type: ComboActivityType,
    /// Proxy wallet the action belongs to.
    pub proxy_wallet: Address,
    /// On-chain combo condition id (`0x03`-prefixed).
    pub combo_condition_id: ConditionId,
    /// Token id of the combo position the action touched.
    pub combo_position_id: TokenId,
    /// Block number of the action.
    pub block_number: i64,
    /// Event time.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Hash of the settling transaction.
    pub transaction_hash: String,
    /// The combo's legs, in leg order.
    pub legs: Vec<ComboLeg>,
    /// Cash amount of the action, in USDC; `None` where no cash leg applies.
    pub amount_usdc: Option<Decimal>,
    /// Redemption payout in USDC on `REDEEM` rows; `None` otherwise.
    pub payout_usdc: Option<Decimal>,
}

impl DataClient {
    /// Lists trades for a wallet, markets, events, or the global feed
    /// (`GET /v2/trades`, cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/feeds/list-trades>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::{StreamExt as _, TryStreamExt as _};
    /// use polyoxide::{data::DataClient, types::Side};
    ///
    /// let data = DataClient::new()?;
    /// // The 500 most recent buys in one market, walking pages lazily.
    /// let trades: Vec<_> = data
    ///     .list_trades()
    ///     .conditions(["0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79"])
    ///     .side(Side::Buy)
    ///     .limit(250)
    ///     .into_stream()
    ///     .take(500)
    ///     .try_collect()
    ///     .await?;
    /// # let _ = trades;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_trades(&self) -> ListTrades {
        ListTrades {
            client: self.clone(),
            user: None,
            limit: None,
            cursor: None,
            taker_only: None,
            filter_type: None,
            filter_amount: None,
            start: None,
            end: None,
            conditions: Vec::new(),
            event_ids: Vec::new(),
            side: None,
        }
    }

    /// Lists a user's account activity: trades, splits, merges, redeems, ...
    /// (`GET /v2/activity`, cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/feeds/list-account-activity>.
    pub fn list_activity(&self, user: impl Into<Address>) -> ListActivity {
        ListActivity {
            client: self.clone(),
            user: user.into(),
            limit: None,
            cursor: None,
            types: Vec::new(),
            conditions: Vec::new(),
            event_ids: Vec::new(),
            side: None,
            start: None,
            end: None,
            sort_by: None,
            sort_direction: None,
            exclude_deposits_withdrawals: None,
        }
    }

    /// Lists a user's combo lifecycle and redemption events
    /// (`GET /v2/activity/combos`, cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/feeds/list-combo-activity>.
    pub fn list_combo_activity(&self, user: impl Into<Address>) -> ListComboActivity {
        ListComboActivity {
            client: self.clone(),
            user: user.into(),
            limit: None,
            cursor: None,
            conditions: Vec::new(),
        }
    }
}

/// Request builder for [`DataClient::list_trades`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTrades {
    client: DataClient,
    user: Option<Address>,
    limit: Option<u32>,
    cursor: Option<String>,
    taker_only: Option<bool>,
    filter_type: Option<FilterType>,
    filter_amount: Option<Decimal>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    conditions: Vec<ConditionId>,
    event_ids: Vec<EventId>,
    side: Option<Side>,
}

impl ListTrades {
    /// Only this wallet's trades (`user`); omit for the market/event/global feed.
    pub fn user(mut self, user: impl Into<Address>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
    /// supplied.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`).
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// `true` (default): each fill once, on its taker side; `false`: maker rows too
    /// (`taker_only`).
    pub fn taker_only(mut self, taker_only: bool) -> Self {
        self.taker_only = Some(taker_only);
        self
    }

    /// Unit of [`filter_amount`](Self::filter_amount) (`filter_type`, default
    /// [`FilterType::Tokens`]).
    pub fn filter_type(mut self, filter_type: FilterType) -> Self {
        self.filter_type = Some(filter_type);
        self
    }

    /// Minimum trade size (`filter_amount`, default `0.01`; `0` means the same).
    pub fn filter_amount(mut self, filter_amount: Decimal) -> Self {
        self.filter_amount = Some(filter_amount);
        self
    }

    /// Inclusive window start on the block timestamp (`start`). Honoured with
    /// [`user`](Self::user) only: omitted floors to three years back, and the Unix
    /// timestamp `1` asks for full history.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Inclusive window end (`end`). Honoured with [`user`](Self::user) only; omitted
    /// means now plus one day.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Condition ids (`condition`, at most 20 distinct values).
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
        self
    }

    /// Gamma event ids (`event_id`, at most 20 distinct values).
    pub fn event_ids<I>(mut self, event_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<EventId>,
    {
        self.event_ids = collect_ids(event_ids);
        self
    }

    /// Only buys or only sells (`side`).
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_FEED_LIMIT)?;
        check_list("condition", &self.conditions)?;
        check_list("event_id", &self.event_ids)?;
        let mut q = Query::new();
        q.push_opt("user", self.user.as_ref())
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_opt("taker_only", self.taker_only)
            .push_opt("filter_type", self.filter_type.as_ref())
            .push_opt("filter_amount", self.filter_amount)
            .push_opt("start", self.start.map(|t| t.timestamp()))
            .push_opt("end", self.end.map(|t| t.timestamp()))
            .push_csv("condition", &self.conditions)
            .push_csv("event_id", &self.event_ids)
            .push_opt("side", self.side.as_ref());
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000 or
    /// more than 20 condition or event ids are given; otherwise see
    /// [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<Trade>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(TRADES, query).await
    }

    /// Streams every trade from the configured cursor onwards, fetching pages lazily.
    ///
    /// The cursor carries only the seek anchor and page size, so every page re-sends the
    /// same filters (changing one mid-walk would silently re-anchor the feed). The stream
    /// ends when `next_cursor` is `null`.
    pub fn into_stream(self) -> impl Stream<Item = Result<Trade>> + Send + 'static {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, TRADES, start, move |cursor| self.query(cursor))
    }
}

/// Request builder for [`DataClient::list_activity`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListActivity {
    client: DataClient,
    user: Address,
    limit: Option<u32>,
    cursor: Option<String>,
    types: Vec<ActivityType>,
    conditions: Vec<ConditionId>,
    event_ids: Vec<EventId>,
    side: Option<Side>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    sort_by: Option<ActivitySortBy>,
    sort_direction: Option<SortDirection>,
    exclude_deposits_withdrawals: Option<bool>,
}

impl ListActivity {
    /// Page size (`limit`, default 100, at most 1000).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`). The
    /// cursor binds the sort direction it was minted under.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Activity types (`type`, comma-separated). [`ActivityType::Tip`] is never in the
    /// default set and is only returned when named here.
    pub fn types(mut self, types: impl IntoIterator<Item = ActivityType>) -> Self {
        self.types = types.into_iter().collect();
        self
    }

    /// Condition ids (`condition`, at most 20 distinct values). Mutually exclusive with
    /// [`event_ids`](Self::event_ids).
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
        self
    }

    /// Gamma event ids, resolved to the events' markets (`event_id`, at most 20 distinct
    /// values). Mutually exclusive with [`conditions`](Self::conditions).
    pub fn event_ids<I>(mut self, event_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<EventId>,
    {
        self.event_ids = collect_ids(event_ids);
        self
    }

    /// Only buys or only sells (`side`).
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    /// Inclusive window start on the block timestamp (`start`). Omitted floors to three
    /// years back; the Unix timestamp `1` asks for full history.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Inclusive window end (`end`); omitted means now plus one day.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Sort key (`sort_by`).
    pub fn sort_by(mut self, sort_by: ActivitySortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// Sort direction (`sort_direction`, default [`SortDirection::Desc`]).
    pub fn sort_direction(mut self, sort_direction: SortDirection) -> Self {
        self.sort_direction = Some(sort_direction);
        self
    }

    /// Exclude deposits and withdrawals (`exclude_deposits_withdrawals`, default `true`).
    pub fn exclude_deposits_withdrawals(mut self, exclude: bool) -> Self {
        self.exclude_deposits_withdrawals = Some(exclude);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_FEED_LIMIT)?;
        check_list("condition", &self.conditions)?;
        check_list("event_id", &self.event_ids)?;
        if !self.conditions.is_empty() && !self.event_ids.is_empty() {
            return Err(ValidationError::new(
                "event_id",
                "`event_id` and `condition` are mutually exclusive",
            )
            .into());
        }
        let mut q = Query::new();
        q.push("user", &self.user)
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_csv("type", &self.types)
            .push_csv("condition", &self.conditions)
            .push_csv("event_id", &self.event_ids)
            .push_opt("side", self.side.as_ref())
            .push_opt("start", self.start.map(|t| t.timestamp()))
            .push_opt("end", self.end.map(|t| t.timestamp()))
            .push_opt("sort_by", self.sort_by.as_ref())
            .push_opt("sort_direction", self.sort_direction.as_ref())
            .push_opt(
                "exclude_deposits_withdrawals",
                self.exclude_deposits_withdrawals,
            );
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000,
    /// more than 20 condition or event ids are given, or both are given; otherwise see
    /// [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<Activity>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(ACTIVITY, query).await
    }

    /// Streams every activity event from the configured cursor onwards, fetching pages
    /// lazily.
    ///
    /// The cursor carries only the seek anchor, page size and sort direction, so every
    /// page re-sends the same filters (changing one mid-walk would silently re-anchor the
    /// feed). The stream ends when `next_cursor` is `null`.
    pub fn into_stream(self) -> impl Stream<Item = Result<Activity>> + Send + 'static {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, ACTIVITY, start, move |cursor| self.query(cursor))
    }
}

/// Request builder for [`DataClient::list_combo_activity`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboActivity {
    client: DataClient,
    user: Address,
    limit: Option<u32>,
    cursor: Option<String>,
    conditions: Vec<ConditionId>,
}

impl ListComboActivity {
    /// First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
    /// supplied.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`).
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Combo condition ids (`condition`, at most 20 distinct values).
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_FEED_LIMIT)?;
        check_list("condition", &self.conditions)?;
        let mut q = Query::new();
        q.push("user", &self.user)
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_csv("condition", &self.conditions);
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000 or
    /// more than 20 condition ids are given; otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<ComboActivity>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(COMBO_ACTIVITY, query).await
    }

    /// Streams every combo activity event from the configured cursor onwards, fetching
    /// pages lazily.
    ///
    /// Every page re-sends `user` and the same filters with the cursor (the feed rule:
    /// changing a filter mid-walk re-anchors it). The stream ends when `next_cursor` is
    /// `null`.
    pub fn into_stream(self) -> impl Stream<Item = Result<ComboActivity>> + Send + 'static {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, COMBO_ACTIVITY, start, move |cursor| {
            self.query(cursor)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Field names and types from `components/schemas/Trade` in
    /// `docs/specs/data-v2-openapi.json`.
    #[test]
    fn deserializes_trade() {
        let json = r#"{
            "proxy_wallet": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
            "side": "BUY",
            "token_id": "31974447302330162086995746309500877260929998201718217388109724292047967921664",
            "condition_id": "0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79",
            "size": 100,
            "price": 0.5203,
            "timestamp": 1787133600,
            "title": "Will ŠK Slovan Bratislava win on 2026-08-19?",
            "slug": "s",
            "icon": "",
            "event_slug": "e",
            "outcome": "Yes",
            "outcome_index": 0,
            "name": "",
            "pseudonym": "Calm-Owl",
            "bio": "",
            "profile_image": "",
            "profile_image_optimized": "",
            "transaction_hash": "0xfeed"
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.size, Decimal::from(100));
        assert_eq!(trade.price.to_string(), "0.5203");
        assert_eq!(trade.timestamp.timestamp(), 1_787_133_600);
    }

    /// Field names and types from `components/schemas/Activity`; `side` is empty where a
    /// side does not apply and `is_combo` is omitted from non-combo rows.
    #[test]
    fn deserializes_activity() {
        let json = r#"{
            "proxy_wallet": "0xabc",
            "timestamp": 1787133600,
            "condition_id": "0xdef",
            "type": "REDEEM",
            "size": 10,
            "usdc_size": 10,
            "transaction_hash": "0xfeed",
            "price": 0,
            "token_id": "1",
            "side": "",
            "outcome_index": 999,
            "title": "",
            "slug": "",
            "icon": "",
            "event_slug": "",
            "outcome": "",
            "name": "",
            "pseudonym": "",
            "bio": "",
            "profile_image": "",
            "profile_image_optimized": ""
        }"#;
        let activity: Activity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, ActivityType::Redeem);
        assert_eq!(activity.side, None);
        assert_eq!(activity.is_combo, None);
        assert_eq!(activity.outcome_index, crate::data::UNLABELED_OUTCOME_INDEX);

        let tip = json
            .replace(r#""type": "REDEEM""#, r#""type": "TIP""#)
            .replace(r#""side": """#, r#""side": "OUT""#);
        let tip: Activity = serde_json::from_str(&tip).unwrap();
        assert_eq!(tip.activity_type, ActivityType::Tip);
        assert_eq!(tip.side, Some(ActivitySide::Out));

        // `side` is required by the schema (even when empty).
        let missing = json.replace(r#""side": "","#, "");
        assert!(serde_json::from_str::<Activity>(&missing).is_err());

        let unknown = json.replace(r#""type": "REDEEM""#, r#""type": "YIELD""#);
        let unknown: Activity = serde_json::from_str(&unknown).unwrap();
        assert_eq!(
            unknown.activity_type,
            ActivityType::Unknown("YIELD".to_owned())
        );
    }

    /// Field names and types from `components/schemas/ComboActivity`.
    #[test]
    fn deserializes_combo_activity() {
        let json = r#"{
            "id": "0xfeed-3",
            "type": "REDEEM",
            "proxy_wallet": "0xabc",
            "combo_condition_id": "0x03aa",
            "combo_position_id": "123",
            "block_number": 75000000,
            "timestamp": 1787133600,
            "transaction_hash": "0xfeed",
            "legs": [],
            "amount_usdc": null,
            "payout_usdc": 12.5
        }"#;
        let row: ComboActivity = serde_json::from_str(json).unwrap();
        assert_eq!(row.activity_type, ComboActivityType::Redeem);
        assert_eq!(row.amount_usdc, None);
        assert_eq!(row.payout_usdc, Some(Decimal::new(125, 1)));
    }

    #[test]
    fn activity_query() {
        let client = DataClient::new().unwrap();
        assert!(
            client
                .list_activity("0xabc")
                .conditions(["0x1"])
                .event_ids(["1"])
                .query(None)
                .is_err()
        );
        let q = client
            .list_activity("0xabc")
            .types([ActivityType::Trade, ActivityType::Tip])
            .sort_direction(SortDirection::Asc)
            .query(Some("c"))
            .unwrap();
        assert_eq!(
            q.to_string(),
            "user=0xabc&cursor=c&type=TRADE%2CTIP&sort_direction=ASC"
        );
    }
}
