//! Markets: `/v2/holders`, `/v2/oi`, `/v2/live-volume`, `/v2/prices-history`,
//! `/v2/resolutions`.

use crate::Paginated;
use chrono::{DateTime, Utc};
use marcasite_core::{Query, Result, ValidationError, serde_util, validate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DataClient,
    types::{
        Page, check_limit, collect_ids, condition_id, distinct_values, integer_id, page_stream,
        positive_integer_id, required_values,
    },
};
use crate::types::{Address, ConditionId, EventId, QuestionId, TokenId};

const HOLDERS: &[&str] = &["v2", "holders"];
const PRICES_HISTORY: &[&str] = &["v2", "prices-history"];

/// Maximum `limit` of `/v2/holders`.
const MAX_HOLDERS_LIMIT: u32 = 1000;
/// Maximum `limit` of `/v2/holders` with `include_pnl=true`.
const MAX_HOLDERS_PNL_LIMIT: u32 = 100;
/// Maximum `limit` of `/v2/prices-history`.
const MAX_PRICES_HISTORY_LIMIT: u32 = 10_000;
/// Accepted range of `bucket_seconds` on `/v2/prices-history`.
const BUCKET_SECONDS: std::ops::RangeInclusive<u32> = 60..=86_400;
/// Longest explicit `start`..`end` window of `/v2/prices-history`, in seconds (15 days).
const MAX_WINDOW_SECONDS: i64 = 15 * 86_400;

/// One outcome token's holder group (`components/schemas/MetaHolder`).
///
/// A multi-market request interleaves tokens across the page, so merge groups across
/// pages by [`token_id`](Self::token_id), not by position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HolderGroup {
    /// The outcome token this group ranks.
    pub token_id: TokenId,
    /// Top holders of that token, amount descending.
    pub holders: Vec<Holder>,
}

/// One market holder, enriched with their public profile (`components/schemas/Holder`).
///
/// The seven position-economics fields (`avg_price` to `total_pnl`) are only served with
/// [`ListHolders::include_pnl`]; they are `None` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Holder {
    /// The holding wallet.
    pub proxy_wallet: Address,
    /// Profile bio text.
    pub bio: String,
    /// Outcome token held.
    pub token_id: TokenId,
    /// Generated fallback handle for profiles without a display name.
    pub pseudonym: String,
    /// Holding in shares: net across the market's outcomes by default, per-side gross with
    /// `include_pnl`.
    #[serde(with = "serde_util::decimal_number")]
    pub amount: Decimal,
    /// Whether the profile chose to show its name publicly.
    pub display_username_public: bool,
    /// Index of the held outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](super::UNLABELED_OUTCOME_INDEX) means unlabelable.
    pub outcome_index: i32,
    /// Profile display name of the wallet.
    pub name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Resized profile image URL (always empty on this route).
    pub profile_image_optimized: String,
    /// Profile verification badge.
    pub verified: bool,
    /// Historical entry price per share.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub avg_price: Option<Decimal>,
    /// Cost basis of the held size in USDC, excluding entry fees.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub entry_cost_usdc: Option<Decimal>,
    /// Current price of the held outcome token, in `[0, 1]`.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub current_price: Option<Decimal>,
    /// `amount × current_price`.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub current_value: Option<Decimal>,
    /// Profit already locked in by sells and redemptions.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub realized_pnl: Option<Decimal>,
    /// `current_value - entry_cost_usdc`.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub unrealized_pnl: Option<Decimal>,
    /// `realized_pnl + unrealized_pnl`.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_pnl: Option<Decimal>,
}

/// The priced gross open interest of a market, or the global figure
/// (`components/schemas/OpenInterest`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OpenInterest {
    /// Condition id the row answers for. On the global figure it holds the sentinel
    /// `GLOBAL`, which is not a condition id: check [`is_global`](Self::is_global) first.
    pub condition_id: ConditionId,
    /// Priced gross open interest, in USDC; `0` when nothing is held.
    #[serde(with = "serde_util::decimal_number")]
    pub value: Decimal,
}

impl OpenInterest {
    /// The `condition_id` of the global figure.
    pub const GLOBAL: &'static str = "GLOBAL";

    /// `true` for the global (all markets) figure.
    #[must_use]
    pub fn is_global(&self) -> bool {
        self.condition_id == Self::GLOBAL
    }
}

/// Cumulative one-side (taker) volume per market of the requested events
/// (`components/schemas/LiveVolume`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LiveVolume {
    /// Sum of the rows' `taker_volume`, in shares.
    #[serde(with = "serde_util::decimal_number")]
    pub taker_volume_total: Decimal,
    /// One row per market, `taker_volume` descending; empty when the events resolve to no
    /// markets.
    pub conditions: Vec<ConditionVolume>,
}

/// One market's cumulative taker volume (`components/schemas/ConditionVolume`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConditionVolume {
    /// On-chain condition id of the market.
    pub condition_id: ConditionId,
    /// Cumulative one-side (taker) volume in shares, truncated to 6 decimals.
    #[serde(with = "serde_util::decimal_number")]
    pub taker_volume: Decimal,
}

marcasite_core::string_enum! {
    /// Relative window of `/v2/prices-history` (`interval`).
    pub enum PriceHistoryInterval {
        /// The market's whole life.
        Max => "max",
        /// The market's whole life (same as [`Max`](Self::Max)).
        All => "all",
        /// One month.
        OneMonth => "1m",
        /// One week.
        OneWeek => "1w",
        /// One day.
        OneDay => "1d",
        /// Six hours.
        SixHours => "6h",
        /// One hour.
        OneHour => "1h",
    }
}

/// One price-history point (`components/schemas/PricePoint`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PricePoint {
    /// The observation's own time (a bucket's start for aggregates), never the time asked
    /// for.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Price, in `0..=1`.
    #[serde(with = "serde_util::decimal_number")]
    pub price: Decimal,
    /// Width in seconds of the window the price was observed in: `0` for an exact tick,
    /// the bucket width for an aggregate.
    ///
    /// On a request that pins [`bucket_seconds`](ListPricesHistory::bucket_seconds), it
    /// echoes the requested grid, not the density of what filled it; counting rows is the
    /// only density measure.
    pub resolution_seconds: i64,
}

marcasite_core::string_enum! {
    /// Lifecycle state of a resolution row (`status`).
    pub enum ResolutionStatus {
        /// Initialized.
        Initialized => "initialized",
        /// Posed.
        Posed => "posed",
        /// Proposed.
        Proposed => "proposed",
        /// Challenged.
        Challenged => "challenged",
        /// Re-proposed.
        Reproposed => "reproposed",
        /// Disputed.
        Disputed => "disputed",
        /// Resolved.
        Resolved => "resolved",
        /// Active (condition-keyed rows only).
        Active => "active",
        /// In arbitration (condition-keyed rows only).
        Arbitration => "arbitration",
    }
}

marcasite_core::string_enum! {
    /// Market type of a condition-keyed resolution row (`market_type`).
    pub enum ResolutionMarketType {
        /// A binary market.
        Binary => "BINARY",
        /// An incremental neg-risk market.
        IncrementalNegrisk => "INCREMENTAL_NEGRISK",
        /// An atomic neg-risk market.
        AtomicNegrisk => "ATOMIC_NEGRISK",
    }
}

marcasite_core::string_enum! {
    /// Reporter family that resolved a market (`reporter`).
    pub enum Reporter {
        /// The UMA optimistic oracle.
        UmaOo => "UMA_OO",
        /// Chainlink.
        Chainlink => "CHAINLINK",
        /// An externally owned account.
        Eoa => "EOA",
    }
}

marcasite_core::string_enum! {
    /// How a resolution came about (`resolution_source`).
    pub enum ResolutionSource {
        /// An oracle reported it.
        Reported => "reported",
        /// Derived from a neg-risk sibling resolution.
        Derived => "derived",
    }
}

marcasite_core::string_enum! {
    /// Source of the `expected_settlement_time` estimate (`settlement_time_basis`).
    pub enum SettlementTimeBasis {
        /// The managed proposal's expiration.
        ManagedProposalExpiration => "managed_proposal_expiration",
        /// The liveness period.
        Liveness => "liveness",
        /// A DVM voting-round estimate.
        DvmRoundEstimate => "dvm_round_estimate",
    }
}

/// One resolution-state row (`components/schemas/ResolutionWithSettlementTime`: the
/// `Resolution` fields plus `settlement_time_basis`).
///
/// UMA lifecycle rows populate the numeric-string price fields; question lookups omit
/// `condition_id`. Native and terminal rows populate the condition lifecycle, payout and
/// finality fields where available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Resolution {
    /// Lifecycle state.
    pub status: ResolutionStatus,
    /// `true` while a managed proposal sits past its normal expiry in extended review.
    pub extended_review: bool,
    /// Whether the resolution was disputed at any point.
    pub was_disputed: bool,
    /// Whether the question rules were updated after posing.
    pub new_version_q: bool,
    /// Transaction of the latest lifecycle event; empty on condition-keyed rows without
    /// one.
    pub transaction_hash: String,
    /// Log index of the latest lifecycle event, as a numeric string; empty where
    /// `transaction_hash` is empty. Kept as served (a string, as the schema types it).
    pub log_index: String,
    /// Latest lifecycle change, as served: epoch seconds on question-keyed rows, RFC 3339
    /// on condition-keyed rows. See [`last_update_time`](Self::last_update_time).
    pub last_update_timestamp: String,
    /// Condition id the row answers for; absent on question-keyed rows.
    pub condition_id: Option<ConditionId>,
    /// UMA question id serving the row; absent on condition-keyed rows.
    pub question_id: Option<QuestionId>,
    /// Estimated settlement time (an estimate, not a deadline); `None` when timing is
    /// unavailable.
    #[serde(default, with = "serde_util::datetime_option")]
    pub expected_settlement_time: Option<DateTime<Utc>>,
    /// Source of `expected_settlement_time`.
    pub settlement_time_basis: Option<SettlementTimeBasis>,
    /// Market type (condition-keyed rows only).
    pub market_type: Option<ResolutionMarketType>,
    /// Per-outcome payout in micro-USDC per share (`[outcome0, outcome1]`), on resolved
    /// condition-keyed rows.
    pub payouts: Option<Vec<i64>>,
    /// Final settlement price, as a numeric string (same conventions as
    /// `proposed_price`).
    pub price: Option<String>,
    /// Price of the first proposal, as a numeric string; `69` means unset. Question-keyed
    /// rows only.
    ///
    /// The UMA price fields are kept as the served strings: the docs give neither their
    /// scale nor their range, so they are not converted to a [`Decimal`].
    pub proposed_price: Option<String>,
    /// Price of the second proposal, as a numeric string (same conventions as
    /// `proposed_price`).
    pub reproposed_price: Option<String>,
    /// Reporter family that resolved the market.
    pub reporter: Option<Reporter>,
    /// Whether an oracle reported the resolution or it was derived.
    pub resolution_source: Option<ResolutionSource>,
    /// When the condition resolved.
    #[serde(default, with = "serde_util::datetime_option")]
    pub resolved_at: Option<DateTime<Utc>>,
    /// Block the condition resolved at.
    pub resolved_block: Option<i64>,
    /// Whether arbitration was triggered on the request.
    pub was_arbitrated: Option<bool>,
}

impl Resolution {
    /// Parses [`last_update_timestamp`](Self::last_update_timestamp), which is served as
    /// epoch seconds on question-keyed rows and RFC 3339 on condition-keyed rows.
    ///
    /// Returns `None` if the value is empty or in neither format.
    #[must_use]
    pub fn last_update_time(&self) -> Option<DateTime<Utc>> {
        let raw = self.last_update_timestamp.trim();
        match raw.parse::<i64>() {
            Ok(secs) => DateTime::from_timestamp(secs, 0),
            Err(_) => serde_util::parse_datetime(raw),
        }
    }
}

/// Which resolution rows to fetch with [`DataClient::get_resolutions`]: exactly one
/// selector family per request.
///
/// The ids are checked when the request is sent: a question id must be `0x` followed by
/// 64 hex digits, condition ids likewise, and event ids positive integers; lists accept
/// at most 20 distinct values, and duplicates are sent once.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionSelector {
    /// One UMA question (`question_id`).
    Question(QuestionId),
    /// Up to 20 condition ids (`condition`).
    Conditions(Vec<ConditionId>),
    /// Up to 20 Gamma event ids (`event_id`).
    Events(Vec<EventId>),
}

impl ResolutionSelector {
    /// Selects one UMA question.
    pub fn question(question_id: impl Into<QuestionId>) -> Self {
        Self::Question(question_id.into())
    }

    /// Selects condition ids (at most 20 distinct values).
    pub fn conditions<I>(conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        Self::Conditions(collect_ids(conditions))
    }

    /// Selects Gamma event ids (at most 20 distinct values).
    pub fn events<I>(event_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<EventId>,
    {
        Self::Events(collect_ids(event_ids))
    }

    fn query(&self) -> Result<Query> {
        let mut q = Query::new();
        match self {
            Self::Question(id) => {
                validate::bytes32("question_id", id.as_str())?;
                q.push("question_id", id);
            }
            Self::Conditions(ids) => {
                q.push_csv(
                    "condition",
                    required_values("condition", ids, condition_id)?,
                );
            }
            Self::Events(ids) => {
                q.push_csv(
                    "event_id",
                    required_values("event_id", ids, positive_integer_id)?,
                );
            }
        }
        Ok(q)
    }
}

impl From<QuestionId> for ResolutionSelector {
    fn from(question_id: QuestionId) -> Self {
        Self::Question(question_id)
    }
}

impl DataClient {
    /// Lists a market's top holders, grouped by outcome token
    /// (`GET /v2/holders`, cursor-paginated).
    ///
    /// `conditions` is required: between 1 and 20 distinct condition ids (exactly one with
    /// [`include_pnl`](ListHolders::include_pnl)). Duplicates are sent once.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/list-a-markets-top-holders>.
    pub fn list_holders<I>(&self, conditions: I) -> ListHolders
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        ListHolders {
            client: self.clone(),
            conditions: collect_ids(conditions),
            limit: None,
            cursor: None,
            min_balance: None,
            include_pnl: None,
        }
    }

    /// Gets the priced gross open interest per market, or the global figure when no
    /// condition is given (`GET /v2/oi`).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-open-interest>.
    pub fn get_open_interest(&self) -> GetOpenInterest {
        GetOpenInterest {
            client: self.clone(),
            conditions: Vec::new(),
        }
    }

    /// Gets the cumulative taker volume of every market under the given events
    /// (`GET /v2/live-volume`); between 1 and 20 distinct integer event ids.
    ///
    /// The server ignores ids it cannot parse, so a typo would silently drop an event;
    /// this client rejects any id that is not made of ASCII digits instead. Duplicates are
    /// sent once.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-live-volume-for-an-event>.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if no event id is given, an
    /// id is not an integer, or more than 20 distinct ids are given; otherwise see
    /// [`Error`](crate::Error).
    pub async fn get_live_volume<I>(&self, event_ids: I) -> Result<LiveVolume>
    where
        I: IntoIterator,
        I::Item: Into<EventId>,
    {
        let event_ids: Vec<EventId> = collect_ids(event_ids);
        let event_ids = required_values("event_id", &event_ids, integer_id)?;
        let mut query = Query::new();
        query.push_csv("event_id", &event_ids);
        self.fetch_data(&["v2", "live-volume"], query).await
    }

    /// Lists a token's price history, or a single point-in-time observation
    /// (`GET /v2/prices-history`, cursor-paginated).
    ///
    /// Choose exactly one window form: [`start`](ListPricesHistory::start) (with an
    /// optional [`end`](ListPricesHistory::end)), [`interval`](ListPricesHistory::interval),
    /// or [`as_of`](ListPricesHistory::as_of).
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-a-tokens-price-history>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use marcasite::data::{DataClient, PriceHistoryInterval};
    ///
    /// let data = DataClient::new()?;
    /// let points: Vec<_> = data
    ///     .list_prices_history(
    ///         "31974447302330162086995746309500877260929998201718217388109724292047967921664",
    ///     )
    ///     .interval(PriceHistoryInterval::OneWeek)
    ///     .bucket_seconds(1800)
    ///     .into_stream()
    ///     .try_collect()
    ///     .await?;
    /// // The last point is the terminal point: the latest observation in the window.
    /// # let _ = points;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_prices_history(&self, token_id: impl Into<TokenId>) -> ListPricesHistory {
        ListPricesHistory {
            client: self.clone(),
            token_id: token_id.into(),
            start: None,
            end: None,
            interval: None,
            bucket_seconds: None,
            as_of: None,
            limit: None,
            cursor: None,
        }
    }

    /// Gets the resolution state of a UMA question, markets or events
    /// (`GET /v2/resolutions`). Misses return an empty list.
    ///
    /// See <https://docs.polymarket.com/api-reference/markets/get-resolution-state>.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) for an empty selector, one
    /// with more than 20 distinct ids, or a malformed id (see [`ResolutionSelector`]);
    /// otherwise see [`Error`](crate::Error).
    pub async fn get_resolutions(&self, selector: ResolutionSelector) -> Result<Vec<Resolution>> {
        let query = selector.query()?;
        self.fetch_data(&["v2", "resolutions"], query).await
    }
}

/// Request builder for [`DataClient::list_holders`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListHolders {
    client: DataClient,
    conditions: Vec<ConditionId>,
    limit: Option<u32>,
    cursor: Option<String>,
    min_balance: Option<Decimal>,
    include_pnl: Option<bool>,
}

impl ListHolders {
    /// Rows per outcome token (`limit`, default 100, at most 1000; at most 100 with
    /// [`include_pnl`](Self::include_pnl)). A cursor carries its own window and overrides
    /// it.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`).
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Minimum balance in shares (`min_balance`, default 0).
    pub fn min_balance(mut self, min_balance: Decimal) -> Self {
        self.min_balance = Some(min_balance);
        self
    }

    /// Opt into per-holder position economics and per-side gross amounts
    /// (`include_pnl`, default `false`). Requires exactly one condition id and a `limit`
    /// of at most 100.
    pub fn include_pnl(mut self, include_pnl: bool) -> Self {
        self.include_pnl = Some(include_pnl);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        let conditions = required_values("condition", &self.conditions, condition_id)?;
        check_limit(self.limit, MAX_HOLDERS_LIMIT)?;
        if self.include_pnl == Some(true) {
            if conditions.len() != 1 {
                return Err(ValidationError::new(
                    "condition",
                    "exactly one condition id is accepted with `include_pnl`",
                )
                .into());
            }
            check_limit(self.limit, MAX_HOLDERS_PNL_LIMIT)?;
        }
        let mut q = Query::new();
        q.push_csv("condition", &conditions)
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_opt("min_balance", self.min_balance)
            .push_opt("include_pnl", self.include_pnl);
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if no condition id or more
    /// than 20 distinct ones are given, one is not `0x` followed by 64 hex digits, `limit`
    /// is above 1000, or `include_pnl` is combined with several condition ids or a `limit`
    /// above 100; otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<HolderGroup>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(HOLDERS, query).await
    }

    /// Streams every holder group from the configured cursor onwards, fetching pages
    /// lazily.
    ///
    /// Every page re-sends the same `condition` list with the cursor, as the API requires.
    /// Page walks advance every token group together, so a token's holders are spread
    /// over several groups: merge them by [`HolderGroup::token_id`]. The stream ends when
    /// the server reports no further page.
    pub fn into_stream(self) -> Paginated<HolderGroup> {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, HOLDERS, start, move |cursor| self.query(cursor))
    }
}

/// Request builder for [`DataClient::get_open_interest`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetOpenInterest {
    client: DataClient,
    conditions: Vec<ConditionId>,
}

impl GetOpenInterest {
    /// Condition ids (`condition`, at most 20 distinct values). Omit for the global
    /// figure. Duplicates are sent once.
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
        self
    }

    /// Sends the request. An id that does not resolve to a servable market is absent from
    /// the result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if a condition id is not
    /// `0x` followed by 64 hex digits or more than 20 distinct ids are given; otherwise
    /// see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Vec<OpenInterest>> {
        let conditions = distinct_values("condition", &self.conditions, condition_id)?;
        let mut query = Query::new();
        query.push_csv("condition", &conditions);
        self.client.fetch_data(&["v2", "oi"], query).await
    }
}

/// Request builder for [`DataClient::list_prices_history`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListPricesHistory {
    client: DataClient,
    token_id: TokenId,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    interval: Option<PriceHistoryInterval>,
    bucket_seconds: Option<u32>,
    as_of: Option<DateTime<Utc>>,
    limit: Option<u32>,
    cursor: Option<String>,
}

impl ListPricesHistory {
    /// Inclusive window start (`start`, epoch seconds; must be after the Unix epoch).
    /// Alone it means "up to the present".
    ///
    /// An explicit window spans **at most 15 days** (`end - start`, or now `- start`
    /// without an `end`); the docs say a longer one is capped, but live it is a `400`
    /// (`'start' to 'end' must span at most 15 days`, not clamped). With both `start` and
    /// [`end`](Self::end) set the client rejects a longer span itself; with `start` alone
    /// it cannot (it would depend on the clock), so the server answers. Set `end` too when
    /// paging, because the span is re-checked on every page. The
    /// [`interval`](Self::interval) presets are the long-range path.
    ///
    /// Points are bucket-aligned: the first one can precede `start` by less than one
    /// bucket.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Exclusive window end (`end`, epoch seconds). Requires [`start`](Self::start), and
    /// must not precede it.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Relative window instead of `start`/`end` (`interval`).
    pub fn interval(mut self, interval: PriceHistoryInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Bucket width in seconds (`bucket_seconds`, 60 to 86400). Omit to let the server
    /// pick the densest width that covers the window (a 15-day window, for instance, is
    /// served in 30-minute buckets); when set, it is served exactly (possibly as an empty
    /// page where that resolution has expired).
    pub fn bucket_seconds(mut self, bucket_seconds: u32) -> Self {
        self.bucket_seconds = Some(bucket_seconds);
        self
    }

    /// Point-in-time read: the latest observation at or before this instant (`as_of`,
    /// epoch seconds). Cannot be combined with a window.
    pub fn as_of(mut self, as_of: DateTime<Utc>) -> Self {
        self.as_of = Some(as_of);
        self
    }

    /// First-page size (`limit`, default and maximum 10 000). Ignored by the server once a
    /// cursor is supplied (the cursor's own page size wins).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Resumes from a previous page's [`next_cursor`](Page::next_cursor) (`cursor`).
    ///
    /// A cursor page must restate the window (`start`/`end`, `interval` or `as_of`): live,
    /// a cursor with only the token is a `400` ("provide a time component"), so the client
    /// rejects it too. [`into_stream`](Self::into_stream) always restates the window.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        if self.token_id.as_str().trim().is_empty() {
            return Err(ValidationError::new("token_id", "must not be empty").into());
        }
        if self.end.is_some() && self.start.is_none() {
            return Err(ValidationError::new("end", "requires `start`").into());
        }
        let forms = [
            self.start.is_some(),
            self.interval.is_some(),
            self.as_of.is_some(),
        ]
        .into_iter()
        .filter(|set| *set)
        .count();
        if forms > 1 {
            return Err(ValidationError::new(
                "interval",
                "`start`/`end`, `interval` and `as_of` are mutually exclusive",
            )
            .into());
        }
        // Exactly one window form per request, restated on every cursor page (live: a
        // cursor with only the token is a 400).
        if forms == 0 {
            return Err(ValidationError::new(
                "start",
                "one window is required, also with a cursor: `start` (with optional `end`), \
                 `interval` or `as_of`",
            )
            .into());
        }
        // A `0` bound is a documented `400`; a negative one is never meaningful.
        for (name, bound) in [
            ("start", self.start),
            ("end", self.end),
            ("as_of", self.as_of),
        ] {
            if bound.is_some_and(|t| t.timestamp() <= 0) {
                return Err(ValidationError::new(name, "must be after the Unix epoch").into());
            }
        }
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if end < start {
                return Err(ValidationError::new("end", "must not precede `start`").into());
            }
            if (end - start).num_seconds() > MAX_WINDOW_SECONDS {
                return Err(ValidationError::new(
                    "end",
                    "an explicit `start`..`end` window spans at most 15 days; use `interval` \
                     for longer ranges",
                )
                .into());
            }
        }
        if let Some(bucket) = self.bucket_seconds
            && !BUCKET_SECONDS.contains(&bucket)
        {
            return Err(ValidationError::new(
                "bucket_seconds",
                format!(
                    "must be between {} and {}, got {bucket}",
                    BUCKET_SECONDS.start(),
                    BUCKET_SECONDS.end()
                ),
            )
            .into());
        }
        check_limit(self.limit, MAX_PRICES_HISTORY_LIMIT)?;

        let mut q = Query::new();
        q.push("token_id", &self.token_id)
            .push_opt("start", self.start.map(|t| t.timestamp()))
            .push_opt("end", self.end.map(|t| t.timestamp()))
            .push_opt("interval", self.interval.as_ref())
            .push_opt("bucket_seconds", self.bucket_seconds)
            .push_opt("as_of", self.as_of.map(|t| t.timestamp()))
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor);
        Ok(q)
    }

    /// Fetches one page of points, oldest first.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if the token id is empty,
    /// the window is missing (also with a cursor) or ambiguous, `end` is set without
    /// `start` or before it, a bound is not after the Unix epoch, `start` and `end` span
    /// more than 15 days, `bucket_seconds` is outside 60..=86400, or `limit` is above
    /// 10 000; otherwise see [`Error`](crate::Error). A `start` alone more than 15 days
    /// back is rejected by the server only.
    pub async fn send(self) -> Result<Page<PricePoint>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(PRICES_HISTORY, query).await
    }

    /// Streams every point from the configured cursor onwards, fetching pages lazily.
    ///
    /// Every page re-sends the same token and window with the cursor. The terminal point
    /// (the latest observation in the window) and a resolved market's settlement point
    /// arrive on the final page. The stream ends when the server reports no further page.
    pub fn into_stream(self) -> Paginated<PricePoint> {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, PRICES_HISTORY, start, move |cursor| {
            self.query(cursor)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::{
        Envelope,
        test_ids::{CONDITION, CONDITION_2},
    };
    use marcasite_core::Error;

    /// A holder row from `components/schemas/Holder` without `include_pnl`: none of the
    /// seven position-economics fields is served.
    const HOLDER: &str = r#"{
      "proxy_wallet": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
      "bio": "",
      "token_id": "1",
      "pseudonym": "Calm-Owl",
      "amount": 1500.25,
      "display_username_public": true,
      "outcome_index": 0,
      "name": "",
      "profile_image": "",
      "profile_image_optimized": "",
      "verified": false
    }"#;

    /// Field names and types from `components/schemas/HoldersPage`, `MetaHolder` and
    /// `Holder` in `docs/specs/data-v2-openapi.json`.
    #[test]
    fn deserializes_holders_page() {
        let json = format!(
            r#"{{
              "data": [{{"token_id": "1", "holders": [{HOLDER}]}}],
              "pagination": {{"limit": 100, "offset": 0, "has_more": false, "next_cursor": null}}
            }}"#
        );
        let page: Page<HolderGroup> = serde_json::from_str(&json).unwrap();
        let group = page.items().first().unwrap();
        let holder = group.holders.first().unwrap();
        assert_eq!(holder.amount.to_string(), "1500.25");
        assert_eq!(holder.avg_price, None);
        assert_eq!(holder.total_pnl, None);
        assert_eq!(page.next_cursor(), None);
    }

    /// With `include_pnl=true` all seven position-economics fields are served.
    #[test]
    fn deserializes_holder_with_pnl() {
        let json = HOLDER.replace(
            r#""verified": false"#,
            r#""verified": false,
              "avg_price": 0.4,
              "entry_cost_usdc": 600.1,
              "current_price": 0.5,
              "current_value": 750.125,
              "realized_pnl": -2,
              "unrealized_pnl": 150.025,
              "total_pnl": 148.025"#,
        );
        let holder: Holder = serde_json::from_str(&json).unwrap();
        assert_eq!(holder.avg_price, Some(Decimal::new(4, 1)));
        assert_eq!(holder.current_price, Some(Decimal::new(5, 1)));
        assert_eq!(holder.total_pnl, Some(Decimal::new(148_025, 3)));
        let value = serde_json::to_value(&holder).unwrap();
        assert_eq!(value["realized_pnl"], serde_json::json!(-2));
        assert_eq!(value["amount"], serde_json::json!(1500.25));
    }

    /// `/v2/oi` without `condition` serves the global figure (`condition_id = "GLOBAL"`,
    /// `components/schemas/Envelope_Vec_OpenInterest`).
    #[test]
    fn deserializes_open_interest() {
        let env: Envelope<Vec<OpenInterest>> =
            serde_json::from_str(r#"{"data":[{"condition_id":"GLOBAL","value":123456.78}]}"#)
                .unwrap();
        let oi = env.data.first().unwrap();
        assert!(oi.is_global());
        assert_eq!(oi.value.to_string(), "123456.78");
    }

    /// Events that resolve to no markets serve `{ taker_volume_total: 0.0, conditions: [] }`
    /// (`components/schemas/LiveVolume`).
    #[test]
    fn deserializes_live_volume() {
        let env: Envelope<LiveVolume> =
            serde_json::from_str(r#"{"data":{"taker_volume_total":0.0,"conditions":[]}}"#).unwrap();
        assert_eq!(env.data.taker_volume_total, Decimal::ZERO);
        assert!(env.data.conditions.is_empty());
        assert_eq!(
            serde_json::to_value(&env.data).unwrap(),
            serde_json::json!({"taker_volume_total": 0, "conditions": []})
        );
    }

    /// Field names and types from `components/schemas/ResolutionWithSettlementTime` /
    /// `Resolution`: a question-keyed row and a condition-keyed row.
    #[test]
    fn deserializes_resolutions() {
        let json = r#"{"data":[
          {
            "question_id": "0x1111111111111111111111111111111111111111111111111111111111111111",
            "status": "proposed",
            "extended_review": false,
            "was_disputed": false,
            "new_version_q": false,
            "transaction_hash": "0xfeed",
            "log_index": "12",
            "last_update_timestamp": "1787133600",
            "proposed_price": "69",
            "expected_settlement_time": "2026-08-19T12:00:00Z",
            "settlement_time_basis": "liveness"
          },
          {
            "condition_id": "0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79",
            "status": "resolved",
            "extended_review": false,
            "was_disputed": true,
            "new_version_q": false,
            "transaction_hash": "",
            "log_index": "",
            "last_update_timestamp": "2026-08-19T10:00:00Z",
            "market_type": "BINARY",
            "payouts": [1000000, 0],
            "reporter": "UMA_OO",
            "resolution_source": "reported",
            "resolved_at": "2026-08-19T10:00:00Z",
            "resolved_block": 75000000,
            "was_arbitrated": null
          }
        ]}"#;
        let rows = serde_json::from_str::<Envelope<Vec<Resolution>>>(json)
            .unwrap()
            .data;
        let [question, condition] = rows.as_slice() else {
            panic!("expected two rows")
        };
        assert_eq!(question.status, ResolutionStatus::Proposed);
        assert_eq!(question.condition_id, None);
        assert_eq!(
            question.settlement_time_basis,
            Some(SettlementTimeBasis::Liveness)
        );
        assert_eq!(
            question.last_update_time().map(|t| t.timestamp()),
            Some(1_787_133_600)
        );
        assert_eq!(condition.payouts.as_deref(), Some(&[1_000_000, 0][..]));
        assert_eq!(condition.reporter, Some(Reporter::UmaOo));
        assert_eq!(condition.market_type, Some(ResolutionMarketType::Binary));
        assert_eq!(
            condition.last_update_time().map(|t| t.timestamp()),
            Some(1_787_133_600)
        );
    }

    #[test]
    fn deserializes_price_point() {
        let point: PricePoint = serde_json::from_str(
            r#"{"timestamp":1787133600,"price":0.515,"resolution_seconds":0}"#,
        )
        .unwrap();
        assert_eq!(point.price.to_string(), "0.515");
        assert_eq!(point.resolution_seconds, 0);
        assert_eq!(
            serde_json::to_value(&point).unwrap(),
            serde_json::json!({"timestamp": 1787133600, "price": 0.515, "resolution_seconds": 0})
        );
    }

    fn validation_parameter(result: Result<Query>) -> String {
        match result {
            Err(Error::Validation(v)) => v.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn prices_history_validation() {
        let client = DataClient::new().unwrap();
        let at = DateTime::from_timestamp(1_787_133_600, 0).unwrap();
        // No window.
        assert_eq!(
            validation_parameter(client.list_prices_history("1").query(None)),
            "start"
        );
        // A cursor page must restate the window too: live, a cursor with only the token is
        // a 400 ("provide a time component").
        assert_eq!(
            validation_parameter(client.list_prices_history("1").query(Some("c"))),
            "start"
        );
        assert!(
            client
                .list_prices_history("1")
                .interval(PriceHistoryInterval::OneDay)
                .query(Some("c"))
                .is_ok()
        );
        // Two windows.
        assert_eq!(
            validation_parameter(
                client
                    .list_prices_history("1")
                    .start(at)
                    .interval(PriceHistoryInterval::Max)
                    .query(None)
            ),
            "interval"
        );
        // `end` without `start`.
        assert_eq!(
            validation_parameter(client.list_prices_history("1").end(at).query(Some("c"))),
            "end"
        );
        // `end` before `start`.
        assert_eq!(
            validation_parameter(
                client
                    .list_prices_history("1")
                    .start(at)
                    .end(at - chrono::Duration::seconds(1))
                    .query(None)
            ),
            "end"
        );
        // An explicit window spans at most 15 days (live: a longer one is a 400, not
        // clamped); exactly 15 days is fine.
        let fifteen_days = chrono::Duration::days(15);
        assert!(
            client
                .list_prices_history("1")
                .start(at)
                .end(at + fifteen_days)
                .query(None)
                .is_ok()
        );
        assert_eq!(
            validation_parameter(
                client
                    .list_prices_history("1")
                    .start(at)
                    .end(at + fifteen_days + chrono::Duration::seconds(1))
                    .query(None)
            ),
            "end"
        );
        // Bucket width out of range.
        assert_eq!(
            validation_parameter(
                client
                    .list_prices_history("1")
                    .as_of(at)
                    .bucket_seconds(59)
                    .query(None)
            ),
            "bucket_seconds"
        );
        // A zero bound is a documented `400`; reject it client-side.
        assert_eq!(
            validation_parameter(
                client
                    .list_prices_history("1")
                    .start(DateTime::UNIX_EPOCH)
                    .query(None)
            ),
            "start"
        );
        assert_eq!(
            validation_parameter(client.list_prices_history(" ").as_of(at).query(None)),
            "token_id"
        );
        let q = client
            .list_prices_history("1")
            .interval(PriceHistoryInterval::OneWeek)
            .bucket_seconds(300)
            .query(None)
            .unwrap();
        assert_eq!(q.to_string(), "token_id=1&interval=1w&bucket_seconds=300");
    }

    #[test]
    fn holders_and_resolutions_validation() {
        let client = DataClient::new().unwrap();
        assert_eq!(
            validation_parameter(client.list_holders(Vec::<String>::new()).query(None)),
            "condition"
        );
        assert_eq!(
            validation_parameter(
                client
                    .list_holders([CONDITION, CONDITION_2])
                    .include_pnl(true)
                    .query(None)
            ),
            "condition"
        );
        // The same id twice is one distinct id.
        let q = client
            .list_holders([CONDITION, CONDITION])
            .include_pnl(true)
            .query(None)
            .unwrap();
        assert_eq!(q.get("condition"), Some(CONDITION));
        assert_eq!(
            validation_parameter(
                client
                    .list_holders([CONDITION])
                    .include_pnl(true)
                    .limit(101)
                    .query(None)
            ),
            "limit"
        );
        assert!(
            client
                .list_holders([CONDITION])
                .include_pnl(true)
                .limit(100)
                .query(None)
                .is_ok()
        );
        assert_eq!(
            validation_parameter(client.list_holders(["0x1"]).query(None)),
            "condition"
        );

        let selector_parameter = |selector: ResolutionSelector| match selector.query() {
            Err(Error::Validation(v)) => v.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        };
        assert_eq!(
            selector_parameter(ResolutionSelector::events(Vec::<String>::new())),
            "event_id"
        );
        assert_eq!(
            selector_parameter(ResolutionSelector::events(["0"])),
            "event_id"
        );
        assert_eq!(
            selector_parameter(ResolutionSelector::events(["12a"])),
            "event_id"
        );
        assert_eq!(
            selector_parameter(ResolutionSelector::question("")),
            "question_id"
        );
        assert_eq!(
            selector_parameter(ResolutionSelector::question("0x1234")),
            "question_id"
        );
        assert_eq!(
            selector_parameter(ResolutionSelector::conditions(["0x1"])),
            "condition"
        );
        assert_eq!(
            ResolutionSelector::conditions([CONDITION, CONDITION_2, CONDITION])
                .query()
                .unwrap()
                .to_string(),
            format!("condition={CONDITION}%2C{CONDITION_2}")
        );
        assert_eq!(
            ResolutionSelector::events(["20", "10", "20"])
                .query()
                .unwrap()
                .to_string(),
            "event_id=20%2C10"
        );
    }
}
