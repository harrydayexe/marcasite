# Module `marcasite::bridge`

> Generated from marcasite 0.1.1 (all features) by `just docs-md`. Do not edit.

Bridge API client (`https://bridge.polymarket.com`).

The Bridge API moves funds between other chains and a Polymarket wallet: it lists the
supported assets, quotes transfers, creates deposit and withdrawal addresses, and reports
the transfers seen at those addresses. Start from [`BridgeClient`](bridge.md#struct.BridgeClient).

| Endpoint | Method |
|---|---|
| `GET /supported-assets` | [`BridgeClient::get_supported_assets`](bridge.md#BridgeClient.fn.get_supported_assets) |
| `POST /quote` | [`BridgeClient::get_quote`](bridge.md#BridgeClient.fn.get_quote) |
| `POST /deposit` | [`BridgeClient::create_deposit_addresses`](bridge.md#BridgeClient.fn.create_deposit_addresses) |
| `POST /withdraw` | [`BridgeClient::create_withdrawal_addresses`](bridge.md#BridgeClient.fn.create_withdrawal_addresses) |
| `GET /status/{address}` | [`BridgeClient::list_transactions`](bridge.md#BridgeClient.fn.list_transactions) |

Every Bridge API endpoint is public; none requires authentication.

Token amounts are exchanged as integer strings in the token's **base units** (no
decimals, e.g. `"10000000"` for 10 USDC with 6 decimals) and are kept as `String`s so no
precision is lost. USD values, fees and percentages are `Decimal`s.

`POST /quote` is read-only and retried like a `GET` under the client's
[`RetryPolicy`](marcasite.md#struct.RetryPolicy); `POST /deposit` and `POST /withdraw` create addresses
and are never retried automatically.

See <https://docs.polymarket.com/api-reference/bridge/get-supported-assets>.

## Index

- **Structs:** [`BridgeAddresses`](#struct.BridgeAddresses), [`BridgeClient`](#struct.BridgeClient), [`BridgeClientBuilder`](#struct.BridgeClientBuilder), [`ChainAddresses`](#struct.ChainAddresses), [`ChainId`](#struct.ChainId), [`CreateDepositAddresses`](#struct.CreateDepositAddresses), [`CreateWithdrawalAddresses`](#struct.CreateWithdrawalAddresses), [`FeeBreakdown`](#struct.FeeBreakdown), [`ListTransactions`](#struct.ListTransactions), [`Quote`](#struct.Quote), [`QuoteId`](#struct.QuoteId), [`QuoteRequest`](#struct.QuoteRequest), [`SupportedAsset`](#struct.SupportedAsset), [`SupportedAssets`](#struct.SupportedAssets), [`Token`](#struct.Token), [`Transaction`](#struct.Transaction), [`TransactionStatusPage`](#struct.TransactionStatusPage), [`WithdrawalRequest`](#struct.WithdrawalRequest)
- **Enums:** [`TransactionStatus`](#enum.TransactionStatus)

## Structs

### <a id="struct.BridgeAddresses"></a>`struct BridgeAddresses`

```rust
#[non_exhaustive]
pub struct BridgeAddresses {
    /// The bridge addresses, one per blockchain network family.
    pub address: Option<ChainAddresses>,
    /// Additional information about the bridge addresses.
    pub note: Option<String>,
}
```

Bridge addresses created by `POST /deposit` or `POST /withdraw`
(`components/schemas/DepositResponse`).

Send funds to one of these addresses to bridge them. Pass an address to
[`BridgeClient::list_transactions`](bridge.md#BridgeClient.fn.list_transactions) to track the transfers it receives. Every field is
optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BridgeClient"></a>`struct BridgeClient`

```rust
pub struct BridgeClient { /* private fields */ }
```

Client for the Bridge API (`https://bridge.polymarket.com`).

Covers supported assets, quotes, deposit/withdrawal addresses and transfer status. Cheap to clone: clones share one connection pool.

```rust
use marcasite::bridge::BridgeClient;

let client = BridgeClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="BridgeClient.fn.create_deposit_addresses"></a>`create_deposit_addresses`

```rust
pub fn create_deposit_addresses(&self, address: impl Into<Address>) -> CreateDepositAddresses
```

Creates bridge addresses that credit deposits to the Polymarket wallet `address`
as pUSD (`POST /deposit`).

The request creates server-side state, so it is **never** retried automatically.
Errors are reported by [`CreateDepositAddresses::send`](bridge.md#CreateDepositAddresses.fn.send); read its "Retrying" section
before repeating a failed request yourself.

See <https://docs.polymarket.com/api-reference/bridge/create-bridge-addresses>.

```rust
let bridge = marcasite::bridge::BridgeClient::new()?;
let created = bridge
    .create_deposit_addresses("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
    .builder_code("0x00000000000000000000000000000000000000000000000000000000abcd1234")
    .send()
    .await?;
println!("send USDC on an EVM chain to {:?}", created.address.and_then(|a| a.evm));
```

##### <a id="BridgeClient.fn.create_withdrawal_addresses"></a>`create_withdrawal_addresses`

```rust
pub fn create_withdrawal_addresses(&self, request: WithdrawalRequest) -> CreateWithdrawalAddresses
```

Creates bridge addresses that withdraw funds from a Polymarket wallet to another
chain and token (`POST /withdraw`).

The request creates server-side state, so it is **never** retried automatically.
Errors (including fields of `request` that were not set) are reported by
[`CreateWithdrawalAddresses::send`](bridge.md#CreateWithdrawalAddresses.fn.send); read its "Retrying" section before repeating a
failed request yourself.

See <https://docs.polymarket.com/api-reference/bridge/create-withdrawal-addresses>.

```rust
use marcasite::bridge::{BridgeClient, WithdrawalRequest};

let bridge = BridgeClient::new()?;
let request = WithdrawalRequest::new()
    .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
    .to_chain_id("1")
    .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    .recipient_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
let created = bridge.create_withdrawal_addresses(request).send().await?;
println!("{:?}", created.note);
```

##### <a id="BridgeClient.fn.get_supported_assets"></a>`get_supported_assets`

```rust
pub async fn get_supported_assets(&self) -> Result<SupportedAssets>
```

Lists the chains and tokens the bridge supports, with minimum amounts
(`GET /supported-assets`).

See <https://docs.polymarket.com/api-reference/bridge/get-supported-assets>.

```rust
let bridge = marcasite::bridge::BridgeClient::new()?;
for asset in bridge.get_supported_assets().await?.assets() {
    println!("{:?} on {:?}", asset.token, asset.chain_name);
}
```

###### Errors

[`Error::Api`](marcasite.md#enum.Error) with status `500` on a server error, or any other
[`Error`](marcasite.md#enum.Error) for transport or decoding failures.

##### <a id="BridgeClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://bridge.polymarket.com";
```

The production base URL.

##### <a id="BridgeClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="BridgeClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> BridgeClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="BridgeClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="BridgeClient.fn.get_quote"></a>`get_quote`

```rust
pub async fn get_quote(&self, request: QuoteRequest) -> Result<Quote>
```

Gets a quote for a transfer (`POST /quote`).

The request only reads data, so it is retried like a `GET` under the client's
[`RetryPolicy`](marcasite.md#struct.RetryPolicy).

See <https://docs.polymarket.com/api-reference/bridge/get-a-quote>.

```rust
use marcasite::bridge::{BridgeClient, QuoteRequest};

let bridge = BridgeClient::new()?;
let request = QuoteRequest::new()
    .from_amount_base_unit("10000000")
    .from_chain_id("137")
    .from_token_address("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")
    .to_chain_id("137")
    .to_token_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
    .recipient_address("0x17eC161f126e82A8ba337f4022d574DBEaFef575");
let quote = bridge.get_quote(request).await?;
println!("receive {:?} base units", quote.est_to_token_base_unit);
```

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if a field of `request` was not
  set; its [`parameter`](marcasite.md#ValidationError.fn.parameter) is the field's wire name,
  e.g. `fromAmountBaseUnit` (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` if the server rejects a field
  (e.g. `"fromAmountBaseUnit is required"`), or `500` if no quote can be made
  (`"cannot get quote"`).
- Any other [`Error`](marcasite.md#enum.Error) for transport, rate limiting or decoding
  failures.

##### <a id="BridgeClient.fn.list_transactions"></a>`list_transactions`

```rust
pub fn list_transactions(&self, address: impl Into<String>) -> ListTransactions
```

Lists the deposits and withdrawals seen at a bridge address, newest first
(`GET /status/{address}`, cursor-paginated).

`address` is a bridge address from [`BridgeClient::create_deposit_addresses`](bridge.md#BridgeClient.fn.create_deposit_addresses) or
[`BridgeClient::create_withdrawal_addresses`](bridge.md#BridgeClient.fn.create_withdrawal_addresses); EVM, Solana, Tron and Bitcoin formats
are supported. Use [`ListTransactions::send`](bridge.md#ListTransactions.fn.send) for one page (repeat it without a
cursor to track recent activity) or [`ListTransactions::into_stream`](bridge.md#ListTransactions.fn.into_stream) to walk the
full history.

**A plain wallet address is not a bridge address.** The live API answers `500`
`{"error":"cannot get transaction status"}` (a server bug; the spec only documents
that body as a generic 500 example) for an address it does not know as a bridge
address, which surfaces as [`Error::Api`](marcasite.md#enum.Error) with status `500`. See
`SPEC_DEVIATIONS.md`.

See <https://docs.polymarket.com/api-reference/bridge/get-transaction-status>.

```rust
use futures_util::TryStreamExt as _;

let bridge = marcasite::bridge::BridgeClient::new()?;
let history: Vec<_> = bridge
    .list_transactions("EXoZue2avJae1d45B3fVw2unhkrtToSYQqHtHgfZ2cbE")
    .limit(100)
    .into_stream()
    .try_collect()
    .await?;
```

### <a id="struct.BridgeClientBuilder"></a>`struct BridgeClientBuilder`

```rust
#[must_use]
pub struct BridgeClientBuilder { /* private fields */ }
```

Builder for [`BridgeClient`](bridge.md#struct.BridgeClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="BridgeClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`BridgeClient::DEFAULT_BASE_URL`](bridge.md#BridgeClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="BridgeClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="BridgeClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<BridgeClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.ChainAddresses"></a>`struct ChainAddresses`

```rust
#[non_exhaustive]
pub struct ChainAddresses {
    /// EVM-compatible bridge address (Ethereum, Polygon, Arbitrum, Base, etc.).
    pub evm: Option<String>,
    /// Solana Virtual Machine bridge address.
    pub svm: Option<String>,
    /// Bitcoin bridge address.
    pub btc: Option<String>,
    /// Tron bridge address.
    pub tron: Option<String>,
}
```

Bridge addresses for different blockchain networks (`DepositResponse.address`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ChainId"></a>`struct ChainId`

```rust
pub struct ChainId(/* private fields */);
```

A chain id as the Bridge API sends it: a decimal string, e.g. `"1"` (Ethereum),
`"8453"` (Base) or `"1151111081099710"` (Solana).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&ChainId>`, `From<&String>`, `From<&str>`, `From<ChainId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="ChainId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="ChainId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="ChainId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.CreateDepositAddresses"></a>`struct CreateDepositAddresses`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CreateDepositAddresses { /* private fields */ }
```

Request builder for [`BridgeClient::create_deposit_addresses`](bridge.md#BridgeClient.fn.create_deposit_addresses).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="CreateDepositAddresses.fn.builder_code"></a>`builder_code`

```rust
pub fn builder_code(self, code: impl Into<String>) -> Self
```

Attributes the request to your integration with your builder code (a bytes32 hex
string, `0x` followed by 64 hex digits), sent as the `X-Builder-Code` header.

The header is optional; per the docs, omitting it still succeeds but the server
returns a `missing_builder_code` warning.

##### <a id="CreateDepositAddresses.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<BridgeAddresses>
```

Sends the request.

###### Retrying

This request is sent exactly once, whatever the client's
[`RetryPolicy`](marcasite.md#struct.RetryPolicy). If it fails with a timeout, a connection
error or a `5xx` status, the server may still have created the addresses: such a
failure does not say whether the request was processed, and the docs do not say
whether repeating it returns the same addresses. [`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) being `true`
only means that the failure is transient, not that repeating this request is safe;
decide that yourself before sending it again.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if the address is not `0x`
  followed by 40 hex digits, or the builder code is not `0x` followed by 64 hex
  digits (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for an address, body or
  builder code the server rejects, or `500` on a server error.
- Any other [`Error`](marcasite.md#enum.Error) for transport, rate limiting or decoding
  failures (see "Retrying" above).

### <a id="struct.CreateWithdrawalAddresses"></a>`struct CreateWithdrawalAddresses`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CreateWithdrawalAddresses { /* private fields */ }
```

Request builder for [`BridgeClient::create_withdrawal_addresses`](bridge.md#BridgeClient.fn.create_withdrawal_addresses).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="CreateWithdrawalAddresses.fn.builder_code"></a>`builder_code`

```rust
pub fn builder_code(self, code: impl Into<String>) -> Self
```

Attributes the request to your integration with your builder code (a bytes32 hex
string, `0x` followed by 64 hex digits), sent as the `X-Builder-Code` header.

The header is optional; per the docs, omitting it still succeeds but the server
returns a `missing_builder_code` warning.

##### <a id="CreateWithdrawalAddresses.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<BridgeAddresses>
```

Sends the request.

###### Retrying

This request is sent exactly once, whatever the client's
[`RetryPolicy`](marcasite.md#struct.RetryPolicy). If it fails with a timeout, a connection
error or a `5xx` status, the server may still have created the addresses: such a
failure does not say whether the request was processed, and the docs do not say
whether repeating it returns the same addresses. [`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) being `true`
only means that the failure is transient, not that repeating this request is safe;
decide that yourself before sending it again.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if a field of the
  [`WithdrawalRequest`](bridge.md#struct.WithdrawalRequest) was not set (the error's
  [`parameter`](marcasite.md#ValidationError.fn.parameter) is its wire name, e.g.
  `recipientAddr`), if its `address` is not `0x` followed by 40 hex digits, or if the
  builder code is not `0x` followed by 64 hex digits (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for invalid or missing
  parameters or a builder code the server rejects, or `500` on a server error.
- Any other [`Error`](marcasite.md#enum.Error) for transport, rate limiting or decoding
  failures (see "Retrying" above).

### <a id="struct.FeeBreakdown"></a>`struct FeeBreakdown`

```rust
#[non_exhaustive]
pub struct FeeBreakdown {
    /// Label of the app fee, e.g. `"Fun.xyz fee"`.
    pub app_fee_label: Option<String>,
    /// App fees as a percentage of the total amount sent.
    pub app_fee_percent: Option<Decimal>,
    /// App fees in USD.
    pub app_fee_usd: Option<Decimal>,
    /// Fill cost as a percentage of the total amount sent.
    pub fill_cost_percent: Option<Decimal>,
    /// Fill cost in USD.
    pub fill_cost_usd: Option<Decimal>,
    /// Gas fee in USD.
    pub gas_usd: Option<Decimal>,
    /// Maximum potential slippage, as a percentage.
    pub max_slippage: Option<Decimal>,
    /// Amount after factoring in slippage.
    pub min_received: Option<Decimal>,
    /// Swap impact as a percentage of the total amount sent.
    pub swap_impact: Option<Decimal>,
    /// Swap impact in USD.
    pub swap_impact_usd: Option<Decimal>,
    /// Total impact as a percentage of the total amount sent.
    pub total_impact: Option<Decimal>,
    /// Impact cost of the transaction (USD).
    pub total_impact_usd: Option<Decimal>,
}
```

Breakdown of the estimated fees of a [`Quote`](bridge.md#struct.Quote) (`components/schemas/FeeBreakdown`).

Percentages are as sent by the API and use a scale where `1` means 1% (observed live:
`swapImpact` `0.0226` next to `swapImpactUsd` `0.002261` on a roughly 10 USD transfer is
0.0226%); the spec does not say.
Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ListTransactions"></a>`struct ListTransactions`

```rust
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTransactions { /* private fields */ }
```

Request builder for [`BridgeClient::list_transactions`](bridge.md#BridgeClient.fn.list_transactions).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="ListTransactions.fn.limit"></a>`limit`

```rust
pub fn limit(self, limit: u32) -> Self
```

Maximum number of transfers per page, `1..=100` (server default `50`). A page may
hold fewer transfers than requested and still have a following page.

##### <a id="ListTransactions.fn.cursor"></a>`cursor`

```rust
pub fn cursor(self, cursor: impl Into<String>) -> Self
```

Continue from the `nextCursor` of a previous page (for the same address). Omit it to
request the first page. An invalid, tampered or cross-address cursor fails with
status `400`; restart without a cursor when that happens.

##### <a id="ListTransactions.fn.paginate"></a>`paginate`

```rust
pub fn paginate(self) -> Self
```

Sends the compatibility parameter `paginate=true`, forwarded upstream for existing
integrations.

Pagination applies whether or not it is sent; the docs advise new integrations to
omit it and rely on [`cursor`](bridge.md#ListTransactions.fn.cursor) and [`limit`](bridge.md#ListTransactions.fn.limit) alone.

##### <a id="ListTransactions.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<TransactionStatusPage>
```

Fetches one page.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if the address is empty (or `.` or
  `..`) or the limit is outside `1..=100` (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for an invalid address, limit
  or cursor (e.g. a stale cursor), or `500` on a server error (also what the live API returns for an address that is not a
  bridge address).
- [`Error::Decode`](marcasite.md#enum.Error) if the response does not match the
  documented schema, including a missing `transactions` or `nextCursor`.
- Any other [`Error`](marcasite.md#enum.Error) for transport or rate limiting failures.

##### <a id="ListTransactions.fn.into_stream"></a>`into_stream`

```rust
pub fn into_stream(self) -> Paginated<Transaction>
```

Streams every transfer from the configured cursor (or the newest) onwards, following
`nextCursor` until it is `null` (or empty).

The stream yields the first error (any error of [`send`](bridge.md#ListTransactions.fn.send)) and then
ends.

### <a id="struct.Quote"></a>`struct Quote`

```rust
#[non_exhaustive]
pub struct Quote {
    /// Estimated time to complete the checkout, in milliseconds.
    pub est_checkout_time_ms: Option<u64>,
    /// Breakdown of the estimated fees.
    pub est_fee_breakdown: Option<FeeBreakdown>,
    /// `estInputUsd`: the estimated value of the amount **sent** (the input), in USD.
    ///
    /// The spec's descriptions of `estInputUsd` and `estOutputUsd` are swapped; the live
    /// API (2026-10-02) sends the input value here (about the amount sent) and the lower,
    /// after-fees value in [`est_output_usd`](bridge.md#struct.Quote). See `SPEC_DEVIATIONS.md`.
    pub est_input_usd: Option<Decimal>,
    /// `estOutputUsd`: the estimated value of the amount **received** (the output), in USD.
    /// See [`est_input_usd`](bridge.md#struct.Quote) for the swapped spec descriptions.
    pub est_output_usd: Option<Decimal>,
    /// Estimated amount of the destination token received (`estToTokenBaseUnit`; the
    /// documented example is `"14491203"`).
    pub est_to_token_base_unit: Option<String>,
    /// Unique quote id of the request.
    pub quote_id: Option<QuoteId>,
}
```

A bridge quote (`components/schemas/QuoteResponse`), returned by
[`BridgeClient::get_quote`](bridge.md#BridgeClient.fn.get_quote).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.QuoteId"></a>`struct QuoteId`

```rust
pub struct QuoteId(/* private fields */);
```

A bridge quote id, e.g.
`"0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&QuoteId>`, `From<&String>`, `From<&str>`, `From<QuoteId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="QuoteId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="QuoteId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="QuoteId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.QuoteRequest"></a>`struct QuoteRequest`

```rust
#[must_use]
pub struct QuoteRequest { /* private fields */ }
```

The body of `POST /quote` (`components/schemas/QuoteRequest`), for
[`BridgeClient::get_quote`](bridge.md#BridgeClient.fn.get_quote).

The API requires every field. Each one has a named setter, and there is deliberately no
constructor with positional arguments, so that a source and a destination value (or a
token and a recipient address) cannot be swapped by accident. [`BridgeClient::get_quote`](bridge.md#BridgeClient.fn.get_quote)
reports a field that was not set as an [`Error::Validation`](marcasite.md#enum.Error) naming it (by its wire
name), before anything is sent.

```rust
use marcasite::bridge::QuoteRequest;

// The documented example: from Polygon to Polygon.
let request = QuoteRequest::new()
    .from_amount_base_unit("10000000")
    .from_chain_id("137")
    .from_token_address("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")
    .to_chain_id("137")
    .to_token_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
    .recipient_address("0x17eC161f126e82A8ba337f4022d574DBEaFef575");
```

**Implements:** `Clone`, `Debug`, `Default`, `Eq`, `PartialEq`

#### Methods

##### <a id="QuoteRequest.fn.new"></a>`new`

```rust
pub fn new() -> Self
```

An empty request; set every field with the setters.

##### <a id="QuoteRequest.fn.from_amount_base_unit"></a>`from_amount_base_unit`

```rust
pub fn from_amount_base_unit(self, amount: impl Into<String>) -> Self
```

Amount of tokens to send (`fromAmountBaseUnit`), in the source token's base units
(the documented example is `"10000000"`).

##### <a id="QuoteRequest.fn.from_chain_id"></a>`from_chain_id`

```rust
pub fn from_chain_id(self, chain_id: impl Into<ChainId>) -> Self
```

Source chain id (`fromChainId`, e.g. `"137"`).

##### <a id="QuoteRequest.fn.from_token_address"></a>`from_token_address`

```rust
pub fn from_token_address(self, address: impl Into<String>) -> Self
```

Source token address (`fromTokenAddress`).

##### <a id="QuoteRequest.fn.recipient_address"></a>`recipient_address`

```rust
pub fn recipient_address(self, address: impl Into<String>) -> Self
```

Address of the recipient (`recipientAddress`).

##### <a id="QuoteRequest.fn.to_chain_id"></a>`to_chain_id`

```rust
pub fn to_chain_id(self, chain_id: impl Into<ChainId>) -> Self
```

Destination chain id (`toChainId`, e.g. `"137"`).

##### <a id="QuoteRequest.fn.to_token_address"></a>`to_token_address`

```rust
pub fn to_token_address(self, address: impl Into<String>) -> Self
```

Destination token address (`toTokenAddress`).

### <a id="struct.SupportedAsset"></a>`struct SupportedAsset`

```rust
#[non_exhaustive]
pub struct SupportedAsset {
    /// Chain id.
    pub chain_id: Option<ChainId>,
    /// Human-readable chain name, e.g. `"Ethereum"`.
    pub chain_name: Option<String>,
    /// The token.
    pub token: Option<Token>,
    /// Minimum amount in USD for deposits and withdrawals (a JSON number on the wire).
    pub min_checkout_usd: Option<Decimal>,
}
```

A token the bridge supports on one chain (`components/schemas/SupportedAsset`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.SupportedAssets"></a>`struct SupportedAssets`

```rust
#[non_exhaustive]
pub struct SupportedAssets {
    /// Supported assets with the minimum amounts for deposits and withdrawals.
    pub supported_assets: Option<Vec<SupportedAsset>>,
    /// A human-readable remark, e.g. `"These are the currently supported chains and assets
    /// for deposits and withdrawals."`. Undocumented; observed live (2026-10-02).
    pub note: Option<String>,
}
```

The assets the bridge supports (`components/schemas/SupportedAssetsResponse`), returned
by [`BridgeClient::get_supported_assets`](bridge.md#BridgeClient.fn.get_supported_assets).

The field is optional because the spec does not mark it as required; use
[`SupportedAssets::assets`](bridge.md#SupportedAssets.fn.assets) for a slice that is empty when it is absent.

The live API also sends a top-level `note` string the spec does not document
(see [`note`](bridge.md#struct.SupportedAssets) and `SPEC_DEVIATIONS.md`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SupportedAssets.fn.assets"></a>`assets`

```rust
#[must_use]
pub fn assets(&self) -> &[SupportedAsset]
```

The supported assets, or an empty slice if the field was absent.

### <a id="struct.Token"></a>`struct Token`

```rust
#[non_exhaustive]
pub struct Token {
    /// Full token name, e.g. `"USD Coin"`.
    pub name: Option<String>,
    /// Token symbol, e.g. `"USDC"`.
    pub symbol: Option<String>,
    /// Token contract address, in the chain's own address format (not necessarily an EVM
    /// address).
    pub address: Option<String>,
    /// Token decimals: the number of decimal places between base units and whole tokens.
    pub decimals: Option<u32>,
}
```

A token on a supported chain (`components/schemas/Token`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Transaction"></a>`struct Transaction`

```rust
#[non_exhaustive]
pub struct Transaction {
    /// Source chain id.
    pub from_chain_id: Option<ChainId>,
    /// Source token contract address, in the source chain's address format.
    pub from_token_address: Option<String>,
    /// Amount sent, in the source token's base units (an integer string with no decimal
    /// point).
    pub from_amount_base_unit: Option<String>,
    /// Destination chain id.
    pub to_chain_id: Option<ChainId>,
    /// Destination token contract address.
    pub to_token_address: Option<String>,
    /// Current status of the transfer.
    pub status: Option<TransactionStatus>,
    /// Transaction hash, only available when the status is
    /// [`Completed`](bridge.md#enum.TransactionStatus). Its format depends on the chain.
    pub tx_hash: Option<String>,
    /// When the transfer was created (wire name `createdTimeMs`, Unix milliseconds).
    /// Missing while the status is [`DepositDetected`](bridge.md#enum.TransactionStatus).
    /// Always an integer live (never fractional).
    pub created_time: Option<DateTime<Utc>>,
}
```

A deposit or withdrawal seen at a bridge address (`components/schemas/Transaction`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TransactionStatusPage"></a>`struct TransactionStatusPage`

```rust
#[non_exhaustive]
pub struct TransactionStatusPage {
    /// One page of transfers, newest first. This is a page, not the full history.
    pub transactions: Vec<Transaction>,
    /// Opaque continuation token for the next page (wire name `nextCursor`), as sent:
    /// `None` when the walk is complete. Prefer [`next_cursor()`](bridge.md#TransactionStatusPage.fn.next_cursor),
    /// which also treats an empty string as the end. Pass it back unchanged with
    /// [`ListTransactions::cursor`](bridge.md#ListTransactions.fn.cursor), for the same address only. Stop on `None`, not on an
    /// empty or short page.
    pub next_cursor: Option<String>,
}
```

One page of transfers seen at a bridge address
(`components/schemas/TransactionStatusResponse`), returned by
[`ListTransactions::send`](bridge.md#ListTransactions.fn.send).

Both fields are required by the spec, so a response without `transactions` or without
`nextCursor` fails to decode (with [`Error::Decode`](marcasite.md#enum.Error)) instead of
being mistaken for the last page.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TransactionStatusPage.fn.items"></a>`items`

```rust
#[must_use]
pub fn items(&self) -> &[Transaction]
```

The transfers on this page, newest first.

##### <a id="TransactionStatusPage.fn.into_items"></a>`into_items`

```rust
#[must_use]
pub fn into_items(self) -> Vec<Transaction>
```

The transfers on this page, newest first, by value.

##### <a id="TransactionStatusPage.fn.next_cursor"></a>`next_cursor`

```rust
#[must_use]
pub fn next_cursor(&self) -> Option<&str>
```

The cursor for the next page, to pass to [`ListTransactions::cursor`](bridge.md#ListTransactions.fn.cursor); `None` on
the last page (a `null` or empty `nextCursor`).

### <a id="struct.WithdrawalRequest"></a>`struct WithdrawalRequest`

```rust
#[must_use]
pub struct WithdrawalRequest { /* private fields */ }
```

The body of `POST /withdraw` (`components/schemas/WithdrawalRequest`), for
[`BridgeClient::create_withdrawal_addresses`](bridge.md#BridgeClient.fn.create_withdrawal_addresses).

The API requires every field. Each one has a named setter, and there is deliberately no
constructor with positional arguments, so that the source wallet, the destination token
and the recipient (all addresses) cannot be swapped by accident.
[`CreateWithdrawalAddresses::send`](bridge.md#CreateWithdrawalAddresses.fn.send) reports a field that was not set as an
[`Error::Validation`](marcasite.md#enum.Error) naming it (by its wire name), before
anything is sent.

```rust
use marcasite::bridge::WithdrawalRequest;

// The documented example: withdraw to USDC on Ethereum.
let request = WithdrawalRequest::new()
    .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
    .to_chain_id("1")
    .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    .recipient_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
```

**Implements:** `Clone`, `Debug`, `Default`, `Eq`, `PartialEq`

#### Methods

##### <a id="WithdrawalRequest.fn.new"></a>`new`

```rust
pub fn new() -> Self
```

An empty request; set every field with the setters.

##### <a id="WithdrawalRequest.fn.address"></a>`address`

```rust
pub fn address(self, address: impl Into<Address>) -> Self
```

Source Polymarket wallet address on Polygon (`address`): `0x` followed by 40 hex
digits.

##### <a id="WithdrawalRequest.fn.to_chain_id"></a>`to_chain_id`

```rust
pub fn to_chain_id(self, chain_id: impl Into<ChainId>) -> Self
```

Destination chain id (`toChainId`, e.g. `"1"` for Ethereum, `"8453"` for Base,
`"1151111081099710"` for Solana).

##### <a id="WithdrawalRequest.fn.to_token_address"></a>`to_token_address`

```rust
pub fn to_token_address(self, address: impl Into<String>) -> Self
```

Destination token contract address (`toTokenAddress`).

##### <a id="WithdrawalRequest.fn.recipient_address"></a>`recipient_address`

```rust
pub fn recipient_address(self, address: impl Into<String>) -> Self
```

Destination wallet address where funds will be sent (wire name `recipientAddr`).

## Enums

### <a id="enum.TransactionStatus"></a>`enum TransactionStatus`

```rust
#[non_exhaustive]
pub enum TransactionStatus {
    /// `DEPOSIT_DETECTED`. Transfers in this status have no
    /// [`created_time`](bridge.md#struct.Transaction).
    DepositDetected,
    /// `PROCESSING`.
    Processing,
    /// `ORIGIN_TX_CONFIRMED`.
    OriginTxConfirmed,
    /// `SUBMITTED`.
    Submitted,
    /// `COMPLETED`. Only transfers in this status carry a
    /// [`tx_hash`](bridge.md#struct.Transaction).
    Completed,
    /// `FAILED`.
    Failed,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The status of a bridge transfer (`Transaction.status`).

If a transfer fails, remains stuck, or funds are held due to a compliance check, the
docs direct users to the Bridge API provider's support
(<https://intercom.help/funxyz/en/articles/10732578-contact-us>).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TransactionStatus.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="TransactionStatus.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.
