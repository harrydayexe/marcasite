# Module `marcasite::data`

> Generated from marcasite 0.1.1 (all features) by `just docs-md`. Do not edit.

Data API v2 client (`https://data-api.polymarket.com`, routes under `/v2`).

Covers wallet portfolios, trade and activity feeds, per-market state, ranked boards and
data freshness. Every route is public: no authentication is needed. Start from
[`DataClient`](data.md#struct.DataClient).

See <https://docs.polymarket.com/api-reference/data-api/overview>.

## Endpoints

| Endpoint | Method | Returns |
|---|---|---|
| **Wallet** | | |
| `GET /v2/positions` | [`list_positions`](data.md#DataClient.fn.list_positions) | [`Page`](data.md#struct.Page)`<`[`Position`](data.md#struct.Position)`>` |
| `GET /v2/positions/combos` | [`list_combo_positions`](data.md#DataClient.fn.list_combo_positions) | [`Page`](data.md#struct.Page)`<`[`ComboPosition`](data.md#struct.ComboPosition)`>` |
| `GET /v2/value` | [`get_portfolio_value`](data.md#DataClient.fn.get_portfolio_value) | [`PortfolioValue`](data.md#struct.PortfolioValue) |
| `GET /v2/approvals` | [`get_approvals`](data.md#DataClient.fn.get_approvals) | [`Approvals`](data.md#struct.Approvals) |
| `GET /v2/user-pnl` | [`get_user_pnl`](data.md#DataClient.fn.get_user_pnl) | [`UserPnlSeries`](data.md#struct.UserPnlSeries) |
| `GET /v2/user-stats` | [`get_user_stats`](data.md#DataClient.fn.get_user_stats) | `Option<`[`UserStats`](data.md#struct.UserStats)`>` |
| `GET /v2/user-volume` | [`get_user_volume`](data.md#DataClient.fn.get_user_volume) | [`UserVolume`](data.md#struct.UserVolume) |
| **Feeds** | | |
| `GET /v2/trades` | [`list_trades`](data.md#DataClient.fn.list_trades) | [`Page`](data.md#struct.Page)`<`[`Trade`](data.md#struct.Trade)`>` |
| `GET /v2/activity` | [`list_activity`](data.md#DataClient.fn.list_activity) | [`Page`](data.md#struct.Page)`<`[`Activity`](data.md#struct.Activity)`>` |
| `GET /v2/activity/combos` | [`list_combo_activity`](data.md#DataClient.fn.list_combo_activity) | [`Page`](data.md#struct.Page)`<`[`ComboActivity`](data.md#struct.ComboActivity)`>` |
| **Markets** | | |
| `GET /v2/holders` | [`list_holders`](data.md#DataClient.fn.list_holders) | [`Page`](data.md#struct.Page)`<`[`HolderGroup`](data.md#struct.HolderGroup)`>` |
| `GET /v2/oi` | [`get_open_interest`](data.md#DataClient.fn.get_open_interest) | `Vec<`[`OpenInterest`](data.md#struct.OpenInterest)`>` |
| `GET /v2/live-volume` | [`get_live_volume`](data.md#DataClient.fn.get_live_volume) | [`LiveVolume`](data.md#struct.LiveVolume) |
| `GET /v2/prices-history` | [`list_prices_history`](data.md#DataClient.fn.list_prices_history) | [`Page`](data.md#struct.Page)`<`[`PricePoint`](data.md#struct.PricePoint)`>` |
| `GET /v2/resolutions` | [`get_resolutions`](data.md#DataClient.fn.get_resolutions) | `Vec<`[`Resolution`](data.md#struct.Resolution)`>` |
| **Boards** | | |
| `GET /v2/leaderboard` | [`list_leaderboard`](data.md#DataClient.fn.list_leaderboard) | [`Page`](data.md#struct.Page)`<`[`LeaderboardEntry`](data.md#struct.LeaderboardEntry)`>` |
| `GET /v2/leaderboard?user=` | [`get_leaderboard_standing`](data.md#DataClient.fn.get_leaderboard_standing) | `Option<`[`LeaderboardUserEntry`](data.md#struct.LeaderboardUserEntry)`>` |
| `GET /v2/biggest-winners` | [`list_biggest_winners`](data.md#DataClient.fn.list_biggest_winners) | [`Page`](data.md#struct.Page)`<`[`BiggestWinner`](data.md#struct.BiggestWinner)`>` |
| `GET /v2/builders/leaderboard` | [`list_builders_leaderboard`](data.md#DataClient.fn.list_builders_leaderboard) | [`Page`](data.md#struct.Page)`<`[`BuilderStanding`](data.md#struct.BuilderStanding)`>` |
| `GET /v2/builders/volume` | [`get_builders_volume`](data.md#DataClient.fn.get_builders_volume) | `Vec<`[`BuilderVolumePoint`](data.md#struct.BuilderVolumePoint)`>` |
| **Service** | | |
| `GET /v2/status` | [`get_status`](data.md#DataClient.fn.get_status) | [`ServiceStatus`](data.md#struct.ServiceStatus) |

Methods named `list_*` are cursor-paginated: their builders have a `cursor(..)` setter
and an `into_stream()`. Methods named `get_*` return a single resource or a fixed
series.

## Conventions

- **Envelope.** Every response wraps its payload in `data`; the client unwraps it.
  Non-paginated routes return the payload directly. A documented miss is `data: null`
  (returned as `Ok(None)`, e.g. [`DataClient::get_user_stats`](data.md#DataClient.fn.get_user_stats)) or an empty list,
  never an error.
- **Cursor pagination.** Paginated routes return a [`Page`](data.md#struct.Page): its rows
  ([`Page::items`](data.md#Page.fn.items)), the raw [`Pagination`](data.md#struct.Pagination) envelope, and the response's `x-trace-id`
  ([`Page::trace_id`](data.md#Page.fn.trace_id)). There is no `offset` request parameter (the API rejects it,
  and this client never sends it). Follow [`Page::next_cursor`](data.md#Page.fn.next_cursor) with the request's
  `cursor(..)` setter until it is `None`, or use `into_stream()` to walk every page
  lazily. `has_more` is exact, so a short or empty page does not end the walk, and a
  page with `has_more: false` ends it even if it carries a cursor. Streams also stop if
  the server hands back a cursor already visited during the walk.
- **Re-sent parameters.** Streams re-send every parameter on every page. The API
  requires it on the feeds (`/v2/trades`, `/v2/activity`, `/v2/activity/combos`, whose
  cursors carry only their seek anchor), `/v2/positions` (the `user`/`condition`
  anchor, `title` and the `start`/`end` window), `/v2/positions/combos` (`user`) and
  `/v2/holders` (`condition`); elsewhere restating the values the cursor was minted on
  is allowed. `limit` is re-sent too, which is harmless: it only sizes the first page,
  and once a cursor is supplied the cursor's own page size wins.
- **Units.** Bare `volume` and `size` values are outcome **shares**; fields suffixed
  `_usdc` are USD amounts; volumes prefixed `taker_` count one side of each trade. All
  amounts are `Decimal`s; they are JSON numbers on the wire and
  serialize back as JSON numbers.
- **Sentinels.** Documented sentinels are kept as served and documented on each field:
  `outcome_index` [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) (`999`) means the outcome could not be
  labeled, [`Position::last_event_at`](data.md#struct.Position) `0` means no native state,
  [`Position::end_date`](data.md#struct.Position) `1970-01-01` means Gamma has none, and
  [`BiggestWinner::event_id`](data.md#struct.BiggestWinner) `0` marks a combo row. Strings the API serves as `""`
  when absent are `None` and serialize back as `""`. A missing or `null` numeric field
  means *unavailable*, never zero, and is modelled as `None`.
- **Identifiers.** Condition ids are [`ConditionId`](types.md#struct.ConditionId)s, outcome token ids [`TokenId`](types.md#struct.TokenId)s,
  wallets [`Address`](types.md#struct.Address)es, and Gamma ids [`MarketId`](types.md#struct.MarketId) / [`EventId`](types.md#struct.EventId). Before sending, the
  client checks the formats the docs state: condition ids (combo condition ids
  included) and question ids must be `0x` followed by 64 hex digits, `user` must be an
  EVM address on `/v2/approvals` and `/v2/user-stats` and non-empty elsewhere, and
  event ids must be integers on `/v2/live-volume` and positive integers on
  `/v2/resolutions`. Comma-separated list parameters accept at most
  [`MAX_LIST_VALUES`](data.md#constant.MAX_LIST_VALUES) (20) distinct values; duplicates are sent once, and a value
  containing a comma is rejected. Violations are
  [`Error::Validation`](marcasite.md#enum.Error)s.
- **Time windows.** `start` / `end` style setters take a
  `DateTime<Utc>` and are sent as epoch seconds; a time before the
  Unix epoch is rejected. How an omitted or `0` bound behaves differs by route; see
  each setter.
- **Errors.** Error bodies carry `error`, `code`, `retryable`, `trace_id` and, for
  validation failures, `parameter`; they are exposed by [`ApiError`](marcasite.md#struct.ApiError).
  Use [`ErrorCode::from_error`](data.md#ErrorCode.fn.from_error) to branch on the typed code.
  [`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) follows the body's `retryable`
  flag. `429` and retryable `503` responses carry `Retry-After`
  ([`Error::retry_after`](marcasite.md#Error.fn.retry_after)), and every response carries an
  `x-trace-id` header ([`Error::trace_id`](marcasite.md#Error.fn.trace_id),
  [`Page::trace_id`](data.md#Page.fn.trace_id)).

## Example

```rust
use futures_util::TryStreamExt as _;
use marcasite::data::DataClient;

let data = DataClient::new()?;
let wallet = "0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748";
let value = data.get_portfolio_value(wallet).send().await?;
let positions: Vec<_> = data
    .list_positions()
    .user(wallet)
    .into_stream()
    .try_collect()
    .await?;
```

## Index

- **Re-exports:** `Address`, `ConditionId`, `EventId`, `MarketId`, `QuestionId`, `Side`, `TokenId`
- **Structs:** [`Activity`](#struct.Activity), [`ApprovalContract`](#struct.ApprovalContract), [`Approvals`](#struct.Approvals), [`BiggestWinner`](#struct.BiggestWinner), [`BuilderCode`](#struct.BuilderCode), [`BuilderStanding`](#struct.BuilderStanding), [`BuilderVolumePoint`](#struct.BuilderVolumePoint), [`ComboActivity`](#struct.ComboActivity), [`ComboLeg`](#struct.ComboLeg), [`ComboLegEvent`](#struct.ComboLegEvent), [`ComboLegMarket`](#struct.ComboLegMarket), [`ComboPosition`](#struct.ComboPosition), [`ConditionVolume`](#struct.ConditionVolume), [`CursorLag`](#struct.CursorLag), [`DataClient`](#struct.DataClient), [`DataClientBuilder`](#struct.DataClientBuilder), [`GetBuildersVolume`](#struct.GetBuildersVolume), [`GetLeaderboardStanding`](#struct.GetLeaderboardStanding), [`GetOpenInterest`](#struct.GetOpenInterest), [`GetPortfolioValue`](#struct.GetPortfolioValue), [`GetUserPnl`](#struct.GetUserPnl), [`GetUserVolume`](#struct.GetUserVolume), [`Holder`](#struct.Holder), [`HolderGroup`](#struct.HolderGroup), [`IngestionFreshness`](#struct.IngestionFreshness), [`LeaderboardEntry`](#struct.LeaderboardEntry), [`LeaderboardUserEntry`](#struct.LeaderboardUserEntry), [`ListActivity`](#struct.ListActivity), [`ListBiggestWinners`](#struct.ListBiggestWinners), [`ListBuildersLeaderboard`](#struct.ListBuildersLeaderboard), [`ListComboActivity`](#struct.ListComboActivity), [`ListComboPositions`](#struct.ListComboPositions), [`ListHolders`](#struct.ListHolders), [`ListLeaderboard`](#struct.ListLeaderboard), [`ListPositions`](#struct.ListPositions), [`ListPricesHistory`](#struct.ListPricesHistory), [`ListTrades`](#struct.ListTrades), [`LiveVolume`](#struct.LiveVolume), [`OpenInterest`](#struct.OpenInterest), [`Page`](#struct.Page), [`Pagination`](#struct.Pagination), [`PortfolioValue`](#struct.PortfolioValue), [`Position`](#struct.Position), [`PricePoint`](#struct.PricePoint), [`Resolution`](#struct.Resolution), [`ServiceStatus`](#struct.ServiceStatus), [`ServingFreshness`](#struct.ServingFreshness), [`ServingMechanism`](#struct.ServingMechanism), [`Trade`](#struct.Trade), [`UserPnlPoint`](#struct.UserPnlPoint), [`UserPnlSeries`](#struct.UserPnlSeries), [`UserStats`](#struct.UserStats), [`UserVolume`](#struct.UserVolume)
- **Enums:** [`ActivitySide`](#enum.ActivitySide), [`ActivitySortBy`](#enum.ActivitySortBy), [`ActivityType`](#enum.ActivityType), [`ComboActivityType`](#enum.ComboActivityType), [`ComboLegStatus`](#enum.ComboLegStatus), [`ComboPositionSortBy`](#enum.ComboPositionSortBy), [`ComboPositionStatus`](#enum.ComboPositionStatus), [`ErrorCode`](#enum.ErrorCode), [`FilterType`](#enum.FilterType), [`LeaderboardSortBy`](#enum.LeaderboardSortBy), [`PnlFidelity`](#enum.PnlFidelity), [`PnlInterval`](#enum.PnlInterval), [`PositionSortBy`](#enum.PositionSortBy), [`PositionStatus`](#enum.PositionStatus), [`PriceHistoryInterval`](#enum.PriceHistoryInterval), [`Reporter`](#enum.Reporter), [`ResolutionMarketType`](#enum.ResolutionMarketType), [`ResolutionSelector`](#enum.ResolutionSelector), [`ResolutionSource`](#enum.ResolutionSource), [`ResolutionStatus`](#enum.ResolutionStatus), [`ServingMechanismName`](#enum.ServingMechanismName), [`SettlementTimeBasis`](#enum.SettlementTimeBasis), [`SortDirection`](#enum.SortDirection), [`TimePeriod`](#enum.TimePeriod), [`TokenStandard`](#enum.TokenStandard), [`WinKind`](#enum.WinKind)
- **Constants:** [`MAX_LIST_VALUES`](#constant.MAX_LIST_VALUES), [`UNLABELED_OUTCOME_INDEX`](#constant.UNLABELED_OUTCOME_INDEX)

## Re-exports

- `Address`: re-export of [`marcasite::types::Address`](types.md#struct.Address).
- `ConditionId`: re-export of [`marcasite::types::ConditionId`](types.md#struct.ConditionId).
- `EventId`: re-export of [`marcasite::types::EventId`](types.md#struct.EventId).
- `MarketId`: re-export of [`marcasite::types::MarketId`](types.md#struct.MarketId).
- `QuestionId`: re-export of [`marcasite::types::QuestionId`](types.md#struct.QuestionId).
- `Side`: re-export of [`marcasite::types::Side`](types.md#enum.Side).
- `TokenId`: re-export of [`marcasite::types::TokenId`](types.md#struct.TokenId).

## Structs

### <a id="struct.Activity"></a>`struct Activity`

```rust
#[non_exhaustive]
pub struct Activity {
    /// Proxy wallet the row belongs to.
    pub proxy_wallet: Address,
    /// Block timestamp of the action.
    pub timestamp: DateTime<Utc>,
    /// On-chain condition id of the market; `None` on rows that touch no market (rebates,
    /// yield, rewards, deposits, withdrawals, ...), which the API serves as `""` (and which
    /// is serialized back as `""`).
    pub condition_id: Option<ConditionId>,
    /// Kind of event (`type`).
    pub activity_type: ActivityType,
    /// Share quantity of the action (for tips, the amount transferred).
    pub size: Decimal,
    /// Cash value of the action, in USDC.
    pub usdc_size: Decimal,
    /// Hash of the settling transaction.
    pub transaction_hash: String,
    /// Price per share in USDC (trades; `0` where no price applies).
    pub price: Decimal,
    /// CLOB asset id of the outcome token the action touched; `None` on rows that touch no
    /// single token (merges, splits and conversions included; served as `""`, and
    /// serialized back as `""`).
    pub token_id: Option<TokenId>,
    /// Direction: `BUY`/`SELL` on trade rows, `IN`/`OUT` on tips; `None` where a side does
    /// not apply (served as `""`, and serialized back as `""`).
    pub side: Option<ActivitySide>,
    /// Index of the outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means it could not be
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
    /// Set on combo trade rows; omitted (`None`, and left out when serialized) on
    /// non-combo rows. Combo detail lives on [`DataClient::list_combo_activity`](data.md#DataClient.fn.list_combo_activity).
    pub is_combo: Option<bool>,
}
```

One activity-feed event: a trade, split, merge, redeem, ... (`components/schemas/Activity`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ApprovalContract"></a>`struct ApprovalContract`

```rust
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
```

One trusted approval contract (`components/schemas/ApprovalContract`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ApprovalContract.fn.is_unlimited"></a>`is_unlimited`

```rust
#[must_use]
pub fn is_unlimited(&self) -> bool
```

`true` if the ERC-20 allowance is unlimited (`amount` is `max`).

### <a id="struct.Approvals"></a>`struct Approvals`

```rust
#[non_exhaustive]
pub struct Approvals {
    /// The checked wallet.
    pub address: Address,
    /// EVM chain id the state was read on (137 = Polygon).
    pub chain_id: i32,
    /// When the on-chain state was read (served from a 20-second cache).
    pub checked_at: DateTime<Utc>,
    /// One row per catalog pair the wallet may need.
    pub contracts: Vec<ApprovalContract>,
}
```

A wallet's Polygon token/operator approval state (`components/schemas/Approvals`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BiggestWinner"></a>`struct BiggestWinner`

```rust
#[non_exhaustive]
pub struct BiggestWinner {
    /// Unique 1-based ordinal within the window and category (equal PnL does not share a
    /// rank).
    pub win_rank: u32,
    /// Whether this is a market or a combo position; check it before building an event
    /// link.
    pub kind: WinKind,
    /// The winning wallet.
    pub user_id: Address,
    /// `final_value - initial_value`, in USDC.
    pub pnl: Decimal,
    /// Cost basis of the winning position, in USDC.
    pub initial_value: Decimal,
    /// Value at resolution, in USDC.
    pub final_value: Decimal,
    /// When the position resolved.
    pub resolved_at: DateTime<Utc>,
    /// On-chain condition id of the market (the combo condition on combo rows).
    pub condition_id: ConditionId,
    /// Token id of the winning position.
    pub position_id: TokenId,
    /// Gamma event id of the parent event. Served as a JSON integer here (unlike the
    /// string event ids of the other routes), and serialized back as one.
    ///
    /// Combo rows have no Gamma event and carry the sentinel `0`, kept as served: branch
    /// on [`kind`](data.md#struct.BiggestWinner) before building an event link.
    pub event_id: EventId,
    /// Parent event slug; empty on combo rows.
    pub event_slug: String,
    /// Parent event title; on combo rows, the `" / "`-joined leg questions.
    pub event_title: String,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
}
```

One winning position (`components/schemas/BiggestWinner`). One row per position, not
per user.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BuilderCode"></a>`struct BuilderCode`

```rust
pub struct BuilderCode(/* private fields */);
```

A builder's stable identifier (`builder_code`) on the builder boards.

The Data API documents it only as a string ("Stable identifier of the builder"),
with no format, so it is kept exactly as received and never validated. It is a
separate type from the CLOB's `BuilderCode` (documented as `0x` followed by 64 hex
characters) because the Data API does not document the same wire format.

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

### <a id="struct.BuilderStanding"></a>`struct BuilderStanding`

```rust
#[non_exhaustive]
pub struct BuilderStanding {
    /// Board rank; ties share a rank and the next one skips.
    pub rank: u64,
    /// Display name, falling back to `builder_code`. Cosmetic: key on
    /// [`builder_code`](data.md#struct.BuilderStanding).
    pub builder_name: String,
    /// Stable identifier of the builder.
    pub builder_code: BuilderCode,
    /// Builder profile image URL.
    pub profile_image: String,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Volume attributed to the builder in the window, in shares.
    pub volume: Decimal,
    /// Distinct active users (makers) attributed to the builder in the window.
    pub active_users: u64,
}
```

One builder's standing for the window (`components/schemas/BuilderStanding`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BuilderVolumePoint"></a>`struct BuilderVolumePoint`

```rust
#[non_exhaustive]
pub struct BuilderVolumePoint {
    /// Bucket start date (UTC); the request's interval sets the width.
    pub date: NaiveDate,
    /// The builder's rank within that bucket.
    pub rank: u64,
    /// Display name, falling back to `builder_code`. Cosmetic: key on
    /// [`builder_code`](data.md#struct.BuilderVolumePoint).
    pub builder_name: String,
    /// Stable identifier of the builder.
    pub builder_code: BuilderCode,
    /// Builder profile image URL.
    pub profile_image: String,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Volume attributed in that bucket, in shares.
    pub volume: Decimal,
    /// Distinct active users attributed in that bucket.
    pub active_users: u64,
}
```

A builder's volume in one bucket of the series
(`components/schemas/BuilderVolumePoint`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ComboActivity"></a>`struct ComboActivity`

```rust
#[non_exhaustive]
pub struct ComboActivity {
    /// `tx_hash-log_index`.
    pub id: String,
    /// Canonical action verb (`type`).
    pub activity_type: ComboActivityType,
    /// Proxy wallet the action belongs to.
    pub proxy_wallet: Address,
    /// On-chain combo condition id: `0x` plus 62 hex digits live (not a bytes32), e.g.
    /// `0x037cb523f88f4c6ef6a31c33f8a2e72be70000000000000000000000000000`. The docs only
    /// say `0x03`-prefixed.
    pub combo_condition_id: ConditionId,
    /// Token id of the combo position the action touched.
    pub combo_position_id: TokenId,
    /// Block number of the action.
    pub block_number: i64,
    /// Event time.
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
```

One combo lifecycle/redemption event (`components/schemas/ComboActivity`).

Ordered by on-chain position `(block_number, log_index)`; `timestamp` is the event's
wall-clock time, not the ordering key.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ComboLeg"></a>`struct ComboLeg`

```rust
#[non_exhaustive]
pub struct ComboLeg {
    /// Position of the leg within the combo, 0-based.
    pub leg_index: i32,
    /// Outcome token id of the leg.
    pub leg_position_id: TokenId,
    /// On-chain condition id of the leg's market.
    ///
    /// Live, the two routes that carry combo legs disagree on its format for the same leg:
    /// `/v2/activity/combos` serves the market's bytes32 condition id (`0x` plus 64 hex
    /// digits), while `/v2/positions/combos` serves a 62-digit, `0x01`/`0x02`-prefixed
    /// value (e.g. `0x0104db2bc4f21eef1caf06186806e548800000000000000000000000000000`)
    /// that is not the market's condition id. The same leg has the same
    /// [`leg_position_id`](data.md#struct.ComboLeg) on both. The value is kept as served;
    /// do not use it as a `condition` filter on the single-market routes.
    pub leg_condition_id: ConditionId,
    /// Index of the outcome the combo takes on this leg;
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means the outcome could not be labeled.
    pub leg_outcome_index: i32,
    /// Label of the outcome the combo takes on this leg.
    pub leg_outcome_label: String,
    /// Live resolution state of the leg.
    pub leg_status: ComboLegStatus,
    /// Live price of the leg outcome (Gamma marks).
    pub leg_current_price: Decimal,
    /// When the leg's market closed (Gamma `closed_time`); `None` while open.
    pub leg_resolved_at: Option<DateTime<Utc>>,
    /// The leg's market, with its event nested.
    pub market: ComboLegMarket,
}
```

One leg of a combo, with its market and event enrichment
(`components/schemas/ComboLeg`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ComboLegEvent"></a>`struct ComboLegEvent`

```rust
#[non_exhaustive]
pub struct ComboLegEvent {
    /// Gamma event id.
    pub event_id: EventId,
    /// Event slug; the URL segment on polymarket.com.
    pub event_slug: String,
    /// Event title.
    pub event_title: String,
    /// Event image URL.
    pub event_image: String,
}
```

A combo leg market's event (`components/schemas/ComboLegEvent`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ComboLegMarket"></a>`struct ComboLegMarket`

```rust
#[non_exhaustive]
pub struct ComboLegMarket {
    /// Gamma's own market id; not the on-chain condition id (that is the leg's
    /// `leg_condition_id`).
    pub market_id: MarketId,
    /// Market slug; the URL segment on polymarket.com.
    pub slug: String,
    /// Short per-leg label (`group_item_title`), falling back to the question.
    pub title: String,
    /// Label of the leg's outcome on this market.
    pub outcome: String,
    /// Market image URL.
    pub image_url: String,
    /// Market icon URL.
    pub icon_url: String,
    /// Gamma market category (e.g. `sports`).
    pub category: String,
    /// Gamma market subcategory.
    pub subcategory: String,
    /// Reserved; always empty today.
    pub tags: Vec<String>,
    /// Market end date; `None` when Gamma has none (served as `""`, and serialized back
    /// as `""`).
    pub end_date: Option<DateTime<Utc>>,
    /// The market's parent event.
    pub event: ComboLegEvent,
    /// The market's full question; `""` when Gamma has none. `None` when absent from the
    /// payload (an older cached payload).
    pub question: Option<String>,
    /// Raw short per-leg label, without the fallback `title` applies; `""` when Gamma has
    /// none. `None` when absent from the payload (an older cached payload).
    pub group_item_title: Option<String>,
    /// The market's outcome labels, in outcome-index order; empty when Gamma has none.
    /// `None` when absent from the payload (an older cached payload).
    pub outcomes: Option<Vec<String>>,
    /// Granular sports market type (e.g. `totals`); `""` for non-sports markets. `None`
    /// when absent from the payload (an older cached payload).
    pub sports_market_type: Option<String>,
    /// The sports line the market is quoted on; `None` when it has none (served as
    /// `null`) or when absent from an older cached payload.
    pub line: Option<Decimal>,
}
```

A combo leg's market, with its (single) event nested
(`components/schemas/ComboLegMarket`).

The docs say the display-metadata fields `question`, `group_item_title`, `outcomes`,
`sports_market_type` and `line` default when absent, so that cached payloads written
before they existed still deserialize, while the query always serves them.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ComboPosition"></a>`struct ComboPosition`

```rust
#[non_exhaustive]
pub struct ComboPosition {
    /// On-chain combo condition id: `0x` plus 62 hex digits live (not a bytes32), e.g.
    /// `0x037cb523f88f4c6ef6a31c33f8a2e72be70000000000000000000000000000`. The docs only
    /// say `0x03`-prefixed.
    pub combo_condition_id: ConditionId,
    /// Index of the combo outcome held;
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means unlabelable.
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
    /// First acquisition time; `None` on rows without one.
    ///
    /// The schema declares `first_entry_at` a required RFC 3339 string, but also says that
    /// [`first_entry_at_micros`](data.md#struct.ComboPosition), its microsecond form, is
    /// `null` or absent on "the NULL tail", so some rows have no first-entry time. The
    /// docs do not say how `first_entry_at` is served on those rows: `""` and `null` both
    /// decode as `None` (the key itself stays required), and `None` serializes back as
    /// `""`.
    pub first_entry_at: Option<DateTime<Utc>>,
    /// `first_entry_at` at microsecond precision (`first_entry_at_micros`); `None` on the
    /// NULL tail.
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
    pub resolved_at: Option<DateTime<Utc>>,
    /// Last event touching the position.
    pub updated_at: DateTime<Utc>,
    /// `updated_at` at microsecond precision (`updated_at_micros`).
    pub updated_at_micros: DateTime<Utc>,
}
```

One combo position: a user's holding in a single combo outcome, with leg rollups
(`components/schemas/ComboPosition`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ConditionVolume"></a>`struct ConditionVolume`

```rust
#[non_exhaustive]
pub struct ConditionVolume {
    /// On-chain condition id of the market.
    pub condition_id: ConditionId,
    /// Cumulative one-side (taker) volume in shares, truncated to 6 decimals.
    pub taker_volume: Decimal,
}
```

One market's cumulative taker volume (`components/schemas/ConditionVolume`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.CursorLag"></a>`struct CursorLag`

```rust
#[non_exhaustive]
pub struct CursorLag {
    /// The stream, as `<contract>_<event>`.
    pub source: String,
    /// Last block the stream has ingested through.
    pub block: i64,
    /// How many blocks this stream trails the furthest-along stream; `0` for the leader.
    pub behind_max: i64,
}
```

One ingestion stream's distance from the furthest-along stream
(`components/schemas/CursorLag`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.DataClient"></a>`struct DataClient`

```rust
pub struct DataClient { /* private fields */ }
```

Client for the Data API v2 (`https://data-api.polymarket.com`).

Covers wallet portfolios, trade and activity feeds, market state and leaderboards. Cheap to clone: clones share one connection pool.

```rust
use marcasite::data::DataClient;

let client = DataClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="DataClient.fn.list_leaderboard"></a>`list_leaderboard`

```rust
pub fn list_leaderboard(&self) -> ListLeaderboard
```

Lists the trader leaderboard of PnL or volume (`GET /v2/leaderboard`,
cursor-paginated). For one wallet's standing use
[`get_leaderboard_standing`](data.md#DataClient.fn.get_leaderboard_standing), the `user=` arm of the
same route.

See <https://docs.polymarket.com/api-reference/boards/get-the-trader-leaderboard>.

```rust
use marcasite::data::{DataClient, LeaderboardSortBy, TimePeriod};

let data = DataClient::new()?;
let page = data
    .list_leaderboard()
    .time_period(TimePeriod::Week)
    .sort_by(LeaderboardSortBy::Volume)
    .limit(25)
    .send()
    .await?;
for entry in page.items() {
    let _ = (entry.rank, &entry.user_name, entry.volume);
}
```

##### <a id="DataClient.fn.get_leaderboard_standing"></a>`get_leaderboard_standing`

```rust
pub fn get_leaderboard_standing(&self, user: impl Into<Address>) -> GetLeaderboardStanding
```

Looks up one wallet's standing on both trader boards
(`GET /v2/leaderboard?user=`). For the board itself use
[`list_leaderboard`](data.md#DataClient.fn.list_leaderboard), the other arm of the same route.

See <https://docs.polymarket.com/api-reference/boards/get-the-trader-leaderboard>.

##### <a id="DataClient.fn.list_biggest_winners"></a>`list_biggest_winners`

```rust
pub fn list_biggest_winners(&self) -> ListBiggestWinners
```

Lists the biggest single winning positions (`GET /v2/biggest-winners`,
cursor-paginated).

See <https://docs.polymarket.com/api-reference/boards/list-the-biggest-wins>.

##### <a id="DataClient.fn.list_builders_leaderboard"></a>`list_builders_leaderboard`

```rust
pub fn list_builders_leaderboard(&self) -> ListBuildersLeaderboard
```

Lists the builders leaderboard, ranked by volume
(`GET /v2/builders/leaderboard`, cursor-paginated).

See <https://docs.polymarket.com/api-reference/boards/get-the-builders-leaderboard>.

##### <a id="DataClient.fn.get_builders_volume"></a>`get_builders_volume`

```rust
pub fn get_builders_volume(&self) -> GetBuildersVolume
```

Gets the per-builder volume time series, newest bucket first
(`GET /v2/builders/volume`).

See <https://docs.polymarket.com/api-reference/boards/get-builder-volume-over-time>.

##### <a id="DataClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://data-api.polymarket.com";
```

The production base URL.

##### <a id="DataClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="DataClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> DataClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="DataClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="DataClient.fn.list_trades"></a>`list_trades`

```rust
pub fn list_trades(&self) -> ListTrades
```

Lists trades for a wallet, markets, events, or the global feed
(`GET /v2/trades`, cursor-paginated).

See <https://docs.polymarket.com/api-reference/feeds/list-trades>.

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{data::DataClient, types::Side};

let data = DataClient::new()?;
// The 500 most recent buys in one market, walking pages lazily.
let trades: Vec<_> = data
    .list_trades()
    .conditions(["0xd9b06e2fd9ddb7ab61c9e3d5d8e074c555802478bbf75145804ff709a4246f79"])
    .side(Side::Buy)
    .limit(250)
    .into_stream()
    .take(500)
    .try_collect()
    .await?;
```

##### <a id="DataClient.fn.list_activity"></a>`list_activity`

```rust
pub fn list_activity(&self, user: impl Into<Address>) -> ListActivity
```

Lists a user's account activity: trades, splits, merges, redeems, ...
(`GET /v2/activity`, cursor-paginated).

See <https://docs.polymarket.com/api-reference/feeds/list-account-activity>.

##### <a id="DataClient.fn.list_combo_activity"></a>`list_combo_activity`

```rust
pub fn list_combo_activity(&self, user: impl Into<Address>) -> ListComboActivity
```

Lists a user's combo lifecycle and redemption events
(`GET /v2/activity/combos`, cursor-paginated).

See <https://docs.polymarket.com/api-reference/feeds/list-combo-activity>.

##### <a id="DataClient.fn.list_holders"></a>`list_holders`

```rust
pub fn list_holders<I>(&self, conditions: I) -> ListHolders
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Lists a market's top holders, grouped by outcome token
(`GET /v2/holders`, cursor-paginated).

`conditions` is required: between 1 and 20 distinct condition ids (exactly one with
[`include_pnl`](data.md#ListHolders.fn.include_pnl)). Duplicates are sent once.

See <https://docs.polymarket.com/api-reference/markets/list-a-markets-top-holders>.

##### <a id="DataClient.fn.get_open_interest"></a>`get_open_interest`

```rust
pub fn get_open_interest(&self) -> GetOpenInterest
```

Gets the priced gross open interest per market, or the global figure when no
condition is given (`GET /v2/oi`).

See <https://docs.polymarket.com/api-reference/markets/get-open-interest>.

##### <a id="DataClient.fn.get_live_volume"></a>`get_live_volume`

```rust
pub async fn get_live_volume<I>(&self, event_ids: I) -> Result<LiveVolume>
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Gets the cumulative taker volume of every market under the given events
(`GET /v2/live-volume`); between 1 and 20 distinct integer event ids.

The server ignores ids it cannot parse, so a typo would silently drop an event;
this client rejects any id that is not made of ASCII digits instead. Duplicates are
sent once.

See <https://docs.polymarket.com/api-reference/markets/get-live-volume-for-an-event>.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if no event id is given, an
id is not an integer, or more than 20 distinct ids are given; otherwise see
[`Error`](marcasite.md#enum.Error).

##### <a id="DataClient.fn.list_prices_history"></a>`list_prices_history`

```rust
pub fn list_prices_history(&self, token_id: impl Into<TokenId>) -> ListPricesHistory
```

Lists a token's price history, or a single point-in-time observation
(`GET /v2/prices-history`, cursor-paginated).

Choose exactly one window form: [`start`](data.md#ListPricesHistory.fn.start) (with an
optional [`end`](data.md#ListPricesHistory.fn.end)), [`interval`](data.md#ListPricesHistory.fn.interval),
or [`as_of`](data.md#ListPricesHistory.fn.as_of).

See <https://docs.polymarket.com/api-reference/markets/get-a-tokens-price-history>.

```rust
use futures_util::TryStreamExt as _;
use marcasite::data::{DataClient, PriceHistoryInterval};

let data = DataClient::new()?;
let points: Vec<_> = data
    .list_prices_history(
        "31974447302330162086995746309500877260929998201718217388109724292047967921664",
    )
    .interval(PriceHistoryInterval::OneWeek)
    .bucket_seconds(1800)
    .into_stream()
    .try_collect()
    .await?;
// The last point is the terminal point: the latest observation in the window.
```

##### <a id="DataClient.fn.get_resolutions"></a>`get_resolutions`

```rust
pub async fn get_resolutions(&self, selector: ResolutionSelector) -> Result<Vec<Resolution>>
```

Gets the resolution state of a UMA question, markets or events
(`GET /v2/resolutions`). Misses return an empty list.

See <https://docs.polymarket.com/api-reference/markets/get-resolution-state>.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) for an empty selector, one
with more than 20 distinct ids, or a malformed id (see [`ResolutionSelector`](data.md#enum.ResolutionSelector));
otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="DataClient.fn.get_status"></a>`get_status`

```rust
pub async fn get_status(&self) -> Result<ServiceStatus>
```

Gets how fresh the data behind the Data API is (`GET /v2/status`).

See <https://docs.polymarket.com/api-reference/service/get-data-freshness>.

###### Errors

Before the first freshness snapshot exists, on a timeout or on a serving-dependency
outage, the API answers `503`, returned as an [`Error::Api`](marcasite.md#enum.Error)
(retry after [`Error::retry_after`](marcasite.md#Error.fn.retry_after) when present).
[`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) follows the error body's
`retryable` flag: a `503` with `"retryable": false` is not retryable (and is not
retried by a [`RetryPolicy`](marcasite.md#struct.RetryPolicy)); only a `503` without the flag
falls back to the status. Otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="DataClient.fn.list_positions"></a>`list_positions`

```rust
pub fn list_positions(&self) -> ListPositions
```

Lists positions for a user or a market, across the whole lifecycle
(`GET /v2/positions`, cursor-paginated).

Set at least one of [`user`](data.md#ListPositions.fn.user) and
[`conditions`](data.md#ListPositions.fn.conditions).

See <https://docs.polymarket.com/api-reference/wallet/list-positions-for-a-user-or-market>.

```rust
use marcasite::data::{DataClient, PositionStatus};

let data = DataClient::new()?;
let page = data
    .list_positions()
    .user("0x983eedfbd75803602e4a6e6ea9aab6dc6b9c6748")
    .status(PositionStatus::Open)
    .limit(50)
    .send()
    .await?;
for position in page.items() {
    let _ = (&position.title, position.current_value, position.total_pnl);
}
// Pass `page.next_cursor()` to `.cursor(..)` for the next page, or use
// `.into_stream()` to walk every page.
```

##### <a id="DataClient.fn.list_combo_positions"></a>`list_combo_positions`

```rust
pub fn list_combo_positions(&self, user: impl Into<Address>) -> ListComboPositions
```

Lists a user's combo positions (`GET /v2/positions/combos`, cursor-paginated).

See <https://docs.polymarket.com/api-reference/wallet/list-combo-positions>.

##### <a id="DataClient.fn.get_portfolio_value"></a>`get_portfolio_value`

```rust
pub fn get_portfolio_value(&self, user: impl Into<Address>) -> GetPortfolioValue
```

Gets a user's portfolio value: single-market holdings marked to market plus
unresolved combo positions at cost basis (`GET /v2/value`).

See <https://docs.polymarket.com/api-reference/wallet/get-portfolio-value>.

##### <a id="DataClient.fn.get_approvals"></a>`get_approvals`

```rust
pub async fn get_approvals(&self, user: impl Into<Address>) -> Result<Approvals>
```

Gets a wallet's Polygon token/operator approval state (`GET /v2/approvals`).

See <https://docs.polymarket.com/api-reference/wallet/get-wallet-approvals>.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is not an EVM
address (`0x` followed by 40 hex digits), as the route requires; a known protocol
contract address is an [`Error::Api`](marcasite.md#enum.Error) with status `400`
([`ErrorCode::InvalidRequest`](data.md#enum.ErrorCode)); otherwise see
[`Error`](marcasite.md#enum.Error).

##### <a id="DataClient.fn.get_user_pnl"></a>`get_user_pnl`

```rust
pub fn get_user_pnl(&self, user: impl Into<Address>) -> GetUserPnl
```

Gets a user's cumulative PnL series (`GET /v2/user-pnl`).

See <https://docs.polymarket.com/api-reference/wallet/get-a-users-pnl-series>.

##### <a id="DataClient.fn.get_user_stats"></a>`get_user_stats`

```rust
pub async fn get_user_stats(&self, user: impl Into<Address>) -> Result<Option<UserStats>>
```

Gets a wallet's profile stats (`GET /v2/user-stats`).

Returns `Ok(None)` when the server has no stats for the wallet (`data: null`).

The docs say `null` means "not a known user". Live, an arbitrary unknown wallet does
get `null`, but so do some active, ranked users (several top places of the day
leaderboard on 2026-10-02), and the wallet
`0x0000000000000000000000000000000000000001` gets a row of zeros instead (no join
date, no `all_time_pnl`). Treat `None` as "no stats available", not as "no such
user".

See <https://docs.polymarket.com/api-reference/wallet/get-a-users-profile-stats>.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is not `0x`
followed by 40 hex digits, as the route requires; a known protocol contract address
is an [`Error::Api`](marcasite.md#enum.Error) with status `400`; otherwise see
[`Error`](marcasite.md#enum.Error).

##### <a id="DataClient.fn.get_user_volume"></a>`get_user_volume`

```rust
pub fn get_user_volume(&self, user: impl Into<Address>) -> GetUserVolume
```

Gets a wallet's trading volume over a window (`GET /v2/user-volume`).

See <https://docs.polymarket.com/api-reference/wallet/get-a-users-trading-volume>.

### <a id="struct.DataClientBuilder"></a>`struct DataClientBuilder`

```rust
#[must_use]
pub struct DataClientBuilder { /* private fields */ }
```

Builder for [`DataClient`](data.md#struct.DataClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="DataClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`DataClient::DEFAULT_BASE_URL`](data.md#DataClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="DataClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="DataClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<DataClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.GetBuildersVolume"></a>`struct GetBuildersVolume`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetBuildersVolume { /* private fields */ }
```

Request builder for [`DataClient::get_builders_volume`](data.md#DataClient.fn.get_builders_volume).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetBuildersVolume.fn.interval"></a>`interval`

```rust
pub fn interval(self, interval: TimePeriod) -> Self
```

Bucket width (`interval`, default [`TimePeriod::Day`](data.md#enum.TimePeriod); [`TimePeriod::All`](data.md#enum.TimePeriod) buckets
by calendar year).

##### <a id="GetBuildersVolume.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

How many of the most recent **buckets** to return (`limit`, default 30, at most
90).

##### <a id="GetBuildersVolume.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<BuilderVolumePoint>>
```

Sends the request.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `limit` is above 90;
otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetLeaderboardStanding"></a>`struct GetLeaderboardStanding`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetLeaderboardStanding { /* private fields */ }
```

Request builder for [`DataClient::get_leaderboard_standing`](data.md#DataClient.fn.get_leaderboard_standing).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetLeaderboardStanding.fn.time_period"></a>`time_period`

```rust
pub fn time_period(self, time_period: TimePeriod) -> Self
```

The window (`time_period`, default [`TimePeriod::Day`](data.md#enum.TimePeriod)).

##### <a id="GetLeaderboardStanding.fn.category"></a>`category`

```rust
pub fn category(self, category: impl Into<String>) -> Self
```

The category (`category`): `overall` (default), a Gamma market category, or the
synthetic `combos` or `esports`.

##### <a id="GetLeaderboardStanding.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Option<LeaderboardUserEntry>>
```

Sends the request. Returns `Ok(None)` when the API answers `data: null`.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty (an empty
`user` would select the board arm of the route instead); an invalid `time_period`
or a known protocol contract address is an [`Error::Api`](marcasite.md#enum.Error) with
status `400`; otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetOpenInterest"></a>`struct GetOpenInterest`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetOpenInterest { /* private fields */ }
```

Request builder for [`DataClient::get_open_interest`](data.md#DataClient.fn.get_open_interest).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetOpenInterest.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Condition ids (`condition`, at most 20 distinct values). Omit for the global
figure. Duplicates are sent once.

##### <a id="GetOpenInterest.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Vec<OpenInterest>>
```

Sends the request. An id that does not resolve to a servable market is absent from
the result.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if a condition id is not
`0x` followed by 64 hex digits or more than 20 distinct ids are given; otherwise
see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetPortfolioValue"></a>`struct GetPortfolioValue`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetPortfolioValue { /* private fields */ }
```

Request builder for [`DataClient::get_portfolio_value`](data.md#DataClient.fn.get_portfolio_value).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetPortfolioValue.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Scopes the single-market term to these condition ids (`condition`, at most 20
distinct values). Any condition filter excludes the portfolio-level combo term.
Duplicates are sent once.

##### <a id="GetPortfolioValue.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<PortfolioValue>
```

Sends the request.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty, a
condition id is not `0x` followed by 64 hex digits, or more than 20 distinct
condition ids are given; otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetUserPnl"></a>`struct GetUserPnl`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetUserPnl { /* private fields */ }
```

Request builder for [`DataClient::get_user_pnl`](data.md#DataClient.fn.get_user_pnl).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetUserPnl.fn.interval"></a>`interval`

```rust
pub fn interval(self, interval: PnlInterval) -> Self
```

The window (`interval`, default [`PnlInterval::OneDay`](data.md#enum.PnlInterval)).

##### <a id="GetUserPnl.fn.fidelity"></a>`fidelity`

```rust
pub fn fidelity(self, fidelity: PnlFidelity) -> Self
```

The output grid (`fidelity`, default [`PnlFidelity::OneHour`](data.md#enum.PnlFidelity)).

##### <a id="GetUserPnl.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<UserPnlSeries>
```

Sends the request.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is not `0x`
followed by 40 hex digits (live, the route answers `400` `invalid user address`
otherwise); an unknown interval or fidelity is an [`Error::Api`](marcasite.md#enum.Error)
with status `400`; otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.GetUserVolume"></a>`struct GetUserVolume`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct GetUserVolume { /* private fields */ }
```

Request builder for [`DataClient::get_user_volume`](data.md#DataClient.fn.get_user_volume).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="GetUserVolume.fn.start"></a>`start`

```rust
pub fn start(self, start: DateTime<Utc>) -> Self
```

Inclusive window start (`start`, epoch seconds), floored to its UTC day. Omitted
(or the Unix epoch, sent as `0`): no lower bound.

##### <a id="GetUserVolume.fn.end"></a>`end`

```rust
pub fn end(self, end: DateTime<Utc>) -> Self
```

Inclusive window end (`end`, epoch seconds), floored to its UTC day. Omitted (or
the Unix epoch, sent as `0`): no upper bound.

##### <a id="GetUserVolume.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<UserVolume>
```

Sends the request. A wallet with no trades in the window (or an inverted window)
yields zeros.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty or a
bound is before the Unix epoch; otherwise see [`Error`](marcasite.md#enum.Error).

### <a id="struct.Holder"></a>`struct Holder`

```rust
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
    pub amount: Decimal,
    /// Whether the profile chose to show its name publicly.
    pub display_username_public: bool,
    /// Index of the held outcome;
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means unlabelable.
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
    pub avg_price: Option<Decimal>,
    /// Cost basis of the held size in USDC, excluding entry fees.
    pub entry_cost_usdc: Option<Decimal>,
    /// Current price of the held outcome token, in `[0, 1]`.
    pub current_price: Option<Decimal>,
    /// `amount × current_price`.
    pub current_value: Option<Decimal>,
    /// Profit already locked in by sells and redemptions.
    pub realized_pnl: Option<Decimal>,
    /// `current_value - entry_cost_usdc`.
    pub unrealized_pnl: Option<Decimal>,
    /// `realized_pnl + unrealized_pnl`.
    pub total_pnl: Option<Decimal>,
}
```

One market holder, enriched with their public profile (`components/schemas/Holder`).

The seven position-economics fields (`avg_price` to `total_pnl`) are only served with
[`ListHolders::include_pnl`](data.md#ListHolders.fn.include_pnl); they are `None` otherwise.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.HolderGroup"></a>`struct HolderGroup`

```rust
#[non_exhaustive]
pub struct HolderGroup {
    /// The outcome token this group ranks.
    pub token_id: TokenId,
    /// Top holders of that token, amount descending.
    pub holders: Vec<Holder>,
}
```

One outcome token's holder group (`components/schemas/MetaHolder`).

A multi-market request interleaves tokens across the page, so merge groups across
pages by [`token_id`](data.md#struct.HolderGroup), not by position.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.IngestionFreshness"></a>`struct IngestionFreshness`

```rust
#[non_exhaustive]
pub struct IngestionFreshness {
    /// How many live streams were found; `0` means no ingestion cursors are visible.
    pub cursors: u64,
    /// The chain id this service is configured for.
    pub chain_id: i64,
    /// The furthest-behind live streams, `most_lagged` first.
    pub lagging: Vec<CursorLag>,
    /// The tail: the furthest-along cursor's block (over all cursors).
    pub max_synced_block: Option<i64>,
    /// The furthest-behind live stream's block.
    pub min_synced_block: Option<i64>,
    /// The single furthest-behind live stream.
    pub most_lagged: Option<CursorLag>,
    /// The chain the ingestion cursors were written for, as the datastore names it;
    /// `None` when no stream declares one.
    pub network: Option<String>,
}
```

Ingestion health over the live streams (`components/schemas/IngestionFreshness`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.LeaderboardEntry"></a>`struct LeaderboardEntry`

```rust
#[non_exhaustive]
pub struct LeaderboardEntry {
    /// Competition rank: tied users share a rank and the next one skips. Page with the
    /// cursor; never derive a page from a rank.
    pub rank: u32,
    /// The ranked wallet.
    pub user_id: Address,
    /// Window PnL in USDC: the marked equity change net of flows for finite windows, the
    /// realized-only lifetime ledger for [`TimePeriod::All`](data.md#enum.TimePeriod).
    pub pnl: Decimal,
    /// Both-sides traded volume, in shares.
    pub volume: Decimal,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Linked X handle, when one exists.
    pub x_username: String,
    /// Profile verification badge.
    pub verified: bool,
}
```

One trader-leaderboard row (`components/schemas/LeaderboardEntry`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.LeaderboardUserEntry"></a>`struct LeaderboardUserEntry`

```rust
#[non_exhaustive]
pub struct LeaderboardUserEntry {
    /// The looked-up wallet.
    pub user_id: Address,
    /// Window PnL in USDC (same semantics as [`LeaderboardEntry::pnl`](data.md#struct.LeaderboardEntry)).
    pub pnl: Decimal,
    /// Both-sides traded volume, in shares.
    pub volume: Decimal,
    /// Profile display name of the wallet.
    pub user_name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Linked X handle, when one exists.
    pub x_username: String,
    /// Profile verification badge.
    pub verified: bool,
    /// Rank on the PnL board; `None` means unranked on that board.
    ///
    /// The docs give two readings of "unranked": the schema says a `null` rank, while the
    /// `/v2/leaderboard` description says a rank of `0`. Live it is always `null`: `0` was
    /// never observed (checked 2026-10-02 over several hundred wallet/board combinations),
    /// and a wallet ranked on one board but not the other has `null` for the unranked one.
    /// The value is kept as served, so a `Some(0)` would still decode; treat it as
    /// unranked as well.
    pub rank_pnl: Option<u32>,
    /// Rank on the volume board; `None` means unranked on that board (see
    /// [`rank_pnl`](data.md#struct.LeaderboardUserEntry)).
    pub rank_volume: Option<u32>,
}
```

One wallet's standing on both trader boards
(`components/schemas/LeaderboardUserEntry`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ListActivity"></a>`struct ListActivity`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListActivity { /* private fields */ }
```

Request builder for [`DataClient::list_activity`](data.md#DataClient.fn.list_activity).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListActivity.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Page size (`limit`, default 100, at most 1000). Per the overview, it only applies
to the first page: once a cursor is supplied, the cursor's own page size wins.

##### <a id="ListActivity.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`). The
cursor binds the sort direction it was minted under.

##### <a id="ListActivity.fn.types"></a>`types`

```rust
pub fn types(self, types: impl IntoIterator<Item = ActivityType>) -> Self
```

Activity types (`type`, comma-separated). [`ActivityType::Tip`](data.md#enum.ActivityType) is never in the
default set and is only returned when named here.

[`ActivityType::Deposit`](data.md#enum.ActivityType) and [`ActivityType::Withdrawal`](data.md#enum.ActivityType) are excluded by default,
so filtering by them alone returns an empty page unless
[`exclude_deposits_withdrawals(false)`](data.md#ListActivity.fn.exclude_deposits_withdrawals) is set
too. An unknown type is a `400` naming the `type` parameter.

##### <a id="ListActivity.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Condition ids (`condition`, at most 20 distinct values). Mutually exclusive with
[`event_ids`](data.md#ListActivity.fn.event_ids). Duplicates are sent once.

##### <a id="ListActivity.fn.event_ids"></a>`event_ids`

```rust
pub fn event_ids<I>(self, event_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Gamma event ids, resolved to the events' markets (`event_id`, at most 20 distinct
values). Mutually exclusive with [`conditions`](data.md#ListActivity.fn.conditions). Duplicates are
sent once.

##### <a id="ListActivity.fn.side"></a>`side`

```rust
pub fn side(self, side: Side) -> Self
```

Only buys or only sells (`side`).

##### <a id="ListActivity.fn.start"></a>`start`

```rust
pub fn start(self, start: DateTime<Utc>) -> Self
```

Inclusive window start on the block timestamp (`start`, epoch seconds).

Omitted, or the Unix epoch (sent as `0`), floors the window to three years back;
use [`full_history`](data.md#ListActivity.fn.full_history) for the full history.

##### <a id="ListActivity.fn.full_history"></a>`full_history`

```rust
pub fn full_history(self) -> Self
```

Asks for the full history instead of the default three years: sends `start=1`, as
documented. Replaces any [`start`](data.md#ListActivity.fn.start).

##### <a id="ListActivity.fn.end"></a>`end`

```rust
pub fn end(self, end: DateTime<Utc>) -> Self
```

Inclusive window end (`end`, epoch seconds); omitted (or the Unix epoch, sent as
`0`) means now plus one day.

##### <a id="ListActivity.fn.sort_by"></a>`sort_by`

```rust
pub fn sort_by(self, sort_by: ActivitySortBy) -> Self
```

Sort key (`sort_by`).

##### <a id="ListActivity.fn.sort_direction"></a>`sort_direction`

```rust
pub fn sort_direction(self, sort_direction: SortDirection) -> Self
```

Sort direction (`sort_direction`, default [`SortDirection::Desc`](data.md#enum.SortDirection)). The cursor
binds it, and streams re-send it on every page.

##### <a id="ListActivity.fn.exclude_deposits_withdrawals"></a>`exclude_deposits_withdrawals`

```rust
pub fn exclude_deposits_withdrawals(self, exclude: bool) -> Self
```

Exclude deposits and withdrawals (`exclude_deposits_withdrawals`, default `true`).
Set `false` to see [`ActivityType::Deposit`](data.md#enum.ActivityType) and [`ActivityType::Withdrawal`](data.md#enum.ActivityType) rows,
including when filtering by those types.

##### <a id="ListActivity.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<Activity>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty, `limit`
is above 1000, a condition id is not `0x` followed by 64 hex digits, more than 20
distinct condition or event ids are given, both are given, or a bound is before
the Unix epoch; otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListActivity.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Activity>
```

Streams every activity event from the configured cursor onwards, fetching pages
lazily.

The cursor carries only the seek anchor, page size and sort direction, so every
page re-sends the same filters and sort direction (changing a filter mid-walk would
silently re-anchor the feed). The stream ends when the server reports no further
page.

### <a id="struct.ListBiggestWinners"></a>`struct ListBiggestWinners`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBiggestWinners { /* private fields */ }
```

Request builder for [`DataClient::list_biggest_winners`](data.md#DataClient.fn.list_biggest_winners).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListBiggestWinners.fn.time_period"></a>`time_period`

```rust
pub fn time_period(self, time_period: TimePeriod) -> Self
```

Window on the resolution time (`time_period`, default [`TimePeriod::Day`](data.md#enum.TimePeriod)).

##### <a id="ListBiggestWinners.fn.category"></a>`category`

```rust
pub fn category(self, category: impl Into<String>) -> Self
```

The category (`category`): `overall` (default), a Gamma market category, or the
synthetic `combos` or `esports`.

##### <a id="ListBiggestWinners.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListBiggestWinners.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`). The
cursor pins the window and category it was minted on.

##### <a id="ListBiggestWinners.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<BiggestWinner>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `limit` is above 1000;
otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListBiggestWinners.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<BiggestWinner>
```

Streams every row from the configured cursor onwards, fetching pages lazily.

Every page restates the same window and category with the cursor. The stream ends
when the server reports no further page.

### <a id="struct.ListBuildersLeaderboard"></a>`struct ListBuildersLeaderboard`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListBuildersLeaderboard { /* private fields */ }
```

Request builder for [`DataClient::list_builders_leaderboard`](data.md#DataClient.fn.list_builders_leaderboard).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListBuildersLeaderboard.fn.time_period"></a>`time_period`

```rust
pub fn time_period(self, time_period: TimePeriod) -> Self
```

The window (`time_period`, default [`TimePeriod::Day`](data.md#enum.TimePeriod)).

##### <a id="ListBuildersLeaderboard.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListBuildersLeaderboard.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`). The
cursor pins the window it was minted on.

##### <a id="ListBuildersLeaderboard.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<BuilderStanding>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `limit` is above 1000;
otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListBuildersLeaderboard.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<BuilderStanding>
```

Streams every row from the configured cursor onwards, fetching pages lazily.

Every page restates the same window with the cursor. The stream ends when
the server reports no further page.

### <a id="struct.ListComboActivity"></a>`struct ListComboActivity`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboActivity { /* private fields */ }
```

Request builder for [`DataClient::list_combo_activity`](data.md#DataClient.fn.list_combo_activity).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListComboActivity.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied.

##### <a id="ListComboActivity.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

##### <a id="ListComboActivity.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Combo condition ids (`condition`, at most 20 distinct values). Duplicates are sent
once.

A combo condition id is `0x` plus **62** hex digits live (not the 64 of a regular
condition id): copy it from a [`ComboActivity::combo_condition_id`](data.md#struct.ComboActivity) or a
combo position. Any other length is a `400` live, so the client
rejects it before sending.

##### <a id="ListComboActivity.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<ComboActivity>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty, `limit`
is above 1000, a condition id is not `0x` followed by 62 hex digits, or more
than 20 distinct condition ids are given; otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListComboActivity.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<ComboActivity>
```

Streams every combo activity event from the configured cursor onwards, fetching
pages lazily.

Every page re-sends `user` and the same filters with the cursor (the feed rule:
changing a filter mid-walk re-anchors it). The stream ends when the server reports
no further page.

### <a id="struct.ListComboPositions"></a>`struct ListComboPositions`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboPositions { /* private fields */ }
```

Request builder for [`DataClient::list_combo_positions`](data.md#DataClient.fn.list_combo_positions).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListComboPositions.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListComboPositions.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

##### <a id="ListComboPositions.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Combo condition ids (`condition`, at most 20 distinct values). Duplicates are sent
once.

A combo condition id is `0x` plus **62** hex digits live (not the 64 of a regular
condition id): copy it from a [`ComboPosition::combo_condition_id`](data.md#struct.ComboPosition) or
[`ComboActivity::combo_condition_id`](data.md#struct.ComboActivity). The
server answers `400` to any other length; the client only checks for `0x` plus 1 to
64 hex digits.

##### <a id="ListComboPositions.fn.statuses"></a>`statuses`

```rust
pub fn statuses(self, statuses: impl IntoIterator<Item = ComboPositionStatus>) -> Self
```

Lifecycle filter (`status`, comma-separated). [`ComboPositionStatus::Redeemable`](data.md#enum.ComboPositionStatus)
must be the only value. Default (none): the held-visibility listing.

##### <a id="ListComboPositions.fn.sort_by"></a>`sort_by`

```rust
pub fn sort_by(self, sort_by: ComboPositionSortBy) -> Self
```

Sort key (`sort_by`, default [`ComboPositionSortBy::FirstEntry`](data.md#enum.ComboPositionSortBy)).

##### <a id="ListComboPositions.fn.sort_direction"></a>`sort_direction`

```rust
pub fn sort_direction(self, sort_direction: SortDirection) -> Self
```

Sort direction (`sort_direction`, default [`SortDirection::Desc`](data.md#enum.SortDirection)).

##### <a id="ListComboPositions.fn.updated_after"></a>`updated_after`

```rust
pub fn updated_after(self, updated_after: DateTime<Utc>) -> Self
```

Incremental-sync watermark: inclusive lower bound on `updated_at`
(`updated_after`, epoch seconds). Without a status filter, switches to the
mirror-complete sync view.

##### <a id="ListComboPositions.fn.updated_before"></a>`updated_before`

```rust
pub fn updated_before(self, updated_before: DateTime<Utc>) -> Self
```

Incremental-sync watermark: inclusive upper bound on `updated_at`
(`updated_before`, epoch seconds); must not precede
[`updated_after`](data.md#ListComboPositions.fn.updated_after).

##### <a id="ListComboPositions.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<ComboPosition>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is empty, `limit`
is above 1000, a condition id is not `0x` followed by 62 hex digits, more than
20 distinct condition ids are given, `REDEEMABLE` is combined with other statuses, or
the sync watermarks are negative or inverted; otherwise see
[`Error`](marcasite.md#enum.Error).

##### <a id="ListComboPositions.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<ComboPosition>
```

Streams every combo position from the configured cursor onwards, fetching pages
lazily.

Every page re-sends `user` (always required on this route) and the same filters
with the cursor. The stream ends when the server reports no further page.

### <a id="struct.ListHolders"></a>`struct ListHolders`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListHolders { /* private fields */ }
```

Request builder for [`DataClient::list_holders`](data.md#DataClient.fn.list_holders).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListHolders.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Rows per outcome token (`limit`, default 100, at most 1000; at most 100 with
[`include_pnl`](data.md#ListHolders.fn.include_pnl)). A cursor carries its own window and overrides
it.

##### <a id="ListHolders.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

##### <a id="ListHolders.fn.min_balance"></a>`min_balance`

```rust
pub fn min_balance(self, min_balance: Decimal) -> Self
```

Minimum balance in shares (`min_balance`, default 0).

##### <a id="ListHolders.fn.include_pnl"></a>`include_pnl`

```rust
pub fn include_pnl(self, include_pnl: bool) -> Self
```

Opt into per-holder position economics and per-side gross amounts
(`include_pnl`, default `false`). Requires exactly one condition id and a `limit`
of at most 100.

##### <a id="ListHolders.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<HolderGroup>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if no condition id or more
than 20 distinct ones are given, one is not `0x` followed by 64 hex digits, `limit`
is above 1000, or `include_pnl` is combined with several condition ids or a `limit`
above 100; otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListHolders.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<HolderGroup>
```

Streams every holder group from the configured cursor onwards, fetching pages
lazily.

Every page re-sends the same `condition` list with the cursor, as the API requires.
Page walks advance every token group together, so a token's holders are spread
over several groups: merge them by [`HolderGroup::token_id`](data.md#struct.HolderGroup). The stream ends when
the server reports no further page.

### <a id="struct.ListLeaderboard"></a>`struct ListLeaderboard`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListLeaderboard { /* private fields */ }
```

Request builder for [`DataClient::list_leaderboard`](data.md#DataClient.fn.list_leaderboard).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListLeaderboard.fn.time_period"></a>`time_period`

```rust
pub fn time_period(self, time_period: TimePeriod) -> Self
```

The window (`time_period`, default [`TimePeriod::Day`](data.md#enum.TimePeriod)).

##### <a id="ListLeaderboard.fn.category"></a>`category`

```rust
pub fn category(self, category: impl Into<String>) -> Self
```

The category (`category`): `overall` (default), a Gamma market category (e.g.
`sports`), or the synthetic `combos` (PnL board only) or `esports`.

##### <a id="ListLeaderboard.fn.sort_by"></a>`sort_by`

```rust
pub fn sort_by(self, sort_by: LeaderboardSortBy) -> Self
```

Which board (`sort_by`, default [`LeaderboardSortBy::Pnl`](data.md#enum.LeaderboardSortBy)).

##### <a id="ListLeaderboard.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListLeaderboard.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`). The
cursor pins the board (sort, window, category) it was minted on.

##### <a id="ListLeaderboard.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<LeaderboardEntry>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `limit` is above 1000;
otherwise see [`Error`](marcasite.md#enum.Error) (a parameter contradicting the cursor's
board is an [`Error::Api`](marcasite.md#enum.Error) with status `400`).

##### <a id="ListLeaderboard.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<LeaderboardEntry>
```

Streams every row from the configured cursor onwards, fetching pages lazily.

Every page restates the same board parameters with the cursor (allowed, since they
agree with the board the cursor pins). The stream ends when the server reports
no further page.

### <a id="struct.ListPositions"></a>`struct ListPositions`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListPositions { /* private fields */ }
```

Request builder for [`DataClient::list_positions`](data.md#DataClient.fn.list_positions).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListPositions.fn.user"></a>`user`

```rust
pub fn user(self, user: impl Into<Address>) -> Self
```

The wallet to anchor on (`user`).

##### <a id="ListPositions.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Condition ids (`condition`, at most 20 distinct values). With [`user`](data.md#ListPositions.fn.user),
narrows that user's positions; without it, anchors on the market's holders and
exactly one id is accepted. Duplicates are sent once.

##### <a id="ListPositions.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListPositions.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

##### <a id="ListPositions.fn.status"></a>`status`

```rust
pub fn status(self, status: PositionStatus) -> Self
```

Lifecycle filter (`status`, default [`PositionStatus::Open`](data.md#enum.PositionStatus)).

##### <a id="ListPositions.fn.event_ids"></a>`event_ids`

```rust
pub fn event_ids<I>(self, event_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Gamma event ids (`event_id`, at most 20 distinct values). Requires
[`user`](data.md#ListPositions.fn.user), and is mutually exclusive with
[`conditions`](data.md#ListPositions.fn.conditions) (the docs do not say so, but the server answers
`400` "must provide either eventId or condition, not both"). Duplicates are sent
once.

##### <a id="ListPositions.fn.title"></a>`title`

```rust
pub fn title(self, title: impl Into<String>) -> Self
```

Case-insensitive market-title substring filter (`title`, at most 200 characters).
SQL `LIKE` wildcards (`%`, `_`) keep their meaning.

##### <a id="ListPositions.fn.filter_type"></a>`filter_type`

```rust
pub fn filter_type(self, filter_type: FilterType) -> Self
```

Unit of [`filter_amount`](data.md#ListPositions.fn.filter_amount) (`filter_type`, default
[`FilterType::Tokens`](data.md#enum.FilterType)).

##### <a id="ListPositions.fn.filter_amount"></a>`filter_amount`

```rust
pub fn filter_amount(self, filter_amount: Decimal) -> Self
```

Minimum current holding (`filter_amount`): shares for [`FilterType::Tokens`](data.md#enum.FilterType)
(default `0.1`), USDC `current_value` for [`FilterType::Cash`](data.md#enum.FilterType).

##### <a id="ListPositions.fn.include_archived"></a>`include_archived`

```rust
pub fn include_archived(self, include_archived: bool) -> Self
```

Also include positions on archived markets (`include_archived`, default `false`).
Cannot be combined with [`PositionStatus::Closed`](data.md#enum.PositionStatus).

##### <a id="ListPositions.fn.sort_by"></a>`sort_by`

```rust
pub fn sort_by(self, sort_by: PositionSortBy) -> Self
```

Sort key (`sort_by`); the default follows the status.

##### <a id="ListPositions.fn.start"></a>`start`

```rust
pub fn start(self, start: DateTime<Utc>) -> Self
```

Inclusive lower bound on `last_event_at` (`start`, epoch seconds). Omitted (or the
Unix epoch, sent as `0`) means unbounded. Any bound excludes positions without
native state.

##### <a id="ListPositions.fn.end"></a>`end`

```rust
pub fn end(self, end: DateTime<Utc>) -> Self
```

Inclusive upper bound on `last_event_at` (`end`, epoch seconds). Omitted (or the
Unix epoch, sent as `0`) means unbounded.

##### <a id="ListPositions.fn.sort_direction"></a>`sort_direction`

```rust
pub fn sort_direction(self, sort_direction: SortDirection) -> Self
```

Sort direction (`sort_direction`, default [`SortDirection::Desc`](data.md#enum.SortDirection)).

##### <a id="ListPositions.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<Position>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if a documented constraint is
violated (no `user`/`condition` anchor, an empty `user`, a condition id that is not
`0x` followed by 64 hex digits, more than 20 distinct condition or event ids,
condition ids and event ids together, several condition ids without `user`,
`event_id` or `REDEEMABLE_LOST` without `user`, `limit` above 1000, `title` over 200 characters, `include_archived` with
`CLOSED`, or a `start`/`end` before the Unix epoch); otherwise see
[`Error`](marcasite.md#enum.Error).

##### <a id="ListPositions.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Position>
```

Streams every position from the configured cursor onwards, fetching pages lazily.

Every page re-sends the same `user`/`condition` anchor and filters with the cursor,
as the API requires (a bare cursor is rejected, and `title` and the `start`/`end`
window are not carried by the cursor). The stream ends when the server reports no
further page.

### <a id="struct.ListPricesHistory"></a>`struct ListPricesHistory`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListPricesHistory { /* private fields */ }
```

Request builder for [`DataClient::list_prices_history`](data.md#DataClient.fn.list_prices_history).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListPricesHistory.fn.start"></a>`start`

```rust
pub fn start(self, start: DateTime<Utc>) -> Self
```

Inclusive window start (`start`, epoch seconds; must be after the Unix epoch).
Alone it means "up to the present".

An explicit window spans **at most 15 days** (`end - start`, or now `- start`
without an `end`); the docs say a longer one is capped, but live it is a `400`
(`'start' to 'end' must span at most 15 days`, not clamped). With both `start` and
[`end`](data.md#ListPricesHistory.fn.end) set the client rejects a longer span itself; with `start` alone
it cannot (it would depend on the clock), so the server answers. Set `end` too when
paging, because the span is re-checked on every page. The
[`interval`](data.md#ListPricesHistory.fn.interval) presets are the long-range path.

Points are bucket-aligned: the first one can precede `start` by less than one
bucket.

##### <a id="ListPricesHistory.fn.end"></a>`end`

```rust
pub fn end(self, end: DateTime<Utc>) -> Self
```

Exclusive window end (`end`, epoch seconds). Requires [`start`](data.md#ListPricesHistory.fn.start), and
must not precede it.

##### <a id="ListPricesHistory.fn.interval"></a>`interval`

```rust
pub fn interval(self, interval: PriceHistoryInterval) -> Self
```

Relative window instead of `start`/`end` (`interval`).

##### <a id="ListPricesHistory.fn.bucket_seconds"></a>`bucket_seconds`

```rust
pub fn bucket_seconds(self, bucket_seconds: u32) -> Self
```

Bucket width in seconds (`bucket_seconds`, 60 to 86400). Omit to let the server
pick the densest width that covers the window (a 15-day window, for instance, is
served in 30-minute buckets); when set, it is served exactly (possibly as an empty
page where that resolution has expired).

##### <a id="ListPricesHistory.fn.as_of"></a>`as_of`

```rust
pub fn as_of(self, as_of: DateTime<Utc>) -> Self
```

Point-in-time read: the latest observation at or before this instant (`as_of`,
epoch seconds). Cannot be combined with a window.

##### <a id="ListPricesHistory.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, default and maximum 10 000). Ignored by the server once a
cursor is supplied (the cursor's own page size wins).

##### <a id="ListPricesHistory.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

A cursor page must restate the window (`start`/`end`, `interval` or `as_of`): live,
a cursor with only the token is a `400` ("provide a time component"), so the client
rejects it too. [`into_stream`](data.md#ListPricesHistory.fn.into_stream) always restates the window.

##### <a id="ListPricesHistory.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<PricePoint>>
```

Fetches one page of points, oldest first.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the token id is empty,
the window is missing (also with a cursor) or ambiguous, `end` is set without
`start` or before it, a bound is not after the Unix epoch, `start` and `end` span
more than 15 days, `bucket_seconds` is outside 60..=86400, or `limit` is above
10 000; otherwise see [`Error`](marcasite.md#enum.Error). A `start` alone more than 15 days
back is rejected by the server only.

##### <a id="ListPricesHistory.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<PricePoint>
```

Streams every point from the configured cursor onwards, fetching pages lazily.

Every page re-sends the same token and window with the cursor. The terminal point
(the latest observation in the window) and a resolved market's settlement point
arrive on the final page. The stream ends when the server reports no further page.

### <a id="struct.ListTrades"></a>`struct ListTrades`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTrades { /* private fields */ }
```

Request builder for [`DataClient::list_trades`](data.md#DataClient.fn.list_trades).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListTrades.fn.user"></a>`user`

```rust
pub fn user(self, user: impl Into<Address>) -> Self
```

Only this wallet's trades (`user`); omit for the market/event/global feed.

##### <a id="ListTrades.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

First-page size (`limit`, at most 1000). Ignored by the server once a cursor is
supplied (the cursor's own page size wins).

##### <a id="ListTrades.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Resumes from a previous page's [`next_cursor`](data.md#Page.fn.next_cursor) (`cursor`).

##### <a id="ListTrades.fn.taker_only"></a>`taker_only`

```rust
pub fn taker_only(self, taker_only: bool) -> Self
```

`true` (default): each fill once, on its taker side; `false`: maker rows too
(`taker_only`).

##### <a id="ListTrades.fn.filter_type"></a>`filter_type`

```rust
pub fn filter_type(self, filter_type: FilterType) -> Self
```

Unit of [`filter_amount`](data.md#ListTrades.fn.filter_amount) (`filter_type`, default
[`FilterType::Tokens`](data.md#enum.FilterType)).

##### <a id="ListTrades.fn.filter_amount"></a>`filter_amount`

```rust
pub fn filter_amount(self, filter_amount: Decimal) -> Self
```

Minimum trade size (`filter_amount`, default `0.01`; `0` means the same).

##### <a id="ListTrades.fn.start"></a>`start`

```rust
pub fn start(self, start: DateTime<Utc>) -> Self
```

Inclusive window start on the block timestamp (`start`, epoch seconds). Honoured
with [`user`](data.md#ListTrades.fn.user) only; the other shapes ignore it.

Omitted, or the Unix epoch (sent as `0`), floors the window to three years back;
use [`full_history`](data.md#ListTrades.fn.full_history) for the full history.

##### <a id="ListTrades.fn.full_history"></a>`full_history`

```rust
pub fn full_history(self) -> Self
```

Asks for the full history instead of the default three years: sends `start=1`, as
documented. Honoured with [`user`](data.md#ListTrades.fn.user) only. Replaces any
[`start`](data.md#ListTrades.fn.start).

##### <a id="ListTrades.fn.end"></a>`end`

```rust
pub fn end(self, end: DateTime<Utc>) -> Self
```

Inclusive window end (`end`, epoch seconds). Honoured with [`user`](data.md#ListTrades.fn.user)
only; omitted (or the Unix epoch, sent as `0`) means now plus one day.

##### <a id="ListTrades.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(self, conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Condition ids (`condition`, at most 20 distinct values). Duplicates are sent once.

##### <a id="ListTrades.fn.event_ids"></a>`event_ids`

```rust
pub fn event_ids<I>(self, event_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Gamma event ids (`event_id`, at most 20 distinct values). Mutually exclusive with
[`conditions`](data.md#ListTrades.fn.conditions) (the docs do not say so, but the server answers
`400` "must provide either eventId or condition, not both"). Duplicates are sent
once.

##### <a id="ListTrades.fn.side"></a>`side`

```rust
pub fn side(self, side: Side) -> Self
```

Only buys or only sells (`side`).

##### <a id="ListTrades.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<Page<Trade>>
```

Fetches one page.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `user` is set but empty,
`limit` is above 1000, a condition id is not `0x` followed by 64 hex digits, more
than 20 distinct condition or event ids are given, both are given, or a bound is
before the Unix epoch; otherwise see [`Error`](marcasite.md#enum.Error).

##### <a id="ListTrades.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Trade>
```

Streams every trade from the configured cursor onwards, fetching pages lazily.

The cursor carries only the seek anchor and page size, so every page re-sends the
same filters (changing one mid-walk would silently re-anchor the feed). The stream
ends when the server reports no further page.

### <a id="struct.LiveVolume"></a>`struct LiveVolume`

```rust
#[non_exhaustive]
pub struct LiveVolume {
    /// Sum of the rows' `taker_volume`, in shares.
    pub taker_volume_total: Decimal,
    /// One row per market, `taker_volume` descending; empty when the events resolve to no
    /// markets.
    pub conditions: Vec<ConditionVolume>,
}
```

Cumulative one-side (taker) volume per market of the requested events
(`components/schemas/LiveVolume`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.OpenInterest"></a>`struct OpenInterest`

```rust
#[non_exhaustive]
pub struct OpenInterest {
    /// Condition id the row answers for. On the global figure it holds the sentinel
    /// `GLOBAL`, which is not a condition id: check [`is_global`](data.md#OpenInterest.fn.is_global) first.
    pub condition_id: ConditionId,
    /// Priced gross open interest, in USDC; `0` when nothing is held.
    pub value: Decimal,
}
```

The priced gross open interest of a market, or the global figure
(`components/schemas/OpenInterest`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Associated items

##### <a id="OpenInterest.constant.GLOBAL"></a>`GLOBAL`

```rust
pub const GLOBAL: &'static str = "GLOBAL";
```

The `condition_id` of the global figure.

##### <a id="OpenInterest.fn.is_global"></a>`is_global`

```rust
#[must_use]
pub fn is_global(&self) -> bool
```

`true` for the global (all markets) figure.

### <a id="struct.Page"></a>`struct Page`

```rust
#[non_exhaustive]
pub struct Page<T> {
    /// The page's rows.
    pub data: Vec<T>,
    /// The paging envelope, as served.
    pub pagination: Pagination,
    /* private fields */
}
```

One page of a cursor-paginated Data API v2 listing (the `{ data, pagination }` envelope).

Read the rows with [`items`](data.md#Page.fn.items) (or [`into_items`](data.md#Page.fn.into_items)) and pass
[`next_cursor`](data.md#Page.fn.next_cursor) to the request's `cursor(..)` setter for the next
page, or use the request's `into_stream()` to walk every page. The walk is over when
`next_cursor()` is `None`.

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

The page's rows.

##### <a id="Page.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<T>
```

Consumes the page and returns its rows.

##### <a id="Page.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor for the next page, or `None` when there is no next page.

This is [`Pagination::next_cursor`](data.md#struct.Pagination), except that it is also `None` when the server
reports [`has_more`](data.md#struct.Pagination) `false` or serves an empty cursor, so
that a contradictory page can never restart a walk.

##### <a id="Page.fn.has_more"></a>`has_more`

```rust
#[must_use]
pub fn has_more(&self) -> bool
```

`true` if another page exists, as reported by the server (`pagination.has_more`,
documented as exact: never inferred from page fullness).

##### <a id="Page.fn.trace_id"></a>`trace_id`

```rust
#[must_use]
pub fn trace_id(&self) -> Option<&str>
```

The trace id the server sent with this page in its `x-trace-id` header, if any.

The API echoes one on every response; quote it when reporting wrong-looking data.
It is `None` for a page that was not received from the API (e.g. deserialized from
JSON by the caller).

### <a id="struct.Pagination"></a>`struct Pagination`

```rust
#[non_exhaustive]
pub struct Pagination {
    /// Page size this page was served with.
    pub limit: u32,
    /// Running item offset, for numbering rows across pages. Cosmetic: the cursor drives
    /// the actual seek, there is no total, and `offset` is never a request parameter.
    pub offset: u32,
    /// Exact: `true` iff another page exists.
    pub has_more: bool,
    /// Opaque, signed cursor for the next page; `None` on the last page. Prefer
    /// [`Page::next_cursor`](data.md#Page.fn.next_cursor), which is also `None` when `has_more` is `false`.
    pub next_cursor: Option<String>,
}
```

The paging envelope of a Data API v2 page (`components/schemas/Pagination`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `Hash`, `PartialEq`, `Serialize`

### <a id="struct.PortfolioValue"></a>`struct PortfolioValue`

```rust
#[non_exhaustive]
pub struct PortfolioValue {
    /// The wallet the value was computed for.
    pub proxy_wallet: Address,
    /// Portfolio value in USDC, rounded to 4 decimals: holdings marked to market plus
    /// non-terminal combos at cost basis. `0` for a user with no positions.
    pub value: Decimal,
}
```

A user's portfolio value (`/v2/value`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Position"></a>`struct Position`

```rust
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
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means it could not be
    /// labeled.
    pub outcome_index: i32,
    /// Label of the market's other outcome.
    pub opposite_outcome: String,
    /// Token id of the market's other outcome.
    pub opposite_token_id: TokenId,
    /// Market end date. The sentinel `1970-01-01` (kept as served) means Gamma has none.
    pub end_date: NaiveDate,
    /// The row's last economics event. The sentinel Unix epoch (`0` on the wire, kept as
    /// served) means the position has no native state: such rows carry no clock and are
    /// excluded by any [`start`](data.md#ListPositions.fn.start) / [`end`](data.md#ListPositions.fn.end)
    /// bound.
    pub last_event_at: DateTime<Utc>,
    /// When the wallet first entered the position (epoch seconds on the wire); `None` when
    /// the position has no native state (served as `0`, like the sentinel
    /// [`last_event_at`](data.md#struct.Position); it also decodes from a missing or `null`
    /// key).
    ///
    /// **Undocumented**: the docs' `Position` schema does not list this field, but every
    /// `/v2/positions` row carries it live.
    pub first_entry_at: Option<DateTime<Utc>>,
    /// Profile display name of the wallet.
    pub name: String,
    /// Profile image URL.
    pub profile_image: String,
    /// Profile verification badge.
    pub verified: bool,
}
```

One position: a holding in a single outcome token, priced and enriched
(`components/schemas/Position`).

The shape is the same for user-anchored (open or closed) and market-anchored
requests. On closed positions `current_size`, `current_value` and `unrealized_pnl` are
approximately zero.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PricePoint"></a>`struct PricePoint`

```rust
#[non_exhaustive]
pub struct PricePoint {
    /// The observation's own time (a bucket's start for aggregates), never the time asked
    /// for.
    pub timestamp: DateTime<Utc>,
    /// Price, in `0..=1`.
    pub price: Decimal,
    /// Width in seconds of the window the price was observed in: `0` for an exact tick,
    /// the bucket width for an aggregate.
    ///
    /// On a request that pins [`bucket_seconds`](data.md#ListPricesHistory.fn.bucket_seconds), it
    /// echoes the requested grid, not the density of what filled it; counting rows is the
    /// only density measure.
    pub resolution_seconds: i64,
}
```

One price-history point (`components/schemas/PricePoint`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Resolution"></a>`struct Resolution`

```rust
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
    /// on condition-keyed rows. See [`last_update_time`](data.md#Resolution.fn.last_update_time).
    pub last_update_timestamp: String,
    /// Condition id the row answers for; absent on question-keyed rows.
    pub condition_id: Option<ConditionId>,
    /// UMA question id serving the row; absent on condition-keyed rows.
    pub question_id: Option<QuestionId>,
    /// Estimated settlement time (an estimate, not a deadline); `None` when timing is
    /// unavailable.
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
    /// scale nor their range, so they are not converted to a `Decimal`.
    pub proposed_price: Option<String>,
    /// Price of the second proposal, as a numeric string (same conventions as
    /// `proposed_price`).
    pub reproposed_price: Option<String>,
    /// Reporter family that resolved the market.
    pub reporter: Option<Reporter>,
    /// Whether an oracle reported the resolution or it was derived.
    pub resolution_source: Option<ResolutionSource>,
    /// When the condition resolved.
    pub resolved_at: Option<DateTime<Utc>>,
    /// Block the condition resolved at.
    pub resolved_block: Option<i64>,
    /// Whether arbitration was triggered on the request.
    pub was_arbitrated: Option<bool>,
}
```

One resolution-state row (`components/schemas/ResolutionWithSettlementTime`: the
`Resolution` fields plus `settlement_time_basis`).

UMA lifecycle rows populate the numeric-string price fields; question lookups omit
`condition_id`. Native and terminal rows populate the condition lifecycle, payout and
finality fields where available.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="Resolution.fn.last_update_time"></a>`last_update_time`

```rust
#[must_use]
pub fn last_update_time(&self) -> Option<DateTime<Utc>>
```

Parses [`last_update_timestamp`](data.md#struct.Resolution), which is served as
epoch seconds on question-keyed rows and RFC 3339 on condition-keyed rows.

Returns `None` if the value is empty or in neither format.

### <a id="struct.ServiceStatus"></a>`struct ServiceStatus`

```rust
#[non_exhaustive]
pub struct ServiceStatus {
    /// When this snapshot was taken.
    pub computed_at: DateTime<Utc>,
    /// How old the snapshot is, in seconds. A value that keeps climbing means the
    /// refresher is not completing.
    pub age_seconds: i64,
    /// Freshness of the projections behind the feeds.
    pub serving: ServingFreshness,
    /// Ingestion health: one cursor per `(contract, event)` stream.
    pub ingestion: IngestionFreshness,
}
```

How fresh the data behind the Data API is (`components/schemas/ServiceStatus`).

Served from a snapshot refreshed in the background: [`computed_at`](data.md#struct.ServiceStatus)
and [`age_seconds`](data.md#struct.ServiceStatus) say how old it is.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ServingFreshness"></a>`struct ServingFreshness`

```rust
#[non_exhaustive]
pub struct ServingFreshness {
    /// Every mechanism that produced a candidate freshness row, in a fixed order.
    pub mechanisms: Vec<ServingMechanism>,
    /// The worst age across `mechanisms`, in seconds; `None` only when no mechanism
    /// reported.
    pub lag_seconds: Option<i64>,
    /// Which mechanism `lag_seconds` came from.
    pub worst: Option<ServingMechanismName>,
}
```

Freshness of the projections behind the feeds (`components/schemas/ServingFreshness`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ServingMechanism"></a>`struct ServingMechanism`

```rust
#[non_exhaustive]
pub struct ServingMechanism {
    /// What this mechanism produces.
    pub name: ServingMechanismName,
    /// Seconds since it last advanced, by its own clock.
    pub age_seconds: i64,
    /// How far behind the ingestion tail it has projected, in blocks; `None` for a
    /// mechanism that records a time but no block.
    pub blocks_behind: Option<i64>,
}
```

One serving mechanism's freshness (`components/schemas/ServingMechanism`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Trade"></a>`struct Trade`

```rust
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
    /// [`UNLABELED_OUTCOME_INDEX`](data.md#constant.UNLABELED_OUTCOME_INDEX) means it could not be
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
```

A trade (`components/schemas/Trade`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.UserPnlPoint"></a>`struct UserPnlPoint`

```rust
#[non_exhaustive]
pub struct UserPnlPoint {
    /// Point time.
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
    /// Collateral moved into the wallet. **Always `None` live** (served as `null` on every
    /// point observed), although the docs list it as an amount.
    pub deposits: Option<Decimal>,
    /// Collateral moved out of the wallet. **Always `None` live** (see
    /// [`deposits`](data.md#struct.UserPnlPoint)).
    pub withdrawals: Option<Decimal>,
    /// `deposits - withdrawals`. **Always `None` live** (see
    /// [`deposits`](data.md#struct.UserPnlPoint)).
    pub cashflow_net: Option<Decimal>,
}
```

One dense cumulative PnL point, in USDC (`components/schemas/UserPnlPoint`).

Every amount is cumulative through [`timestamp`](data.md#struct.UserPnlPoint). An optional amount
is `None` when its source was unavailable; it never means zero.

Live (checked 2026-10-02 over ~188k points of 25 wallets) only
[`deposits`](data.md#struct.UserPnlPoint), [`withdrawals`](data.md#struct.UserPnlPoint) and
[`cashflow_net`](data.md#struct.UserPnlPoint) are ever `None`, on every point; every other
optional amount, [`unrealized_pnl`](data.md#struct.UserPnlPoint) and
[`position_pnl`](data.md#struct.UserPnlPoint) included, is always present.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.UserPnlSeries"></a>`struct UserPnlSeries`

```rust
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
    ///
    /// The docs do not list the values of this field. It is decoded as a [`PnlFidelity`](data.md#enum.PnlFidelity)
    /// (the vocabulary of [`fidelity`](data.md#struct.UserPnlSeries)); any other value is kept as
    /// [`PnlFidelity::Unknown`](data.md#enum.PnlFidelity). Live it was `1d` for every interval and fidelity tried
    /// (`fidelity` itself echoes the request, `1h` by default).
    pub source_fidelity: PnlFidelity,
    /// Dense cumulative points on the requested grid, oldest first.
    pub points: Vec<UserPnlPoint>,
}
```

A user's cumulative PnL series (`components/schemas/UserPnlSeries`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.UserStats"></a>`struct UserStats`

```rust
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
    pub join_date: Option<DateTime<Utc>>,
    /// Newest persisted cumulative all-time PnL point; `None` when the user is known but
    /// has no observation yet.
    pub all_time_pnl: Option<UserPnlPoint>,
}
```

A wallet's profile card (`components/schemas/UserStats`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.UserVolume"></a>`struct UserVolume`

```rust
#[non_exhaustive]
pub struct UserVolume {
    /// Both-sides traded volume over the window, in shares.
    pub volume: Decimal,
    /// Both-sides cash volume over the window, in USD.
    pub volume_usdc: Decimal,
    /// Number of fills in the window.
    pub trade_count: u64,
}
```

A wallet's trading volume over a whole-day window (`components/schemas/UserVolume`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

## Enums

### <a id="enum.ActivitySide"></a>`enum ActivitySide`

```rust
#[non_exhaustive]
pub enum ActivitySide {
    /// A buy (trade rows).
    Buy,
    /// A sell (trade rows).
    Sell,
    /// Received (tip rows).
    In,
    /// Sent (tip rows).
    Out,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Direction of an activity-feed event (`side`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ActivitySide.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ActivitySide.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ActivitySortBy"></a>`enum ActivitySortBy`

```rust
#[non_exhaustive]
pub enum ActivitySortBy {
    /// `(block_timestamp, sequence_id)` order.
    Timestamp,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Sort key of `/v2/activity` (`sort_by`). Only [`Timestamp`](data.md#enum.ActivitySortBy) is
supported (the feed pages by keyset).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ActivitySortBy.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ActivitySortBy.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ActivityType"></a>`enum ActivityType`

```rust
#[non_exhaustive]
pub enum ActivityType {
    /// A trade.
    Trade,
    /// A split of collateral into outcome tokens.
    Split,
    /// A merge of outcome tokens back into collateral.
    Merge,
    /// A redemption.
    Redeem,
    /// A reward.
    Reward,
    /// A conversion.
    Conversion,
    /// A user-to-user transfer that is not a trade-settlement leg. Opt-in: only
    /// returned when requested through [`ListActivity::types`](data.md#ListActivity.fn.types).
    Tip,
    /// A maker-side fee rebate credit. **Undocumented** (not in the docs' `type`
    /// list); served live. A non-trade row: `condition_id`, `token_id` and `side` are
    /// empty.
    MakerRebate,
    /// A taker-side fee rebate credit. **Undocumented**; served live, like
    /// [`MakerRebate`](data.md#enum.ActivityType).
    TakerRebate,
    /// A yield income credit. **Undocumented**; served live, like
    /// [`MakerRebate`](data.md#enum.ActivityType).
    Yield,
    /// A referral-program reward credit. **Undocumented**; served live, like
    /// [`MakerRebate`](data.md#enum.ActivityType).
    ReferralReward,
    /// A collateral deposit. **Undocumented**; served live only with
    /// [`exclude_deposits_withdrawals(false)`](data.md#ListActivity.fn.exclude_deposits_withdrawals).
    /// Filtering by it without that flag returns an empty page (the default
    /// excludes deposits and withdrawals).
    Deposit,
    /// A collateral withdrawal. **Undocumented**; see [`Deposit`](data.md#enum.ActivityType).
    Withdrawal,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Kind of an activity-feed event (`type`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ActivityType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ActivityType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ComboActivityType"></a>`enum ComboActivityType`

```rust
#[non_exhaustive]
pub enum ComboActivityType {
    /// A split.
    Split,
    /// A merge.
    Merge,
    /// A conversion.
    Convert,
    /// A compression.
    Compress,
    /// A wrap.
    Wrap,
    /// An unwrap.
    Unwrap,
    /// A redemption.
    Redeem,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Kind of a combo lifecycle/redemption event (`type`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboActivityType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ComboActivityType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ComboLegStatus"></a>`enum ComboLegStatus`

```rust
#[non_exhaustive]
pub enum ComboLegStatus {
    /// Unresolved.
    Open,
    /// Resolved in the combo's favour.
    ResolvedWin,
    /// Resolved against the combo.
    ResolvedLoss,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Live resolution state of a combo leg (`leg_status`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboLegStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ComboLegStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ComboPositionSortBy"></a>`enum ComboPositionSortBy`

```rust
#[non_exhaustive]
pub enum ComboPositionSortBy {
    /// First entry time (default, except under `status=REDEEMABLE`).
    FirstEntry,
    /// Entry cost.
    EntryCost,
    /// Alias of [`EntryCost`](data.md#enum.ComboPositionSortBy).
    CurrentValue,
    /// Last update time; pair with ascending order for incremental sync.
    Updated,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Sort key of `/v2/positions/combos` (`sort_by`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboPositionSortBy.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ComboPositionSortBy.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ComboPositionStatus"></a>`enum ComboPositionStatus`

```rust
#[non_exhaustive]
pub enum ComboPositionStatus {
    /// Open: the superset, including still-held redeemable positions.
    Open,
    /// Exactly the rows whose `redeemable` flag is `true`. Must be the only status
    /// filter value.
    Redeemable,
    /// Partially resolved (filter value).
    Partial,
    /// Resolved as a win.
    ResolvedWin,
    /// Resolved as a loss.
    ResolvedLoss,
    /// Resolved partially.
    ResolvedPartial,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Lifecycle state of a combo position (`status` filter and row field of
`/v2/positions/combos`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboPositionStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ComboPositionStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ErrorCode"></a>`enum ErrorCode`

```rust
#[non_exhaustive]
pub enum ErrorCode {
    /// `400`: invalid query parameters, cursor or selector.
    InvalidRequest,
    /// Unauthorized.
    Unauthorized,
    /// `404`: not found.
    NotFound,
    /// `405`: method not allowed.
    MethodNotAllowed,
    /// `503`: the request deadline, the datastore statement timeout or the connection
    /// pool's acquire budget was exceeded.
    RequestTimeout,
    /// `429`: rate limited. Retry after `Retry-After`.
    RateLimited,
    /// `503`: a serving dependency is unavailable.
    DependencyUnavailable,
    /// `500`: internal server error.
    Internal,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Stable machine-readable classification of a Data API v2 failure
(`components/schemas/ErrorCode`), carried in the `code` field of the error body.

See <https://docs.polymarket.com/api-reference/data-api/overview>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ErrorCode.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ErrorCode.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

##### <a id="ErrorCode.fn.from_error"></a>`from_error`

```rust
#[must_use]
pub fn from_error(err: &Error) -> Option<Self>
```

The Data API v2 error code of `err`, if it is an API error returned by the Data API
with a `code` field.

`None` for other errors, and for a Data API error body without `code` (such as the
`{ "error": "..." }` body shown in the overview), whose message is still available
from [`ApiError::message`](marcasite.md#ApiError.fn.message).

```rust
use marcasite::data::ErrorCode;

fn should_fix_request(err: &marcasite::Error) -> bool {
    ErrorCode::from_error(err) == Some(ErrorCode::InvalidRequest)
}
```

### <a id="enum.FilterType"></a>`enum FilterType`

```rust
#[non_exhaustive]
pub enum FilterType {
    /// A USDC amount.
    Cash,
    /// A number of shares (tokens).
    Tokens,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Which unit a `filter_amount` floor is expressed in (`filter_type`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="FilterType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="FilterType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.LeaderboardSortBy"></a>`enum LeaderboardSortBy`

```rust
#[non_exhaustive]
pub enum LeaderboardSortBy {
    /// Ranked by PnL (default).
    Pnl,
    /// Ranked by volume.
    Volume,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Which trader leaderboard to read (`sort_by`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="LeaderboardSortBy.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="LeaderboardSortBy.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PnlFidelity"></a>`enum PnlFidelity`

```rust
#[non_exhaustive]
pub enum PnlFidelity {
    /// One day.
    OneDay,
    /// Eighteen hours.
    EighteenHours,
    /// Twelve hours.
    TwelveHours,
    /// Three hours.
    ThreeHours,
    /// One hour (the default).
    OneHour,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Output grid of `/v2/user-pnl` (`fidelity`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PnlFidelity.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PnlFidelity.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PnlInterval"></a>`enum PnlInterval`

```rust
#[non_exhaustive]
pub enum PnlInterval {
    /// The maximum window.
    Max,
    /// All time.
    All,
    /// One month.
    OneMonth,
    /// One week.
    OneWeek,
    /// One day (the default).
    OneDay,
    /// Twelve hours.
    TwelveHours,
    /// Six hours.
    SixHours,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Window of `/v2/user-pnl` (`interval`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PnlInterval.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PnlInterval.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PositionSortBy"></a>`enum PositionSortBy`

```rust
#[non_exhaustive]
pub enum PositionSortBy {
    /// Current value (default for `OPEN` / `REDEEMABLE`).
    CurrentValue,
    /// The effective `current_price`.
    Price,
    /// Number of tokens held.
    Tokens,
    /// Unrealized PnL.
    UnrealizedPnl,
    /// Realized PnL (default for `CLOSED`).
    RealizedPnl,
    /// Total PnL.
    TotalPnl,
    /// The row's `last_event_at`.
    Timestamp,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Sort key of `/v2/positions` (`sort_by`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PositionSortBy.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PositionSortBy.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PositionStatus"></a>`enum PositionStatus`

```rust
#[non_exhaustive]
pub enum PositionStatus {
    /// Open positions: the **superset**, including settled-but-unredeemed winners. The
    /// default filter.
    Open,
    /// Open positions whose market resolved and whose tokens are still held.
    Redeemable,
    /// Redeemable positions that lost (filter only, requires `user`; rows keep status
    /// [`Redeemable`](data.md#enum.PositionStatus)).
    RedeemableLost,
    /// Open positions narrowed to live complementary pairs (filter only, user-scoped).
    Mergeable,
    /// Exited positions.
    Closed,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Lifecycle state of a position (`status` filter and row field of `/v2/positions`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PositionStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PositionStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PriceHistoryInterval"></a>`enum PriceHistoryInterval`

```rust
#[non_exhaustive]
pub enum PriceHistoryInterval {
    /// The market's whole life.
    Max,
    /// The market's whole life (same as [`Max`](data.md#enum.PriceHistoryInterval)).
    All,
    /// One month.
    OneMonth,
    /// One week.
    OneWeek,
    /// One day.
    OneDay,
    /// Six hours.
    SixHours,
    /// One hour.
    OneHour,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Relative window of `/v2/prices-history` (`interval`).

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

### <a id="enum.Reporter"></a>`enum Reporter`

```rust
#[non_exhaustive]
pub enum Reporter {
    /// The UMA optimistic oracle.
    UmaOo,
    /// Chainlink.
    Chainlink,
    /// An externally owned account.
    Eoa,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Reporter family that resolved a market (`reporter`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="Reporter.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="Reporter.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ResolutionMarketType"></a>`enum ResolutionMarketType`

```rust
#[non_exhaustive]
pub enum ResolutionMarketType {
    /// A binary market.
    Binary,
    /// An incremental neg-risk market.
    IncrementalNegrisk,
    /// An atomic neg-risk market.
    AtomicNegrisk,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Market type of a condition-keyed resolution row (`market_type`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ResolutionMarketType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ResolutionMarketType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ResolutionSelector"></a>`enum ResolutionSelector`

```rust
#[non_exhaustive]
pub enum ResolutionSelector {
    /// One UMA question (`question_id`).
    Question(QuestionId),
    /// Up to 20 condition ids (`condition`).
    Conditions(Vec<ConditionId>),
    /// Up to 20 Gamma event ids (`event_id`).
    Events(Vec<EventId>),
}
```

Which resolution rows to fetch with [`DataClient::get_resolutions`](data.md#DataClient.fn.get_resolutions): exactly one
selector family per request.

The ids are checked when the request is sent: a question id must be `0x` followed by
64 hex digits, condition ids likewise, and event ids positive integers; lists accept
at most 20 distinct values, and duplicates are sent once.

**Implements:** `Clone`, `Debug`, `Eq`, `From<QuestionId>`, `PartialEq`

#### Methods

##### <a id="ResolutionSelector.fn.question"></a>`question`

```rust
pub fn question(question_id: impl Into<QuestionId>) -> Self
```

Selects one UMA question.

##### <a id="ResolutionSelector.fn.conditions"></a>`conditions`

```rust
pub fn conditions<I>(conditions: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Selects condition ids (at most 20 distinct values).

##### <a id="ResolutionSelector.fn.events"></a>`events`

```rust
pub fn events<I>(event_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<EventId>,
```

Selects Gamma event ids (at most 20 distinct values).

### <a id="enum.ResolutionSource"></a>`enum ResolutionSource`

```rust
#[non_exhaustive]
pub enum ResolutionSource {
    /// An oracle reported it.
    Reported,
    /// Derived from a neg-risk sibling resolution.
    Derived,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

How a resolution came about (`resolution_source`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ResolutionSource.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ResolutionSource.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ResolutionStatus"></a>`enum ResolutionStatus`

```rust
#[non_exhaustive]
pub enum ResolutionStatus {
    /// Initialized.
    Initialized,
    /// Posed.
    Posed,
    /// Proposed.
    Proposed,
    /// Challenged.
    Challenged,
    /// Re-proposed.
    Reproposed,
    /// Disputed.
    Disputed,
    /// Resolved.
    Resolved,
    /// Active (condition-keyed rows only).
    Active,
    /// In arbitration (condition-keyed rows only).
    Arbitration,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Lifecycle state of a resolution row (`status`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ResolutionStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ResolutionStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.ServingMechanismName"></a>`enum ServingMechanismName`

```rust
#[non_exhaustive]
pub enum ServingMechanismName {
    /// Activity-feed enrichment.
    ActivityFeed,
    /// Custody-balance ingestion.
    CustodyBalances,
    /// The PnL engine.
    Pnl,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

What a serving mechanism produces.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ServingMechanismName.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="ServingMechanismName.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.SettlementTimeBasis"></a>`enum SettlementTimeBasis`

```rust
#[non_exhaustive]
pub enum SettlementTimeBasis {
    /// The managed proposal's expiration.
    ManagedProposalExpiration,
    /// The liveness period.
    Liveness,
    /// A DVM voting-round estimate.
    DvmRoundEstimate,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Source of the `expected_settlement_time` estimate (`settlement_time_basis`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SettlementTimeBasis.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="SettlementTimeBasis.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.SortDirection"></a>`enum SortDirection`

```rust
#[non_exhaustive]
pub enum SortDirection {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

A sort direction (`sort_direction`).

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

### <a id="enum.TimePeriod"></a>`enum TimePeriod`

```rust
#[non_exhaustive]
pub enum TimePeriod {
    /// One day.
    Day,
    /// One week.
    Week,
    /// One month.
    Month,
    /// All time (on `/v2/builders/volume`: yearly buckets).
    All,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

A board window (`time_period`, and the bucket width `interval` of
`/v2/builders/volume`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TimePeriod.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="TimePeriod.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.TokenStandard"></a>`enum TokenStandard`

```rust
#[non_exhaustive]
pub enum TokenStandard {
    /// An ERC-20 token (allowance-based approval).
    Erc20,
    /// An ERC-1155 token (all-or-nothing operator approval).
    Erc1155,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Token standard of an approval pair.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TokenStandard.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="TokenStandard.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.WinKind"></a>`enum WinKind`

```rust
#[non_exhaustive]
pub enum WinKind {
    /// A single-market position.
    Market,
    /// A combo position: no Gamma event (`event_id` is `0`, `event_slug` is empty).
    Combo,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

Kind of a biggest-winner row (`kind`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="WinKind.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="WinKind.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

## Constants

### <a id="constant.MAX_LIST_VALUES"></a>`const MAX_LIST_VALUES`

```rust
pub const MAX_LIST_VALUES: usize = 20;
```

The maximum number of distinct values accepted by the comma-separated `condition` and
`event_id` list parameters.

### <a id="constant.UNLABELED_OUTCOME_INDEX"></a>`const UNLABELED_OUTCOME_INDEX`

```rust
pub const UNLABELED_OUTCOME_INDEX: i32 = 999;
```

The `outcome_index` sentinel meaning "the outcome could not be labeled".

See the *Units and Sentinels* section of
<https://docs.polymarket.com/api-reference/data-api/overview>.
