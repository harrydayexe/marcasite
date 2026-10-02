# Module `marcasite::clob`

> Generated from marcasite 0.1.1 (all features) by `just docs-md`. Do not edit.

CLOB API client (`https://clob.polymarket.com`), public (unauthenticated) endpoints.

Covers market data (order books, prices, midpoints, spreads, last trade prices, fee
rates, tick sizes, neg-risk flags), market listings and lookups, price history, liquidity
rewards configurations, maker rebates, builder trades and the server time. Start from
[`ClobClient`](clob.md#struct.ClobClient); every endpoint is a method on it.

See <https://docs.polymarket.com/api-reference/predictions/overview>.

## Endpoints

| Endpoint | Method |
|---|---|
| `GET /time` | [`ClobClient::get_server_time`](clob.md#ClobClient.fn.get_server_time) |
| `GET /book` | [`ClobClient::get_order_book`](clob.md#ClobClient.fn.get_order_book) |
| `POST /books` | [`ClobClient::get_order_books`](clob.md#ClobClient.fn.get_order_books) |
| `GET /price` | [`ClobClient::get_price`](clob.md#ClobClient.fn.get_price) |
| `POST /prices` | [`ClobClient::get_prices`](clob.md#ClobClient.fn.get_prices) |
| `GET /midpoint` | [`ClobClient::get_midpoint`](clob.md#ClobClient.fn.get_midpoint) |
| `POST /midpoints` | [`ClobClient::get_midpoints`](clob.md#ClobClient.fn.get_midpoints) |
| `GET /spread` | [`ClobClient::get_spread`](clob.md#ClobClient.fn.get_spread) |
| `POST /spreads` | [`ClobClient::get_spreads`](clob.md#ClobClient.fn.get_spreads) |
| `GET /last-trade-price` | [`ClobClient::get_last_trade_price`](clob.md#ClobClient.fn.get_last_trade_price) |
| `POST /last-trades-prices` | [`ClobClient::get_last_trade_prices`](clob.md#ClobClient.fn.get_last_trade_prices) |
| `GET /fee-rate` | [`ClobClient::get_fee_rate`](clob.md#ClobClient.fn.get_fee_rate) |
| `GET /fee-rate/{token_id}` | [`ClobClient::get_fee_rate_by_path`](clob.md#ClobClient.fn.get_fee_rate_by_path) |
| `GET /tick-size` | [`ClobClient::get_tick_size`](clob.md#ClobClient.fn.get_tick_size) |
| `GET /tick-size/{token_id}` | [`ClobClient::get_tick_size_by_path`](clob.md#ClobClient.fn.get_tick_size_by_path) |
| `GET /neg-risk` | [`ClobClient::get_neg_risk`](clob.md#ClobClient.fn.get_neg_risk) |
| `GET /neg-risk/{token_id}` | [`ClobClient::get_neg_risk_by_path`](clob.md#ClobClient.fn.get_neg_risk_by_path) |
| `GET /simplified-markets` | [`ClobClient::list_simplified_markets`](clob.md#ClobClient.fn.list_simplified_markets) |
| `GET /sampling-markets` | [`ClobClient::list_sampling_markets`](clob.md#ClobClient.fn.list_sampling_markets) |
| `GET /sampling-simplified-markets` | [`ClobClient::list_sampling_simplified_markets`](clob.md#ClobClient.fn.list_sampling_simplified_markets) |
| `GET /clob-markets/{condition_id}` | [`ClobClient::get_clob_market_info`](clob.md#ClobClient.fn.get_clob_market_info) |
| `GET /markets-by-token/{token_id}` | [`ClobClient::get_market_by_token`](clob.md#ClobClient.fn.get_market_by_token) |
| `POST /markets/live-activity` | [`ClobClient::get_markets_live_activity`](clob.md#ClobClient.fn.get_markets_live_activity) |
| `GET /markets/live-activity/{condition_id}` | [`ClobClient::get_market_live_activity`](clob.md#ClobClient.fn.get_market_live_activity) |
| `GET /prices-history` | [`ClobClient::get_prices_history`](clob.md#ClobClient.fn.get_prices_history) |
| `POST /batch-prices-history` | [`ClobClient::get_batch_prices_history`](clob.md#ClobClient.fn.get_batch_prices_history) |
| `GET /rewards/markets/current` | [`ClobClient::list_current_rewards`](clob.md#ClobClient.fn.list_current_rewards) |
| `GET /rewards/markets/{condition_id}` | [`ClobClient::list_raw_rewards_for_market`](clob.md#ClobClient.fn.list_raw_rewards_for_market) |
| `GET /rewards/markets/multi` | [`ClobClient::list_markets_with_rewards`](clob.md#ClobClient.fn.list_markets_with_rewards) |
| `GET /rebates/current` | [`ClobClient::get_current_rebated_fees`](clob.md#ClobClient.fn.get_current_rebated_fees) |
| `GET /builder/trades` | [`ClobClient::list_builder_trades`](clob.md#ClobClient.fn.list_builder_trades) |

Authenticated CLOB endpoints (orders, trades of a user, API keys, user rewards, ...) are
not implemented yet; see `ENDPOINTS.md`. The places where the live API differs from the
docs, and what the SDK does about each, are listed in `SPEC_DEVIATIONS.md`.

## Naming

Cursor-paginated listings are named `list_*`; they return a [`Page`](clob.md#struct.Page) or
[`MarketsPage`](clob.md#struct.MarketsPage) from `.send()` and every item from `.into_stream()`. The starting cursor
is set with `.cursor(..)`, and `page.next_cursor()` returns the cursor of the next page,
or `None` on the last one: the CLOB marks the last page with the `next_cursor` value
[`END_CURSOR`](clob.md#constant.END_CURSOR) (`"LTE="`). Every other endpoint is a `get_*` method.

Several market-data endpoints are documented in more than one form. The batch endpoints
(`books`, `prices`, `midpoints`, `last-trades-prices`) are implemented as their `POST`
(JSON array body) form only, under the plain plural name (`get_<things>`): the documented
`GET` forms with a `token_ids` query parameter answer `400 Invalid payload` for every
encoding on the live API, so they cannot be called (see `SPEC_DEVIATIONS.md`). The
single-token endpoints with a path parameter keep a `_by_path` form:

| Form | Method name | Example |
|---|---|---|
| `GET` with a single `token_id` query parameter | `get_<thing>` | [`ClobClient::get_midpoint`](clob.md#ClobClient.fn.get_midpoint) |
| `POST` with a JSON array request body | `get_<things>` | [`ClobClient::get_midpoints`](clob.md#ClobClient.fn.get_midpoints) |
| `GET` with the token id as a path parameter | `get_<thing>_by_path` | [`ClobClient::get_fee_rate_by_path`](clob.md#ClobClient.fn.get_fee_rate_by_path) |

`POST /spreads` has no `GET` counterpart and is simply [`ClobClient::get_spreads`](clob.md#ClobClient.fn.get_spreads).

## Validation

Requests are checked before they are sent, and a violation is an
[`Error::Validation`](marcasite.md#enum.Error) naming the parameter: required ids must
not be empty, required lists (including the request bodies of the batch `POST` forms)
must not be empty, and the documented limits ([`MAX_LAST_TRADE_PRICES_TOKEN_IDS`](clob.md#constant.MAX_LAST_TRADE_PRICES_TOKEN_IDS),
[`MAX_BATCH_PRICES_HISTORY_MARKETS`](clob.md#constant.MAX_BATCH_PRICES_HISTORY_MARKETS), [`MAX_REWARDS_MARKETS_PAGE_SIZE`](clob.md#constant.MAX_REWARDS_MARKETS_PAGE_SIZE)) and patterns
(builder codes and markets of [`ClobClient::list_builder_trades`](clob.md#ClobClient.fn.list_builder_trades)) are enforced. The
price-history `fidelity` minimums that the live server enforces for the `1m` and `1w`
intervals ([`MIN_FIDELITY_ONE_MONTH`](clob.md#constant.MIN_FIDELITY_ONE_MONTH), [`MIN_FIDELITY_ONE_WEEK`](clob.md#constant.MIN_FIDELITY_ONE_WEEK)) are checked as well.

## Index

- **Re-exports:** `Address`, `ConditionId`, `EventId`, `MarketId`, `Side`, `TokenId`
- **Structs:** [`BatchPricesHistory`](#struct.BatchPricesHistory), [`BookRequest`](#struct.BookRequest), [`BuilderCode`](#struct.BuilderCode), [`BuilderTrade`](#struct.BuilderTrade), [`ClobClient`](#struct.ClobClient), [`ClobClientBuilder`](#struct.ClobClientBuilder), [`ClobMarketDetails`](#struct.ClobMarketDetails), [`ClobToken`](#struct.ClobToken), [`CurrentReward`](#struct.CurrentReward), [`CurrentRewardConfig`](#struct.CurrentRewardConfig), [`FeeDetails`](#struct.FeeDetails), [`FeeRate`](#struct.FeeRate), [`GetBatchPricesHistory`](#struct.GetBatchPricesHistory), [`GetFeeRate`](#struct.GetFeeRate), [`GetNegRisk`](#struct.GetNegRisk), [`GetPricesHistory`](#struct.GetPricesHistory), [`GetTickSize`](#struct.GetTickSize), [`LastTradePrice`](#struct.LastTradePrice), [`ListBuilderTrades`](#struct.ListBuilderTrades), [`ListCurrentRewards`](#struct.ListCurrentRewards), [`ListMarketsWithRewards`](#struct.ListMarketsWithRewards), [`ListRawRewardsForMarket`](#struct.ListRawRewardsForMarket), [`ListSamplingMarkets`](#struct.ListSamplingMarkets), [`ListSamplingSimplifiedMarkets`](#struct.ListSamplingSimplifiedMarkets), [`ListSimplifiedMarkets`](#struct.ListSimplifiedMarkets), [`LiveActivityMarket`](#struct.LiveActivityMarket), [`Market`](#struct.Market), [`MarketByToken`](#struct.MarketByToken), [`MarketReward`](#struct.MarketReward), [`MarketsPage`](#struct.MarketsPage), [`Midpoint`](#struct.Midpoint), [`MultiMarketInfo`](#struct.MultiMarketInfo), [`NegRisk`](#struct.NegRisk), [`OrderBookSummary`](#struct.OrderBookSummary), [`OrderId`](#struct.OrderId), [`OrderSummary`](#struct.OrderSummary), [`Page`](#struct.Page), [`Price`](#struct.Price), [`PricePoint`](#struct.PricePoint), [`PricesHistory`](#struct.PricesHistory), [`RebatedFees`](#struct.RebatedFees), [`RewardRate`](#struct.RewardRate), [`Rewards`](#struct.Rewards), [`RewardsConfig`](#struct.RewardsConfig), [`RewardsToken`](#struct.RewardsToken), [`SimplifiedMarket`](#struct.SimplifiedMarket), [`Spread`](#struct.Spread), [`TickSize`](#struct.TickSize), [`Token`](#struct.Token), [`TokenLastTradePrice`](#struct.TokenLastTradePrice), [`TradeId`](#struct.TradeId)
- **Enums:** [`PriceHistoryInterval`](#enum.PriceHistoryInterval), [`RewardsMarketsOrderBy`](#enum.RewardsMarketsOrderBy), [`SortDirection`](#enum.SortDirection)
- **Constants:** [`END_CURSOR`](#constant.END_CURSOR), [`MAX_BATCH_PRICES_HISTORY_MARKETS`](#constant.MAX_BATCH_PRICES_HISTORY_MARKETS), [`MAX_LAST_TRADE_PRICES_TOKEN_IDS`](#constant.MAX_LAST_TRADE_PRICES_TOKEN_IDS), [`MAX_REWARDS_MARKETS_PAGE_SIZE`](#constant.MAX_REWARDS_MARKETS_PAGE_SIZE), [`MIN_FIDELITY_ONE_MONTH`](#constant.MIN_FIDELITY_ONE_MONTH), [`MIN_FIDELITY_ONE_WEEK`](#constant.MIN_FIDELITY_ONE_WEEK)

## Re-exports

- `Address`: re-export of [`marcasite::types::Address`](types.md#struct.Address).
- `ConditionId`: re-export of [`marcasite::types::ConditionId`](types.md#struct.ConditionId).
- `EventId`: re-export of [`marcasite::types::EventId`](types.md#struct.EventId).
- `MarketId`: re-export of [`marcasite::types::MarketId`](types.md#struct.MarketId).
- `Side`: re-export of [`marcasite::types::Side`](types.md#enum.Side).
- `TokenId`: re-export of [`marcasite::types::TokenId`](types.md#struct.TokenId).

## Structs

### <a id="struct.BatchPricesHistory"></a>`struct BatchPricesHistory`

```rust
#[non_exhaustive]
pub struct BatchPricesHistory {
    /// Map of market asset id (token id) to its price points.
    pub history: Option<HashMap<TokenId, Vec<PricePoint>>>,
}
```

The price histories of several markets
(`components/schemas/BatchPricesHistoryResponse`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BookRequest"></a>`struct BookRequest`

```rust
#[non_exhaustive]
pub struct BookRequest {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Order side. Optional; not used for midpoint calculation.
    pub side: Option<Side>,
}
```

An item of the request body of the batch market-data endpoints
(`components/schemas/BookRequest`): a token id and an optional side.

Every `impl Into<TokenId>` value (`&str`, `String`, [`TokenId`](types.md#struct.TokenId)) converts into a
`BookRequest` without a side, and a `(token id, Side)` pair (owned or borrowed) into one
with a side, so token ids and pairs can be passed directly.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `From<&(T, Side)>`, `From<&BookRequest>`, `From<&String>`, `From<&TokenId>`, `From<&str>`, `From<(T, Side)>`, `From<String>`, `From<TokenId>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="BookRequest.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(token_id: impl Into<TokenId>) -> Self
```

Creates a request item for `token_id` without a side.

##### <a id="BookRequest.fn.with_side"></a>`with_side`

```rust
#[must_use]
pub fn with_side(self, side: Side) -> Self
```

Sets the side.

### <a id="struct.BuilderCode"></a>`struct BuilderCode`

```rust
pub struct BuilderCode(/* private fields */);
```

A builder code: `0x` followed by 64 hex characters, e.g.
`"0x0000000000000000000000000000000000000000000000000000000000000001"`.

Requests validate the documented pattern `^0x[a-fA-F0-9]{64}$` before sending.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&BuilderCode>`, `From<&String>`, `From<&str>`, `From<BuilderCode>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="BuilderCode.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="BuilderCode.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="BuilderCode.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.BuilderTrade"></a>`struct BuilderTrade`

```rust
#[non_exhaustive]
pub struct BuilderTrade {
    /// Trade id.
    pub id: TradeId,
    /// Trade type, e.g. `"TAKER"` or `"MAKER"` (the spec documents no fixed set of values).
    pub trade_type: String,
    /// Hash of the taker order.
    pub taker_order_hash: OrderId,
    /// The spec's builder-code field. Live it is always an empty string; the code is in
    /// [`builder_code`](clob.md#struct.BuilderTrade) instead.
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
    /// [`builder`](clob.md#struct.BuilderTrade) field leaves empty.
    pub builder_code: Option<BuilderCode>,
    /// Error message, if any (wire name `err_msg`).
    pub err_msg: Option<String>,
    /// Creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    pub updated_at: Option<DateTime<Utc>>,
}
```

A trade attributed to a builder code (`components/schemas/BuilderTrade`).

Amounts (`size`, `size_usdc`, `fee`, `fee_usdc`, `builder_fee`) are decimal units, not
the micro-units the spec examples show (live check, 2026-10-02): a trade of 5 shares at
`0.01` has `size` `"5"` and `sizeUsdc` `"0.05"`. See `SPEC_DEVIATIONS.md`.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ClobClient"></a>`struct ClobClient`

```rust
pub struct ClobClient { /* private fields */ }
```

Client for the CLOB API (`https://clob.polymarket.com`).

Covers the public endpoints: order books, prices, spreads, markets, price history,
rewards, rebates, builder trades and the server time (see the [module docs](clob.md)
for the full list). Cheap to clone: clones share one connection pool.

```rust
use marcasite::clob::ClobClient;

let client = ClobClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="ClobClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://clob.polymarket.com";
```

The production base URL.

##### <a id="ClobClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="ClobClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> ClobClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="ClobClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="ClobClient.fn.get_order_book"></a>`get_order_book`

```rust
pub async fn get_order_book(&self, token_id: impl Into<TokenId>) -> Result<OrderBookSummary>
```

Gets the order book of a token (`GET /book`).

See <https://docs.polymarket.com/api-reference/market-data/get-order-book>.

```rust
let clob = marcasite::clob::ClobClient::new()?;
let book = clob
    .get_order_book(
        "71321045679252212594626385532706912750332728571942532289631379312455583992563",
    )
    .await?;
if let Some(best_bid) = book.bids.first() {
    println!("best bid {} x {}", best_bid.price, best_bid.size);
}
```

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. A token
without an order book is an [`Error::Api`](marcasite.md#enum.Error) with status `404`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_order_books"></a>`get_order_books`

```rust
pub async fn get_order_books(&self, requests: impl IntoIterator<Item = impl Into<BookRequest>>) -> Result<Vec<OrderBookSummary>>
```

Gets the order books of several tokens (`POST /books`).

Accepts token ids directly or [`BookRequest`](clob.md#struct.BookRequest)s.

The `GET /books?token_ids=...` form that the OpenAPI spec lists (`getBooksGet`) always
answers `400 Invalid payload` live, so it is not implemented (see
`SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/market-data/get-order-books-request-body>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `token_ids`) if `requests`
is empty (the request body is required) or a token id is empty. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_price"></a>`get_price`

```rust
pub async fn get_price(&self, token_id: impl Into<TokenId>, side: Side) -> Result<Price>
```

Gets the best price of a token for a side (`GET /price`): the best bid for
[`Side::Buy`](types.md#enum.Side), the best ask for [`Side::Sell`](types.md#enum.Side).

See <https://docs.polymarket.com/api-reference/market-data/get-market-price>.

```rust
use marcasite::types::Side;

let clob = marcasite::clob::ClobClient::new()?;
let best_bid = clob
    .get_price(
        "71321045679252212594626385532706912750332728571942532289631379312455583992563",
        Side::Buy,
    )
    .await?;
println!("best bid: {}", best_bid.price);
```

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. A token
without an order book is an [`Error::Api`](marcasite.md#enum.Error) with status `404`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_prices"></a>`get_prices`

```rust
pub async fn get_prices(&self, requests: impl IntoIterator<Item = impl Into<BookRequest>>) -> Result<HashMap<TokenId, HashMap<Side, Decimal>>>
```

Gets the best prices of several `(token id, side)` pairs (`POST /prices`).

Accepts `(token id, Side)` pairs (owned or borrowed) or [`BookRequest`](clob.md#struct.BookRequest)s with a side.
Returns a map of token id to a map of side to price (the prices are numeric strings on
the wire).

Every request needs a side, as the docs say. Live, an item without a side is accepted
but silently priced as `SELL` only, so this is rejected client-side instead. The
`GET /prices?token_ids=...&sides=...` form always answers `400 Invalid payload` live,
so it is not implemented (see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/market-data/get-market-prices-request-body>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `requests` is empty or a token id
is empty (parameter `token_ids`), or if a request has no side (parameter `side`; the
docs require "both token_id and side"). See [`Error`](marcasite.md#enum.Error) for the other
cases.

##### <a id="ClobClient.fn.get_midpoint"></a>`get_midpoint`

```rust
pub async fn get_midpoint(&self, token_id: impl Into<TokenId>) -> Result<Midpoint>
```

Gets the midpoint price of a token (`GET /midpoint`).

See <https://docs.polymarket.com/api-reference/data/get-midpoint-price>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. A token
without an order book is an [`Error::Api`](marcasite.md#enum.Error) with status `404`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_midpoints"></a>`get_midpoints`

```rust
pub async fn get_midpoints(&self, requests: impl IntoIterator<Item = impl Into<BookRequest>>) -> Result<HashMap<TokenId, Decimal>>
```

Gets the midpoint prices of several tokens (`POST /midpoints`). Returns a map of
token id to midpoint.

Accepts token ids directly or [`BookRequest`](clob.md#struct.BookRequest)s (the side is not used). The
`GET /midpoints?token_ids=...` form always answers `400 Invalid payload` live, so it
is not implemented (see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/market-data/get-midpoint-prices-request-body>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `token_ids`) if `requests`
is empty (the request body is required) or a token id is empty. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_spread"></a>`get_spread`

```rust
pub async fn get_spread(&self, token_id: impl Into<TokenId>) -> Result<Spread>
```

Gets the spread of a token (`GET /spread`).

See <https://docs.polymarket.com/api-reference/market-data/get-spread>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. A token
without an order book is an [`Error::Api`](marcasite.md#enum.Error) with status `404`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_spreads"></a>`get_spreads`

```rust
pub async fn get_spreads(&self, requests: impl IntoIterator<Item = impl Into<BookRequest>>) -> Result<HashMap<TokenId, Decimal>>
```

Gets the spreads of several tokens (`POST /spreads`, the only form).
Returns a map of token id to spread.

Accepts token ids directly or [`BookRequest`](clob.md#struct.BookRequest)s.

See <https://docs.polymarket.com/api-reference/market-data/get-spreads>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `token_ids`) if `requests`
is empty (the request body is required) or a token id is empty. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_last_trade_price"></a>`get_last_trade_price`

```rust
pub async fn get_last_trade_price(&self, token_id: impl Into<TokenId>) -> Result<LastTradePrice>
```

Gets the last trade price and side of a token (`GET /last-trade-price`).

A token without trades yields a price of `0.5` and no side.

See <https://docs.polymarket.com/api-reference/market-data/get-last-trade-price>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_last_trade_prices"></a>`get_last_trade_prices`

```rust
pub async fn get_last_trade_prices(&self, requests: impl IntoIterator<Item = impl Into<BookRequest>>) -> Result<Vec<TokenLastTradePrice>>
```

Gets the last trade prices of up to [`MAX_LAST_TRADE_PRICES_TOKEN_IDS`](clob.md#constant.MAX_LAST_TRADE_PRICES_TOKEN_IDS) tokens
(`POST /last-trades-prices`).

Accepts token ids directly or [`BookRequest`](clob.md#struct.BookRequest)s. An empty side from the server is
mapped to a `None` [`TokenLastTradePrice::side`](clob.md#struct.TokenLastTradePrice). The
`GET /last-trades-prices?token_ids=...` form always answers `400 Invalid payload`
live, so it is not implemented (see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/market-data/get-last-trade-prices-request-body>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `token_ids`) if `requests`
is empty (the request body is required), has more than
[`MAX_LAST_TRADE_PRICES_TOKEN_IDS`](clob.md#constant.MAX_LAST_TRADE_PRICES_TOKEN_IDS) entries, or a token id is empty. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_fee_rate"></a>`get_fee_rate`

```rust
pub fn get_fee_rate(&self) -> GetFeeRate
```

Gets the base fee rate of a token with a query parameter (`GET /fee-rate`).

The spec marks the `token_id` query parameter as optional, so it is a builder setter
here. The endpoint is documented as returning the fee rate "for a specific token ID",
and the docs do not say what the server returns when `token_id` is omitted.
[`ClobClient::get_fee_rate_by_path`](clob.md#ClobClient.fn.get_fee_rate_by_path) takes the token id as a required argument.

See <https://docs.polymarket.com/api-reference/market-data/get-fee-rate>.

##### <a id="ClobClient.fn.get_fee_rate_by_path"></a>`get_fee_rate_by_path`

```rust
pub async fn get_fee_rate_by_path(&self, token_id: impl Into<TokenId>) -> Result<FeeRate>
```

Gets the base fee rate of a token with a path parameter
(`GET /fee-rate/{token_id}`). The query-parameter form is
[`ClobClient::get_fee_rate`](clob.md#ClobClient.fn.get_fee_rate).

See <https://docs.polymarket.com/api-reference/market-data/get-fee-rate-by-path-parameter>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. An invalid
token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an unknown market
one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_tick_size"></a>`get_tick_size`

```rust
pub fn get_tick_size(&self) -> GetTickSize
```

Gets the minimum tick size of a token with a query parameter (`GET /tick-size`).

The spec marks the `token_id` query parameter as optional, so it is a builder setter
here. The endpoint is documented as returning the tick size "for a specific token
ID", and the docs do not say what the server returns when `token_id` is omitted.
[`ClobClient::get_tick_size_by_path`](clob.md#ClobClient.fn.get_tick_size_by_path) takes the token id as a required argument.

See <https://docs.polymarket.com/api-reference/market-data/get-tick-size>.

##### <a id="ClobClient.fn.get_tick_size_by_path"></a>`get_tick_size_by_path`

```rust
pub async fn get_tick_size_by_path(&self, token_id: impl Into<TokenId>) -> Result<TickSize>
```

Gets the minimum tick size of a token with a path parameter
(`GET /tick-size/{token_id}`). The query-parameter form is
[`ClobClient::get_tick_size`](clob.md#ClobClient.fn.get_tick_size).

See <https://docs.polymarket.com/api-reference/market-data/get-tick-size-by-path-parameter>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. An invalid
token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an unknown market
one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_neg_risk"></a>`get_neg_risk`

```rust
pub fn get_neg_risk(&self) -> GetNegRisk
```

Gets the negative-risk flag of a token's market with a query parameter
(`GET /neg-risk`).

The spec marks the `token_id` query parameter as optional, so it is a builder setter
here. The endpoint is documented as returning the flag "for a specific token ID", and
the docs do not say what the server returns when `token_id` is omitted.
[`ClobClient::get_neg_risk_by_path`](clob.md#ClobClient.fn.get_neg_risk_by_path) takes the token id as a required argument.

Documented only in the CLOB OpenAPI spec (operation `getNegRisk`); there is no
reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.

##### <a id="ClobClient.fn.get_neg_risk_by_path"></a>`get_neg_risk_by_path`

```rust
pub async fn get_neg_risk_by_path(&self, token_id: impl Into<TokenId>) -> Result<NegRisk>
```

Gets the negative-risk flag of a token's market with a path parameter
(`GET /neg-risk/{token_id}`). The query-parameter form is
[`ClobClient::get_neg_risk`](clob.md#ClobClient.fn.get_neg_risk).

Documented only in the CLOB OpenAPI spec (operation `getNegRiskByPath`); there is no
reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty. An invalid
token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an unknown market
one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.list_simplified_markets"></a>`list_simplified_markets`

```rust
pub fn list_simplified_markets(&self) -> ListSimplifiedMarkets
```

Lists markets in simplified form (`GET /simplified-markets`, cursor pagination).

Live pages hold up to 1000 markets (all states, including closed ones, whose
`rewards.rates` is `null`), and the cursors are opaque base64 strings. See
<https://docs.polymarket.com/api-reference/markets/get-simplified-markets>.

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};

let clob = marcasite::clob::ClobClient::new()?;

// One page, with the cursor of the next one.
let page = clob.list_simplified_markets().send().await?;
println!("{} markets, next page: {:?}", page.items().len(), page.next_cursor());

// Or every market, fetching pages lazily.
let first_1000: Vec<_> = clob
    .list_simplified_markets()
    .into_stream()
    .take(1000)
    .try_collect()
    .await?;
```

##### <a id="ClobClient.fn.list_sampling_markets"></a>`list_sampling_markets`

```rust
pub fn list_sampling_markets(&self) -> ListSamplingMarkets
```

Lists sampling markets (`GET /sampling-markets`, cursor pagination).

See <https://docs.polymarket.com/api-reference/markets/get-sampling-markets>.

##### <a id="ClobClient.fn.list_sampling_simplified_markets"></a>`list_sampling_simplified_markets`

```rust
pub fn list_sampling_simplified_markets(&self) -> ListSamplingSimplifiedMarkets
```

Lists sampling markets in simplified form (`GET /sampling-simplified-markets`, cursor
pagination).

See <https://docs.polymarket.com/api-reference/markets/get-sampling-simplified-markets>.

##### <a id="ClobClient.fn.get_clob_market_info"></a>`get_clob_market_info`

```rust
pub async fn get_clob_market_info(&self, condition_id: impl Into<ConditionId>) -> Result<ClobMarketDetails>
```

Gets all CLOB-level parameters of a market (`GET /clob-markets/{condition_id}`).

See <https://docs.polymarket.com/api-reference/markets/get-clob-market-info>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `condition_id` is empty. An invalid
condition id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_market_by_token"></a>`get_market_by_token`

```rust
pub async fn get_market_by_token(&self, token_id: impl Into<TokenId>) -> Result<MarketByToken>
```

Gets the parent market of a token (`GET /markets-by-token/{token_id}`).

See <https://docs.polymarket.com/api-reference/markets/get-market-by-token>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `token_id` is empty (which the
server documents as `400`). An unknown token is an [`Error::Api`](marcasite.md#enum.Error)
with status `404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_markets_live_activity"></a>`get_markets_live_activity`

```rust
pub async fn get_markets_live_activity(&self, condition_ids: impl IntoIterator<Item = impl Into<ConditionId>>) -> Result<Vec<LiveActivityMarket>>
```

Gets live-activity summaries of several markets (`POST /markets/live-activity`).

Documented only in the CLOB OpenAPI spec (operation `getMarketsLiveActivity`); there
is no reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `condition_ids`) if
`condition_ids` is empty (the server rejects an empty body with `400`) or a condition
id is empty. Unknown markets are an [`Error::Api`](marcasite.md#enum.Error) with status
`404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_market_live_activity"></a>`get_market_live_activity`

```rust
pub async fn get_market_live_activity(&self, condition_id: impl Into<ConditionId>) -> Result<LiveActivityMarket>
```

Gets the live-activity summary of a market
(`GET /markets/live-activity/{condition_id}`).

Documented only in the CLOB OpenAPI spec (operation `getMarketLiveActivity`); there is
no reference page. See <https://docs.polymarket.com/api-reference/predictions/overview>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `condition_id` is empty. An invalid
condition id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an unknown
market one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.get_prices_history"></a>`get_prices_history`

```rust
pub fn get_prices_history(&self, market: impl Into<TokenId>) -> GetPricesHistory
```

Gets the price history of a market (`GET /prices-history`).

`market` is the token id (asset id) to query. The batch form for up to
[`MAX_BATCH_PRICES_HISTORY_MARKETS`](clob.md#constant.MAX_BATCH_PRICES_HISTORY_MARKETS) markets is
[`ClobClient::get_batch_prices_history`](clob.md#ClobClient.fn.get_batch_prices_history).

See <https://docs.polymarket.com/api-reference/markets/get-prices-history>.

```rust
use marcasite::clob::PriceHistoryInterval;

let clob = marcasite::clob::ClobClient::new()?;
let history = clob
    .get_prices_history(
        "71321045679252212594626385532706912750332728571942532289631379312455583992563",
    )
    .interval(PriceHistoryInterval::OneDay)
    .fidelity(60)
    .send()
    .await?;
for point in history.history.unwrap_or_default() {
    println!("{:?}: {:?}", point.timestamp, point.price);
}
```

##### <a id="ClobClient.fn.get_batch_prices_history"></a>`get_batch_prices_history`

```rust
pub fn get_batch_prices_history(&self, markets: impl IntoIterator<Item = impl Into<TokenId>>) -> GetBatchPricesHistory
```

Gets the price histories of up to [`MAX_BATCH_PRICES_HISTORY_MARKETS`](clob.md#constant.MAX_BATCH_PRICES_HISTORY_MARKETS) markets
(`POST /batch-prices-history`).

`markets` are token ids (asset ids). The single-market form is
[`ClobClient::get_prices_history`](clob.md#ClobClient.fn.get_prices_history).

See <https://docs.polymarket.com/api-reference/markets/get-batch-prices-history>.

```rust
use marcasite::clob::PriceHistoryInterval;

let clob = marcasite::clob::ClobClient::new()?;
let batch = clob
    .get_batch_prices_history(["1", "2"])
    .interval(PriceHistoryInterval::OneWeek)
    .send()
    .await?;
for (token_id, points) in batch.history.unwrap_or_default() {
    println!("{token_id}: {} points", points.len());
}
```

##### <a id="ClobClient.fn.get_current_rebated_fees"></a>`get_current_rebated_fees`

```rust
pub async fn get_current_rebated_fees(&self, date: NaiveDate, maker_address: impl Into<Address>) -> Result<Vec<RebatedFees>>
```

Gets the fees rebated to a maker on a given date, per market
(`GET /rebates/current`). No authentication is required.

A maker without rebates on the date gets an empty list. The spec documents `[]`, but
live answers with the JSON body `null` (HTTP 200), which is mapped to an empty list
too (see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/rebates/get-current-rebated-fees-for-a-maker>.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `maker_address` is empty. An
invalid date or maker address is an [`Error::Api`](marcasite.md#enum.Error) with status
`400`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ClobClient.fn.list_current_rewards"></a>`list_current_rewards`

```rust
pub fn list_current_rewards(&self) -> ListCurrentRewards
```

Lists all current active reward configurations, grouped by market
(`GET /rewards/markets/current`, cursor pagination, 500 items per page).

See <https://docs.polymarket.com/api-reference/rewards/get-current-active-rewards-configurations>.

##### <a id="ClobClient.fn.list_raw_rewards_for_market"></a>`list_raw_rewards_for_market`

```rust
pub fn list_raw_rewards_for_market(&self, condition_id: impl Into<ConditionId>) -> ListRawRewardsForMarket
```

Lists the present and future reward configurations of a market
(`GET /rewards/markets/{condition_id}`, cursor pagination, 100 items per page).

See <https://docs.polymarket.com/api-reference/rewards/get-raw-rewards-for-a-specific-market>.

##### <a id="ClobClient.fn.list_markets_with_rewards"></a>`list_markets_with_rewards`

```rust
pub fn list_markets_with_rewards(&self) -> ListMarketsWithRewards
```

Lists active markets with their reward configurations, with search, filters and
sorting (`GET /rewards/markets/multi`, cursor pagination).

See <https://docs.polymarket.com/api-reference/rewards/get-multiple-markets-with-rewards>.

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::clob::{RewardsMarketsOrderBy, SortDirection};

let clob = marcasite::clob::ClobClient::new()?;
let markets: Vec<_> = clob
    .list_markets_with_rewards()
    .tag_slugs(["politics", "sports"])
    .order_by(RewardsMarketsOrderBy::Volume24hr)
    .position(SortDirection::Desc)
    .page_size(500)
    .into_stream()
    .take(1000)
    .try_collect()
    .await?;
for market in markets {
    println!("{}: {:?}", market.question, market.volume_24hr);
}
```

##### <a id="ClobClient.fn.get_server_time"></a>`get_server_time`

```rust
pub async fn get_server_time(&self) -> Result<DateTime<Utc>>
```

Gets the server's current time (`GET /time`), e.g. to synchronise the local clock.

The server returns a Unix timestamp in seconds.

See <https://docs.polymarket.com/api-reference/data/get-server-time>.

###### Errors

See [`Error`](marcasite.md#enum.Error).

##### <a id="ClobClient.fn.list_builder_trades"></a>`list_builder_trades`

```rust
pub fn list_builder_trades(&self, builder_code: impl Into<BuilderCode>) -> ListBuilderTrades
```

Lists trades attributed to a builder code (`GET /builder/trades`, cursor pagination).

See <https://docs.polymarket.com/api-reference/trade/get-builder-trades>.

```rust
use marcasite::chrono::{Duration, Utc};

let clob = marcasite::clob::ClobClient::new()?;
let page = clob
    .list_builder_trades("0x0000000000000000000000000000000000000000000000000000000000000001")
    .after(Utc::now() - Duration::days(1))
    .send()
    .await?;
for trade in page.items() {
    println!("{} {} {} @ {}", trade.match_time, trade.side, trade.size, trade.price);
}
if let Some(cursor) = page.next_cursor() {
    println!("more trades after cursor {cursor}");
}
```

### <a id="struct.ClobClientBuilder"></a>`struct ClobClientBuilder`

```rust
#[must_use]
pub struct ClobClientBuilder { /* private fields */ }
```

Builder for [`ClobClient`](clob.md#struct.ClobClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="ClobClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`ClobClient::DEFAULT_BASE_URL`](clob.md#ClobClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="ClobClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="ClobClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<ClobClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.ClobMarketDetails"></a>`struct ClobMarketDetails`

```rust
#[non_exhaustive]
pub struct ClobMarketDetails {
    /// Game start time for sports markets, or `None` (wire name `gst`).
    pub game_start_time: Option<DateTime<Utc>>,
    /// Rewards configuration (wire name `r`).
    ///
    /// The spec describes it as an object with arbitrary properties
    /// (`components/schemas/ClobRewards`), so it is kept as raw JSON. This deliberately
    /// exposes `serde_json` 1.x in the public API (see the crate docs
    /// on dependencies in the public API); a typed representation would replace it if the
    /// docs ever describe the object's properties.
    pub rewards: Option<Map<String, Value>>,
    /// Tokens of this market (wire name `t`).
    pub tokens: Option<Vec<ClobToken>>,
    /// Minimum order size (wire name `mos`; a JSON number on the wire).
    pub min_order_size: Option<Decimal>,
    /// Minimum tick size, the price increment (wire name `mts`; a JSON number on the wire).
    pub min_tick_size: Option<Decimal>,
    /// Maker base fee in basis points (wire name `mbf`).
    pub maker_base_fee: Option<i64>,
    /// Taker base fee in basis points (wire name `tbf`).
    pub taker_base_fee: Option<i64>,
    /// Whether RFQ (request for quote) is enabled (wire name `rfqe`).
    pub rfq_enabled: Option<bool>,
    /// Whether the taker order delay is enabled (wire name `itode`): marketable orders are
    /// then held for the 250 ms taker-delay window before processing. The server omits the
    /// field when `false`.
    pub taker_order_delay_enabled: Option<bool>,
    /// Whether the Blockaid check is enabled (wire name `ibce`).
    pub blockaid_check_enabled: Option<bool>,
    /// Fee curve parameters (wire name `fd`).
    pub fee_details: Option<FeeDetails>,
    /// Minimum order age in seconds (wire name `oas`).
    pub min_order_age_seconds: Option<i64>,
    /// Condition id of the market (wire name `c`). Undocumented; observed live.
    pub condition_id: Option<ConditionId>,
    /// Delay in seconds applied to marketable orders (wire name `sd`). Undocumented; observed
    /// live as `1` or `3` on sports markets and omitted when the delay is 0. It always equals
    /// the [`Market::seconds_delay`](clob.md#struct.Market) of the market listings (live check, 2026-10-02); what
    /// the delay applies to is not documented.
    pub seconds_delay: Option<u64>,
    /// Whether the market accepts orders (wire name `ao`). Undocumented; observed live.
    pub accepting_orders: Option<bool>,
    /// Since when the market accepts orders (wire name `aot`, an RFC 3339 date-time).
    /// Undocumented; observed live.
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    /// Whether negative risk is enabled for this market (wire name `nr`). Undocumented;
    /// observed live, and sent only for neg-risk markets.
    pub neg_risk: Option<bool>,
    /// The `cbos` flag (wire name `cbos`). Undocumented; observed live as a boolean on every
    /// market. Its meaning is unknown, so the field keeps the wire name.
    pub cbos: Option<bool>,
    /// Version tag of the market (wire name `v`). Undocumented; observed live as `"v1"` on
    /// every market.
    pub version: Option<String>,
}
```

All CLOB-level parameters of a market (`components/schemas/ClobMarketDetails`): tokens,
tick size, base fees, rewards, RFQ status and fee details.

The wire format uses abbreviated field names (noted on each field). The spec marks no
field as required. Fields marked "undocumented" are sent by the live API (observed
2026-10-02) but are not in the spec; see `SPEC_DEVIATIONS.md`. Any field may be absent:
which ones appear depends on the market (e.g. `mbf`, `tbf` and `fd` only on markets with
fees, `gst`/`sd` on sports markets, `ao`/`aot` on markets that have been opened).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ClobToken"></a>`struct ClobToken`

```rust
#[non_exhaustive]
pub struct ClobToken {
    /// The token id (wire name `t`).
    pub token_id: Option<TokenId>,
    /// Outcome label, e.g. `"Yes"` (wire name `o`).
    pub outcome: Option<String>,
}
```

A token of a market in [`ClobMarketDetails`](clob.md#struct.ClobMarketDetails) (`components/schemas/ClobToken`).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CurrentReward"></a>`struct CurrentReward`

```rust
#[non_exhaustive]
pub struct CurrentReward {
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Maximum spread for rewards eligibility (a JSON number on the wire).
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    pub rewards_min_size: Option<Decimal>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<CurrentRewardConfig>>,
    /// Sponsored daily rate (a JSON number on the wire; omitted when zero).
    pub sponsored_daily_rate: Option<Decimal>,
    /// Number of sponsors (omitted when zero).
    pub sponsors_count: Option<u64>,
    /// Native daily rate, excluding sponsors (a JSON number on the wire; omitted when zero).
    pub native_daily_rate: Option<Decimal>,
    /// Total daily rate, including sponsors (a JSON number on the wire; omitted when zero).
    pub total_daily_rate: Option<Decimal>,
}
```

The current reward configuration of a market (`components/schemas/CurrentReward`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CurrentRewardConfig"></a>`struct CurrentRewardConfig`

```rust
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
    pub rate_per_day: Decimal,
    /// Total rewards amount (a JSON number on the wire).
    pub total_rewards: Option<Decimal>,
}
```

A reward configuration of a market in the current-rewards listing
(`components/schemas/CurrentRewardConfig`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.FeeDetails"></a>`struct FeeDetails`

```rust
#[non_exhaustive]
pub struct FeeDetails {
    /// Fee rate (wire name `r`; a JSON number on the wire).
    pub rate: Option<Decimal>,
    /// Fee curve exponent (wire name `e`; a JSON number on the wire).
    pub exponent: Option<Decimal>,
    /// Whether fees apply to takers only (wire name `to`).
    pub takers_only: Option<bool>,
}
```

Fee curve parameters of a market (`components/schemas/FeeDetails`).

Every field is optional and nullable.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.FeeRate"></a>`struct FeeRate`

```rust
#[non_exhaustive]
pub struct FeeRate {
    /// Base fee in basis points.
    pub base_fee: i64,
}
```

The base fee rate of a token (`components/schemas/FeeRate`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `Hash`, `PartialEq`, `Serialize`

### <a id="struct.GetBatchPricesHistory"></a>`struct GetBatchPricesHistory`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetBatchPricesHistory { /* private fields */ }
```

Request builder for [`ClobClient::get_batch_prices_history`](clob.md#ClobClient.fn.get_batch_prices_history).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetBatchPricesHistory.fn.start_ts"></a>`start_ts`

```rust
pub fn start_ts(self, start: DateTime<Utc>) -> Self
```

Only points after this time (`start_ts`, sent as Unix seconds).

##### <a id="GetBatchPricesHistory.fn.end_ts"></a>`end_ts`

```rust
pub fn end_ts(self, end: DateTime<Utc>) -> Self
```

Only points before this time (`end_ts`, sent as Unix seconds).

##### <a id="GetBatchPricesHistory.fn.interval"></a>`interval`

```rust
pub fn interval(self, interval: PriceHistoryInterval) -> Self
```

Time interval for data aggregation.

##### <a id="GetBatchPricesHistory.fn.fidelity"></a>`fidelity`

```rust
pub fn fidelity(self, minutes: u32) -> Self
```

Accuracy of the data in minutes (server default: 1 minute). The `1m` and `1w`
intervals need a minimum (see [`PriceHistoryInterval`](clob.md#enum.PriceHistoryInterval)).

##### <a id="GetBatchPricesHistory.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<BatchPricesHistory>
```

Sends the request.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) (parameter `markets`) if there are no
markets (the field is required), more than [`MAX_BATCH_PRICES_HISTORY_MARKETS`](clob.md#constant.MAX_BATCH_PRICES_HISTORY_MARKETS), or
an empty one; (parameter `fidelity`) if the interval is `1m` or `1w` and the fidelity
is missing or below the server's minimum (10 and 5 minutes). Missing or invalid
parameters are an [`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

### <a id="struct.GetFeeRate"></a>`struct GetFeeRate`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetFeeRate { /* private fields */ }
```

Request builder for [`ClobClient::get_fee_rate`](clob.md#ClobClient.fn.get_fee_rate).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetFeeRate.fn.token_id"></a>`token_id`

```rust
pub fn token_id(self, token_id: impl Into<TokenId>) -> Self
```

Token id (asset id), the `token_id` query parameter.

##### <a id="GetFeeRate.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<FeeRate>
```

Sends the request.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if the token id is set but empty. An
invalid token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an
unknown market one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other
cases.

### <a id="struct.GetNegRisk"></a>`struct GetNegRisk`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetNegRisk { /* private fields */ }
```

Request builder for [`ClobClient::get_neg_risk`](clob.md#ClobClient.fn.get_neg_risk).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetNegRisk.fn.token_id"></a>`token_id`

```rust
pub fn token_id(self, token_id: impl Into<TokenId>) -> Self
```

Token id (asset id), the `token_id` query parameter.

##### <a id="GetNegRisk.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<NegRisk>
```

Sends the request.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if the token id is set but empty. An
invalid token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an
unknown market one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other
cases.

### <a id="struct.GetPricesHistory"></a>`struct GetPricesHistory`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetPricesHistory { /* private fields */ }
```

Request builder for [`ClobClient::get_prices_history`](clob.md#ClobClient.fn.get_prices_history).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetPricesHistory.fn.start_ts"></a>`start_ts`

```rust
pub fn start_ts(self, start: DateTime<Utc>) -> Self
```

Only points after this time (`startTs`, sent as Unix seconds).

This endpoint's docs say only "unix timestamp"; live confirms seconds (milliseconds
are rejected as an interval that is too long). The server also appends one point at
the current time even when it lies after `end_ts` (see `SPEC_DEVIATIONS.md`).

##### <a id="GetPricesHistory.fn.end_ts"></a>`end_ts`

```rust
pub fn end_ts(self, end: DateTime<Utc>) -> Self
```

Only points before this time (`endTs`, sent as Unix seconds; see
[`start_ts`](clob.md#GetPricesHistory.fn.start_ts) on the unit).

##### <a id="GetPricesHistory.fn.interval"></a>`interval`

```rust
pub fn interval(self, interval: PriceHistoryInterval) -> Self
```

Time interval for data aggregation.

##### <a id="GetPricesHistory.fn.fidelity"></a>`fidelity`

```rust
pub fn fidelity(self, minutes: u32) -> Self
```

Accuracy of the data in minutes (server default: 1 minute). The `1m` and `1w`
intervals need a minimum (see [`PriceHistoryInterval`](clob.md#enum.PriceHistoryInterval)).

##### <a id="GetPricesHistory.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<PricesHistory>
```

Sends the request.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if `market` is empty, or (parameter
`fidelity`) if the interval is `1m` or `1w` and the fidelity is missing or below the
server's minimum (10 and 5 minutes). Missing or invalid parameters are an
[`Error::Api`](marcasite.md#enum.Error) with status `400`. See [`Error`](marcasite.md#enum.Error) for
the other cases.

### <a id="struct.GetTickSize"></a>`struct GetTickSize`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetTickSize { /* private fields */ }
```

Request builder for [`ClobClient::get_tick_size`](clob.md#ClobClient.fn.get_tick_size).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetTickSize.fn.token_id"></a>`token_id`

```rust
pub fn token_id(self, token_id: impl Into<TokenId>) -> Self
```

Token id (asset id), the `token_id` query parameter.

##### <a id="GetTickSize.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<TickSize>
```

Sends the request.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if the token id is set but empty. An
invalid token id is an [`Error::Api`](marcasite.md#enum.Error) with status `400`, an
unknown market one with status `404`. See [`Error`](marcasite.md#enum.Error) for the other
cases.

### <a id="struct.LastTradePrice"></a>`struct LastTradePrice`

```rust
#[non_exhaustive]
pub struct LastTradePrice {
    /// Last trade price (`0.5` if there were no trades; a numeric string on the wire).
    pub price: Decimal,
    /// Last trade side; `None` when the server sends the documented empty string (no
    /// trades). `None` serializes back to `""`.
    pub side: Option<Side>,
}
```

The last trade of a token (`GET /last-trade-price` response).

If the token has no trades, the server returns a price of `0.5` and an empty side.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ListBuilderTrades"></a>`struct ListBuilderTrades`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBuilderTrades { /* private fields */ }
```

Request builder for [`ClobClient::list_builder_trades`](clob.md#ClobClient.fn.list_builder_trades).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListBuilderTrades.fn.id"></a>`id`

```rust
pub fn id(self, id: impl Into<TradeId>) -> Self
```

Only the trade with this id.

##### <a id="ListBuilderTrades.fn.market"></a>`market`

```rust
pub fn market(self, market: impl Into<ConditionId>) -> Self
```

Only trades in this market (condition id).

##### <a id="ListBuilderTrades.fn.asset_id"></a>`asset_id`

```rust
pub fn asset_id(self, asset_id: impl Into<TokenId>) -> Self
```

Only trades of this asset id (token id).

##### <a id="ListBuilderTrades.fn.before"></a>`before`

```rust
pub fn before(self, before: DateTime<Utc>) -> Self
```

Only trades before this time (sent as Unix seconds).

##### <a id="ListBuilderTrades.fn.after"></a>`after`

```rust
pub fn after(self, after: DateTime<Utc>) -> Self
```

Only trades after this time (sent as Unix seconds).

##### <a id="ListBuilderTrades.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a previous
page's [`next_cursor()`](clob.md#Page.fn.next_cursor). Omit for the first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no more
pages: [`into_stream`](clob.md#ListBuilderTrades.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListBuilderTrades.fn.send) still sends it as given.

##### <a id="ListBuilderTrades.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<BuilderTrade>>
```

Fetches one page.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if the builder code or market does not
match the documented pattern (`0x` followed by 64 hex characters), the trade id or
asset id is set but empty, or a time filter is before the Unix epoch. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListBuilderTrades.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<BuilderTrade>
```

Streams every trade from the configured cursor onwards, fetching pages lazily until
the last page (`next_cursor` `"LTE="`). The stream ends after yielding the first
error (e.g. a validation error, before any request is sent).

### <a id="struct.ListCurrentRewards"></a>`struct ListCurrentRewards`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListCurrentRewards { /* private fields */ }
```

Request builder for [`ClobClient::list_current_rewards`](clob.md#ClobClient.fn.list_current_rewards).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListCurrentRewards.fn.sponsored"></a>`sponsored`

```rust
pub fn sponsored(self, sponsored: bool) -> Self
```

If `true`, returns sponsored reward configurations instead of the standard ones
(server default `false`).

##### <a id="ListCurrentRewards.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a previous
page's [`next_cursor()`](clob.md#Page.fn.next_cursor). Omit for the first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no more
pages: [`into_stream`](clob.md#ListCurrentRewards.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListCurrentRewards.fn.send) still sends it as given.

##### <a id="ListCurrentRewards.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<CurrentReward>>
```

Fetches one page.

###### Errors

An invalid cursor is an [`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListCurrentRewards.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<CurrentReward>
```

Streams every configuration from the configured cursor onwards, fetching pages
lazily until the last page (`next_cursor` `"LTE="`). The stream ends after yielding
the first error.

### <a id="struct.ListMarketsWithRewards"></a>`struct ListMarketsWithRewards`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListMarketsWithRewards { /* private fields */ }
```

Request builder for [`ClobClient::list_markets_with_rewards`](clob.md#ClobClient.fn.list_markets_with_rewards).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListMarketsWithRewards.fn.q"></a>`q`

```rust
pub fn q(self, q: impl Into<String>) -> Self
```

Text search on the market question and description.

##### <a id="ListMarketsWithRewards.fn.tag_slugs"></a>`tag_slugs`

```rust
pub fn tag_slugs(self, tag_slugs: impl IntoIterator<Item = impl Into<String>>) -> Self
```

Only markets with any of these tag slugs (OR), sent as one `tag_slug` query
parameter per value. Replaces previously set slugs.

##### <a id="ListMarketsWithRewards.fn.event_ids"></a>`event_ids`

```rust
pub fn event_ids(self, event_ids: impl IntoIterator<Item = impl Into<EventId>>) -> Self
```

Only markets of these events, sent as one `event_id` query parameter per value.
Replaces previously set ids.

##### <a id="ListMarketsWithRewards.fn.event_title"></a>`event_title`

```rust
pub fn event_title(self, event_title: impl Into<String>) -> Self
```

Search event titles (case-insensitive pattern matching).

##### <a id="ListMarketsWithRewards.fn.order_by"></a>`order_by`

```rust
pub fn order_by(self, order_by: RewardsMarketsOrderBy) -> Self
```

Field to sort by.

##### <a id="ListMarketsWithRewards.fn.position"></a>`position`

```rust
pub fn position(self, position: SortDirection) -> Self
```

Sort direction.

##### <a id="ListMarketsWithRewards.fn.min_volume_24hr"></a>`min_volume_24hr`

```rust
pub fn min_volume_24hr(self, min: Decimal) -> Self
```

Minimum 24-hour volume.

##### <a id="ListMarketsWithRewards.fn.max_volume_24hr"></a>`max_volume_24hr`

```rust
pub fn max_volume_24hr(self, max: Decimal) -> Self
```

Maximum 24-hour volume.

##### <a id="ListMarketsWithRewards.fn.min_spread"></a>`min_spread`

```rust
pub fn min_spread(self, min: Decimal) -> Self
```

Minimum spread.

##### <a id="ListMarketsWithRewards.fn.max_spread"></a>`max_spread`

```rust
pub fn max_spread(self, max: Decimal) -> Self
```

Maximum spread.

##### <a id="ListMarketsWithRewards.fn.min_price"></a>`min_price`

```rust
pub fn min_price(self, min: Decimal) -> Self
```

Minimum price of the first token.

##### <a id="ListMarketsWithRewards.fn.max_price"></a>`max_price`

```rust
pub fn max_price(self, max: Decimal) -> Self
```

Maximum price of the first token.

##### <a id="ListMarketsWithRewards.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a previous
page's [`next_cursor()`](clob.md#Page.fn.next_cursor). Omit for the first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no more
pages: [`into_stream`](clob.md#ListMarketsWithRewards.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListMarketsWithRewards.fn.send) still sends it as given.

##### <a id="ListMarketsWithRewards.fn.page_size"></a>`page_size`

```rust
pub fn page_size(self, page_size: u32) -> Self
```

Number of items per page (server default 100).

The docs give a maximum of [`MAX_REWARDS_MARKETS_PAGE_SIZE`](clob.md#constant.MAX_REWARDS_MARKETS_PAGE_SIZE) and say the server caps
larger values; this client rejects them instead (see [`send`](clob.md#ListMarketsWithRewards.fn.send)), so a
page never silently holds fewer items than asked for.

##### <a id="ListMarketsWithRewards.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<MultiMarketInfo>>
```

Fetches one page.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if
[`page_size`](clob.md#ListMarketsWithRewards.fn.page_size) exceeds [`MAX_REWARDS_MARKETS_PAGE_SIZE`](clob.md#constant.MAX_REWARDS_MARKETS_PAGE_SIZE). An invalid
`order_by`, `position` or cursor is an [`Error::Api`](marcasite.md#enum.Error) with status
`400`. See [`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListMarketsWithRewards.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<MultiMarketInfo>
```

Streams every market from the configured cursor onwards, fetching pages lazily until
the last page (`next_cursor` `"LTE="`). The stream ends after yielding the first
error.

### <a id="struct.ListRawRewardsForMarket"></a>`struct ListRawRewardsForMarket`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListRawRewardsForMarket { /* private fields */ }
```

Request builder for [`ClobClient::list_raw_rewards_for_market`](clob.md#ClobClient.fn.list_raw_rewards_for_market).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListRawRewardsForMarket.fn.sponsored"></a>`sponsored`

```rust
pub fn sponsored(self, sponsored: bool) -> Self
```

If `true`, folds sponsored daily rates into each configuration's `rate_per_day`
(server default `false`).

##### <a id="ListRawRewardsForMarket.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a previous
page's [`next_cursor()`](clob.md#Page.fn.next_cursor). Omit for the first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no more
pages: [`into_stream`](clob.md#ListRawRewardsForMarket.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListRawRewardsForMarket.fn.send) still sends it as given.

##### <a id="ListRawRewardsForMarket.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<MarketReward>>
```

Fetches one page.

###### Errors

[`Error::Validation`](marcasite.md#enum.Error) if the condition id is empty (which
the server documents as `400` "Invalid market") or is `current` or `multi` (the path
of another endpoint). An invalid market or cursor is an
[`Error::Api`](marcasite.md#enum.Error) with status `400`. See [`Error`](marcasite.md#enum.Error) for
the other cases.

##### <a id="ListRawRewardsForMarket.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<MarketReward>
```

Streams every configured market entry from the configured cursor onwards, fetching
pages lazily until the last page (`next_cursor` `"LTE="`). The stream ends after
yielding the first error.

### <a id="struct.ListSamplingMarkets"></a>`struct ListSamplingMarkets`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSamplingMarkets { /* private fields */ }
```

Request builder for [`ClobClient::list_sampling_markets`](clob.md#ClobClient.fn.list_sampling_markets).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListSamplingMarkets.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a
previous page's [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor). Omit for the
first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no
more pages: [`into_stream`](clob.md#ListSamplingMarkets.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListSamplingMarkets.fn.send) still sends it as given.

##### <a id="ListSamplingMarkets.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<MarketsPage<Market>>
```

Fetches one page.

###### Errors

An invalid request (e.g. an invalid cursor) is an
[`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListSamplingMarkets.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Market>
```

Streams every market from the configured cursor onwards, fetching pages
lazily until the last page. The stream ends after yielding the first error.

### <a id="struct.ListSamplingSimplifiedMarkets"></a>`struct ListSamplingSimplifiedMarkets`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSamplingSimplifiedMarkets { /* private fields */ }
```

Request builder for [`ClobClient::list_sampling_simplified_markets`](clob.md#ClobClient.fn.list_sampling_simplified_markets).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListSamplingSimplifiedMarkets.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a
previous page's [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor). Omit for the
first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no
more pages: [`into_stream`](clob.md#ListSamplingSimplifiedMarkets.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListSamplingSimplifiedMarkets.fn.send) still sends it as given.

##### <a id="ListSamplingSimplifiedMarkets.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<MarketsPage<SimplifiedMarket>>
```

Fetches one page.

###### Errors

An invalid request (e.g. an invalid cursor) is an
[`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListSamplingSimplifiedMarkets.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<SimplifiedMarket>
```

Streams every market from the configured cursor onwards, fetching pages
lazily until the last page. The stream ends after yielding the first error.

### <a id="struct.ListSimplifiedMarkets"></a>`struct ListSimplifiedMarkets`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListSimplifiedMarkets { /* private fields */ }
```

Request builder for [`ClobClient::list_simplified_markets`](clob.md#ClobClient.fn.list_simplified_markets).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListSimplifiedMarkets.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Cursor of the page to fetch (the `next_cursor` query parameter), from a
previous page's [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor). Omit for the
first page.

[`END_CURSOR`](clob.md#constant.END_CURSOR) (or an empty cursor) means there are no
more pages: [`into_stream`](clob.md#ListSimplifiedMarkets.fn.into_stream) then yields nothing, while
[`send`](clob.md#ListSimplifiedMarkets.fn.send) still sends it as given.

##### <a id="ListSimplifiedMarkets.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<MarketsPage<SimplifiedMarket>>
```

Fetches one page.

###### Errors

An invalid request (e.g. an invalid cursor) is an
[`Error::Api`](marcasite.md#enum.Error) with status `400`. See
[`Error`](marcasite.md#enum.Error) for the other cases.

##### <a id="ListSimplifiedMarkets.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<SimplifiedMarket>
```

Streams every market from the configured cursor onwards, fetching pages
lazily until the last page. The stream ends after yielding the first error.

### <a id="struct.LiveActivityMarket"></a>`struct LiveActivityMarket`

```rust
#[non_exhaustive]
pub struct LiveActivityMarket {
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Market id (a JSON integer).
    ///
    /// This is the same id as the Gamma market's `id` (live check, 2026-10-02), i.e. the
    /// integer behind a [`MarketId`](types.md#struct.MarketId); it stays an integer because
    /// the CLOB sends a JSON number (Gamma sends it as a string).
    pub id: Option<i64>,
    /// The market question.
    pub question: Option<String>,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// URL slug of the parent event.
    pub event_slug: Option<String>,
    /// URL slug of the series, if any.
    pub series_slug: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Tag slugs of the market.
    pub tags: Option<Vec<String>>,
}
```

Minimal market information for live-activity widgets
(`components/schemas/LiveActivityMarket`).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `Hash`, `PartialEq`, `Serialize`

### <a id="struct.Market"></a>`struct Market`

```rust
#[non_exhaustive]
pub struct Market {
    /// Whether the order book is enabled.
    pub enable_order_book: Option<bool>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// Whether the market is closed.
    pub closed: Option<bool>,
    /// Whether the market is archived.
    pub archived: Option<bool>,
    /// Whether the market accepts orders.
    pub accepting_orders: Option<bool>,
    /// Since when the market accepts orders.
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    /// Minimum order size (a JSON number on the wire).
    pub minimum_order_size: Option<Decimal>,
    /// Minimum tick size, the price increment (a JSON number on the wire).
    pub minimum_tick_size: Option<Decimal>,
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Question id, exactly as sent.
    ///
    /// This is the same id as the Gamma market's `questionID` (live check, 2026-10-02), i.e.
    /// a [`QuestionId`](types.md#struct.QuestionId) value; it stays a plain string because the
    /// spec types it as one.
    pub question_id: Option<String>,
    /// The market question.
    pub question: Option<String>,
    /// Market description.
    pub description: Option<String>,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// End date.
    pub end_date_iso: Option<DateTime<Utc>>,
    /// Game start time (sports markets).
    pub game_start_time: Option<DateTime<Utc>>,
    /// The `seconds_delay` value (integer; the spec does not document it further).
    pub seconds_delay: Option<i64>,
    /// The `fpmm` value (string; the spec does not document it further).
    pub fpmm: Option<String>,
    /// Maker base fee (integer; the spec does not state the unit).
    pub maker_base_fee: Option<i64>,
    /// Taker base fee (integer; the spec does not state the unit).
    pub taker_base_fee: Option<i64>,
    /// Whether notifications are enabled.
    pub notifications_enabled: Option<bool>,
    /// Whether negative risk is enabled for this market.
    pub neg_risk: Option<bool>,
    /// Negative-risk market id (a string; the spec does not document it further).
    pub neg_risk_market_id: Option<String>,
    /// Negative-risk request id (a string; the spec does not document it further).
    pub neg_risk_request_id: Option<String>,
    /// Icon URL.
    pub icon: Option<String>,
    /// Image URL.
    pub image: Option<String>,
    /// Liquidity rewards.
    pub rewards: Option<Rewards>,
    /// Whether the market is a 50/50 outcome market.
    pub is_50_50_outcome: Option<bool>,
    /// Outcome tokens.
    pub tokens: Option<Vec<Token>>,
    /// Tags.
    pub tags: Option<Vec<String>>,
}
```

A CLOB market (`components/schemas/Market`), as listed by
[`ClobClient::list_sampling_markets`](clob.md#ClobClient.fn.list_sampling_markets).

The spec marks no field as required and documents none of them; descriptions here are
limited to what the field names state.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MarketByToken"></a>`struct MarketByToken`

```rust
#[non_exhaustive]
pub struct MarketByToken {
    /// Condition id of the market containing the token.
    pub condition_id: ConditionId,
    /// The primary token id.
    ///
    /// The spec calls it "the primary (Yes) token", but live it is not reliably the first or
    /// "Yes" token of the market: it is one of the market's two tokens, chosen independently
    /// of outcome order (see `SPEC_DEVIATIONS.md`). Look the outcome up with
    /// [`ClobClient::get_clob_market_info`](clob.md#ClobClient.fn.get_clob_market_info) if it matters.
    pub primary_token_id: TokenId,
    /// The secondary token id (the market's other token; see
    /// [`primary_token_id`](clob.md#struct.MarketByToken)).
    pub secondary_token_id: TokenId,
}
```

The parent market of a token (`components/schemas/MarketByTokenResponse`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `Hash`, `PartialEq`, `Serialize`

### <a id="struct.MarketReward"></a>`struct MarketReward`

```rust
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
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    pub rewards_min_size: Option<Decimal>,
    /// Competitiveness score of the market (a JSON number on the wire).
    pub market_competitiveness: Option<Decimal>,
    /// Outcome tokens.
    pub tokens: Vec<RewardsToken>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<RewardsConfig>>,
}
```

A market with its raw reward configurations (`components/schemas/MarketReward`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MarketsPage"></a>`struct MarketsPage`

```rust
#[non_exhaustive]
pub struct MarketsPage<T> {
    /// Maximum number of items per page.
    pub limit: Option<u32>,
    /// Cursor for the next page, exactly as sent by the server (wire name `next_cursor`);
    /// prefer the [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor) method, which maps the
    /// [`END_CURSOR`](clob.md#constant.END_CURSOR) sentinel and an empty cursor to `None`.
    pub next_cursor: Option<String>,
    /// Number of items in this page.
    pub count: Option<u32>,
    /// The items on this page (`None` if the server omitted the field).
    pub data: Option<Vec<T>>,
}
```

One page of the CLOB market listings (`/simplified-markets`, `/sampling-markets`,
`/sampling-simplified-markets`).

Mirrors `PaginatedSimplifiedMarkets` and `PaginatedMarkets` in the CLOB OpenAPI spec,
which mark no field as required, so every field is optional. The public fields hold the
wire values; [`items`](clob.md#MarketsPage.fn.items) and [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor) are the
service-independent accessors.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

```rust
impl<T> MarketsPage<T>
```

##### <a id="MarketsPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[T]
```

The items on this page (empty if the server omitted them).

##### <a id="MarketsPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<T>
```

Consumes the page and returns its items (empty if the server omitted them).

##### <a id="MarketsPage.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor to request the next page with (the listing's `cursor` setter), or `None`
on the last page: when the server sent no cursor, an empty cursor or
[`END_CURSOR`](clob.md#constant.END_CURSOR).

The spec documents no end sentinel for these listings; the CLOB-wide `"LTE="`
sentinel documented for its other cursor listings is assumed.

##### <a id="MarketsPage.fn.is_last_page"></a>`is_last_page`

```rust
#[must_use]
pub fn is_last_page(&self) -> bool
```

`true` if this is the last page (see [`next_cursor()`](clob.md#MarketsPage.fn.next_cursor)).

### <a id="struct.Midpoint"></a>`struct Midpoint`

```rust
#[non_exhaustive]
pub struct Midpoint {
    /// Midpoint price (a numeric string on the wire).
    ///
    /// The wire name is `mid`; the spec documents `mid_price`, which live never sends (see
    /// `SPEC_DEVIATIONS.md`). `mid_price` is still accepted when reading.
    pub mid: Decimal,
}
```

The midpoint price of a token: the average of the best bid and best ask
(`GET /midpoint` response).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MultiMarketInfo"></a>`struct MultiMarketInfo`

```rust
#[non_exhaustive]
pub struct MultiMarketInfo {
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Event id.
    pub event_id: Option<EventId>,
    /// URL slug of the event.
    pub event_slug: Option<String>,
    /// Market creation time.
    pub created_at: Option<DateTime<Utc>>,
    /// Title within an event group.
    pub group_item_title: Option<String>,
    /// Market image URL.
    pub image: Option<String>,
    /// Competitiveness score of the market (a JSON number on the wire).
    pub market_competitiveness: Option<Decimal>,
    /// Market id.
    pub market_id: MarketId,
    /// URL slug of the market.
    pub market_slug: Option<String>,
    /// Price change over the last 24 hours (a JSON number on the wire).
    pub one_day_price_change: Option<Decimal>,
    /// The market question.
    pub question: String,
    /// Maximum spread for rewards eligibility (a JSON number on the wire).
    pub rewards_max_spread: Option<Decimal>,
    /// Minimum order size for rewards eligibility (a JSON number on the wire).
    pub rewards_min_size: Option<Decimal>,
    /// Current spread (a JSON number on the wire).
    pub spread: Option<Decimal>,
    /// Market end date.
    ///
    /// The spec documents no format; the docs example is `"2024-08-10 00:00:00"` and live
    /// sends `"2025-05-01 12:00:00+00"` (UTC, a space instead of `T`, a short `+00` offset),
    /// or `""` for some markets, which is `None`. Both spellings are read as UTC. Serializes
    /// as RFC 3339.
    pub end_date: Option<DateTime<Utc>>,
    /// Outcome tokens.
    pub tokens: Vec<RewardsToken>,
    /// 24-hour trading volume (a JSON number on the wire).
    pub volume_24hr: Option<Decimal>,
    /// Reward configurations.
    pub rewards_config: Option<Vec<RewardsConfig>>,
}
```

A market with its reward configurations and trading metrics
(`components/schemas/MultiMarketInfo`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.NegRisk"></a>`struct NegRisk`

```rust
#[non_exhaustive]
pub struct NegRisk {
    /// Whether negative risk is enabled for this market.
    pub neg_risk: bool,
}
```

The negative-risk flag of a token's market (`components/schemas/NegRisk`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `Hash`, `PartialEq`, `Serialize`

### <a id="struct.OrderBookSummary"></a>`struct OrderBookSummary`

```rust
#[non_exhaustive]
pub struct OrderBookSummary {
    /// Market condition id.
    pub market: ConditionId,
    /// Token id (asset id).
    pub asset_id: TokenId,
    /// Timestamp of the snapshot.
    ///
    /// Sent as a numeric string of Unix **milliseconds** (e.g. `"1790934258308"`; the REST
    /// spec does not document the unit, live confirms it) and serialized back to a string.
    pub timestamp: DateTime<Utc>,
    /// Hash of the order book summary, exactly as sent.
    ///
    /// Live this is 40 hex characters without a `0x` prefix (the spec only has a placeholder
    /// example), so it is kept as a plain string.
    pub hash: String,
    /// Bids, sorted by price descending.
    pub bids: Vec<OrderSummary>,
    /// Asks, sorted by price ascending.
    pub asks: Vec<OrderSummary>,
    /// Minimum order size (a numeric string on the wire).
    pub min_order_size: Decimal,
    /// Minimum price increment (tick size; a numeric string on the wire).
    pub tick_size: Decimal,
    /// Whether negative risk is enabled for this market.
    pub neg_risk: bool,
    /// Last trade price (a numeric string on the wire).
    pub last_trade_price: Decimal,
}
```

An order book snapshot for one token (`components/schemas/OrderBookSummary`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.OrderId"></a>`struct OrderId`

```rust
pub struct OrderId(/* private fields */);
```

A CLOB order id, which is the order hash (e.g. a builder trade's `takerOrderHash`).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&OrderId>`, `From<&String>`, `From<&str>`, `From<OrderId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="OrderId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="OrderId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="OrderId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.OrderSummary"></a>`struct OrderSummary`

```rust
#[non_exhaustive]
pub struct OrderSummary {
    /// Price of the level (a numeric string on the wire).
    pub price: Decimal,
    /// Total size resting at this price (a numeric string on the wire).
    pub size: Decimal,
}
```

A price level of an order book (`components/schemas/OrderSummary`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Page"></a>`struct Page`

```rust
#[non_exhaustive]
pub struct Page<T> {
    /// Maximum number of items per page.
    pub limit: u32,
    /// Number of items in this page.
    pub count: u32,
    /// Cursor for the next page, exactly as sent by the server (wire name `next_cursor`).
    /// [`END_CURSOR`](clob.md#constant.END_CURSOR) (`"LTE="`) marks the last page; prefer the
    /// [`next_cursor()`](clob.md#Page.fn.next_cursor) method, which maps it to `None`.
    pub next_cursor: String,
    /// The items on this page.
    pub data: Vec<T>,
}
```

One page of a CLOB cursor-paginated listing whose envelope fields are all required
(rewards listings and builder trades).

Mirrors `PaginatedCurrentReward`, `PaginatedMarketReward`, `PaginatedMultiMarketInfo`
and `BuilderTradesResponse` in the CLOB OpenAPI spec. The public fields hold the wire
values; [`items`](clob.md#Page.fn.items) and [`next_cursor()`](clob.md#Page.fn.next_cursor) are the
service-independent accessors.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

```rust
impl<T> Page<T>
```

##### <a id="Page.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[T]
```

The items on this page.

##### <a id="Page.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<T>
```

Consumes the page and returns its items.

##### <a id="Page.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor to request the next page with (the listing's `cursor` setter), or `None`
on the last page: when the server sent [`END_CURSOR`](clob.md#constant.END_CURSOR) or an empty cursor.

##### <a id="Page.fn.is_last_page"></a>`is_last_page`

```rust
#[must_use]
pub fn is_last_page(&self) -> bool
```

`true` if this is the last page (see [`next_cursor()`](clob.md#Page.fn.next_cursor)).

### <a id="struct.Price"></a>`struct Price`

```rust
#[non_exhaustive]
pub struct Price {
    /// Market price (a numeric string on the wire; the spec documents a JSON number, which
    /// is accepted as well).
    pub price: Decimal,
}
```

The best price for a token and side (`GET /price` response): the best bid for
[`Side::Buy`](types.md#enum.Side), the best ask for [`Side::Sell`](types.md#enum.Side).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PricePoint"></a>`struct PricePoint`

```rust
#[non_exhaustive]
pub struct PricePoint {
    /// Time of the point (wire name `t`).
    ///
    /// The spec types it as a `uint32` without stating the unit; it is decoded as Unix
    /// seconds, the only Unix-timestamp unit a `uint32` can hold for current dates and the
    /// unit documented for the `start_ts` / `end_ts` filters.
    pub timestamp: Option<DateTime<Utc>>,
    /// Price at that time (wire name `p`; a JSON number on the wire).
    pub price: Option<Decimal>,
}
```

One point of a price history (`components/schemas/MarketPrice`).

The spec marks neither field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PricesHistory"></a>`struct PricesHistory`

```rust
#[non_exhaustive]
pub struct PricesHistory {
    /// The price points.
    pub history: Option<Vec<PricePoint>>,
}
```

The price history of one market (`components/schemas/PricesHistoryResponse`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RebatedFees"></a>`struct RebatedFees`

```rust
#[non_exhaustive]
pub struct RebatedFees {
    /// Date of the rebate.
    ///
    /// The spec documents `format: date` (`YYYY-MM-DD`), but live sends an RFC 3339
    /// date-time at midnight UTC (e.g. `"2026-09-25T00:00:00Z"`). Both are accepted; the
    /// time of day is dropped. Serializes as `YYYY-MM-DD`.
    pub date: NaiveDate,
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Asset address (e.g. the USDC contract).
    pub asset_address: Address,
    /// The maker's address.
    pub maker_address: Address,
    /// Rebated fee amount in USDC (a numeric string on the wire).
    pub rebated_fees_usdc: Decimal,
}
```

Fees rebated to a maker on one market and date (`components/schemas/RebatedFees`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RewardRate"></a>`struct RewardRate`

```rust
#[non_exhaustive]
pub struct RewardRate {
    /// Address of the reward asset.
    pub asset_address: Option<Address>,
    /// Daily reward rate (a JSON number on the wire).
    pub rewards_daily_rate: Option<Decimal>,
}
```

A reward rate of a market (an item of `Rewards.rates`).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Rewards"></a>`struct Rewards`

```rust
#[non_exhaustive]
pub struct Rewards {
    /// Reward rates per reward asset.
    pub rates: Option<Vec<RewardRate>>,
    /// Minimum order size to be eligible for rewards (a JSON number on the wire).
    pub min_size: Option<Decimal>,
    /// Maximum spread to be eligible for rewards (a JSON number on the wire).
    pub max_spread: Option<Decimal>,
}
```

The liquidity rewards of a market (`components/schemas/Rewards`).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RewardsConfig"></a>`struct RewardsConfig`

```rust
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
    pub rate_per_day: Decimal,
    /// Total rewards amount (a JSON number on the wire).
    pub total_rewards: Option<Decimal>,
    /// Remaining reward amount (a JSON number on the wire).
    pub remaining_reward_amount: Option<Decimal>,
    /// Total number of days in the rewards period.
    pub total_days: Option<i64>,
}
```

A reward configuration of a market (`components/schemas/RewardsConfig`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RewardsToken"></a>`struct RewardsToken`

```rust
#[non_exhaustive]
pub struct RewardsToken {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Outcome name (e.g. `"YES"`).
    pub outcome: String,
    /// Current price of the token (a JSON number on the wire).
    pub price: Option<Decimal>,
}
```

A token of a rewards market (`components/schemas/RewardsToken`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SimplifiedMarket"></a>`struct SimplifiedMarket`

```rust
#[non_exhaustive]
pub struct SimplifiedMarket {
    /// Condition id of the market.
    pub condition_id: Option<ConditionId>,
    /// Liquidity rewards.
    pub rewards: Option<Rewards>,
    /// Outcome tokens.
    pub tokens: Option<Vec<Token>>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// Whether the market is closed.
    pub closed: Option<bool>,
    /// Whether the market is archived.
    pub archived: Option<bool>,
    /// Whether the market accepts orders.
    pub accepting_orders: Option<bool>,
}
```

A market in its simplified form (`components/schemas/SimplifiedMarket`), as listed by
[`ClobClient::list_simplified_markets`](clob.md#ClobClient.fn.list_simplified_markets) and
[`ClobClient::list_sampling_simplified_markets`](clob.md#ClobClient.fn.list_sampling_simplified_markets).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Spread"></a>`struct Spread`

```rust
#[non_exhaustive]
pub struct Spread {
    /// The spread (a numeric string on the wire).
    pub spread: Decimal,
}
```

The spread of a token: best ask minus best bid (`GET /spread` response).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TickSize"></a>`struct TickSize`

```rust
#[non_exhaustive]
pub struct TickSize {
    /// Minimum tick size (price increment; a JSON number on the wire).
    pub minimum_tick_size: Decimal,
}
```

The minimum tick size of a token (`components/schemas/TickSize`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Token"></a>`struct Token`

```rust
#[non_exhaustive]
pub struct Token {
    /// Token id (asset id).
    pub token_id: Option<TokenId>,
    /// Outcome label (e.g. `"Yes"`).
    pub outcome: Option<String>,
    /// Price of the token (a JSON number on the wire).
    pub price: Option<Decimal>,
    /// Whether this outcome won.
    pub winner: Option<bool>,
}
```

An outcome token of a market (`components/schemas/Token`).

The spec marks no field as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TokenLastTradePrice"></a>`struct TokenLastTradePrice`

```rust
#[non_exhaustive]
pub struct TokenLastTradePrice {
    /// Token id (asset id).
    pub token_id: TokenId,
    /// Last trade price (a numeric string on the wire).
    pub price: Decimal,
    /// Last trade side.
    ///
    /// The spec documents only `BUY` / `SELL` here. `GET /last-trade-price` documents an
    /// empty side for a token without trades; the same value is mapped to `None` here, as
    /// in [`LastTradePrice::side`](clob.md#struct.LastTradePrice), and `None` serializes back to `""`.
    pub side: Option<Side>,
}
```

The last trade of one token in a batch (`GET`/`POST /last-trades-prices` response
items).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TradeId"></a>`struct TradeId`

```rust
pub struct TradeId(/* private fields */);
```

A CLOB trade id, e.g. `"trade-123"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TradeId>`, `From<&str>`, `From<String>`, `From<TradeId>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TradeId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TradeId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TradeId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

## Enums

### <a id="enum.PriceHistoryInterval"></a>`enum PriceHistoryInterval`

```rust
#[non_exhaustive]
pub enum PriceHistoryInterval {
    /// `max`.
    Max,
    /// `all`.
    All,
    /// `1m`: one month (needs `fidelity` of at least 10).
    OneMonth,
    /// `1w`: one week (needs `fidelity` of at least 5).
    OneWeek,
    /// `1d`: one day.
    OneDay,
    /// `6h`: six hours.
    SixHours,
    /// `1h`: one hour.
    OneHour,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Time interval for price-history aggregation (the `interval` parameter).

The CLOB spec lists the values (`max`, `all`, `1m`, `1w`, `1d`, `6h`, `1h`) without
describing them. Live checks (2026-10-02) show they are windows ending now:

- `1h`, `6h`, `1d`, `1w` are one hour, six hours, one day and one week;
- **`1m` is one month** (about 30 days), not one minute;
- `max` and `all` are the same: the whole available history, up to a cap on the number
  of points (about 4,300 observed), so a small `fidelity` only reaches back a few
  weeks (30 days at 10 minutes) while `1440` covers more than a year.

The server enforces a minimum `fidelity` per window: **10 minutes for `1m`**, 5 for
`1w`, and answers `400` below it, including when `fidelity` is omitted (the default is
1 minute). The request builders check this client-side
([`MIN_FIDELITY_ONE_MONTH`](clob.md#constant.MIN_FIDELITY_ONE_MONTH), [`MIN_FIDELITY_ONE_WEEK`](clob.md#constant.MIN_FIDELITY_ONE_WEEK)). When `start_ts`/`end_ts` are
sent as well, the window of the timestamps is used, but the fidelity minimum of the
interval still applies. See `SPEC_DEVIATIONS.md`.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PriceHistoryInterval.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PriceHistoryInterval.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.RewardsMarketsOrderBy"></a>`enum RewardsMarketsOrderBy`

```rust
#[non_exhaustive]
pub enum RewardsMarketsOrderBy {
    /// `market_id`.
    MarketId,
    /// `created_at`.
    CreatedAt,
    /// `volume_24hr`.
    Volume24hr,
    /// `spread`.
    Spread,
    /// `competitiveness`.
    Competitiveness,
    /// `max_spread`.
    MaxSpread,
    /// `min_size`.
    MinSize,
    /// `question`.
    Question,
    /// `one_day_price_change`.
    OneDayPriceChange,
    /// `rate_per_day`.
    RatePerDay,
    /// `price`.
    Price,
    /// `end_date`.
    EndDate,
    /// `start_date`.
    StartDate,
    /// `reward_end_date`.
    RewardEndDate,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Sort field of [`ClobClient::list_markets_with_rewards`](clob.md#ClobClient.fn.list_markets_with_rewards) (the `order_by` parameter).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="RewardsMarketsOrderBy.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="RewardsMarketsOrderBy.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.SortDirection"></a>`enum SortDirection`

```rust
#[non_exhaustive]
pub enum SortDirection {
    /// Ascending (`ASC`).
    Asc,
    /// Descending (`DESC`).
    Desc,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Sort direction (the `position` parameter).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SortDirection.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="SortDirection.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

## Constants

### <a id="constant.END_CURSOR"></a>`const END_CURSOR`

```rust
pub const END_CURSOR: &str = "LTE=";
```

The `next_cursor` value the CLOB returns on the last page of a cursor-paginated listing.

Documented for the rewards and builder-trades listings ("A next_cursor value of `"LTE="`
indicates the last page"). The page types in this module treat it, and an empty cursor,
as "no next page": their `next_cursor()` methods return `None` for it, and a listing's
`into_stream()` started at it yields nothing.

### <a id="constant.MAX_BATCH_PRICES_HISTORY_MARKETS"></a>`const MAX_BATCH_PRICES_HISTORY_MARKETS`

```rust
pub const MAX_BATCH_PRICES_HISTORY_MARKETS: usize = 20;
```

Maximum number of markets per [`ClobClient::get_batch_prices_history`](clob.md#ClobClient.fn.get_batch_prices_history) request.

### <a id="constant.MAX_LAST_TRADE_PRICES_TOKEN_IDS"></a>`const MAX_LAST_TRADE_PRICES_TOKEN_IDS`

```rust
pub const MAX_LAST_TRADE_PRICES_TOKEN_IDS: usize = 500;
```

Maximum number of token ids per last-trade-prices request
([`ClobClient::get_last_trade_prices`](clob.md#ClobClient.fn.get_last_trade_prices)).

### <a id="constant.MAX_REWARDS_MARKETS_PAGE_SIZE"></a>`const MAX_REWARDS_MARKETS_PAGE_SIZE`

```rust
pub const MAX_REWARDS_MARKETS_PAGE_SIZE: u32 = 500;
```

Maximum `page_size` of [`ClobClient::list_markets_with_rewards`](clob.md#ClobClient.fn.list_markets_with_rewards).

### <a id="constant.MIN_FIDELITY_ONE_MONTH"></a>`const MIN_FIDELITY_ONE_MONTH`

```rust
pub const MIN_FIDELITY_ONE_MONTH: u32 = 10;
```

Minimum `fidelity` (in minutes) the server accepts with [`PriceHistoryInterval::OneMonth`](clob.md#enum.PriceHistoryInterval).

### <a id="constant.MIN_FIDELITY_ONE_WEEK"></a>`const MIN_FIDELITY_ONE_WEEK`

```rust
pub const MIN_FIDELITY_ONE_WEEK: u32 = 5;
```

Minimum `fidelity` (in minutes) the server accepts with [`PriceHistoryInterval::OneWeek`](clob.md#enum.PriceHistoryInterval).
