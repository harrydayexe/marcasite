//! Data API v2 client (`https://data-api.polymarket.com`, routes under `/v2`).
//!
//! Covers wallet portfolios, trade and activity feeds, per-market state, ranked boards and
//! data freshness. Every route is public: no authentication is needed. Start from
//! [`DataClient`].
//!
//! See <https://docs.polymarket.com/api-reference/data-api/overview>.
//!
//! # Endpoints
//!
//! | Endpoint | Method | Returns |
//! |---|---|---|
//! | **Wallet** | | |
//! | `GET /v2/positions` | [`list_positions`](DataClient::list_positions) | [`Page`]`<`[`Position`]`>` |
//! | `GET /v2/positions/combos` | [`list_combo_positions`](DataClient::list_combo_positions) | [`Page`]`<`[`ComboPosition`]`>` |
//! | `GET /v2/value` | [`get_portfolio_value`](DataClient::get_portfolio_value) | [`PortfolioValue`] |
//! | `GET /v2/approvals` | [`get_approvals`](DataClient::get_approvals) | [`Approvals`] |
//! | `GET /v2/user-pnl` | [`get_user_pnl`](DataClient::get_user_pnl) | [`UserPnlSeries`] |
//! | `GET /v2/user-stats` | [`get_user_stats`](DataClient::get_user_stats) | `Option<`[`UserStats`]`>` |
//! | `GET /v2/user-volume` | [`get_user_volume`](DataClient::get_user_volume) | [`UserVolume`] |
//! | **Feeds** | | |
//! | `GET /v2/trades` | [`list_trades`](DataClient::list_trades) | [`Page`]`<`[`Trade`]`>` |
//! | `GET /v2/activity` | [`list_activity`](DataClient::list_activity) | [`Page`]`<`[`Activity`]`>` |
//! | `GET /v2/activity/combos` | [`list_combo_activity`](DataClient::list_combo_activity) | [`Page`]`<`[`ComboActivity`]`>` |
//! | **Markets** | | |
//! | `GET /v2/holders` | [`list_holders`](DataClient::list_holders) | [`Page`]`<`[`HolderGroup`]`>` |
//! | `GET /v2/oi` | [`get_open_interest`](DataClient::get_open_interest) | `Vec<`[`OpenInterest`]`>` |
//! | `GET /v2/live-volume` | [`get_live_volume`](DataClient::get_live_volume) | [`LiveVolume`] |
//! | `GET /v2/prices-history` | [`list_prices_history`](DataClient::list_prices_history) | [`Page`]`<`[`PricePoint`]`>` |
//! | `GET /v2/resolutions` | [`get_resolutions`](DataClient::get_resolutions) | `Vec<`[`Resolution`]`>` |
//! | **Boards** | | |
//! | `GET /v2/leaderboard` | [`list_leaderboard`](DataClient::list_leaderboard) | [`Page`]`<`[`LeaderboardEntry`]`>` |
//! | `GET /v2/leaderboard?user=` | [`get_leaderboard_standing`](DataClient::get_leaderboard_standing) | `Option<`[`LeaderboardUserEntry`]`>` |
//! | `GET /v2/biggest-winners` | [`list_biggest_winners`](DataClient::list_biggest_winners) | [`Page`]`<`[`BiggestWinner`]`>` |
//! | `GET /v2/builders/leaderboard` | [`list_builders_leaderboard`](DataClient::list_builders_leaderboard) | [`Page`]`<`[`BuilderStanding`]`>` |
//! | `GET /v2/builders/volume` | [`get_builders_volume`](DataClient::get_builders_volume) | `Vec<`[`BuilderVolumePoint`]`>` |
//! | **Service** | | |
//! | `GET /v2/status` | [`get_status`](DataClient::get_status) | [`ServiceStatus`] |
//!
//! Methods named `list_*` are cursor-paginated: their builders have a `cursor(..)` setter
//! and an `into_stream()`. Methods named `get_*` return a single resource or a fixed
//! series.
//!
//! # Conventions
//!
//! - **Envelope.** Every response wraps its payload in `data`; the client unwraps it.
//!   Non-paginated routes return the payload directly. A documented miss is `data: null`
//!   (returned as `Ok(None)`, e.g. [`DataClient::get_user_stats`]) or an empty list,
//!   never an error.
//! - **Cursor pagination.** Paginated routes return a [`Page`]: its rows
//!   ([`Page::items`]), the raw [`Pagination`] envelope, and the response's `x-trace-id`
//!   ([`Page::trace_id`]). There is no `offset` request parameter (the API rejects it,
//!   and this client never sends it). Follow [`Page::next_cursor`] with the request's
//!   `cursor(..)` setter until it is `None`, or use `into_stream()` to walk every page
//!   lazily. `has_more` is exact, so a short or empty page does not end the walk, and a
//!   page with `has_more: false` ends it even if it carries a cursor. Streams also stop if
//!   the server hands back a cursor already visited during the walk.
//! - **Re-sent parameters.** Streams re-send every parameter on every page. The API
//!   requires it on the feeds (`/v2/trades`, `/v2/activity`, `/v2/activity/combos`, whose
//!   cursors carry only their seek anchor), `/v2/positions` (the `user`/`condition`
//!   anchor, `title` and the `start`/`end` window), `/v2/positions/combos` (`user`) and
//!   `/v2/holders` (`condition`); elsewhere restating the values the cursor was minted on
//!   is allowed. `limit` is re-sent too, which is harmless: it only sizes the first page,
//!   and once a cursor is supplied the cursor's own page size wins.
//! - **Units.** Bare `volume` and `size` values are outcome **shares**; fields suffixed
//!   `_usdc` are USD amounts; volumes prefixed `taker_` count one side of each trade. All
//!   amounts are [`Decimal`](crate::Decimal)s; they are JSON numbers on the wire and
//!   serialize back as JSON numbers.
//! - **Sentinels.** Documented sentinels are kept as served and documented on each field:
//!   `outcome_index` [`UNLABELED_OUTCOME_INDEX`] (`999`) means the outcome could not be
//!   labeled, [`Position::last_event_at`] `0` means no native state,
//!   [`Position::end_date`] `1970-01-01` means Gamma has none, and
//!   [`BiggestWinner::event_id`] `0` marks a combo row. Strings the API serves as `""`
//!   when absent are `None` and serialize back as `""`. A missing or `null` numeric field
//!   means *unavailable*, never zero, and is modelled as `None`.
//! - **Identifiers.** Condition ids are [`ConditionId`]s, outcome token ids [`TokenId`]s,
//!   wallets [`Address`]es, and Gamma ids [`MarketId`] / [`EventId`]. Before sending, the
//!   client checks the formats the docs state: condition ids (combo condition ids
//!   included) and question ids must be `0x` followed by 64 hex digits, `user` must be an
//!   EVM address on `/v2/approvals` and `/v2/user-stats` and non-empty elsewhere, and
//!   event ids must be integers on `/v2/live-volume` and positive integers on
//!   `/v2/resolutions`. Comma-separated list parameters accept at most
//!   [`MAX_LIST_VALUES`] (20) distinct values; duplicates are sent once, and a value
//!   containing a comma is rejected. Violations are
//!   [`Error::Validation`](crate::Error::Validation)s.
//! - **Time windows.** `start` / `end` style setters take a
//!   [`DateTime<Utc>`](chrono::DateTime) and are sent as epoch seconds; a time before the
//!   Unix epoch is rejected. How an omitted or `0` bound behaves differs by route; see
//!   each setter.
//! - **Errors.** Error bodies carry `error`, `code`, `retryable`, `trace_id` and, for
//!   validation failures, `parameter`; they are exposed by [`ApiError`](crate::ApiError).
//!   Use [`ErrorCode::from_error`] to branch on the typed code.
//!   [`Error::is_retryable`](crate::Error::is_retryable) follows the body's `retryable`
//!   flag. `429` and retryable `503` responses carry `Retry-After`
//!   ([`Error::retry_after`](crate::Error::retry_after)), and every response carries an
//!   `x-trace-id` header ([`Error::trace_id`](crate::Error::trace_id),
//!   [`Page::trace_id`]).
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

/// Shared identifiers and enums used by this module's requests and responses, re-exported
/// here for discoverability (also available from [`crate::types`]).
pub use crate::types::{Address, ConditionId, EventId, MarketId, QuestionId, Side, TokenId};
pub use boards::{
    BiggestWinner, BuilderCode, BuilderStanding, BuilderVolumePoint, GetBuildersVolume,
    GetLeaderboardStanding, LeaderboardEntry, LeaderboardSortBy, LeaderboardUserEntry,
    ListBiggestWinners, ListBuildersLeaderboard, ListLeaderboard, WinKind,
};
pub use client::{DataClient, DataClientBuilder};
pub use feeds::{
    Activity, ActivitySide, ActivitySortBy, ActivityType, ComboActivity, ComboActivityType,
    ListActivity, ListComboActivity, ListTrades, Trade,
};
pub use markets::{
    ConditionVolume, GetOpenInterest, Holder, HolderGroup, ListHolders, ListPricesHistory,
    LiveVolume, OpenInterest, PriceHistoryInterval, PricePoint, Reporter, Resolution,
    ResolutionMarketType, ResolutionSelector, ResolutionSource, ResolutionStatus,
    SettlementTimeBasis,
};
pub use service::{
    CursorLag, IngestionFreshness, ServiceStatus, ServingFreshness, ServingMechanism,
    ServingMechanismName,
};
pub use types::{
    ComboLeg, ComboLegEvent, ComboLegMarket, ComboLegStatus, ErrorCode, FilterType,
    MAX_LIST_VALUES, Page, Pagination, SortDirection, TimePeriod, UNLABELED_OUTCOME_INDEX,
};
pub use wallet::{
    ApprovalContract, Approvals, ComboPosition, ComboPositionSortBy, ComboPositionStatus,
    GetPortfolioValue, GetUserPnl, GetUserVolume, ListComboPositions, ListPositions, PnlFidelity,
    PnlInterval, PortfolioValue, Position, PositionSortBy, PositionStatus, TokenStandard,
    UserPnlPoint, UserPnlSeries, UserStats, UserVolume,
};
