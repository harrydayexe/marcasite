# Crate `marcasite`

> Generated from marcasite 0.1.0 (all features) by `just docs-md`. Do not edit.

Unofficial Rust SDK for the Polymarket Predictions APIs.

This project is not affiliated with or endorsed by Polymarket.

marcasite gives typed, async access to the **public (unauthenticated)** Polymarket
Predictions APIs:

| Service | Client | Feature |
|---|---|---|
| Gamma API: events, markets, tags, series, comments, sports, search, profiles | [`gamma::GammaClient`](gamma.md#struct.GammaClient) | `gamma` |
| CLOB API: order books, prices, spreads, markets, price history, rewards, rebates, builder trades | [`clob::ClobClient`](clob.md#struct.ClobClient) | `clob` |
| Data API v2: positions, PnL, trades, activity, leaderboards, holders | [`data::DataClient`](data.md#struct.DataClient) | `data` |
| Relayer API: transaction status, nonces, wallet deployment | [`relayer::RelayerClient`](relayer.md#struct.RelayerClient) | `relayer` |
| Bridge API: supported assets, quotes, deposit/withdrawal addresses | [`bridge::BridgeClient`](bridge.md#struct.BridgeClient) | `bridge` |
| Combos / RFQ API: combo-eligible markets | [`combos::CombosClient`](combos.md#struct.CombosClient) | `combos` |
| WebSockets: market channel, sports results, PolyBolt public prices | [`ws`](ws.md) | `ws` |

All features are enabled by default; disable default features and pick the ones you need
to reduce compile time. [`ENDPOINTS.md`] lists every endpoint and its implementation status.

## Quick start

```rust
use marcasite::Polymarket;

let pm = Polymarket::new()?;
let tags = pm.gamma().list_tags().limit(5).send().await?;
for tag in tags {
    println!("{:?}: {:?}", tag.id, tag.label);
}
```

Each service client can also be created on its own, e.g. [`gamma::GammaClient::new`](gamma.md#GammaClient.fn.new), and
every client accepts a custom base URL (for example to point tests at a mock server).

## Requests

Endpoints with only required parameters are plain `async fn`s. Endpoints with optional
parameters return a request builder: chain setters, then call `.send().await`. Required
parameters are always arguments of the client method, never builder setters.

## Pagination

Paginated endpoints return one page from `.send()` (including the cursor or offset needed
for the next page), and every item of every page from `.into_stream()`, as a
[`Paginated`](marcasite.md#struct.Paginated) `Stream` that fetches the pages lazily. It ends after the last page, or
right after yielding the first error. [`Paginated`](marcasite.md#struct.Paginated) is `Unpin`, so a plain `while let`
loop works:

```rust
use futures_util::StreamExt as _;

let pm = marcasite::Polymarket::new()?;
let mut tags = pm.gamma().list_tags().limit(100).into_stream();
while let Some(tag) = tags.next().await {
    println!("{:?}", tag?.label);
}
```

The usual stream adapters apply, for example to collect a bounded number of items:

```rust
use futures_util::{StreamExt as _, TryStreamExt as _};

let pm = marcasite::Polymarket::new()?;
let first_200: Vec<_> = pm
    .gamma()
    .list_tags()
    .limit(100)
    .into_stream()
    .take(200)
    .try_collect()
    .await?;
```

## Errors

Every fallible operation returns [`Result<T>`](marcasite.md#type.Result), whose error type [`Error`](marcasite.md#enum.Error) distinguishes
API errors (with the parsed error body and trace id), rate limiting (with `Retry-After`),
timeouts, transport failures, decoding failures (with the JSON path of the bad field),
client-side validation failures and configuration errors. See [`Error`](marcasite.md#enum.Error).

## Types

Prices, sizes and amounts are `Decimal`s, never floating point. Values the API sends as
JSON strings are parsed exactly. Values it sends as JSON numbers are exact if they are
integers (within `i64`/`u64`) or have at most 15 significant digits; longer numbers pass
through `f64` while being parsed and may be rounded (see `Decimal`). Timestamps are
`chrono::DateTime<Utc>`. Identifiers are newtypes (see [`types`](types.md)) so
that, for example, a token id cannot be passed where a condition id is expected. String
enums are `#[non_exhaustive]` and have an `Unknown(String)` variant, so values added by the
server later never break deserialization.

## Dependencies in the public API

Types from these crates appear in marcasite's public API, so upgrading any of them to a
new major (or, before 1.0, minor) version is a breaking change of marcasite:
`http` 1 (`StatusCode`, `Method`), `url` 2 (`Url`), `serde_json` 1
(`serde_json`), `chrono` 0.4 (`chrono`), `rust_decimal` 1 (`Decimal`) and
`futures-core` 0.3 (`Stream`). The types are re-exported, so you do not need to
depend on these crates yourself to name them:

```rust
use marcasite::{Method, Url, chrono::Utc, serde_json::Value};

let _ = (Method::GET, Url::parse("https://example.com"), Value::Null, Utc::now());
```

## Logging

This crate emits diagnostics via `tracing`. It never installs a subscriber; install one
in your application (e.g. with `tracing-subscriber`) to see them. Requests, retries and
failures are logged at `DEBUG`, response bodies and WebSocket frames at `TRACE`. Errors
that are returned to you are never logged above `DEBUG`, so they are not reported twice.

Spans (all at `DEBUG`), useful with span-based subscribers such as OpenTelemetry exporters:

- `marcasite.request`: one per HTTP request, covering all retries. Fields `service`,
  `method`, `path`, and on completion `status`, `attempts`, `elapsed_ms` and `trace_id`
  (when the server sends one).
- `marcasite.ws`: one per WebSocket connection, covering the handshake and the background
  task that drives the socket for its whole lifetime. Fields `service`, `host`, `path`.

Both are children of whatever span is current when the request is sent or the connection
is opened, so they nest under your application's own spans.

[`ENDPOINTS.md`]: https://github.com/harrydayexe/marcasite/blob/main/ENDPOINTS.md

## Index

- **Modules:** [`bridge`](bridge.md), [`clob`](clob.md), [`combos`](combos.md), [`data`](data.md), [`gamma`](gamma.md), [`relayer`](relayer.md), [`types`](types.md), [`ws`](ws.md)
- **Re-exports:** `Decimal`, `Method`, `StatusCode`, `Url`, `chrono`, `serde_json`
- **Structs:** [`ApiError`](#struct.ApiError), [`ConfigError`](#struct.ConfigError), [`DecodeError`](#struct.DecodeError), [`HttpClient`](#struct.HttpClient), [`HttpClientBuilder`](#struct.HttpClientBuilder), [`Paginated`](#struct.Paginated), [`Polymarket`](#struct.Polymarket), [`PolymarketBuilder`](#struct.PolymarketBuilder), [`RetryPolicy`](#struct.RetryPolicy), [`TransportError`](#struct.TransportError), [`ValidationError`](#struct.ValidationError), [`WebSocketError`](#struct.WebSocketError)
- **Enums:** [`Error`](#enum.Error), [`Service`](#enum.Service), [`WebSocketErrorKind`](#enum.WebSocketErrorKind)
- **Type aliases:** [`Result`](#type.Result)

## Modules

- [`bridge`](bridge.md): Bridge API client (`https://bridge.polymarket.com`).
- [`clob`](clob.md): CLOB API client (`https://clob.polymarket.com`), public (unauthenticated) endpoints.
- [`combos`](combos.md): Combos / RFQ REST API client (`https://combos-rfq-api.polymarket.com`).
- [`data`](data.md): Data API v2 client (`https://data-api.polymarket.com`, routes under `/v2`).
- [`gamma`](gamma.md): Gamma API client (`https://gamma-api.polymarket.com`).
- [`relayer`](relayer.md): Relayer API client (`https://relayer-v2.polymarket.com`).
- [`types`](types.md): Identifier newtypes and enums shared by several services.
- [`ws`](ws.md): Public (unauthenticated) WebSocket channels.

## Re-exports

- `Decimal`: `pub use rust_decimal::Decimal;` from the [`rust_decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/) crate. Re-export of `rust_decimal::Decimal`, used for all prices, sizes and amounts.
- `Method`: `pub use marcasite_core::Method;` from the [`http`](https://docs.rs/http/latest/http/) crate.
- `StatusCode`: `pub use marcasite_core::StatusCode;` from the [`http`](https://docs.rs/http/latest/http/) crate.
- `Url`: `pub use marcasite_core::Url;` from the [`url`](https://docs.rs/url/latest/url/) crate.
- `chrono`: `pub use chrono;` from the [`chrono`](https://docs.rs/chrono/latest/chrono/) crate. Re-export of the `chrono` crate, whose `DateTime<Utc>` is used for timestamps. Its `now` feature is enabled, so `marcasite::chrono::Utc::now()` works.
- `serde_json`: `pub use serde_json;` from the [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) crate. Re-export of the `serde_json` crate, whose `Value` appears in the public API (e.g. the `Unknown` variants of the WebSocket event enums).

## Structs

### <a id="struct.ApiError"></a>`struct ApiError`

```rust
pub struct ApiError { /* private fields */ }
```

A non-success HTTP response from a Polymarket API.

Every service documents a JSON error body with an `error` message; some add more
fields (Data API v2: `code`, `retryable`, `trace_id`, `parameter`; Gamma: `type`;
CLOB: `code`, `retry_after_seconds`). All documented fields are parsed when present and
exposed through the accessors below; the raw body is kept (truncated) for anything else.

**Implements:** `Clone`, `Debug`, `Display`, `Error`

#### Methods

##### <a id="ApiError.fn.service"></a>`service`

```rust
#[must_use]
pub fn service(&self) -> Service
```

The service that returned the error.

##### <a id="ApiError.fn.method"></a>`method`

```rust
#[must_use]
pub fn method(&self) -> &Method
```

The HTTP method of the failing request.

##### <a id="ApiError.fn.url"></a>`url`

```rust
#[must_use]
pub fn url(&self) -> &str
```

The full URL of the failing request, including the query string.

##### <a id="ApiError.fn.status"></a>`status`

```rust
#[must_use]
pub fn status(&self) -> StatusCode
```

The HTTP status code.

##### <a id="ApiError.fn.message"></a>`message`

```rust
#[must_use]
pub fn message(&self) -> Option<&str>
```

The human-readable error message (`error` field of the body), when present.

##### <a id="ApiError.fn.code"></a>`code`

```rust
#[must_use]
pub fn code(&self) -> Option<&str>
```

The machine-readable error code (`code` field), when the service provides one.

For example Data API v2 returns codes such as `invalid_request` or `rate_limited`.

##### <a id="ApiError.fn.error_type"></a>`error_type`

```rust
#[must_use]
pub fn error_type(&self) -> Option<&str>
```

The error classification (`type` field), when the service provides one (Gamma).

##### <a id="ApiError.fn.retryable"></a>`retryable`

```rust
#[must_use]
pub fn retryable(&self) -> Option<bool>
```

Whether the server marked the failure as retryable (`retryable` field), when
provided. When present, it overrides the status-based rule of
[`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) and of the automatic retries.

##### <a id="ApiError.fn.parameter"></a>`parameter`

```rust
#[must_use]
pub fn parameter(&self) -> Option<&str>
```

The name of the offending request parameter (`parameter` field), when provided.

##### <a id="ApiError.fn.trace_id"></a>`trace_id`

```rust
#[must_use]
pub fn trace_id(&self) -> Option<&str>
```

The server-side trace id (`x-trace-id` header or `trace_id` body field).

##### <a id="ApiError.fn.retry_after"></a>`retry_after`

```rust
#[must_use]
pub fn retry_after(&self) -> Option<Duration>
```

How long to wait before retrying (`Retry-After` header, in seconds or as an HTTP
date, or the CLOB `retry_after_seconds` body field), when provided.

##### <a id="ApiError.fn.body"></a>`body`

```rust
#[must_use]
pub fn body(&self) -> &str
```

The raw response body, truncated to a few KiB.

### <a id="struct.ConfigError"></a>`struct ConfigError`

```rust
pub struct ConfigError { /* private fields */ }
```

The client configuration is invalid (e.g. a base URL that cannot be used).

**Implements:** `Debug`, `Display`, `Error`, `From<ConfigError>`

#### Methods

##### <a id="ConfigError.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(message: impl Into<Cow<'static, str>>) -> Self
```

Creates a configuration error with a message.

##### <a id="ConfigError.fn.with_source"></a>`with_source`

```rust
#[must_use]
pub fn with_source(message: impl Into<Cow<'static, str>>, source: impl Error + Send + Sync + 'static) -> Self
```

Creates a configuration error with a message and an underlying cause.

##### <a id="ConfigError.fn.message"></a>`message`

```rust
#[must_use]
pub fn message(&self) -> &str
```

The error message.

### <a id="struct.DecodeError"></a>`struct DecodeError`

```rust
pub struct DecodeError { /* private fields */ }
```

A successful response whose body could not be deserialized into the expected type.

This usually means the API changed shape or returned a value the documentation does not
describe. [`DecodeError::path`](marcasite.md#DecodeError.fn.path) points at the offending field and
[`DecodeError::body_snippet`](marcasite.md#DecodeError.fn.body_snippet) shows the surrounding JSON. `Display` includes the
deserializer's message; there is no further `source`.

**Implements:** `Debug`, `Display`, `Error`

#### Methods

##### <a id="DecodeError.fn.service"></a>`service`

```rust
#[must_use]
pub fn service(&self) -> Service
```

The service that returned the response.

##### <a id="DecodeError.fn.method"></a>`method`

```rust
#[must_use]
pub fn method(&self) -> &Method
```

The HTTP method of the request.

##### <a id="DecodeError.fn.url"></a>`url`

```rust
#[must_use]
pub fn url(&self) -> &str
```

The full URL of the request.

##### <a id="DecodeError.fn.status"></a>`status`

```rust
#[must_use]
pub fn status(&self) -> StatusCode
```

The HTTP status of the response.

##### <a id="DecodeError.fn.path"></a>`path`

```rust
#[must_use]
pub fn path(&self) -> &str
```

The path of the field that failed to decode, e.g. `data[3].outcome_index`
(`.` when the failure is at the root).

##### <a id="DecodeError.fn.trace_id"></a>`trace_id`

```rust
#[must_use]
pub fn trace_id(&self) -> Option<&str>
```

The server-side trace id, when provided.

##### <a id="DecodeError.fn.body_snippet"></a>`body_snippet`

```rust
#[must_use]
pub fn body_snippet(&self) -> &str
```

A short excerpt of the response body around the failure position.

### <a id="struct.HttpClient"></a>`struct HttpClient`

```rust
pub struct HttpClient { /* private fields */ }
```

A connection-pooled HTTP client shared by all service clients.

Cloning is cheap (the connection pool is reference counted), so one `HttpClient` can
back every service client in an application.

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="HttpClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default settings.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the TLS backend cannot be
initialised.

##### <a id="HttpClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> HttpClientBuilder
```

Returns a builder for customising timeouts, the user agent and retries.

##### <a id="HttpClient.fn.retry_policy"></a>`retry_policy`

```rust
#[must_use]
pub fn retry_policy(&self) -> RetryPolicy
```

The retry policy in effect.

##### <a id="HttpClient.fn.max_response_size"></a>`max_response_size`

```rust
#[must_use]
pub fn max_response_size(&self) -> usize
```

The largest response body accepted, in bytes (see
[`HttpClientBuilder::max_response_size`](marcasite.md#HttpClientBuilder.fn.max_response_size)).

### <a id="struct.HttpClientBuilder"></a>`struct HttpClientBuilder`

```rust
#[must_use]
pub struct HttpClientBuilder { /* private fields */ }
```

Builder for [`HttpClient`](marcasite.md#struct.HttpClient).

```rust
use std::time::Duration;
use marcasite_core::{HttpClient, RetryPolicy};

let http = HttpClient::builder()
    .timeout(Duration::from_secs(10))
    .user_agent("my-app/1.0")
    .retry_policy(RetryPolicy::new(2))
    .build()?;
```

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="HttpClientBuilder.fn.timeout"></a>`timeout`

```rust
pub fn timeout(self, timeout: Duration) -> Self
```

Sets the overall per-request timeout (default 30 s).

##### <a id="HttpClientBuilder.fn.no_timeout"></a>`no_timeout`

```rust
pub fn no_timeout(self) -> Self
```

Disables the overall per-request timeout.

##### <a id="HttpClientBuilder.fn.connect_timeout"></a>`connect_timeout`

```rust
pub fn connect_timeout(self, timeout: Duration) -> Self
```

Sets the connection timeout (default 10 s).

##### <a id="HttpClientBuilder.fn.user_agent"></a>`user_agent`

```rust
pub fn user_agent(self, user_agent: impl Into<String>) -> Self
```

Sets the `User-Agent` header (default `marcasite/<version>`).

##### <a id="HttpClientBuilder.fn.retry_policy"></a>`retry_policy`

```rust
pub fn retry_policy(self, retry: RetryPolicy) -> Self
```

Sets the automatic retry policy (default: no retries).

##### <a id="HttpClientBuilder.fn.max_response_size"></a>`max_response_size`

```rust
pub fn max_response_size(self, bytes: usize) -> Self
```

Sets the largest response body accepted, in bytes (default
`DEFAULT_MAX_RESPONSE_SIZE`, 64 MiB), so a misbehaving server or proxy cannot
exhaust memory.

Bodies are read in chunks and never held beyond the limit. A successful response
whose body is larger fails with [`Error::Transport`](marcasite.md#enum.Error) (whose
`source` says the body was too large); it is not
retried. The body of an error response is only read up to 64 KiB (or the limit, if
smaller) and the error keeps its HTTP status.

##### <a id="HttpClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<HttpClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the user agent is not a valid
header value or the TLS backend cannot be initialised.

### <a id="struct.Paginated"></a>`struct Paginated`

```rust
#[must_use = "streams do nothing unless polled"]
pub struct Paginated<T> { /* private fields */ }
```

The items of a paginated endpoint, fetched page by page as the stream is polled.

Every `into_stream()` returns this type. It yields `Ok(item)` for each item, in order,
and ends after the last page, or right after yielding the first error. It is
`Unpin`, so it can be polled directly in a `while let` loop, and it is a named type,
so it can be stored in a struct. It is `Send + Sync + 'static`, and it implements
`FusedStream`.

```rust
use futures_util::StreamExt as _;
use marcasite_core::pagination::{Paginated, offset_stream};

// A stand-in for a service's `into_stream()`.
let mut stream: Paginated<u64> = offset_stream(0, |offset| async move {
    Ok(if offset < 3 { vec![offset] } else { Vec::new() })
});
while let Some(item) = stream.next().await {
    println!("{}", item?);
}
```

**Implements:** `Debug`, `FusedStream`, `Stream<Item = Result<T, Error>>`

#### Methods

```rust
impl<T> Paginated<T>
```

##### <a id="Paginated.fn.new"></a>`new`

```rust
pub fn new<S>(stream: S) -> Self
where
    S: Stream<Item = Result<T>> + Send + 'static,
```

Wraps a stream of items.

### <a id="struct.Polymarket"></a>`struct Polymarket`

```rust
pub struct Polymarket { /* private fields */ }
```

One handle to every enabled Polymarket service, sharing a single HTTP connection pool.

Cheap to clone. Use [`Polymarket::new`](marcasite.md#Polymarket.fn.new) for the defaults or [`Polymarket::builder`](marcasite.md#Polymarket.fn.builder) to
configure timeouts, retries, the user agent or per-service base URLs.

```rust
use std::time::Duration;
use marcasite::{Polymarket, RetryPolicy};

let pm = Polymarket::builder()
    .timeout(Duration::from_secs(10))
    .retry_policy(RetryPolicy::new(3))
    .build()?;
let tag = pm.gamma().list_tags().limit(1).send().await?;
```

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="Polymarket.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client for every enabled service with default settings and base URLs.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="Polymarket.fn.builder"></a>`builder`

```rust
pub fn builder() -> PolymarketBuilder
```

Returns a builder for customising the shared HTTP client and base URLs.

##### <a id="Polymarket.fn.http_client"></a>`http_client`

```rust
#[must_use]
pub fn http_client(&self) -> &HttpClient
```

The shared HTTP client.

##### <a id="Polymarket.fn.gamma"></a>`gamma`

```rust
#[must_use]
pub fn gamma(&self) -> &GammaClient
```

The Gamma API client (events, markets, tags, series, comments, sports, search,
profiles).

##### <a id="Polymarket.fn.clob"></a>`clob`

```rust
#[must_use]
pub fn clob(&self) -> &ClobClient
```

The CLOB API client (public market data).

##### <a id="Polymarket.fn.data"></a>`data`

```rust
#[must_use]
pub fn data(&self) -> &DataClient
```

The Data API v2 client (wallets, feeds, boards, market state).

##### <a id="Polymarket.fn.relayer"></a>`relayer`

```rust
#[must_use]
pub fn relayer(&self) -> &RelayerClient
```

The Relayer API client (public endpoints).

##### <a id="Polymarket.fn.bridge"></a>`bridge`

```rust
#[must_use]
pub fn bridge(&self) -> &BridgeClient
```

The Bridge API client.

##### <a id="Polymarket.fn.combos"></a>`combos`

```rust
#[must_use]
pub fn combos(&self) -> &CombosClient
```

The Combos / RFQ REST client (public endpoints).

### <a id="struct.PolymarketBuilder"></a>`struct PolymarketBuilder`

```rust
#[must_use]
pub struct PolymarketBuilder { /* private fields */ }
```

Builder for [`Polymarket`](marcasite.md#struct.Polymarket).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="PolymarketBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient); the timeout, user agent, retry and response size
settings of this builder are then ignored.

##### <a id="PolymarketBuilder.fn.timeout"></a>`timeout`

```rust
pub fn timeout(self, timeout: Duration) -> Self
```

Sets the overall per-request timeout (default 30 s).

##### <a id="PolymarketBuilder.fn.connect_timeout"></a>`connect_timeout`

```rust
pub fn connect_timeout(self, timeout: Duration) -> Self
```

Sets the connection timeout (default 10 s).

##### <a id="PolymarketBuilder.fn.user_agent"></a>`user_agent`

```rust
pub fn user_agent(self, user_agent: impl Into<String>) -> Self
```

Sets the `User-Agent` header (default `marcasite/<version>`).

##### <a id="PolymarketBuilder.fn.retry_policy"></a>`retry_policy`

```rust
pub fn retry_policy(self, retry: RetryPolicy) -> Self
```

Sets the automatic retry policy (default: no retries).

##### <a id="PolymarketBuilder.fn.max_response_size"></a>`max_response_size`

```rust
pub fn max_response_size(self, bytes: usize) -> Self
```

Sets the largest response body accepted, in bytes (default 64 MiB); see
[`HttpClientBuilder::max_response_size`](marcasite.md#HttpClientBuilder.fn.max_response_size).

##### <a id="PolymarketBuilder.fn.gamma_base_url"></a>`gamma_base_url`

```rust
pub fn gamma_base_url(self, url: impl Into<String>) -> Self
```

Overrides the Gamma API base URL.

##### <a id="PolymarketBuilder.fn.clob_base_url"></a>`clob_base_url`

```rust
pub fn clob_base_url(self, url: impl Into<String>) -> Self
```

Overrides the CLOB API base URL.

##### <a id="PolymarketBuilder.fn.data_base_url"></a>`data_base_url`

```rust
pub fn data_base_url(self, url: impl Into<String>) -> Self
```

Overrides the Data API base URL.

##### <a id="PolymarketBuilder.fn.relayer_base_url"></a>`relayer_base_url`

```rust
pub fn relayer_base_url(self, url: impl Into<String>) -> Self
```

Overrides the Relayer API base URL.

##### <a id="PolymarketBuilder.fn.bridge_base_url"></a>`bridge_base_url`

```rust
pub fn bridge_base_url(self, url: impl Into<String>) -> Self
```

Overrides the Bridge API base URL.

##### <a id="PolymarketBuilder.fn.combos_base_url"></a>`combos_base_url`

```rust
pub fn combos_base_url(self, url: impl Into<String>) -> Self
```

Overrides the Combos / RFQ REST base URL.

##### <a id="PolymarketBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<Polymarket>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if a base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.RetryPolicy"></a>`struct RetryPolicy`

```rust
pub struct RetryPolicy { /* private fields */ }
```

When and how to retry failed requests automatically.

Only requests that are safe to repeat are retried: `GET` requests, and read-only `POST`
requests that the service clients mark as idempotent (e.g. batch price lookups).
Requests that create server-side state are never retried. A request is retried when
[`Error::is_retryable`](marcasite.md#Error.fn.is_retryable) is `true` for its failure: when the
error body carries an explicit `retryable` flag (Data API v2), that flag decides
(`"retryable": false` is never retried); otherwise `429 Too Many Requests`, `502`,
`503`, `504`, a timeout, or a connection error.

The delay before retry `n` (starting at 0) is the server's `Retry-After` value when
present, otherwise `initial_backoff * 2^n`, capped at `max_backoff`. If the server asks
for a longer delay than `max_backoff`, the error is returned instead of waiting.

Retries are **disabled by default** ([`RetryPolicy::none`](marcasite.md#RetryPolicy.fn.none)).

```rust
use std::time::Duration;
use marcasite_core::RetryPolicy;

let policy = RetryPolicy::new(3)
    .with_initial_backoff(Duration::from_millis(250))
    .with_max_backoff(Duration::from_secs(5));
assert_eq!(policy.max_retries(), 3);
```

**Implements:** `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `PartialEq`

#### Methods

##### <a id="RetryPolicy.fn.none"></a>`none`

```rust
#[must_use]
pub const fn none() -> Self
```

A policy that never retries.

##### <a id="RetryPolicy.fn.new"></a>`new`

```rust
#[must_use]
pub const fn new(max_retries: u32) -> Self
```

A policy that retries up to `max_retries` times with the default backoff
(200 ms initial, 10 s maximum).

##### <a id="RetryPolicy.fn.with_initial_backoff"></a>`with_initial_backoff`

```rust
#[must_use]
pub const fn with_initial_backoff(self, initial_backoff: Duration) -> Self
```

Sets the delay before the first retry when the server gives no `Retry-After`.

##### <a id="RetryPolicy.fn.with_max_backoff"></a>`with_max_backoff`

```rust
#[must_use]
pub const fn with_max_backoff(self, max_backoff: Duration) -> Self
```

Sets the longest delay the client will wait before a retry.

##### <a id="RetryPolicy.fn.max_retries"></a>`max_retries`

```rust
#[must_use]
pub const fn max_retries(&self) -> u32
```

The maximum number of retries after the first attempt.

##### <a id="RetryPolicy.fn.initial_backoff"></a>`initial_backoff`

```rust
#[must_use]
pub const fn initial_backoff(&self) -> Duration
```

The delay before the first retry when the server gives no `Retry-After`.

##### <a id="RetryPolicy.fn.max_backoff"></a>`max_backoff`

```rust
#[must_use]
pub const fn max_backoff(&self) -> Duration
```

The longest delay the client will wait before a retry.

### <a id="struct.TransportError"></a>`struct TransportError`

```rust
pub struct TransportError { /* private fields */ }
```

The request could not be completed at the transport level.

The underlying cause (from the HTTP client) is available through
`std::error::Error::source`; `Display` does not repeat it.

**Implements:** `Debug`, `Display`, `Error`

#### Methods

##### <a id="TransportError.fn.service"></a>`service`

```rust
#[must_use]
pub fn service(&self) -> Service
```

The service the request was sent to.

##### <a id="TransportError.fn.method"></a>`method`

```rust
#[must_use]
pub fn method(&self) -> &Method
```

The HTTP method of the failing request.

##### <a id="TransportError.fn.url"></a>`url`

```rust
#[must_use]
pub fn url(&self) -> &str
```

The full URL of the failing request.

##### <a id="TransportError.fn.is_connect"></a>`is_connect`

```rust
#[must_use]
pub fn is_connect(&self) -> bool
```

`true` if the connection could not be established (DNS, refused, TLS handshake).

### <a id="struct.ValidationError"></a>`struct ValidationError`

```rust
pub struct ValidationError { /* private fields */ }
```

A request parameter violates a documented constraint; the request was not sent.

**Implements:** `Clone`, `Debug`, `Display`, `Eq`, `Error`, `From<ValidationError>`, `PartialEq`

#### Methods

##### <a id="ValidationError.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(parameter: impl Into<Cow<'static, str>>, message: impl Into<Cow<'static, str>>) -> Self
```

Creates a validation error for `parameter` with an explanatory `message`.

##### <a id="ValidationError.fn.parameter"></a>`parameter`

```rust
#[must_use]
pub fn parameter(&self) -> &str
```

The name of the offending parameter.

##### <a id="ValidationError.fn.message"></a>`message`

```rust
#[must_use]
pub fn message(&self) -> &str
```

Why the value was rejected.

### <a id="struct.WebSocketError"></a>`struct WebSocketError`

```rust
pub struct WebSocketError { /* private fields */ }
```

A WebSocket connection failed, closed unexpectedly, or received a message that could
not be decoded. [`WebSocketError::kind`](marcasite.md#WebSocketError.fn.kind) tells these apart.

The underlying cause, if any (e.g. the WebSocket library's error), is available through
`std::error::Error::source`; `Display` does not repeat it.

**Implements:** `Debug`, `Display`, `Error`

#### Methods

##### <a id="WebSocketError.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(service: Service, kind: WebSocketErrorKind, message: impl Into<Cow<'static, str>>) -> Self
```

Creates a WebSocket error of the given `kind` for `service` with a message.

##### <a id="WebSocketError.fn.with_source"></a>`with_source`

```rust
#[must_use]
pub fn with_source(self, source: impl Error + Send + Sync + 'static) -> Self
```

Attaches the underlying cause.

##### <a id="WebSocketError.fn.with_close"></a>`with_close`

```rust
#[must_use]
pub fn with_close(self, code: u16, reason: impl Into<String>) -> Self
```

Attaches the close code and reason sent by the server.

##### <a id="WebSocketError.fn.service"></a>`service`

```rust
#[must_use]
pub fn service(&self) -> Service
```

The channel the error relates to.

##### <a id="WebSocketError.fn.kind"></a>`kind`

```rust
#[must_use]
pub fn kind(&self) -> WebSocketErrorKind
```

What kind of failure this is. In particular, [`WebSocketErrorKind::Decode`](marcasite.md#enum.WebSocketErrorKind) marks a
non-fatal decode error, after which the stream continues; every other kind is
terminal.

##### <a id="WebSocketError.fn.message"></a>`message`

```rust
#[must_use]
pub fn message(&self) -> &str
```

A description of what went wrong.

##### <a id="WebSocketError.fn.close_code"></a>`close_code`

```rust
#[must_use]
pub fn close_code(&self) -> Option<u16>
```

The close code sent by the server, if the connection was closed by the server.

##### <a id="WebSocketError.fn.close_reason"></a>`close_reason`

```rust
#[must_use]
pub fn close_reason(&self) -> Option<&str>
```

The close reason sent by the server, if any.

##### <a id="WebSocketError.fn.http_status"></a>`http_status`

```rust
#[must_use]
pub fn http_status(&self) -> Option<StatusCode>
```

The HTTP status the server answered the WebSocket handshake with, when it refused
the upgrade (e.g. `429 Too Many Requests` or `503 Service Unavailable`). Only set on
errors of kind [`WebSocketErrorKind::Connect`](marcasite.md#enum.WebSocketErrorKind).

##### <a id="WebSocketError.fn.retry_after"></a>`retry_after`

```rust
#[must_use]
pub fn retry_after(&self) -> Option<Duration>
```

How long the server asked the client to wait before connecting again: the
`Retry-After` header (in seconds or as an HTTP date) of a refused handshake, when
present. Only set on
errors of kind [`WebSocketErrorKind::Connect`](marcasite.md#enum.WebSocketErrorKind). Also available as
[`Error::retry_after`](marcasite.md#Error.fn.retry_after).

## Enums

### <a id="enum.Error"></a>`enum Error`

```rust
#[non_exhaustive]
pub enum Error {
    /// The server answered with a non-success HTTP status (other than `429`).
    ///
    /// The payload holds the status, the parsed error body and the trace id, if any.
    Api(Box<ApiError>),
    /// The server answered `429 Too Many Requests`.
    ///
    /// See [`ApiError::retry_after`](marcasite.md#ApiError.fn.retry_after) for the server's suggested delay, when provided.
    RateLimited(Box<ApiError>),
    /// The request did not complete within the configured timeout.
    Timeout(Box<TransportError>),
    /// The request could not be sent or the response could not be read (DNS, connection,
    /// TLS, I/O, ...).
    Transport(Box<TransportError>),
    /// A successful response body did not match the expected shape.
    Decode(Box<DecodeError>),
    /// The request was rejected client-side before being sent, because a parameter
    /// violates a documented constraint (e.g. too many ids in a batch).
    Validation(ValidationError),
    /// The client was misconfigured (e.g. an invalid base URL).
    Config(ConfigError),
    /// A WebSocket connection failed, closed unexpectedly or received an invalid frame.
    WebSocket(Box<WebSocketError>),
}
```

The error type for every marcasite operation.

The enum is `#[non_exhaustive]`: new failure kinds may be added in minor releases, so
include a wildcard arm when matching.

#### Display and cause chain

`Display` describes this failure with its context (service, method, URL without the
query string, status, ...) but does not repeat the underlying cause, such as the HTTP
client's or the WebSocket library's error. The cause is available through
`std::error::Error::source`, so a reporter that walks the chain shows each message
once: for example `anyhow`'s `{:#}`, or a loop over `source()`:

```rust
fn report(err: &marcasite_core::Error) -> String {
    let mut text = err.to_string();
    let mut cause = std::error::Error::source(err);
    while let Some(inner) = cause {
        text.push_str(&format!(": {inner}"));
        cause = inner.source();
    }
    text
}
```

**Implements:** `Debug`, `Display`, `Error`, `From<ConfigError>`, `From<ValidationError>`

#### Methods

##### <a id="Error.fn.service"></a>`service`

```rust
#[must_use]
pub fn service(&self) -> Option<Service>
```

The service the failing request was sent to, when known.

##### <a id="Error.fn.status"></a>`status`

```rust
#[must_use]
pub fn status(&self) -> Option<StatusCode>
```

The HTTP status code, for [`Error::Api`](marcasite.md#enum.Error), [`Error::RateLimited`](marcasite.md#enum.Error) and
[`Error::Decode`](marcasite.md#enum.Error), and for an `Error::WebSocket` whose handshake the server refused
with an HTTP status (see `WebSocketError::http_status`).

##### <a id="Error.fn.api_error"></a>`api_error`

```rust
#[must_use]
pub fn api_error(&self) -> Option<&ApiError>
```

The parsed API error, for [`Error::Api`](marcasite.md#enum.Error) and [`Error::RateLimited`](marcasite.md#enum.Error).

##### <a id="Error.fn.retry_after"></a>`retry_after`

```rust
#[must_use]
pub fn retry_after(&self) -> Option<Duration>
```

How long the server asked the client to wait before retrying, when provided: the
`Retry-After` header or a documented body field of an API error, or the
`Retry-After` header of a refused WebSocket handshake (see
`WebSocketError::retry_after`).

##### <a id="Error.fn.trace_id"></a>`trace_id`

```rust
#[must_use]
pub fn trace_id(&self) -> Option<&str>
```

The server-side trace id, when the service provides one (e.g. Data API v2's
`x-trace-id` header). Include it when reporting a failure to Polymarket.

##### <a id="Error.fn.is_not_found"></a>`is_not_found`

```rust
#[must_use]
pub fn is_not_found(&self) -> bool
```

`true` if the request failed with HTTP `404 Not Found`.

##### <a id="Error.fn.is_retryable"></a>`is_retryable`

```rust
#[must_use]
pub fn is_retryable(&self) -> bool
```

`true` if the failure is transient: the same request may succeed if sent again
later.

When the API error body carries an explicit `retryable` flag (as Data API v2 errors
do), the flag decides, whatever the status: `"retryable": false` is never
retryable, not even for a `429` or `503`. Without the flag, this covers rate
limiting (`429`), `502`/`503`/`504` responses, timeouts and connection failures.
The automatic retries of a [`RetryPolicy`](marcasite.md#struct.RetryPolicy) use the same rule.

This describes the *failure*, not the *request*: it does **not** mean that the
request is safe to repeat. A request that creates server-side state (for example
creating Bridge deposit or withdrawal addresses) may have been processed even though
it failed with a timeout or a `5xx`, and repeating it may do so twice. Such requests
are never retried automatically; the caller must decide whether repeating them is
safe.

### <a id="enum.Service"></a>`enum Service`

```rust
#[non_exhaustive]
pub enum Service {
    /// Gamma API (`gamma-api.polymarket.com`).
    Gamma,
    /// CLOB API (`clob.polymarket.com`).
    Clob,
    /// Data API v2 (`data-api.polymarket.com`).
    Data,
    /// Relayer API (`relayer-v2.polymarket.com`).
    Relayer,
    /// Bridge API (`bridge.polymarket.com`).
    Bridge,
    /// Combos / RFQ REST API (`combos-rfq-api.polymarket.com`).
    Combos,
    /// CLOB market WebSocket channel (`ws-subscriptions-clob.polymarket.com/ws/market`).
    MarketChannel,
    /// Sports results WebSocket channel (`sports-api.polymarket.com/ws`).
    SportsChannel,
    /// PolyBolt live data WebSocket (`ws-live-v2.polymarket.com/ws`).
    PolyBolt,
}
```

A Polymarket service (REST API or WebSocket channel).

Used in errors and logs to say which service a failure relates to.

**Implements:** `Clone`, `Copy`, `Debug`, `Display`, `Eq`, `Hash`, `PartialEq`

#### Methods

##### <a id="Service.fn.name"></a>`name`

```rust
#[must_use]
pub const fn name(self) -> &'static str
```

A short, stable, human-readable name for the service (e.g. `"gamma"`).

### <a id="enum.WebSocketErrorKind"></a>`enum WebSocketErrorKind`

```rust
#[non_exhaustive]
pub enum WebSocketErrorKind {
    /// The connection could not be established: the TCP connection, the TLS or WebSocket
    /// handshake failed (including a handshake refused with an HTTP status such as `429`
    /// or `503`, see [`WebSocketError::http_status`](marcasite.md#WebSocketError.fn.http_status) and [`WebSocketError::retry_after`](marcasite.md#WebSocketError.fn.retry_after)),
    /// the handshake timed out, the messages to send right after connecting (e.g. a
    /// subscription) could not be sent, or the connection was opened outside a Tokio
    /// runtime.
    Connect,
    /// The connection is closed: the server closed it with a close code other than `1000`
    /// (normal), see [`WebSocketError::close_code`](marcasite.md#WebSocketError.fn.close_code); it ended without a close frame; or a
    /// frame was queued after the connection had already terminated.
    Closed,
    /// The established connection failed at the socket or protocol level (an I/O or TLS
    /// error, or a WebSocket protocol violation).
    Protocol,
    /// A frame could not be sent: writing it to the socket failed (which ends the
    /// connection), or the request could not be encoded.
    Send,
    /// The connection stopped responding and was abandoned: no frame of any kind arrived
    /// within the configured idle timeout, or writing a frame to the socket did not
    /// complete within the write timeout (e.g. a half-open connection).
    Timeout,
    /// A received message could not be decoded into the expected type. Not fatal: the
    /// connection stays open and the stream continues with the next message.
    Decode,
}
```

What went wrong on a WebSocket connection; see [`WebSocketError::kind`](marcasite.md#WebSocketError.fn.kind).

Every kind except [`Decode`](marcasite.md#enum.WebSocketErrorKind) means the connection is unusable (or was
never established): a stream yields at most one such error, as its last item.
[`Decode`](marcasite.md#enum.WebSocketErrorKind) errors are **not fatal**: the stream continues with the next
message.

The enum is `#[non_exhaustive]`: include a wildcard arm when matching.

**Implements:** `Clone`, `Copy`, `Debug`, `Display`, `Eq`, `Hash`, `PartialEq`

#### Methods

##### <a id="WebSocketErrorKind.fn.name"></a>`name`

```rust
#[must_use]
pub const fn name(self) -> &'static str
```

A short, stable, human-readable name for the kind (e.g. `"decode"`).

## Type aliases

### <a id="type.Result"></a>`type Result`

```rust
pub type Result<T, E = Error> = Result<T, E>;
```

Convenience alias for `Result<T, marcasite_core::Error>`.
