//! Data API v2 client (`https://data-api.polymarket.com`, routes under `/v2`).
//!
//! Covers wallet portfolios ([`wallet`](#wallet)), trade and activity feeds
//! ([`feeds`](#feeds)), per-market state ([`markets`](#markets)), ranked boards
//! ([`boards`](#boards)) and data freshness. Every route is public: no authentication is
//! needed. Start from [`DataClient`].
//!
//! See <https://docs.polymarket.com/api-reference/data-api/overview>.
//!
//! # Conventions
//!
//! - **Envelope.** Every response wraps its payload in `data`; the client unwraps it.
//!   Non-paginated routes return the payload directly. A documented miss is `data: null`
//!   (returned as `Ok(None)`, e.g. [`DataClient::get_user_stats`]) or an empty list,
//!   never an error.
//! - **Cursor pagination.** Paginated routes return a [`Page`] (rows plus [`Pagination`]).
//!   There is no `offset` request parameter (the API rejects it, and this client never
//!   sends it); follow [`Page::next_cursor`] with the request's `cursor(..)` setter until
//!   it is `None`, or use `into_stream()` to walk every page lazily. `has_more` is exact,
//!   so a short or empty page does not end the walk. `limit` only sizes the first page; a
//!   cursor carries its own page size. Streams re-send the same filters on every page, as
//!   the API requires for the feeds, `/v2/holders` (`condition`) and
//!   `/v2/positions/combos` (`user`).
//! - **Units.** Bare `volume` and `size` values are outcome **shares**; fields suffixed
//!   `_usdc` are USD amounts; volumes prefixed `taker_` count one side of each trade. All
//!   amounts are [`Decimal`](crate::Decimal)s.
//! - **Sentinels.** `outcome_index` [`UNLABELED_OUTCOME_INDEX`] (`999`) means the outcome
//!   could not be labeled. A missing or `null` numeric field means *unavailable*, never
//!   zero, and is modelled as `None`.
//! - **Identifiers.** Condition ids are [`ConditionId`](crate::types::ConditionId)s,
//!   outcome token ids [`TokenId`](crate::types::TokenId)s, wallets
//!   [`Address`](crate::types::Address)es, and Gamma ids [`MarketId`] / [`EventId`].
//!   Comma-separated list parameters accept at most [`MAX_LIST_VALUES`] (20) distinct
//!   values; larger lists are rejected client-side with
//!   [`Error::Validation`](crate::Error::Validation).
//! - **Time windows.** `start` / `end` style setters take a
//!   [`DateTime<Utc>`](chrono::DateTime) and are sent as epoch seconds. How an omitted
//!   bound behaves differs by route; see each setter.
//! - **Errors.** Error bodies carry `error`, `code`, `retryable`, `trace_id` and, for
//!   validation failures, `parameter`; they are exposed by [`ApiError`](crate::ApiError).
//!   Use [`ErrorCode::from_error`] to branch on the typed code. `429` and retryable `503`
//!   responses carry `Retry-After` ([`Error::retry_after`](crate::Error::retry_after)),
//!   and every response carries an `x-trace-id` header
//!   ([`Error::trace_id`](crate::Error::trace_id)).
//!
//! # Endpoints
//!
//! ## Wallet
//!
//! [`list_positions`](DataClient::list_positions),
//! [`list_combo_positions`](DataClient::list_combo_positions),
//! [`get_portfolio_value`](DataClient::get_portfolio_value),
//! [`get_approvals`](DataClient::get_approvals),
//! [`get_user_pnl`](DataClient::get_user_pnl),
//! [`get_user_stats`](DataClient::get_user_stats),
//! [`get_user_volume`](DataClient::get_user_volume).
//!
//! ## Feeds
//!
//! [`list_trades`](DataClient::list_trades), [`list_activity`](DataClient::list_activity),
//! [`list_combo_activity`](DataClient::list_combo_activity).
//!
//! ## Markets
//!
//! [`list_holders`](DataClient::list_holders),
//! [`get_open_interest`](DataClient::get_open_interest),
//! [`get_live_volume`](DataClient::get_live_volume),
//! [`get_prices_history`](DataClient::get_prices_history),
//! [`get_resolutions`](DataClient::get_resolutions).
//!
//! ## Boards
//!
//! [`get_leaderboard`](DataClient::get_leaderboard),
//! [`get_leaderboard_standing`](DataClient::get_leaderboard_standing),
//! [`list_biggest_winners`](DataClient::list_biggest_winners),
//! [`get_builders_leaderboard`](DataClient::get_builders_leaderboard),
//! [`get_builders_volume`](DataClient::get_builders_volume).
//!
//! ## Service
//!
//! [`get_status`](DataClient::get_status).
//!
//! # Example
//!
//! ```no_run
//! # async fn run() -> polyoxide::Result<()> {
//! use futures_util::TryStreamExt as _;
//! use polyoxide::data::DataClient;
//!
//! let data = DataClient::new()?;
//! let wallet = "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748";
//! let value = data.get_portfolio_value(wallet).send().await?;
//! let positions: Vec<_> = data
//!     .list_positions()
//!     .user(wallet)
//!     .into_stream()
//!     .try_collect()
//!     .await?;
//! # let _ = (value, positions);
//! # Ok(())
//! # }
//! ```

mod boards;
mod client;
mod feeds;
mod markets;
mod service;
mod types;
mod wallet;

pub use boards::{
    BiggestWinner, BuilderCode, BuilderStanding, BuilderVolumePoint, GetBuildersLeaderboard,
    GetBuildersVolume, GetLeaderboard, GetLeaderboardStanding, LeaderboardEntry, LeaderboardSortBy,
    LeaderboardUserEntry, ListBiggestWinners, WinKind,
};
pub use client::{DataClient, DataClientBuilder};
pub use feeds::{
    Activity, ActivitySide, ActivitySortBy, ActivityType, ComboActivity, ComboActivityType,
    ListActivity, ListComboActivity, ListTrades, Trade,
};
pub use markets::{
    ConditionVolume, GetOpenInterest, GetPricesHistory, Holder, HolderGroup, ListHolders,
    LiveVolume, OpenInterest, PriceHistoryInterval, PricePoint, Reporter, Resolution,
    ResolutionMarketType, ResolutionSelector, ResolutionSource, ResolutionStatus,
    SettlementTimeBasis,
};
pub use service::{
    CursorLag, IngestionFreshness, ServiceStatus, ServingFreshness, ServingMechanism,
    ServingMechanismName,
};
pub use types::{
    ComboLeg, ComboLegEvent, ComboLegMarket, ComboLegStatus, ErrorCode, EventId, FilterType,
    MAX_LIST_VALUES, MarketId, Page, Pagination, QuestionId, SortDirection, TimePeriod,
    UNLABELED_OUTCOME_INDEX,
};
pub use wallet::{
    ApprovalContract, Approvals, ComboPosition, ComboPositionSortBy, ComboPositionStatus,
    GetPortfolioValue, GetUserPnl, GetUserVolume, ListComboPositions, ListPositions, PnlFidelity,
    PnlInterval, PortfolioValue, Position, PositionSortBy, PositionStatus, TokenStandard,
    UserPnlPoint, UserPnlSeries, UserStats, UserVolume,
};
