//! CLOB API client (`https://clob.polymarket.com`), public (unauthenticated) endpoints.
//!
//! Covers market data (order books, prices, midpoints, spreads, last trade prices, fee
//! rates, tick sizes, neg-risk flags), market listings and lookups, price history, liquidity
//! rewards configurations, maker rebates, builder trades and the server time. Start from
//! [`ClobClient`]; every endpoint is a method on it.
//!
//! Endpoints that exist both as `GET` with query parameters and as `POST` with a request
//! body are exposed twice: the `POST` form has a `_by_body` suffix (e.g.
//! [`ClobClient::get_midpoints`] / [`ClobClient::get_midpoints_by_body`]). Endpoints that
//! also accept the token id as a path parameter have a `_by_path` form (e.g.
//! [`ClobClient::get_fee_rate_by_path`]).
//!
//! Cursor-paginated listings return a [`Page`] or [`MarketsPage`] from `.send()` and every
//! item from `.into_stream()`. The CLOB marks the last page with the `next_cursor` value
//! [`END_CURSOR`] (`"LTE="`).
//!
//! Authenticated CLOB endpoints (orders, trades of a user, API keys, user rewards, ...) are
//! not implemented yet; see `ENDPOINTS.md`.
//!
//! See <https://docs.polymarket.com/api-reference/predictions/overview>.

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
    ClobMarketDetails, ClobToken, FeeDetails, GetSamplingMarkets, GetSamplingSimplifiedMarkets,
    GetSimplifiedMarkets, LiveActivityMarket, Market, MarketByToken, RewardRate, Rewards,
    SimplifiedMarket, Token,
};
pub use prices_history::{
    BatchPricesHistory, GetBatchPricesHistory, GetPricesHistory, MAX_BATCH_PRICES_HISTORY_MARKETS,
    PriceHistoryInterval, PricePoint, PricesHistory,
};
pub use rebates::RebatedFees;
pub use rewards::{
    CurrentReward, CurrentRewardConfig, GetCurrentRewards, GetMarketsWithRewards,
    GetRawRewardsForMarket, MAX_REWARDS_MARKETS_PAGE_SIZE, MarketReward, MultiMarketInfo,
    RewardsConfig, RewardsMarketsOrderBy, RewardsToken, SortDirection,
};
pub use trades::{BuilderTrade, GetBuilderTrades};
pub use types::{
    BookRequest, BuilderCode, END_CURSOR, EventId, MarketId, MarketsPage, OrderId, Page, TradeId,
};
