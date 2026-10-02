# Module `marcasite::relayer`

> Generated from marcasite 0.1.0 (all features) by `just docs-md`. Do not edit.

Relayer API client (`https://relayer-v2.polymarket.com`).

The Relayer submits and tracks gasless transactions for Polymarket wallets. This module
covers its public (unauthenticated) endpoints. Start from [`RelayerClient`](relayer.md#struct.RelayerClient).

| Endpoint | Method |
|---|---|
| `GET /transaction` | [`RelayerClient::get_transaction`](relayer.md#RelayerClient.fn.get_transaction) |
| `GET /nonce` | [`RelayerClient::get_nonce`](relayer.md#RelayerClient.fn.get_nonce) |
| `GET /relay-payload` | [`RelayerClient::get_relay_payload`](relayer.md#RelayerClient.fn.get_relay_payload) |
| `GET /deployed` | [`RelayerClient::check_deployed`](relayer.md#RelayerClient.fn.check_deployed) |

Endpoints that require Builder API key or Relayer API key authentication
(`POST /submit`, `GET /transactions`, `GET /relayer/api/keys`) are not supported yet.

Addresses passed to these endpoints are checked client-side against the spec's
`Address` pattern (`0x` followed by 40 hex digits) and rejected with
[`Error::Validation`](marcasite.md#enum.Error) before any request is sent.

See <https://docs.polymarket.com/api-reference/relayer/get-a-transaction-by-id>.

## Index

- **Structs:** [`CheckDeployed`](#struct.CheckDeployed), [`DeploymentStatus`](#struct.DeploymentStatus), [`Nonce`](#struct.Nonce), [`RelayPayload`](#struct.RelayPayload), [`RelayerClient`](#struct.RelayerClient), [`RelayerClientBuilder`](#struct.RelayerClientBuilder), [`RelayerTransaction`](#struct.RelayerTransaction), [`TransactionHash`](#struct.TransactionHash), [`TransactionId`](#struct.TransactionId)
- **Enums:** [`NonceType`](#enum.NonceType), [`TransactionState`](#enum.TransactionState), [`TransactionType`](#enum.TransactionType), [`WalletType`](#enum.WalletType)

## Structs

### <a id="struct.CheckDeployed"></a>`struct CheckDeployed`

```rust
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CheckDeployed { /* private fields */ }
```

Request builder for [`RelayerClient::check_deployed`](relayer.md#RelayerClient.fn.check_deployed).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="CheckDeployed.fn.wallet_type"></a>`wallet_type`

```rust
pub fn wallet_type(self, wallet_type: WalletType) -> Self
```

The wallet type to check (the `type` query parameter). The server defaults to
[`WalletType::Safe`](relayer.md#enum.WalletType) when it is not sent.

##### <a id="CheckDeployed.fn.send"></a>`send`

```rust
pub async fn send(self) -> Result<DeploymentStatus>
```

Sends the request.

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if the address is not `0x`
  followed by 40 hex digits (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for an address the server
  rejects.
- Any other [`Error`](marcasite.md#enum.Error) for transport, server or decoding failures.

### <a id="struct.DeploymentStatus"></a>`struct DeploymentStatus`

```rust
#[non_exhaustive]
pub struct DeploymentStatus {
    /// Whether the wallet is deployed.
    pub deployed: Option<bool>,
}
```

Whether a wallet is deployed onchain (`components/schemas/DeployedResponse`), returned by
[`CheckDeployed::send`](relayer.md#CheckDeployed.fn.send).

The field is optional because the spec does not mark it as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.Nonce"></a>`struct Nonce`

```rust
#[non_exhaustive]
pub struct Nonce {
    /// Current nonce value, as a decimal string (e.g. `"31"`). Kept as the documented
    /// string so no precision is lost.
    pub nonce: Option<String>,
}
```

The current nonce for a user (`components/schemas/NonceResponse`), returned by
[`RelayerClient::get_nonce`](relayer.md#RelayerClient.fn.get_nonce).

The field is optional because the spec does not mark it as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RelayPayload"></a>`struct RelayPayload`

```rust
#[non_exhaustive]
pub struct RelayPayload {
    /// Relayer address.
    pub address: Option<Address>,
    /// Current nonce value, as a decimal string (e.g. `"31"`).
    pub nonce: Option<String>,
}
```

The relayer address and nonce for a user (`components/schemas/RelayPayloadResponse`),
returned by [`RelayerClient::get_relay_payload`](relayer.md#RelayerClient.fn.get_relay_payload).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.RelayerClient"></a>`struct RelayerClient`

```rust
pub struct RelayerClient { /* private fields */ }
```

Client for the Relayer API (`https://relayer-v2.polymarket.com`).

Covers the public endpoints: transaction status, nonces, relay payloads and wallet deployment checks. Cheap to clone: clones share one connection pool.

```rust
use marcasite::relayer::RelayerClient;

let client = RelayerClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="RelayerClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://relayer-v2.polymarket.com";
```

The production base URL.

##### <a id="RelayerClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="RelayerClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> RelayerClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="RelayerClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="RelayerClient.fn.get_nonce"></a>`get_nonce`

```rust
pub async fn get_nonce(&self, address: impl Into<Address>, nonce_type: NonceType) -> Result<Nonce>
```

Gets the current Proxy or Safe nonce for a user (`GET /nonce`).

`address` is the user's **signer** address. The address is validated client-side
(the live server accepts a malformed address with `200` where the spec says `400`;
see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/relayer/get-current-nonce-for-a-user>.

```rust
use marcasite::relayer::{NonceType, RelayerClient};

let relayer = RelayerClient::new()?;
let nonce = relayer
    .get_nonce("0x77837466dd64fb52ECD00C737F060d0ff5CCB575", NonceType::Proxy)
    .await?;
println!("{:?}", nonce.nonce);
```

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if `address` is not `0x` followed
  by 40 hex digits (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for an address or type the
  server rejects (e.g. an [`NonceType::Unknown`](relayer.md#enum.NonceType) value).
- Any other [`Error`](marcasite.md#enum.Error) for transport, server or decoding failures.

##### <a id="RelayerClient.fn.get_relay_payload"></a>`get_relay_payload`

```rust
pub async fn get_relay_payload(&self, address: impl Into<Address>, nonce_type: NonceType) -> Result<RelayPayload>
```

Gets the relayer address and the current nonce for a user (`GET /relay-payload`).

`address` is the user's **signer** address. The address is validated client-side
(the live server accepts a malformed address with `200`; see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/relayer/get-relayer-address-and-nonce>.

```rust
use marcasite::relayer::{NonceType, RelayerClient};

let relayer = RelayerClient::new()?;
let payload = relayer
    .get_relay_payload("0x77837466dd64fb52ECD00C737F060d0ff5CCB575", NonceType::Safe)
    .await?;
println!("relayer {:?}, nonce {:?}", payload.address, payload.nonce);
```

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if `address` is not `0x` followed
  by 40 hex digits (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for an address or type the
  server rejects.
- Any other [`Error`](marcasite.md#enum.Error) for transport, server or decoding failures.

##### <a id="RelayerClient.fn.get_transaction"></a>`get_transaction`

```rust
pub async fn get_transaction(&self, id: impl Into<TransactionId>) -> Result<Vec<RelayerTransaction>>
```

Gets a transaction submitted to the Relayer by its id (`GET /transaction?id=...`).

Poll this with the `transactionID` returned by `POST /submit` to retrieve the
onchain [`transaction_hash`](relayer.md#struct.RelayerTransaction) once the
transaction has been broadcast. As documented, the API answers with an **array** of
transactions (the documented example holds exactly one).

See <https://docs.polymarket.com/api-reference/relayer/get-a-transaction-by-id>.

```rust
use marcasite::relayer::{RelayerClient, TransactionState};

let relayer = RelayerClient::new()?;
let txs = relayer
    .get_transaction("0190b317-a1d3-7bec-9b91-eeb6dcd3a620")
    .await?;
for tx in txs {
    if tx.state == Some(TransactionState::Confirmed) {
        println!("mined as {:?}", tx.transaction_hash);
    }
}
```

###### Errors

- [`Error::Validation`](marcasite.md#enum.Error) if `id` is empty (nothing is sent).
- [`Error::Api`](marcasite.md#enum.Error) with status `400` for a missing or invalid id, or
  `404` if no transaction has this id (see
  [`Error::is_not_found`](marcasite.md#Error.fn.is_not_found)).
- Any other [`Error`](marcasite.md#enum.Error) for transport, server or decoding failures.

##### <a id="RelayerClient.fn.check_deployed"></a>`check_deployed`

```rust
pub fn check_deployed(&self, address: impl Into<Address>) -> CheckDeployed
```

Checks whether the wallet at `address` is deployed onchain (`GET /deployed`).

Without [`CheckDeployed::wallet_type`](relayer.md#CheckDeployed.fn.wallet_type) the server checks a Gnosis Safe
([`WalletType::Safe`](relayer.md#enum.WalletType)). Errors are reported by [`CheckDeployed::send`](relayer.md#CheckDeployed.fn.send). The address
is validated client-side (the live server accepts a malformed address with `200`
where the spec says `400`; see `SPEC_DEVIATIONS.md`).

See <https://docs.polymarket.com/api-reference/relayer/check-if-a-wallet-is-deployed>.

```rust
use marcasite::relayer::{RelayerClient, WalletType};

let relayer = RelayerClient::new()?;
let status = relayer
    .check_deployed("0x6d8c4e9aDF5748Af82Dabe2C6225207770d6B4fa")
    .wallet_type(WalletType::Wallet)
    .send()
    .await?;
println!("deployed: {:?}", status.deployed);
```

### <a id="struct.RelayerClientBuilder"></a>`struct RelayerClientBuilder`

```rust
#[must_use]
pub struct RelayerClientBuilder { /* private fields */ }
```

Builder for [`RelayerClient`](relayer.md#struct.RelayerClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="RelayerClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`RelayerClient::DEFAULT_BASE_URL`](relayer.md#RelayerClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="RelayerClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="RelayerClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<RelayerClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.RelayerTransaction"></a>`struct RelayerTransaction`

```rust
#[non_exhaustive]
pub struct RelayerTransaction {
    /// Unique identifier of the transaction (wire name `transactionID`).
    pub transaction_id: Option<TransactionId>,
    /// Onchain transaction hash, available once the transaction has been broadcast.
    pub transaction_hash: Option<TransactionHash>,
    /// Signer address.
    pub from: Option<Address>,
    /// Target contract address.
    pub to: Option<Address>,
    /// The user's Polymarket proxy wallet address.
    pub proxy_address: Option<Address>,
    /// Encoded transaction data (`0x`-prefixed hex string).
    pub data: Option<String>,
    /// Transaction nonce, as a decimal string (e.g. `"60"`).
    pub nonce: Option<String>,
    /// Transaction value, as sent by the API (the documented example is `""`).
    pub value: Option<String>,
    /// Transaction signature (`0x`-prefixed hex string).
    pub signature: Option<String>,
    /// Current state of the transaction.
    pub state: Option<TransactionState>,
    /// Transaction type (wire name `type`).
    pub transaction_type: Option<TransactionType>,
    /// Owner address.
    pub owner: Option<Address>,
    /// Transaction metadata, as sent by the API (the documented example is `""`).
    pub metadata: Option<String>,
    /// When the transaction was created.
    pub created_at: Option<DateTime<Utc>>,
    /// When the transaction was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}
```

A transaction submitted to the Relayer (`components/schemas/RelayerTransaction`).

Every field is optional because the spec marks none as required.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.TransactionHash"></a>`struct TransactionHash`

```rust
pub struct TransactionHash(/* private fields */);
```

An onchain transaction hash (`0x`-prefixed hex), e.g.
`"0x38cbfbeae8fffa4e2b187ee5978d3ee9cafc53af0363ed90a35b7ea9016535d8"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TransactionHash>`, `From<&str>`, `From<String>`, `From<TransactionHash>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TransactionHash.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TransactionHash.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TransactionHash.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.TransactionId"></a>`struct TransactionId`

```rust
pub struct TransactionId(/* private fields */);
```

A Relayer transaction id, as returned in `transactionID` by `POST /submit`, e.g.
`"0190b317-a1d3-7bec-9b91-eeb6dcd3a620"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TransactionId>`, `From<&str>`, `From<String>`, `From<TransactionId>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TransactionId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TransactionId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TransactionId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

## Enums

### <a id="enum.NonceType"></a>`enum NonceType`

```rust
#[non_exhaustive]
pub enum NonceType {
    /// `PROXY`: the user's Proxy wallet nonce.
    Proxy,
    /// `SAFE`: the user's Gnosis Safe nonce.
    Safe,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The type of nonce to retrieve (the `type` query parameter of `GET /nonce` and
`GET /relay-payload`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="NonceType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="NonceType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.TransactionState"></a>`enum TransactionState`

```rust
#[non_exhaustive]
pub enum TransactionState {
    /// `STATE_NEW`: the state a transaction has right after `POST /submit`.
    New,
    /// `STATE_EXECUTED`.
    Executed,
    /// `STATE_MINED`.
    Mined,
    /// `STATE_CONFIRMED`.
    Confirmed,
    /// `STATE_INVALID`.
    Invalid,
    /// `STATE_FAILED`.
    Failed,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The state of a Relayer transaction (`RelayerTransaction.state`).

The spec lists the values but does not describe them individually.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TransactionState.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="TransactionState.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.TransactionType"></a>`enum TransactionType`

```rust
#[non_exhaustive]
pub enum TransactionType {
    /// `SAFE`: a Gnosis Safe wallet transaction.
    Safe,
    /// `PROXY`: a Proxy wallet transaction.
    Proxy,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The type of a Relayer transaction (`RelayerTransaction.type`).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TransactionType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="TransactionType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.WalletType"></a>`enum WalletType`

```rust
#[non_exhaustive]
pub enum WalletType {
    /// `SAFE`: the user's Polymarket Gnosis Safe address (signature type `2`).
    Safe,
    /// `WALLET`: the user's Polymarket Deposit Wallet address (signature type `3`).
    Wallet,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The wallet type to check with [`RelayerClient::check_deployed`](relayer.md#RelayerClient.fn.check_deployed) (the `type` query
parameter of `GET /deployed`). The server defaults to [`WalletType::Safe`](relayer.md#enum.WalletType).

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="WalletType.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="WalletType.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.
