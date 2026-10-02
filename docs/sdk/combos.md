# Module `marcasite::combos`

> Generated from marcasite 0.1.0 (all features) by `just docs-md`. Do not edit.

Combos / RFQ REST API client (`https://combos-rfq-api.polymarket.com`).

The combinatorial RFQ (request for quote) system lets traders combine several markets
("legs") into one combo position. This module covers its public catalog endpoint. Start
from [`CombosClient`](combos.md#struct.CombosClient).

| Endpoint | Method |
|---|---|
| `GET /v1/rfq/combo-markets` | [`CombosClient::list_combo_markets`](combos.md#CombosClient.fn.list_combo_markets) |

The authenticated maker (quoter) commands (`POST /v1/maker/quotes`,
`POST /v1/maker/quotes/cancel`, `POST /v1/maker/confirmations`) and the RFQ Quoter
Gateway WebSocket are not supported yet.

See <https://docs.polymarket.com/api-reference/combo-markets/get-combo-markets>.

## Index

- **Structs:** [`ComboMarket`](#struct.ComboMarket), [`ComboMarketId`](#struct.ComboMarketId), [`ComboMarketsPage`](#struct.ComboMarketsPage), [`CombosClient`](#struct.CombosClient), [`CombosClientBuilder`](#struct.CombosClientBuilder), [`ListComboMarkets`](#struct.ListComboMarkets), [`PositionId`](#struct.PositionId)

## Structs

### <a id="struct.ComboMarket"></a>`struct ComboMarket`

```rust
#[non_exhaustive]
pub struct ComboMarket {
    /// Market id.
    pub id: ComboMarketId,
    /// The market's condition id.
    pub condition_id: ConditionId,
    /// Combo position ids; `[0]` is YES, `[1]` is NO.
    pub position_ids: Vec<PositionId>,
    /// Whether the market is pending. Undocumented; observed live (2026-10-02) on every
    /// market. Optional so that a response without it (as in the spec's example) still
    /// decodes.
    pub pending: Option<bool>,
    /// URL slug.
    pub slug: String,
    /// Market title, e.g. `"Will Mexico win on 2026-06-11?"`.
    pub title: String,
    /// Outcome labels, e.g. `["Yes", "No"]`; `[0]` is YES, `[1]` is NO.
    pub outcomes: Vec<String>,
    /// Outcome prices (strings on the wire, e.g. `["0.685", "0.315"]`); `[0]` is YES,
    /// `[1]` is NO.
    pub outcome_prices: Vec<Decimal>,
    /// Image URL.
    pub image: String,
    /// Market volume (a JSON number on the wire). The catalog is ordered by volume,
    /// descending.
    pub volume: Decimal,
    /// Tag slugs, e.g. `["sports", "soccer"]`.
    pub tags: Vec<String>,
}
```

A market that can be used as a combo leg (`components/schemas/ComboMarket`).

[`position_ids`](combos.md#struct.ComboMarket), [`outcomes`](combos.md#struct.ComboMarket) and
[`outcome_prices`](combos.md#struct.ComboMarket) correspond by index: `[0]` is YES and `[1]` is
NO. Use [`yes_position_id`](combos.md#ComboMarket.fn.yes_position_id), [`no_position_id`](combos.md#ComboMarket.fn.no_position_id),
[`yes_price`](combos.md#ComboMarket.fn.yes_price) and [`no_price`](combos.md#ComboMarket.fn.no_price) to read them without
indexing.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboMarket.fn.yes_position_id"></a>`yes_position_id`

```rust
#[must_use]
pub fn yes_position_id(&self) -> Option<&PositionId>
```

The YES combo position id (`position_ids[0]`), if present.

##### <a id="ComboMarket.fn.no_position_id"></a>`no_position_id`

```rust
#[must_use]
pub fn no_position_id(&self) -> Option<&PositionId>
```

The NO combo position id (`position_ids[1]`), if present.

##### <a id="ComboMarket.fn.yes_price"></a>`yes_price`

```rust
#[must_use]
pub fn yes_price(&self) -> Option<Decimal>
```

The YES price (`outcome_prices[0]`), if present.

##### <a id="ComboMarket.fn.no_price"></a>`no_price`

```rust
#[must_use]
pub fn no_price(&self) -> Option<Decimal>
```

The NO price (`outcome_prices[1]`), if present.

### <a id="struct.ComboMarketId"></a>`struct ComboMarketId`

```rust
pub struct ComboMarketId(/* private fields */);
```

The id of a market in the combo catalog (`ComboMarket.id`), e.g. `"1897034"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&ComboMarketId>`, `From<&String>`, `From<&str>`, `From<ComboMarketId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="ComboMarketId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="ComboMarketId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="ComboMarketId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.ComboMarketsPage"></a>`struct ComboMarketsPage`

```rust
#[non_exhaustive]
pub struct ComboMarketsPage {
    /// The markets on this page, by volume descending.
    pub markets: Vec<ComboMarket>,
    /// Cursor for the next page, as sent: `None` on the final page. Prefer
    /// [`next_cursor()`](combos.md#ComboMarketsPage.fn.next_cursor), which also treats an empty string as the end.
    /// Pass it back unchanged with [`ListComboMarkets::cursor`](combos.md#ListComboMarkets.fn.cursor).
    pub next_cursor: Option<String>,
}
```

One page of the combo market catalog (`components/schemas/ComboMarketsResponse`),
returned by [`ListComboMarkets::send`](combos.md#ListComboMarkets.fn.send).

Both fields are required by the spec, so a response without `markets` or without
`next_cursor` fails to decode (with [`Error::Decode`](marcasite.md#enum.Error)) instead of
being mistaken for the last page. Every market of the page must decode: one market that
does not match the schema (for example an `outcome_prices` entry that is not a decimal
number) fails the whole page.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="ComboMarketsPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[ComboMarket]
```

The markets on this page, by volume descending.

##### <a id="ComboMarketsPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<ComboMarket>
```

The markets on this page, by volume descending, by value.

##### <a id="ComboMarketsPage.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor for the next page, to pass to [`ListComboMarkets::cursor`](combos.md#ListComboMarkets.fn.cursor); `None` on the
final page (a `null` or empty `next_cursor`).

### <a id="struct.CombosClient"></a>`struct CombosClient`

```rust
pub struct CombosClient { /* private fields */ }
```

Client for the Combos / RFQ REST API (`https://combos-rfq-api.polymarket.com`).

Covers the public endpoints: combo-eligible markets. Cheap to clone: clones share one connection pool.

```rust
use marcasite::combos::CombosClient;

let client = CombosClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="CombosClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://combos-rfq-api.polymarket.com";
```

The production base URL.

##### <a id="CombosClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="CombosClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> CombosClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="CombosClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="CombosClient.fn.list_combo_markets"></a>`list_combo_markets`

```rust
pub fn list_combo_markets(&self) -> ListComboMarkets
```

Lists active markets that can be used as combo legs, ordered by volume descending
(`GET /v1/rfq/combo-markets`, cursor-paginated). Public; no authentication needed.

Use [`ListComboMarkets::send`](combos.md#ListComboMarkets.fn.send) for one page or [`ListComboMarkets::into_stream`](combos.md#ListComboMarkets.fn.into_stream) to
walk the whole catalog.

See <https://docs.polymarket.com/api-reference/combo-markets/get-combo-markets>.

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};

let combos = marcasite::combos::CombosClient::new()?;
let top: Vec<_> = combos
    .list_combo_markets()
    .limit(100)
    .into_stream()
    .take(250)
    .try_collect()
    .await?;
for market in &top {
    println!("{}: YES {:?}", market.title, market.yes_price());
}
```

### <a id="struct.CombosClientBuilder"></a>`struct CombosClientBuilder`

```rust
#[must_use]
pub struct CombosClientBuilder { /* private fields */ }
```

Builder for [`CombosClient`](combos.md#struct.CombosClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="CombosClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`CombosClient::DEFAULT_BASE_URL`](combos.md#CombosClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="CombosClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="CombosClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<CombosClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.ListComboMarkets"></a>`struct ListComboMarkets`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListComboMarkets { /* private fields */ }
```

Request builder for [`CombosClient::list_combo_markets`](combos.md#CombosClient.fn.list_combo_markets).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListComboMarkets.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Number of markets per page, `1..=10000`.

The spec documents a default of `50` and a maximum of `100`; the live API returns
`1000` markets per page when no limit is set and accepts limits up to `10000`
(`0`, values above `10000` and non-numeric values get `400`). The SDK allows
`1..=10000`. See
`SPEC_DEVIATIONS.md`.

##### <a id="ListComboMarkets.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Continue from the `next_cursor` of a previous page.

##### <a id="ListComboMarkets.fn.exclude"></a>`exclude`

```rust
pub fn exclude<I>(self, condition_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<ConditionId>,
```

Condition ids of markets to omit, such as markets already shown (sent as a
comma-separated `exclude` list). Replaces any previously set list.

##### <a id="ListComboMarkets.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<ComboMarketsPage>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if the limit is outside `1..=10000`
  (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for parameters the server
  rejects.
- [`Error::Decode`](marcasite.md#enum.Error) if the response does not match the
  documented schema, including a missing `markets` or `next_cursor` (see
  [`ComboMarketsPage`](combos.md#struct.ComboMarketsPage)).
- Any other [`Error`](marcasite.md#enum.Error) for transport, server or rate limiting
  failures.

##### <a id="ListComboMarkets.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<ComboMarket>
```

Streams every market from the configured cursor (or the start of the catalog)
onwards, following `next_cursor` until it is `null` (or empty).

The stream yields the first error (any error of [`send`](combos.md#ListComboMarkets.fn.send)) and then
ends.

### <a id="struct.PositionId"></a>`struct PositionId`

```rust
pub struct PositionId(/* private fields */);
```

A combo position id: a `uint256` as a decimal string. Used as the order's `tokenId`
when quoting the YES or NO side of a combo.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&PositionId>`, `From<&String>`, `From<&str>`, `From<PositionId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="PositionId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="PositionId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="PositionId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.
