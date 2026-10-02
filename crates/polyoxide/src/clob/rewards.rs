//! Liquidity rewards configurations (public endpoints): `/rewards/markets/current`,
//! `/rewards/markets/{condition_id}` and `/rewards/markets/multi`.

use crate::Paginated;
use chrono::{DateTime, NaiveDate, Utc};
use polyoxide_core::{
    Query, Result, ValidationError, serde_util,
    types::{Address, ConditionId, EventId, MarketId, TokenId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    ClobClient,
    types::{Page, page_stream, require_id},
};

/// Maximum `page_size` of [`ClobClient::list_markets_with_rewards`].
pub const MAX_REWARDS_MARKETS_PAGE_SIZE: u32 = 500;

polyoxide_core::string_enum! {
    /// Sort field of [`ClobClient::list_markets_with_rewards`] (the `order_by` parameter).
    pub enum RewardsMarketsOrderBy {
        /// `market_id`.
        MarketId => "market_id",
        /// `created_at`.
        CreatedAt => "created_at",
        /// `volume_24hr`.
        Volume24hr => "volume_24hr",
        /// `spread`.
        Spread => "spread",
        /// `competitiveness`.
        Competitiveness => "competitiveness",
        /// `max_spread`.
        MaxSpread => "max_spread",
        /// `min_size`.
        MinSize => "min_size",
        /// `question`.
        Question => "question",
        /// `one_day_price_change`.
        OneDayPriceChange => "one_day_price_change",
        /// `rate_per_day`.
        RatePerDay => "rate_per_day",
        /// `price`.
        Price => "price",
        /// `end_date`.
        EndDate => "end_date",
        /// `start_date`.
        StartDate => "start_date",
        /// `reward_end_date`.
        RewardEndDate => "reward_end_date",
    }
}

polyoxide_core::string_enum! {
    /// Sort direction (the `position` parameter).
    pub enum SortDirection {
        /// Ascending (`ASC`).
        Asc => "ASC",
        /// Descending (`DESC`).
        Desc => "DESC",
    }
}

/// A reward configuration of a market in the current-rewards listing
/// (`components/schemas/CurrentRewardConfig`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CurrentRewardConfig {
    /// Rewards config id (always `0` on this endpoint).
    pub id: Option<i64>,
    /// Address of the reward asset.
    pub asset_address: Address,
    /// Start date of the rewards period.
    pub start_date: NaiveDate,
    /// End date of the rewards period.
    pub end_date: Option<NaiveDate>,
    /// Daily reward rate (a JSON number on the wire).
    #[serde(with = "serde_util::decimal_number")]
    pub rate_per_day: Decimal,
    /// Total rewards amount (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_rewards: Option<Decimal>,
}

/// The current reward configuration of a market (`components/schemas/CurrentReward`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CurrentReward {
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Maximum spread for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_min_size: Option<Decimal>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<CurrentRewardConfig>>,
    /// Sponsored daily rate (a JSON number on the wire; omitted when zero).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub sponsored_daily_rate: Option<Decimal>,
    /// Number of sponsors (omitted when zero).
    pub sponsors_count: Option<u64>,
    /// Native daily rate, excluding sponsors (a JSON number on the wire; omitted when zero).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub native_daily_rate: Option<Decimal>,
    /// Total daily rate, including sponsors (a JSON number on the wire; omitted when zero).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_daily_rate: Option<Decimal>,
}

/// A token of a rewards market (`components/schemas/RewardsToken`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RewardsToken {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Outcome name (e.g. `"YES"`).
    pub outcome: String,
    /// Current price of the token (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub price: Option<Decimal>,
}

/// A reward configuration of a market (`components/schemas/RewardsConfig`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RewardsConfig {
    /// Rewards config id.
    pub id: Option<i64>,
    /// Address of the reward asset.
    pub asset_address: Address,
    /// Start date of the rewards period.
    pub start_date: NaiveDate,
    /// End date of the rewards period.
    pub end_date: Option<NaiveDate>,
    /// Daily reward rate (a JSON number on the wire).
    #[serde(with = "serde_util::decimal_number")]
    pub rate_per_day: Decimal,
    /// Total rewards amount (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_rewards: Option<Decimal>,
    /// Remaining reward amount (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub remaining_reward_amount: Option<Decimal>,
    /// Total number of days in the rewards period.
    pub total_days: Option<i64>,
}

/// A market with its raw reward configurations (`components/schemas/MarketReward`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketReward {
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// The market question.
    pub question: String,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// URL slug of the event.
    pub event_slug: Option<String>,
    /// Market image URL.
    pub image: Option<String>,
    /// Maximum spread for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_min_size: Option<Decimal>,
    /// Competitiveness score of the market (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub market_competitiveness: Option<Decimal>,
    /// Outcome tokens.
    pub tokens: Vec<RewardsToken>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<RewardsConfig>>,
}

/// A market with its reward configurations and trading metrics
/// (`components/schemas/MultiMarketInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MultiMarketInfo {
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Event id.
    pub event_id: Option<EventId>,
    /// URL slug of the event.
    pub event_slug: Option<String>,
    /// Market creation time.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// Title within an event group.
    pub group_item_title: Option<String>,
    /// Market image URL.
    pub image: Option<String>,
    /// Competitiveness score of the market (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub market_competitiveness: Option<Decimal>,
    /// Market id.
    pub market_id: MarketId,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// Price change over the last 24 hours (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub one_day_price_change: Option<Decimal>,
    /// The market question.
    pub question: String,
    /// Maximum spread for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub rewards_min_size: Option<Decimal>,
    /// Current spread (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub spread: Option<Decimal>,
    /// Market end date, exactly as sent (e.g. `"2024-08-10 00:00:00"`).
    ///
    /// Kept as a string because the spec does not document its format.
    pub end_date: Option<String>,
    /// Outcome tokens.
    pub tokens: Vec<RewardsToken>,
    /// 24-hour trading volume (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub volume_24hr: Option<Decimal>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<RewardsConfig>>,
}

impl ClobClient {
    /// Lists all current active reward configurations, grouped by market
    /// (`GET /rewards/markets/current`, cursor pagination, 500 items per page).
    ///
    /// See <https://docs.polymarket.com/api-reference/rewards/get-current-active-rewards-configurations>.
    pub fn list_current_rewards(&self) -> ListCurrentRewards {
        ListCurrentRewards {
            client: self.clone(),
            sponsored: None,
            cursor: None,
        }
    }

    /// Lists the present and future reward configurations of a market
    /// (`GET /rewards/markets/{condition_id}`, cursor pagination, 100 items per page).
    ///
    /// See <https://docs.polymarket.com/api-reference/rewards/get-raw-rewards-for-a-specific-market>.
    pub fn list_raw_rewards_for_market(
        &self,
        condition_id: impl Into<ConditionId>,
    ) -> ListRawRewardsForMarket {
        ListRawRewardsForMarket {
            client: self.clone(),
            condition_id: condition_id.into(),
            sponsored: None,
            cursor: None,
        }
    }

    /// Lists active markets with their reward configurations, with search, filters and
    /// sorting (`GET /rewards/markets/multi`, cursor pagination).
    ///
    /// See <https://docs.polymarket.com/api-reference/rewards/get-multiple-markets-with-rewards>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::{StreamExt as _, TryStreamExt as _};
    /// use polyoxide::clob::{RewardsMarketsOrderBy, SortDirection};
    ///
    /// let clob = polyoxide::clob::ClobClient::new()?;
    /// let markets: Vec<_> = clob
    ///     .list_markets_with_rewards()
    ///     .tag_slugs(["politics", "sports"])
    ///     .order_by(RewardsMarketsOrderBy::Volume24hr)
    ///     .position(SortDirection::Desc)
    ///     .page_size(500)
    ///     .into_stream()
    ///     .take(1000)
    ///     .try_collect()
    ///     .await?;
    /// for market in markets {
    ///     println!("{}: {:?}", market.question, market.volume_24hr);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_markets_with_rewards(&self) -> ListMarketsWithRewards {
        ListMarketsWithRewards {
            client: self.clone(),
            q: None,
            tag_slugs: Vec::new(),
            event_ids: Vec::new(),
            event_title: None,
            order_by: None,
            position: None,
            min_volume_24hr: None,
            max_volume_24hr: None,
            min_spread: None,
            max_spread: None,
            min_price: None,
            max_price: None,
            cursor: None,
            page_size: None,
        }
    }
}

/// Request builder for [`ClobClient::list_current_rewards`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListCurrentRewards {
    client: ClobClient,
    sponsored: Option<bool>,
    cursor: Option<String>,
}

impl ListCurrentRewards {
    /// If `true`, returns sponsored reward configurations instead of the standard ones
    /// (server default `false`).
    pub fn sponsored(mut self, sponsored: bool) -> Self {
        self.sponsored = Some(sponsored);
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

    async fn fetch(self, cursor: Option<String>) -> Result<Page<CurrentReward>> {
        let mut query = Query::new();
        query
            .push_opt("sponsored", self.sponsored)
            .push_opt("next_cursor", cursor);
        self.client
            .transport
            .get(&["rewards", "markets", "current"])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// An invalid cursor is an [`Error::Api`](crate::Error::Api) with status `400`. See
    /// [`Error`](crate::Error) for the other cases.
    pub async fn send(self) -> Result<Page<CurrentReward>> {
        let cursor = self.cursor.clone();
        self.fetch(cursor).await
    }

    /// Streams every configuration from the configured cursor onwards, fetching pages
    /// lazily until the last page (`next_cursor` `"LTE="`). The stream ends after yielding
    /// the first error.
    pub fn into_stream(self) -> Paginated<CurrentReward> {
        let start = self.cursor.clone();
        page_stream(self, start, Self::fetch)
    }
}

/// Path segments that `/rewards/markets/{condition_id}` shares with other endpoints
/// (`/rewards/markets/current`, `/rewards/markets/multi`); as a condition id they would
/// address those endpoints instead.
const RESERVED_REWARDS_MARKET_SEGMENTS: [&str; 2] = ["current", "multi"];

/// Request builder for [`ClobClient::list_raw_rewards_for_market`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListRawRewardsForMarket {
    client: ClobClient,
    condition_id: ConditionId,
    sponsored: Option<bool>,
    cursor: Option<String>,
}

impl ListRawRewardsForMarket {
    /// If `true`, folds sponsored daily rates into each configuration's `rate_per_day`
    /// (server default `false`).
    pub fn sponsored(mut self, sponsored: bool) -> Self {
        self.sponsored = Some(sponsored);
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

    async fn fetch(self, cursor: Option<String>) -> Result<Page<MarketReward>> {
        let condition_id = self.condition_id.as_str();
        require_id("condition_id", condition_id)?;
        if RESERVED_REWARDS_MARKET_SEGMENTS.contains(&condition_id) {
            return Err(ValidationError::new(
                "condition_id",
                format!(
                    "{condition_id:?} is not a condition id: `/rewards/markets/{condition_id}` \
                     is a different endpoint"
                ),
            )
            .into());
        }
        let mut query = Query::new();
        query
            .push_opt("sponsored", self.sponsored)
            .push_opt("next_cursor", cursor);
        self.client
            .transport
            .get(&["rewards", "markets", condition_id])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if the condition id is empty (which
    /// the server documents as `400` "Invalid market") or is `current` or `multi` (the path
    /// of another endpoint). An invalid market or cursor is an
    /// [`Error::Api`](crate::Error::Api) with status `400`. See [`Error`](crate::Error) for
    /// the other cases.
    pub async fn send(self) -> Result<Page<MarketReward>> {
        let cursor = self.cursor.clone();
        self.fetch(cursor).await
    }

    /// Streams every configured market entry from the configured cursor onwards, fetching
    /// pages lazily until the last page (`next_cursor` `"LTE="`). The stream ends after
    /// yielding the first error.
    pub fn into_stream(self) -> Paginated<MarketReward> {
        let start = self.cursor.clone();
        page_stream(self, start, Self::fetch)
    }
}

/// Request builder for [`ClobClient::list_markets_with_rewards`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarketsWithRewards {
    client: ClobClient,
    q: Option<String>,
    tag_slugs: Vec<String>,
    event_ids: Vec<EventId>,
    event_title: Option<String>,
    order_by: Option<RewardsMarketsOrderBy>,
    position: Option<SortDirection>,
    min_volume_24hr: Option<Decimal>,
    max_volume_24hr: Option<Decimal>,
    min_spread: Option<Decimal>,
    max_spread: Option<Decimal>,
    min_price: Option<Decimal>,
    max_price: Option<Decimal>,
    cursor: Option<String>,
    page_size: Option<u32>,
}

impl ListMarketsWithRewards {
    /// Text search on the market question and description.
    pub fn q(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    /// Only markets with any of these tag slugs (OR), sent as one `tag_slug` query
    /// parameter per value. Replaces previously set slugs.
    pub fn tag_slugs(mut self, tag_slugs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tag_slugs = tag_slugs.into_iter().map(Into::into).collect();
        self
    }

    /// Only markets of these events, sent as one `event_id` query parameter per value.
    /// Replaces previously set ids.
    pub fn event_ids(mut self, event_ids: impl IntoIterator<Item = impl Into<EventId>>) -> Self {
        self.event_ids = event_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Search event titles (case-insensitive pattern matching).
    pub fn event_title(mut self, event_title: impl Into<String>) -> Self {
        self.event_title = Some(event_title.into());
        self
    }

    /// Field to sort by.
    pub fn order_by(mut self, order_by: RewardsMarketsOrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Sort direction.
    pub fn position(mut self, position: SortDirection) -> Self {
        self.position = Some(position);
        self
    }

    /// Minimum 24-hour volume.
    pub fn min_volume_24hr(mut self, min: Decimal) -> Self {
        self.min_volume_24hr = Some(min);
        self
    }

    /// Maximum 24-hour volume.
    pub fn max_volume_24hr(mut self, max: Decimal) -> Self {
        self.max_volume_24hr = Some(max);
        self
    }

    /// Minimum spread.
    pub fn min_spread(mut self, min: Decimal) -> Self {
        self.min_spread = Some(min);
        self
    }

    /// Maximum spread.
    pub fn max_spread(mut self, max: Decimal) -> Self {
        self.max_spread = Some(max);
        self
    }

    /// Minimum price of the first token.
    pub fn min_price(mut self, min: Decimal) -> Self {
        self.min_price = Some(min);
        self
    }

    /// Maximum price of the first token.
    pub fn max_price(mut self, max: Decimal) -> Self {
        self.max_price = Some(max);
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

    /// Number of items per page (server default 100).
    ///
    /// The docs give a maximum of [`MAX_REWARDS_MARKETS_PAGE_SIZE`] and say the server caps
    /// larger values; this client rejects them instead (see [`send`](Self::send)), so a
    /// page never silently holds fewer items than asked for.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    async fn fetch(self, cursor: Option<String>) -> Result<Page<MultiMarketInfo>> {
        if let Some(page_size) = self.page_size
            && page_size > MAX_REWARDS_MARKETS_PAGE_SIZE
        {
            return Err(ValidationError::new(
                "page_size",
                format!("must be at most {MAX_REWARDS_MARKETS_PAGE_SIZE}, got {page_size}"),
            )
            .into());
        }
        let mut query = Query::new();
        query
            .push_opt("q", self.q.as_deref())
            .push_all("tag_slug", &self.tag_slugs)
            .push_all("event_id", &self.event_ids)
            .push_opt("event_title", self.event_title.as_deref())
            .push_opt("order_by", self.order_by.as_ref())
            .push_opt("position", self.position.as_ref())
            .push_opt("min_volume_24hr", self.min_volume_24hr)
            .push_opt("max_volume_24hr", self.max_volume_24hr)
            .push_opt("min_spread", self.min_spread)
            .push_opt("max_spread", self.max_spread)
            .push_opt("min_price", self.min_price)
            .push_opt("max_price", self.max_price)
            .push_opt("next_cursor", cursor)
            .push_opt("page_size", self.page_size);
        self.client
            .transport
            .get(&["rewards", "markets", "multi"])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if
    /// [`page_size`](Self::page_size) exceeds [`MAX_REWARDS_MARKETS_PAGE_SIZE`]. An invalid
    /// `order_by`, `position` or cursor is an [`Error::Api`](crate::Error::Api) with status
    /// `400`. See [`Error`](crate::Error) for the other cases.
    pub async fn send(self) -> Result<Page<MultiMarketInfo>> {
        let cursor = self.cursor.clone();
        self.fetch(cursor).await
    }

    /// Streams every market from the configured cursor onwards, fetching pages lazily until
    /// the last page (`next_cursor` `"LTE="`). The stream ends after yielding the first
    /// error.
    pub fn into_stream(self) -> Paginated<MultiMarketInfo> {
        let start = self.cursor.clone();
        page_stream(self, start, Self::fetch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clob::types::test_util::round_trip;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    /// Example response of `GET /rewards/markets/current` in docs/specs/clob-openapi.yaml
    /// (docs/api-reference/rewards/get-current-active-rewards-configurations.md).
    #[test]
    fn deserializes_current_rewards_page() {
        let json = r#"{
            "limit": 500,
            "count": 1,
            "next_cursor": "LTE=",
            "data": [{
                "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
                "rewards_max_spread": 99,
                "rewards_min_size": 10,
                "rewards_config": [
                    {"id": 0, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 2, "total_rewards": 92},
                    {"id": 0, "asset_address": "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 1, "total_rewards": 46}
                ],
                "sponsored_daily_rate": 0.5,
                "sponsors_count": 2,
                "native_daily_rate": 2.5,
                "total_daily_rate": 3.0
            }]
        }"#;
        let page: Page<CurrentReward> = round_trip(json);
        assert_eq!(page.limit, 500);
        assert!(page.is_last_page());
        let reward = &page.data[0];
        assert_eq!(reward.rewards_max_spread, Some(d("99")));
        let config = &reward.rewards_config.as_ref().unwrap()[1];
        assert_eq!(
            config.asset_address,
            "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB"
        );
        assert_eq!(
            config.start_date,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
        assert_eq!(
            config.end_date,
            Some(NaiveDate::from_ymd_opt(2500, 12, 31).unwrap())
        );
        assert_eq!(config.rate_per_day, d("1"));
        assert_eq!(reward.sponsors_count, Some(2));
        assert_eq!(reward.total_daily_rate, Some(d("3")));
    }

    /// Optional fields omitted when zero, per the `CurrentReward` schema descriptions.
    #[test]
    fn current_reward_minimal() {
        let reward: CurrentReward = serde_json::from_str(
            r#"{"condition_id":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af"}"#,
        )
        .unwrap();
        assert_eq!(reward.sponsored_daily_rate, None);
        assert!(serde_json::from_str::<CurrentReward>("{}").is_err());
    }

    /// Example response of `GET /rewards/markets/{condition_id}` in
    /// docs/specs/clob-openapi.yaml
    /// (docs/api-reference/rewards/get-raw-rewards-for-a-specific-market.md).
    #[test]
    fn deserializes_market_rewards_page() {
        let json = r#"{
            "limit": 100,
            "count": 1,
            "next_cursor": "LTE=",
            "data": [{
                "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
                "question": "Will Trump win the 2024 Iowa Caucus?",
                "market_slug": "will-trump-win-the-2024-iowa-caucus",
                "event_slug": "will-trump-win-the-2024-iowa-caucus",
                "image": "https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png",
                "rewards_max_spread": 99,
                "rewards_min_size": 10,
                "market_competitiveness": 0.42,
                "tokens": [
                    {"token_id": "1343197538147866997676250008839231694243646439454152539053893078719042421992", "outcome": "YES", "price": 0.8},
                    {"token_id": "16678291189211314787145083999015737376658799626183230671758641503291735614088", "outcome": "NO", "price": 0.2}
                ],
                "rewards_config": [
                    {"id": 1, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 0.25, "total_rewards": 0, "total_days": 174161},
                    {"id": 2, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2024-05-31", "rate_per_day": 1, "total_rewards": 92, "total_days": 92}
                ]
            }]
        }"#;
        let page: Page<MarketReward> = round_trip(json);
        let market = &page.data[0];
        assert_eq!(market.question, "Will Trump win the 2024 Iowa Caucus?");
        assert_eq!(market.market_competitiveness, Some(d("0.42")));
        assert_eq!(market.tokens[1].outcome, "NO");
        assert_eq!(market.tokens[1].price, Some(d("0.2")));
        let configs = market.rewards_config.as_ref().unwrap();
        assert_eq!(configs[0].rate_per_day, d("0.25"));
        assert_eq!(configs[0].total_days, Some(174_161));
        assert_eq!(configs[1].remaining_reward_amount, None);
    }

    /// Example response of `GET /rewards/markets/multi` in docs/specs/clob-openapi.yaml
    /// (docs/api-reference/rewards/get-multiple-markets-with-rewards.md).
    #[test]
    fn deserializes_multi_market_info_page() {
        let json = r#"{
            "limit": 50,
            "count": 1,
            "next_cursor": "NQ==",
            "data": [{
                "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
                "event_id": "12345",
                "event_slug": "2024-us-election",
                "created_at": "2024-05-01T12:00:00Z",
                "group_item_title": "",
                "image": "https://example.com/image.png",
                "market_competitiveness": 0.42,
                "market_id": "248849",
                "market_slug": "will-trump-win-the-2024-iowa-caucus",
                "one_day_price_change": 0.03,
                "question": "Will Trump win the 2024 Iowa Caucus?",
                "rewards_max_spread": 99,
                "rewards_min_size": 10,
                "spread": 0.12,
                "end_date": "2024-08-10 00:00:00",
                "tokens": [
                    {"token_id": "1343197538147866997676250008839231694243646439454152539053893078719042421992", "outcome": "YES", "price": 0.8},
                    {"token_id": "16678291189211314787145083999015737376658799626183230671758641503291735614088", "outcome": "NO", "price": 0.2}
                ],
                "volume_24hr": 12345.67,
                "rewards_config": [
                    {"id": 7, "asset_address": "0x9c4E1703476E875070EE25b56A58B008CFb8FA78", "start_date": "2024-03-01", "end_date": "2500-12-31", "rate_per_day": 2, "total_rewards": 92}
                ]
            }]
        }"#;
        let page: Page<MultiMarketInfo> = round_trip(json);
        assert_eq!(page.next_cursor(), Some("NQ=="));
        let market = &page.data[0];
        assert_eq!(market.event_id, Some(EventId::from("12345")));
        assert_eq!(market.market_id, "248849");
        assert_eq!(
            market.created_at.map(|t| t.timestamp()),
            Some(1_714_564_800)
        );
        assert_eq!(market.end_date.as_deref(), Some("2024-08-10 00:00:00"));
        assert_eq!(market.volume_24hr, Some(d("12345.67")));
        assert_eq!(market.one_day_price_change, Some(d("0.03")));
    }

    #[test]
    fn enum_wire_values() {
        assert_eq!(RewardsMarketsOrderBy::Volume24hr.as_str(), "volume_24hr");
        assert_eq!(SortDirection::Desc.to_string(), "DESC");
    }
}
