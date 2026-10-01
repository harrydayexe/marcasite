//! Wallet: `/v2/positions`, `/v2/positions/combos`, `/v2/value`, `/v2/approvals`,
//! `/v2/user-pnl`, `/v2/user-stats`, `/v2/user-volume`.

use chrono::{DateTime, NaiveDate, Utc};
use futures_core::Stream;
use polyoxide_core::{Query, Result, ValidationError, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    DataClient,
    types::{
        ComboLeg, EventId, FilterType, Page, SortDirection, check_limit, check_list, collect_ids,
        distinct, page_stream, timestamp_micros, timestamp_micros_option,
    },
};
use crate::types::{Address, ConditionId, TokenId};

const POSITIONS: &[&str] = &["v2", "positions"];
const COMBO_POSITIONS: &[&str] = &["v2", "positions", "combos"];

/// Maximum `limit` of `/v2/positions` and `/v2/positions/combos`.
const MAX_POSITIONS_LIMIT: u32 = 1000;

/// Maximum length, in characters, of the `/v2/positions` `title` filter.
const MAX_TITLE_CHARS: usize = 200;

polyoxide_core::string_enum! {
    /// Lifecycle state of a position (`status` filter and row field of `/v2/positions`).
    pub enum PositionStatus {
        /// Open positions: the **superset**, including settled-but-unredeemed winners. The
        /// default filter.
        Open => "OPEN",
        /// Open positions whose market resolved and whose tokens are still held.
        Redeemable => "REDEEMABLE",
        /// Redeemable positions that lost (filter only, requires `user`; rows keep status
        /// [`Redeemable`](Self::Redeemable)).
        RedeemableLost => "REDEEMABLE_LOST",
        /// Open positions narrowed to live complementary pairs (filter only, user-scoped).
        Mergeable => "MERGEABLE",
        /// Exited positions.
        Closed => "CLOSED",
    }
}

polyoxide_core::string_enum! {
    /// Sort key of `/v2/positions` (`sort_by`).
    pub enum PositionSortBy {
        /// Current value (default for `OPEN` / `REDEEMABLE`).
        CurrentValue => "CURRENT_VALUE",
        /// The effective `current_price`.
        Price => "PRICE",
        /// Number of tokens held.
        Tokens => "TOKENS",
        /// Unrealized PnL.
        UnrealizedPnl => "UNREALIZED_PNL",
        /// Realized PnL (default for `CLOSED`).
        RealizedPnl => "REALIZED_PNL",
        /// Total PnL.
        TotalPnl => "TOTAL_PNL",
        /// The row's `last_event_at`.
        Timestamp => "TIMESTAMP",
    }
}

/// One position: a holding in a single outcome token, priced and enriched
/// (`components/schemas/Position`).
///
/// The shape is the same for user-anchored (open or closed) and market-anchored
/// requests. On closed positions `current_size`, `current_value` and `unrealized_pnl` are
/// approximately zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Position {
    /// Proxy wallet holding the position.
    pub proxy_wallet: Address,
    /// The outcome token id.
    pub token_id: TokenId,
    /// The on-chain condition id.
    pub condition_id: ConditionId,
    /// The current holding, in shares.
    pub current_size: Decimal,
    /// Weighted-average entry price per share, in USDC.
    pub avg_price: Decimal,
    /// The fee-exclusive entry basis, in USDC.
    pub entry_cost_usdc: Decimal,
    /// Attributed buy-fee total, in USDC. Disclosure only: `entry_cost_usdc` is already
    /// fee-exclusive.
    pub entry_fees_usdc: Decimal,
    /// Gross (fee-inclusive) basis: `entry_cost_usdc + entry_fees_usdc`.
    pub total_cost_usdc: Decimal,
    /// Live mark per share, in USDC.
    pub current_price: Decimal,
    /// `current_size × current_price`, in USDC.
    pub current_value: Decimal,
    /// Lifetime bought shares (never the current balance; that is `current_size`).
    pub total_size: Decimal,
    /// Cumulative realized PnL, in USDC.
    pub realized_pnl: Decimal,
    /// Unrealized PnL: `current_value - entry_cost_usdc`, in USDC.
    pub unrealized_pnl: Decimal,
    /// `realized_pnl + unrealized_pnl`, in USDC.
    pub total_pnl: Decimal,
    /// `(current_value - entry_cost_usdc) / entry_cost_usdc`, as a percent.
    pub percent_pnl: Decimal,
    /// `(current_value - total_size × avg_price) / (total_size × avg_price)`, as a percent.
    pub percent_realized_pnl: Decimal,
    /// The row's actual state; can be narrower than the requested status.
    pub status: PositionStatus,
    /// Whether the market resolved and the tokens are still held (losing sides included).
    pub redeemable: bool,
    /// Whether the wallet also holds the opposite outcome, so the pair can be merged.
    pub mergeable: bool,
    /// Whether the market belongs to a neg-risk group.
    pub negative_risk: bool,
    /// Whether the market is archived.
    pub archived: bool,
    /// Market question title (empty when unenriched).
    pub title: String,
    /// Market slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Gamma event id of the parent event.
    pub event_id: EventId,
    /// Parent event slug.
    pub event_slug: String,
    /// Label of the held outcome (e.g. `Yes`).
    pub outcome: String,
    /// Index of the held outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](super::UNLABELED_OUTCOME_INDEX) means it could not be
    /// labeled.
    pub outcome_index: i32,
    /// Label of the market's other outcome.
    pub opposite_outcome: String,
    /// Token id of the market's other outcome.
    pub opposite_token_id: TokenId,
    /// Market end date; `1970-01-01` when Gamma has none.
    pub end_date: NaiveDate,
    /// The row's last economics event. The Unix epoch (`0` on the wire) when the position
    /// has no native state.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub last_event_at: DateTime<Utc>,
    /// Profile display name of the wallet.
    pub name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Profile verification badge.
    pub verified: bool,
}

polyoxide_core::string_enum! {
    /// Lifecycle state of a combo position (`status` filter and row field of
    /// `/v2/positions/combos`).
    pub enum ComboPositionStatus {
        /// Open: the superset, including still-held redeemable positions.
        Open => "OPEN",
        /// Exactly the rows whose `redeemable` flag is `true`. Must be the only status
        /// filter value.
        Redeemable => "REDEEMABLE",
        /// Partially resolved (filter value).
        Partial => "PARTIAL",
        /// Resolved as a win.
        ResolvedWin => "RESOLVED_WIN",
        /// Resolved as a loss.
        ResolvedLoss => "RESOLVED_LOSS",
        /// Resolved partially.
        ResolvedPartial => "RESOLVED_PARTIAL",
    }
}

polyoxide_core::string_enum! {
    /// Sort key of `/v2/positions/combos` (`sort_by`).
    pub enum ComboPositionSortBy {
        /// First entry time (default, except under `status=REDEEMABLE`).
        FirstEntry => "FIRST_ENTRY",
        /// Entry cost.
        EntryCost => "ENTRY_COST",
        /// Alias of [`EntryCost`](Self::EntryCost).
        CurrentValue => "CURRENT_VALUE",
        /// Last update time; pair with ascending order for incremental sync.
        Updated => "UPDATED",
    }
}

/// One combo position: a user's holding in a single combo outcome, with leg rollups
/// (`components/schemas/ComboPosition`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ComboPosition {
    /// On-chain combo condition id (`0x03`-prefixed).
    pub combo_condition_id: ConditionId,
    /// Index of the combo outcome held;
    /// [`UNLABELED_OUTCOME_INDEX`](super::UNLABELED_OUTCOME_INDEX) means unlabelable.
    pub outcome_index: i32,
    /// Label of the combo outcome held.
    pub outcome_label: String,
    /// Token id of the combo position.
    pub combo_position_id: TokenId,
    /// The holder's wallet.
    pub proxy_wallet: Address,
    /// Current holding, in shares.
    pub current_size: Decimal,
    /// Weighted-average entry price per share, in USDC.
    pub entry_avg_price_usdc: Decimal,
    /// Entry cost basis in USDC (rounded weighted-average form).
    pub entry_cost_usdc: Decimal,
    /// Exact fee-inclusive entry basis, in USDC. Do not reconstruct it as
    /// `entry_cost_usdc + entry_fees_usdc`.
    pub gross_entry_cost_usdc: Decimal,
    /// Attributed buy-fee portion of `gross_entry_cost_usdc`, in USDC.
    pub entry_fees_usdc: Decimal,
    /// Gross redemption payout received so far, in USDC (turnover, not profit).
    pub realized_payout_usdc: Decimal,
    /// Lifecycle state of the position.
    pub status: ComboPositionStatus,
    /// Whether the combo can be redeemed now.
    pub redeemable: bool,
    /// First acquisition time.
    #[serde(with = "serde_util::datetime")]
    pub first_entry_at: DateTime<Utc>,
    /// `first_entry_at` at microsecond precision (`first_entry_at_micros`); `None` on the
    /// null tail.
    #[serde(default, with = "timestamp_micros_option")]
    pub first_entry_at_micros: Option<DateTime<Utc>>,
    /// Number of legs in the combo.
    pub legs_total: i32,
    /// Legs whose markets have resolved.
    pub legs_resolved: i32,
    /// Legs still awaiting resolution.
    pub legs_pending: i32,
    /// The combo's legs, in leg order.
    pub legs: Vec<ComboLeg>,
    /// When the combo fully resolved; `None` while any leg is open.
    #[serde(default, with = "serde_util::datetime_option")]
    pub resolved_at: Option<DateTime<Utc>>,
    /// Last event touching the position.
    #[serde(with = "serde_util::datetime")]
    pub updated_at: DateTime<Utc>,
    /// `updated_at` at microsecond precision (`updated_at_micros`).
    #[serde(with = "timestamp_micros")]
    pub updated_at_micros: DateTime<Utc>,
}

/// A user's portfolio value (`/v2/value`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PortfolioValue {
    /// The wallet the value was computed for.
    pub proxy_wallet: Address,
    /// Portfolio value in USDC, rounded to 4 decimals: holdings marked to market plus
    /// non-terminal combos at cost basis. `0` for a user with no positions.
    pub value: Decimal,
}

polyoxide_core::string_enum! {
    /// Token standard of an approval pair.
    pub enum TokenStandard {
        /// An ERC-20 token (allowance-based approval).
        Erc20 => "ERC20",
        /// An ERC-1155 token (all-or-nothing operator approval).
        Erc1155 => "ERC1155",
    }
}

/// A wallet's Polygon token/operator approval state (`components/schemas/Approvals`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Approvals {
    /// The checked wallet.
    pub address: Address,
    /// EVM chain id the state was read on (137 = Polygon).
    pub chain_id: i32,
    /// When the on-chain state was read (served from a 20-second cache).
    #[serde(with = "serde_util::datetime")]
    pub checked_at: DateTime<Utc>,
    /// One row per catalog pair the wallet may need.
    pub contracts: Vec<ApprovalContract>,
}

/// One trusted approval contract (`components/schemas/ApprovalContract`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApprovalContract {
    /// Catalog identifier of this approval pair (token + spender).
    pub id: String,
    /// The product flow the approval enables (e.g. `trading`).
    pub feature: String,
    /// The token contract the approval is granted on.
    pub token: Address,
    /// The contract approved to spend or operate the token.
    pub spender: Address,
    /// Token standard of the pair.
    pub standard: TokenStandard,
    /// Whether the wallet currently grants this approval.
    pub approved: bool,
    /// Current ERC-20 allowance: `max` for an unlimited grant, else the raw integer amount
    /// as a string; `None` on ERC-1155 operator approvals.
    pub amount: Option<String>,
}

impl ApprovalContract {
    /// `true` if the ERC-20 allowance is unlimited (`amount` is `max`).
    #[must_use]
    pub fn is_unlimited(&self) -> bool {
        self.amount.as_deref() == Some("max")
    }
}

polyoxide_core::string_enum! {
    /// Window of `/v2/user-pnl` (`interval`).
    pub enum PnlInterval {
        /// The maximum window.
        Max => "max",
        /// All time.
        All => "all",
        /// One month.
        OneMonth => "1m",
        /// One week.
        OneWeek => "1w",
        /// One day (the default).
        OneDay => "1d",
        /// Twelve hours.
        TwelveHours => "12h",
        /// Six hours.
        SixHours => "6h",
    }
}

polyoxide_core::string_enum! {
    /// Output grid of `/v2/user-pnl` (`fidelity`).
    pub enum PnlFidelity {
        /// One day.
        OneDay => "1d",
        /// Eighteen hours.
        EighteenHours => "18h",
        /// Twelve hours.
        TwelveHours => "12h",
        /// Three hours.
        ThreeHours => "3h",
        /// One hour (the default).
        OneHour => "1h",
    }
}

/// A user's cumulative PnL series (`components/schemas/UserPnlSeries`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserPnlSeries {
    /// The wallet the series was computed for.
    pub proxy_wallet: Address,
    /// The window served.
    pub interval: PnlInterval,
    /// Grid step the points were synthesized on.
    pub fidelity: PnlFidelity,
    /// Grid of the underlying observations: historical observations are daily even when
    /// carried onto a finer grid.
    pub source_fidelity: PnlFidelity,
    /// Dense cumulative points on the requested grid, oldest first.
    pub points: Vec<UserPnlPoint>,
}

/// One dense cumulative PnL point, in USDC (`components/schemas/UserPnlPoint`).
///
/// Every amount is cumulative through [`timestamp`](Self::timestamp). An optional amount
/// is `None` when its source was unavailable; it never means zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserPnlPoint {
    /// Point time.
    #[serde(with = "serde_util::timestamp_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Chain block the point was observed at.
    pub source_block: i64,
    /// Realized PnL from market positions.
    pub realized_market_pnl: Decimal,
    /// Realized PnL from AMM liquidity-provision activity.
    pub realized_lp_pnl: Decimal,
    /// Realized PnL from combo positions.
    pub realized_combo_pnl: Decimal,
    /// `realized_market_pnl + realized_lp_pnl + realized_combo_pnl`.
    pub realized_pnl: Decimal,
    /// Cumulative maker-attributed fill volume, in shares.
    pub volume: Decimal,
    /// Cumulative maker-attributed fill volume, in USDC.
    pub volume_usdc: Decimal,
    /// Cumulative maker-attributed fill count.
    pub trade_count: u64,
    /// Mark-to-market of open inventory.
    pub unrealized_pnl: Option<Decimal>,
    /// `realized_pnl + unrealized_pnl`; the position-only result.
    pub position_pnl: Option<Decimal>,
    /// Compatibility chart series:
    /// `position_pnl - realized_lp_pnl + fees_charged - fees_refunded`.
    pub trade_pnl: Option<Decimal>,
    /// `realized_pnl + wallet_income`; settled economics, no marks.
    pub settled_pnl: Option<Decimal>,
    /// `position_pnl + wallet_income`; the all-in economic result.
    pub economic_pnl: Option<Decimal>,
    /// Negative cumulative fee charges.
    pub fees: Option<Decimal>,
    /// Refunds minus charges; a disclosure, not another PnL adjustment.
    pub fees_paid: Option<Decimal>,
    /// Total fees refunded.
    pub fees_refunded: Option<Decimal>,
    /// Maker-side fee rebates credited.
    pub maker_rebate: Option<Decimal>,
    /// Taker-side fee rebates credited.
    pub taker_rebate: Option<Decimal>,
    /// Reward-program income credited.
    pub reward_income: Option<Decimal>,
    /// Yield income credited.
    pub yield_income: Option<Decimal>,
    /// Referral income credited.
    pub referral_income: Option<Decimal>,
    /// `reward_income + yield_income + referral_income`.
    pub sponsored_income: Option<Decimal>,
    /// All income credited to the wallet: rebates plus reward, yield and referral income.
    pub wallet_income: Option<Decimal>,
    /// Collateral moved into the wallet.
    pub deposits: Option<Decimal>,
    /// Collateral moved out of the wallet.
    pub withdrawals: Option<Decimal>,
    /// `deposits - withdrawals`.
    pub cashflow_net: Option<Decimal>,
}

/// A wallet's profile card (`components/schemas/UserStats`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserStats {
    /// The proxy wallet these lifetime statistics describe.
    pub proxy_wallet: Address,
    /// Distinct **markets** traded (not individual trades); an exact count.
    pub trades: u64,
    /// Largest single resolved win, in USDC; `0` when the wallet has no win over $1.
    pub biggest_win: Decimal,
    /// Profile view count.
    pub views: u64,
    /// When the account joined; `None` when unknown.
    #[serde(default, with = "serde_util::timestamp_seconds_option")]
    pub join_date: Option<DateTime<Utc>>,
    /// Newest persisted cumulative all-time PnL point; `None` when the user is known but
    /// has no observation yet.
    pub all_time_pnl: Option<UserPnlPoint>,
}

/// A wallet's trading volume over a whole-day window (`components/schemas/UserVolume`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserVolume {
    /// Both-sides traded volume over the window, in shares.
    pub volume: Decimal,
    /// Both-sides cash volume over the window, in USD.
    pub volume_usdc: Decimal,
    /// Number of fills in the window.
    pub trade_count: u64,
}

impl DataClient {
    /// Lists positions for a user or a market, across the whole lifecycle
    /// (`GET /v2/positions`, cursor-paginated).
    ///
    /// Set at least one of [`user`](ListPositions::user) and
    /// [`conditions`](ListPositions::conditions).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/list-positions-for-a-user-or-market>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::data::{DataClient, PositionStatus};
    ///
    /// let data = DataClient::new()?;
    /// let page = data
    ///     .list_positions()
    ///     .user("0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748")
    ///     .status(PositionStatus::Open)
    ///     .limit(50)
    ///     .send()
    ///     .await?;
    /// for position in &page.items {
    ///     let _ = (&position.title, position.current_value, position.total_pnl);
    /// }
    /// // Pass `page.next_cursor()` to `.cursor(..)` for the next page, or use
    /// // `.into_stream()` to walk every page.
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_positions(&self) -> ListPositions {
        ListPositions {
            client: self.clone(),
            user: None,
            conditions: Vec::new(),
            limit: None,
            cursor: None,
            status: None,
            event_ids: Vec::new(),
            title: None,
            filter_type: None,
            filter_amount: None,
            include_archived: None,
            sort_by: None,
            start: None,
            end: None,
            sort_direction: None,
        }
    }

    /// Lists a user's combo positions (`GET /v2/positions/combos`, cursor-paginated).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/list-combo-positions>.
    pub fn list_combo_positions(&self, user: impl Into<Address>) -> ListComboPositions {
        ListComboPositions {
            client: self.clone(),
            user: user.into(),
            limit: None,
            cursor: None,
            conditions: Vec::new(),
            statuses: Vec::new(),
            sort_by: None,
            sort_direction: None,
            updated_after: None,
            updated_before: None,
        }
    }

    /// Gets a user's portfolio value: single-market holdings marked to market plus
    /// unresolved combo positions at cost basis (`GET /v2/value`).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/get-portfolio-value>.
    pub fn get_portfolio_value(&self, user: impl Into<Address>) -> GetPortfolioValue {
        GetPortfolioValue {
            client: self.clone(),
            user: user.into(),
            conditions: Vec::new(),
        }
    }

    /// Gets a wallet's Polygon token/operator approval state (`GET /v2/approvals`).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/get-wallet-approvals>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing or invalid address, or a known protocol
    /// contract address, is an [`Error::Api`](crate::Error::Api) with status `400`
    /// ([`ErrorCode::InvalidRequest`](super::ErrorCode::InvalidRequest)).
    pub async fn get_approvals(&self, user: impl Into<Address>) -> Result<Approvals> {
        let user = user.into();
        let mut query = Query::new();
        query.push("user", &user);
        self.fetch_data(&["v2", "approvals"], query).await
    }

    /// Gets a user's cumulative PnL series (`GET /v2/user-pnl`).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/get-a-users-pnl-series>.
    pub fn get_user_pnl(&self, user: impl Into<Address>) -> GetUserPnl {
        GetUserPnl {
            client: self.clone(),
            user: user.into(),
            interval: None,
            fidelity: None,
        }
    }

    /// Gets a wallet's profile stats (`GET /v2/user-stats`).
    ///
    /// Returns `Ok(None)` when the wallet is not a known user (`data: null`). A known user
    /// that never traded returns a row of zeros instead.
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/get-a-users-profile-stats>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); a missing or invalid address, or a known protocol
    /// contract address, is an [`Error::Api`](crate::Error::Api) with status `400`.
    pub async fn get_user_stats(&self, user: impl Into<Address>) -> Result<Option<UserStats>> {
        let user = user.into();
        let mut query = Query::new();
        query.push("user", &user);
        self.fetch_data(&["v2", "user-stats"], query).await
    }

    /// Gets a wallet's trading volume over a window (`GET /v2/user-volume`).
    ///
    /// See <https://docs.polymarket.com/api-reference/wallet/get-a-users-trading-volume>.
    pub fn get_user_volume(&self, user: impl Into<Address>) -> GetUserVolume {
        GetUserVolume {
            client: self.clone(),
            user: user.into(),
            start: None,
            end: None,
        }
    }
}

/// Request builder for [`DataClient::list_positions`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListPositions {
    client: DataClient,
    user: Option<Address>,
    conditions: Vec<ConditionId>,
    limit: Option<u32>,
    cursor: Option<String>,
    status: Option<PositionStatus>,
    event_ids: Vec<EventId>,
    title: Option<String>,
    filter_type: Option<FilterType>,
    filter_amount: Option<Decimal>,
    include_archived: Option<bool>,
    sort_by: Option<PositionSortBy>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    sort_direction: Option<SortDirection>,
}

impl ListPositions {
    /// The wallet to anchor on (`user`).
    pub fn user(mut self, user: impl Into<Address>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Condition ids (`condition`, at most 20 distinct values). With [`user`](Self::user),
    /// narrows that user's positions; without it, anchors on the market's holders and
    /// exactly one id is accepted.
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
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

    /// Lifecycle filter (`status`, default [`PositionStatus::Open`]).
    pub fn status(mut self, status: PositionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Gamma event ids (`event_id`, at most 20 distinct values). Requires
    /// [`user`](Self::user).
    pub fn event_ids<I>(mut self, event_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<EventId>,
    {
        self.event_ids = collect_ids(event_ids);
        self
    }

    /// Case-insensitive market-title substring filter (`title`, at most 200 characters).
    /// SQL `LIKE` wildcards (`%`, `_`) keep their meaning.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Unit of [`filter_amount`](Self::filter_amount) (`filter_type`, default
    /// [`FilterType::Tokens`]).
    pub fn filter_type(mut self, filter_type: FilterType) -> Self {
        self.filter_type = Some(filter_type);
        self
    }

    /// Minimum current holding (`filter_amount`): shares for [`FilterType::Tokens`]
    /// (default `0.1`), USDC `current_value` for [`FilterType::Cash`].
    pub fn filter_amount(mut self, filter_amount: Decimal) -> Self {
        self.filter_amount = Some(filter_amount);
        self
    }

    /// Also include positions on archived markets (`include_archived`, default `false`).
    /// Cannot be combined with [`PositionStatus::Closed`].
    pub fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = Some(include_archived);
        self
    }

    /// Sort key (`sort_by`); the default follows the status.
    pub fn sort_by(mut self, sort_by: PositionSortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// Inclusive lower bound on `last_event_at` (`start`). Any bound excludes positions
    /// without native state.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Inclusive upper bound on `last_event_at` (`end`).
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Sort direction (`sort_direction`, default [`SortDirection::Desc`]).
    pub fn sort_direction(mut self, sort_direction: SortDirection) -> Self {
        self.sort_direction = Some(sort_direction);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        if self.user.is_none() {
            if self.conditions.is_empty() {
                return Err(ValidationError::new(
                    "user",
                    "at least one of `user` or `condition` is required",
                )
                .into());
            }
            if distinct(&self.conditions) > 1 {
                return Err(ValidationError::new(
                    "condition",
                    "without `user`, exactly one condition id is accepted",
                )
                .into());
            }
            if !self.event_ids.is_empty() {
                return Err(ValidationError::new("event_id", "requires `user`").into());
            }
            if self.status == Some(PositionStatus::RedeemableLost) {
                return Err(
                    ValidationError::new("status", "REDEEMABLE_LOST requires `user`").into(),
                );
            }
        }
        check_list("condition", &self.conditions)?;
        check_list("event_id", &self.event_ids)?;
        check_limit(self.limit, MAX_POSITIONS_LIMIT)?;
        if let Some(title) = &self.title {
            let chars = title.chars().count();
            if chars > MAX_TITLE_CHARS {
                return Err(ValidationError::new(
                    "title",
                    format!("must be at most {MAX_TITLE_CHARS} characters, got {chars}"),
                )
                .into());
            }
        }
        if self.include_archived == Some(true) && self.status == Some(PositionStatus::Closed) {
            return Err(ValidationError::new(
                "include_archived",
                "cannot be combined with status CLOSED",
            )
            .into());
        }

        let mut q = Query::new();
        q.push_opt("user", self.user.as_ref())
            .push_csv("condition", &self.conditions)
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_opt("status", self.status.as_ref())
            .push_csv("event_id", &self.event_ids)
            .push_opt("title", self.title.as_deref())
            .push_opt("filter_type", self.filter_type.as_ref())
            .push_opt("filter_amount", self.filter_amount)
            .push_opt("include_archived", self.include_archived)
            .push_opt("sort_by", self.sort_by.as_ref())
            .push_opt("start", self.start.map(|t| t.timestamp()))
            .push_opt("end", self.end.map(|t| t.timestamp()))
            .push_opt("sort_direction", self.sort_direction.as_ref());
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if a documented constraint is
    /// violated (no `user`/`condition` anchor, more than 20 condition or event ids, several
    /// condition ids without `user`, `event_id` or `REDEEMABLE_LOST` without `user`,
    /// `limit` above 1000, `title` over 200 characters, or `include_archived` with
    /// `CLOSED`); otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<Position>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(POSITIONS, query).await
    }

    /// Streams every position from the configured cursor onwards, fetching pages lazily.
    ///
    /// Every page re-sends the same `user`/`condition` anchor and filters with the cursor,
    /// as the API requires (a bare cursor is rejected, and `title` and the `start`/`end`
    /// window are not carried by the cursor). The stream ends when `next_cursor` is
    /// `null`.
    pub fn into_stream(self) -> impl Stream<Item = Result<Position>> + Send + 'static {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, POSITIONS, start, move |cursor| self.query(cursor))
    }
}

/// Request builder for [`DataClient::list_combo_positions`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboPositions {
    client: DataClient,
    user: Address,
    limit: Option<u32>,
    cursor: Option<String>,
    conditions: Vec<ConditionId>,
    statuses: Vec<ComboPositionStatus>,
    sort_by: Option<ComboPositionSortBy>,
    sort_direction: Option<SortDirection>,
    updated_after: Option<DateTime<Utc>>,
    updated_before: Option<DateTime<Utc>>,
}

impl ListComboPositions {
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

    /// Lifecycle filter (`status`, comma-separated). [`ComboPositionStatus::Redeemable`]
    /// must be the only value. Default (none): the held-visibility listing.
    pub fn statuses(mut self, statuses: impl IntoIterator<Item = ComboPositionStatus>) -> Self {
        self.statuses = statuses.into_iter().collect();
        self
    }

    /// Sort key (`sort_by`, default [`ComboPositionSortBy::FirstEntry`]).
    pub fn sort_by(mut self, sort_by: ComboPositionSortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// Sort direction (`sort_direction`, default [`SortDirection::Desc`]).
    pub fn sort_direction(mut self, sort_direction: SortDirection) -> Self {
        self.sort_direction = Some(sort_direction);
        self
    }

    /// Incremental-sync watermark: inclusive lower bound on `updated_at`
    /// (`updated_after`). Without a status filter, switches to the mirror-complete sync
    /// view.
    pub fn updated_after(mut self, updated_after: DateTime<Utc>) -> Self {
        self.updated_after = Some(updated_after);
        self
    }

    /// Incremental-sync watermark: inclusive upper bound on `updated_at`
    /// (`updated_before`); must not precede [`updated_after`](Self::updated_after).
    pub fn updated_before(mut self, updated_before: DateTime<Utc>) -> Self {
        self.updated_before = Some(updated_before);
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        check_limit(self.limit, MAX_POSITIONS_LIMIT)?;
        check_list("condition", &self.conditions)?;
        if self.statuses.contains(&ComboPositionStatus::Redeemable)
            && self
                .statuses
                .iter()
                .any(|s| *s != ComboPositionStatus::Redeemable)
        {
            return Err(
                ValidationError::new("status", "REDEEMABLE must be the only status").into(),
            );
        }
        for (name, bound) in [
            ("updated_after", self.updated_after),
            ("updated_before", self.updated_before),
        ] {
            if bound.is_some_and(|t| t.timestamp() < 0) {
                return Err(ValidationError::new(name, "must not be negative").into());
            }
        }
        if let (Some(after), Some(before)) = (self.updated_after, self.updated_before)
            && before < after
        {
            return Err(
                ValidationError::new("updated_before", "must not precede `updated_after`").into(),
            );
        }

        let mut q = Query::new();
        q.push("user", &self.user)
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_csv("condition", &self.conditions)
            .push_csv("status", &self.statuses)
            .push_opt("sort_by", self.sort_by.as_ref())
            .push_opt("sort_direction", self.sort_direction.as_ref())
            .push_opt("updated_after", self.updated_after.map(|t| t.timestamp()))
            .push_opt("updated_before", self.updated_before.map(|t| t.timestamp()));
        Ok(q)
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is above 1000,
    /// more than 20 condition ids are given, `REDEEMABLE` is combined with other statuses,
    /// or the sync watermarks are negative or inverted; otherwise see
    /// [`Error`](crate::Error).
    pub async fn send(self) -> Result<Page<ComboPosition>> {
        let query = self.query(self.cursor.as_deref())?;
        self.client.fetch_page(COMBO_POSITIONS, query).await
    }

    /// Streams every combo position from the configured cursor onwards, fetching pages
    /// lazily.
    ///
    /// Every page re-sends `user` (always required on this route) and the same filters
    /// with the cursor. The stream ends when `next_cursor` is `null`.
    pub fn into_stream(self) -> impl Stream<Item = Result<ComboPosition>> + Send + 'static {
        let client = self.client.clone();
        let start = self.cursor.clone();
        page_stream(client, COMBO_POSITIONS, start, move |cursor| {
            self.query(cursor)
        })
    }
}

/// Request builder for [`DataClient::get_portfolio_value`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetPortfolioValue {
    client: DataClient,
    user: Address,
    conditions: Vec<ConditionId>,
}

impl GetPortfolioValue {
    /// Scopes the single-market term to these condition ids (`condition`, at most 20
    /// distinct values). Any condition filter excludes the portfolio-level combo term.
    pub fn conditions<I>(mut self, conditions: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<ConditionId>,
    {
        self.conditions = collect_ids(conditions);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if more than 20 condition ids
    /// are given; otherwise see [`Error`](crate::Error).
    pub async fn send(self) -> Result<PortfolioValue> {
        check_list("condition", &self.conditions)?;
        let mut query = Query::new();
        query
            .push("user", &self.user)
            .push_csv("condition", &self.conditions);
        self.client.fetch_data(&["v2", "value"], query).await
    }
}

/// Request builder for [`DataClient::get_user_pnl`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetUserPnl {
    client: DataClient,
    user: Address,
    interval: Option<PnlInterval>,
    fidelity: Option<PnlFidelity>,
}

impl GetUserPnl {
    /// The window (`interval`, default [`PnlInterval::OneDay`]).
    pub fn interval(mut self, interval: PnlInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// The output grid (`fidelity`, default [`PnlFidelity::OneHour`]).
    pub fn fidelity(mut self, fidelity: PnlFidelity) -> Self {
        self.fidelity = Some(fidelity);
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); an invalid wallet, interval or fidelity is an
    /// [`Error::Api`](crate::Error::Api) with status `400`.
    pub async fn send(self) -> Result<UserPnlSeries> {
        let mut query = Query::new();
        query
            .push("user", &self.user)
            .push_opt("interval", self.interval.as_ref())
            .push_opt("fidelity", self.fidelity.as_ref());
        self.client.fetch_data(&["v2", "user-pnl"], query).await
    }
}

/// Request builder for [`DataClient::get_user_volume`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetUserVolume {
    client: DataClient,
    user: Address,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

impl GetUserVolume {
    /// Inclusive window start (`start`), floored to its UTC day. Omitted: no lower bound.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Inclusive window end (`end`), floored to its UTC day. Omitted: no upper bound.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Sends the request. A wallet with no trades in the window (or an inverted window)
    /// yields zeros.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); an empty or invalid `user` is an
    /// [`Error::Api`](crate::Error::Api) with status `400`.
    pub async fn send(self) -> Result<UserVolume> {
        let mut query = Query::new();
        query
            .push("user", &self.user)
            .push_opt("start", self.start.map(|t| t.timestamp()))
            .push_opt("end", self.end.map(|t| t.timestamp()));
        self.client.fetch_data(&["v2", "user-volume"], query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::Envelope;

    /// The example response in `docs/api-reference/data-api/overview.md` ("Make a First
    /// Request") trims the row; the remaining required fields of
    /// `components/schemas/Position` (`docs/specs/data-v2-openapi.json`) are filled with
    /// values of the documented types.
    const POSITION_PAGE: &str = r#"{
      "data": [
        {
          "proxy_wallet": "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748",
          "condition_id": "0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79",
          "token_id": "31974447302330162086995746309500877260929998201718217388109724292047967921664",
          "outcome": "Yes",
          "title": "Will ŠK Slovan Bratislava win on 2026-08-19?",
          "status": "REDEEMABLE",
          "current_size": 86780.64,
          "avg_price": 0.5203,
          "entry_cost_usdc": 45159.4653,
          "current_value": 0.0,
          "realized_pnl": -1082.5533,
          "unrealized_pnl": -45159.4653,
          "entry_fees_usdc": 0.0,
          "total_cost_usdc": 45159.4653,
          "current_price": 0.0,
          "total_size": 86780.64,
          "total_pnl": -46242.0186,
          "percent_pnl": -100.0,
          "percent_realized_pnl": -100.0,
          "redeemable": true,
          "mergeable": false,
          "negative_risk": false,
          "archived": false,
          "slug": "slovan-bratislava",
          "icon": "https://example.com/icon.png",
          "event_id": "12345",
          "event_slug": "slovan-bratislava-2026-08-19",
          "outcome_index": 0,
          "opposite_outcome": "No",
          "opposite_token_id": "1",
          "end_date": "2026-08-19",
          "last_event_at": 1787097600,
          "name": "",
          "profile_image": "",
          "verified": false
        }
      ],
      "pagination": {
        "limit": 1,
        "offset": 0,
        "has_more": true,
        "next_cursor": "eyJkYXRhIjp7InR5cGUiOiJwb3NpdGlvbnMi…"
      }
    }"#;

    #[test]
    fn deserializes_position_page() {
        let page: Page<Position> = serde_json::from_str(POSITION_PAGE).unwrap();
        let p = page.items.first().unwrap();
        assert_eq!(p.proxy_wallet, "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748");
        assert_eq!(p.status, PositionStatus::Redeemable);
        assert_eq!(p.current_size.to_string(), "86780.64");
        assert_eq!(p.entry_cost_usdc.to_string(), "45159.4653");
        assert_eq!(p.realized_pnl.to_string(), "-1082.5533");
        assert_eq!(p.end_date, NaiveDate::from_ymd_opt(2026, 8, 19).unwrap());
        assert_eq!(p.last_event_at.timestamp(), 1_787_097_600);
        assert_eq!(p.event_id, EventId::from("12345"));
        assert!(page.has_more());

        // Every field of the schema's `required` list is required here too.
        let mut value: serde_json::Value = serde_json::from_str(POSITION_PAGE).unwrap();
        value["data"][0].as_object_mut().unwrap().remove("verified");
        assert!(serde_json::from_value::<Page<Position>>(value).is_err());
    }

    /// Field names and types from `components/schemas/ComboPosition`, `ComboLeg`,
    /// `ComboLegMarket` and `ComboLegEvent` in `docs/specs/data-v2-openapi.json`.
    #[test]
    fn deserializes_combo_position() {
        let json = r#"{
          "combo_condition_id": "0x03aa",
          "outcome_index": 1,
          "outcome_label": "Yes",
          "combo_position_id": "123",
          "proxy_wallet": "0xabc",
          "current_size": 10.5,
          "entry_avg_price_usdc": 0.25,
          "entry_cost_usdc": 2.625,
          "gross_entry_cost_usdc": 2.65,
          "entry_fees_usdc": 0.025,
          "realized_payout_usdc": 0,
          "status": "OPEN",
          "redeemable": false,
          "first_entry_at": "2026-08-19T10:00:00Z",
          "first_entry_at_micros": null,
          "legs_total": 1,
          "legs_resolved": 0,
          "legs_pending": 1,
          "legs": [{
            "leg_index": 0,
            "leg_position_id": "456",
            "leg_condition_id": "0xdef",
            "leg_outcome_index": 999,
            "leg_outcome_label": "Over",
            "leg_status": "OPEN",
            "leg_current_price": 0.5,
            "leg_resolved_at": null,
            "market": {
              "market_id": "789",
              "slug": "m",
              "title": "Over 2.5",
              "outcome": "Over",
              "image_url": "",
              "icon_url": "",
              "category": "sports",
              "subcategory": "soccer",
              "tags": [],
              "end_date": "",
              "event": {
                "event_id": "42",
                "event_slug": "e",
                "event_title": "E",
                "event_image": ""
              },
              "line": 2.5,
              "outcomes": ["Over", "Under"],
              "sports_market_type": "totals"
            }
          }],
          "resolved_at": null,
          "updated_at": "2026-08-19T10:00:01.5Z",
          "updated_at_micros": 1787133601500000
        }"#;
        let combo: ComboPosition = serde_json::from_str(json).unwrap();
        assert_eq!(combo.status, ComboPositionStatus::Open);
        assert_eq!(combo.first_entry_at_micros, None);
        assert_eq!(combo.updated_at, combo.updated_at_micros);
        let leg = combo.legs.first().unwrap();
        assert_eq!(leg.leg_outcome_index, crate::data::UNLABELED_OUTCOME_INDEX);
        assert_eq!(leg.market.end_date, None);
        assert_eq!(leg.market.line, Some(Decimal::new(25, 1)));
        assert_eq!(leg.market.question, None);
        assert_eq!(leg.market.event.event_id, EventId::from("42"));
    }

    /// `/v2/user-stats` documents `data: null` for an unknown wallet
    /// (`components/schemas/Envelope_Option_UserStats`).
    #[test]
    fn deserializes_user_stats_envelope() {
        let none: Envelope<Option<UserStats>> = serde_json::from_str(r#"{"data":null}"#).unwrap();
        assert_eq!(none.data, None);

        let json = r#"{"data":{
            "proxy_wallet": "0xabc",
            "trades": 12,
            "biggest_win": 0,
            "views": 3,
            "join_date": null,
            "all_time_pnl": {
              "timestamp": 1700000000,
              "source_block": 123,
              "realized_market_pnl": 1.5,
              "realized_lp_pnl": 0,
              "realized_combo_pnl": 0,
              "realized_pnl": 1.5,
              "volume": 10,
              "volume_usdc": 5,
              "trade_count": 2,
              "unrealized_pnl": null,
              "fees": -0.1
            }
        }}"#;
        let stats = serde_json::from_str::<Envelope<Option<UserStats>>>(json)
            .unwrap()
            .data
            .unwrap();
        assert_eq!(stats.trades, 12);
        assert_eq!(stats.join_date, None);
        let pnl = stats.all_time_pnl.unwrap();
        assert_eq!(pnl.timestamp.timestamp(), 1_700_000_000);
        assert_eq!(pnl.unrealized_pnl, None);
        assert_eq!(pnl.fees, Some(Decimal::new(-1, 1)));
        assert_eq!(pnl.deposits, None);
    }

    /// Field names and types from `components/schemas/Approvals` and `ApprovalContract`.
    #[test]
    fn deserializes_approvals() {
        let json = r#"{
            "address": "0xabc",
            "chain_id": 137,
            "checked_at": "2026-10-01T12:00:00Z",
            "contracts": [
              {"id": "usdc-exchange", "feature": "trading", "token": "0x1", "spender": "0x2",
               "standard": "ERC20", "approved": true, "amount": "max"},
              {"id": "ctf-exchange", "feature": "trading", "token": "0x3", "spender": "0x2",
               "standard": "ERC1155", "approved": false}
            ]
        }"#;
        let approvals: Approvals = serde_json::from_str(json).unwrap();
        assert_eq!(approvals.chain_id, 137);
        let [erc20, erc1155] = approvals.contracts.as_slice() else {
            panic!("expected two contracts")
        };
        assert!(erc20.is_unlimited());
        assert_eq!(erc20.standard, TokenStandard::Erc20);
        assert_eq!(erc1155.amount, None);
        assert!(!erc1155.is_unlimited());
    }

    #[test]
    fn positions_query_validation() {
        let client = DataClient::new().unwrap();
        let err = client.list_positions().query(None).unwrap_err();
        assert!(matches!(err, polyoxide_core::Error::Validation(_)), "{err}");
        assert!(
            client
                .list_positions()
                .conditions(["0x1", "0x2"])
                .query(None)
                .is_err()
        );
        assert!(
            client
                .list_positions()
                .user("0xabc")
                .conditions(["0x1", "0x2"])
                .query(None)
                .is_ok()
        );
        assert!(
            client
                .list_positions()
                .user("0xabc")
                .title("x".repeat(201))
                .query(None)
                .is_err()
        );
        assert!(
            client
                .list_positions()
                .user("0xabc")
                .status(PositionStatus::Closed)
                .include_archived(true)
                .query(None)
                .is_err()
        );
        let q = client
            .list_positions()
            .user("0xabc")
            .conditions(["0x1", "0x2"])
            .filter_amount(Decimal::new(5, 1))
            .start(DateTime::from_timestamp(1_700_000_000, 0).unwrap())
            .query(Some("c"))
            .unwrap();
        assert_eq!(
            q.to_string(),
            "user=0xabc&condition=0x1%2C0x2&cursor=c&filter_amount=0.5&start=1700000000"
        );
    }

    #[test]
    fn combo_positions_query_validation() {
        let client = DataClient::new().unwrap();
        assert!(
            client
                .list_combo_positions("0xabc")
                .statuses([ComboPositionStatus::Redeemable, ComboPositionStatus::Open])
                .query(None)
                .is_err()
        );
        let after = DateTime::from_timestamp(200, 0).unwrap();
        let before = DateTime::from_timestamp(100, 0).unwrap();
        assert!(
            client
                .list_combo_positions("0xabc")
                .updated_after(after)
                .updated_before(before)
                .query(None)
                .is_err()
        );
        let q = client
            .list_combo_positions("0xabc")
            .statuses([
                ComboPositionStatus::ResolvedWin,
                ComboPositionStatus::ResolvedLoss,
            ])
            .query(None)
            .unwrap();
        assert_eq!(q.get("status"), Some("RESOLVED_WIN,RESOLVED_LOSS"));
    }
}
