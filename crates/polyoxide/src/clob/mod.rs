//! CLOB API client (`https://clob.polymarket.com`), public (unauthenticated) endpoints.
//!
//! Covers market data (order books, prices, midpoints, spreads, last trade prices, fee
//! rates, tick sizes, neg-risk flags), market listings and lookups, price history, liquidity
//! rewards configurations, maker rebates, builder trades and the server time. Start from
//! [`ClobClient`]; every endpoint is a method on it.
//!
//! See <https://docs.polymarket.com/api-reference/predictions/overview>.
//!
//! # Endpoints
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /time` | [`ClobClient::get_server_time`] |
//! | `GET /book` | [`ClobClient::get_order_book`] |
//! | `POST /books` | [`ClobClient::get_order_books`] |
//! | `GET /price` | [`ClobClient::get_price`] |
//! | `POST /prices` | [`ClobClient::get_prices`] |
//! | `GET /midpoint` | [`ClobClient::get_midpoint`] |
//! | `POST /midpoints` | [`ClobClient::get_midpoints`] |
//! | `GET /spread` | [`ClobClient::get_spread`] |
//! | `POST /spreads` | [`ClobClient::get_spreads`] |
//! | `GET /last-trade-price` | [`ClobClient::get_last_trade_price`] |
//! | `POST /last-trades-prices` | [`ClobClient::get_last_trade_prices`] |
//! | `GET /fee-rate` | [`ClobClient::get_fee_rate`] |
//! | `GET /fee-rate/{token_id}` | [`ClobClient::get_fee_rate_by_path`] |
//! | `GET /tick-size` | [`ClobClient::get_tick_size`] |
//! | `GET /tick-size/{token_id}` | [`ClobClient::get_tick_size_by_path`] |
//! | `GET /neg-risk` | [`ClobClient::get_neg_risk`] |
//! | `GET /neg-risk/{token_id}` | [`ClobClient::get_neg_risk_by_path`] |
//! | `GET /simplified-markets` | [`ClobClient::list_simplified_markets`] |
//! | `GET /sampling-markets` | [`ClobClient::list_sampling_markets`] |
//! | `GET /sampling-simplified-markets` | [`ClobClient::list_sampling_simplified_markets`] |
//! | `GET /clob-markets/{condition_id}` | [`ClobClient::get_clob_market_info`] |
//! | `GET /markets-by-token/{token_id}` | [`ClobClient::get_market_by_token`] |
//! | `POST /markets/live-activity` | [`ClobClient::get_markets_live_activity`] |
//! | `GET /markets/live-activity/{condition_id}` | [`ClobClient::get_market_live_activity`] |
//! | `GET /prices-history` | [`ClobClient::get_prices_history`] |
//! | `POST /batch-prices-history` | [`ClobClient::get_batch_prices_history`] |
//! | `GET /rewards/markets/current` | [`ClobClient::list_current_rewards`] |
//! | `GET /rewards/markets/{condition_id}` | [`ClobClient::list_raw_rewards_for_market`] |
//! | `GET /rewards/markets/multi` | [`ClobClient::list_markets_with_rewards`] |
//! | `GET /rebates/current` | [`ClobClient::get_current_rebated_fees`] |
//! | `GET /builder/trades` | [`ClobClient::list_builder_trades`] |
//!
//! Authenticated CLOB endpoints (orders, trades of a user, API keys, user rewards, ...) are
//! not implemented yet; see `ENDPOINTS.md`. The places where the live API differs from the
//! docs, and what the SDK does about each, are listed in `SPEC_DEVIATIONS.md`.
//!
//! # Naming
//!
//! Cursor-paginated listings are named `list_*`; they return a [`Page`] or
//! [`MarketsPage`] from `.send()` and every item from `.into_stream()`. The starting cursor
//! is set with `.cursor(..)`, and `page.next_cursor()` returns the cursor of the next page,
//! or `None` on the last one: the CLOB marks the last page with the `next_cursor` value
//! [`END_CURSOR`] (`"LTE="`). Every other endpoint is a `get_*` method.
//!
//! Several market-data endpoints are documented in more than one form. The batch endpoints
//! (`books`, `prices`, `midpoints`, `last-trades-prices`) are implemented as their `POST`
//! (JSON array body) form only, under the plain plural name (`get_<things>`): the documented
//! `GET` forms with a `token_ids` query parameter answer `400 Invalid payload` for every
//! encoding on the live API, so they cannot be called (see `SPEC_DEVIATIONS.md`). The
//! single-token endpoints with a path parameter keep a `_by_path` form:
//!
//! | Form | Method name | Example |
//! |---|---|---|
//! | `GET` with a single `token_id` query parameter | `get_<thing>` | [`ClobClient::get_midpoint`] |
//! | `POST` with a JSON array request body | `get_<things>` | [`ClobClient::get_midpoints`] |
//! | `GET` with the token id as a path parameter | `get_<thing>_by_path` | [`ClobClient::get_fee_rate_by_path`] |
//!
//! `POST /spreads` has no `GET` counterpart and is simply [`ClobClient::get_spreads`].
//!
//! # Validation
//!
//! Requests are checked before they are sent, and a violation is an
//! [`Error::Validation`](crate::Error::Validation) naming the parameter: required ids must
//! not be empty, required lists (including the request bodies of the batch `POST` forms)
//! must not be empty, ids in comma-separated lists must not contain a comma, and the
//! documented limits ([`MAX_LAST_TRADE_PRICES_TOKEN_IDS`],
//! [`MAX_BATCH_PRICES_HISTORY_MARKETS`], [`MAX_REWARDS_MARKETS_PAGE_SIZE`]) and patterns
//! (builder codes and markets of [`ClobClient::list_builder_trades`]) are enforced.
//!

mod client;
mod market_data;
mod markets;
mod prices_history;
mod rebates;
mod rewards;
mod time;
mod trades;
mod types;

pub use client::{ClobClient, ClobClientBuilder};
pub use market_data::{
    FeeRate, GetFeeRate, GetNegRisk, GetTickSize, LastTradePrice, MAX_LAST_TRADE_PRICES_TOKEN_IDS,
    Midpoint, NegRisk, OrderBookSummary, OrderSummary, Price, Spread, TickSize,
    TokenLastTradePrice,
};
pub use markets::{
    ClobMarketDetails, ClobToken, FeeDetails, ListSamplingMarkets, ListSamplingSimplifiedMarkets,
    ListSimplifiedMarkets, LiveActivityMarket, Market, MarketByToken, RewardRate, Rewards,
    SimplifiedMarket, Token,
};
pub use prices_history::{
    BatchPricesHistory, GetBatchPricesHistory, GetPricesHistory, MAX_BATCH_PRICES_HISTORY_MARKETS,
    PriceHistoryInterval, PricePoint, PricesHistory,
};
pub use rebates::RebatedFees;
pub use rewards::{
    CurrentReward, CurrentRewardConfig, ListCurrentRewards, ListMarketsWithRewards,
    ListRawRewardsForMarket, MAX_REWARDS_MARKETS_PAGE_SIZE, MarketReward, MultiMarketInfo,
    RewardsConfig, RewardsMarketsOrderBy, RewardsToken, SortDirection,
};
pub use trades::{BuilderTrade, ListBuilderTrades};
pub use types::{BookRequest, BuilderCode, END_CURSOR, MarketsPage, OrderId, Page, TradeId};

/// Shared identifiers and enums used by this module, re-exported here for discoverability
/// (also available from [`crate::types`]).
pub use crate::types::{Address, ConditionId, EventId, MarketId, Side, TokenId};
