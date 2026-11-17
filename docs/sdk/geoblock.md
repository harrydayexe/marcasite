# Module `marcasite::geoblock`

> Generated from marcasite 0.2.0 (all features) by `just docs-md`. Do not edit.

Geoblock check (`https://polymarket.com/api/geoblock`).

Polymarket rejects orders from some jurisdictions. This module asks Polymarket whether the
IP address the request comes from is blocked, so an application can warn its users before
they try to trade. Start from [`GeoblockClient`](geoblock.md#struct.GeoblockClient).

| Endpoint | Method |
|---|---|
| `GET /api/geoblock` | [`GeoblockClient::get_status`](geoblock.md#GeoblockClient.fn.get_status) |

The endpoint is on `polymarket.com`, not on the API servers, and needs no authentication.
It reports the status of the address the request was sent **from**: it is a check only.
marcasite never provides, and will not add, any way to bypass a geoblock.

See <https://docs.polymarket.com/api-reference/geoblock>.

## Index

- **Structs:** [`GeoblockClient`](#struct.GeoblockClient), [`GeoblockClientBuilder`](#struct.GeoblockClientBuilder), [`GeoblockStatus`](#struct.GeoblockStatus)

## Structs

### <a id="struct.GeoblockClient"></a>`struct GeoblockClient`

```rust
pub struct GeoblockClient { /* private fields */ }
```

Client for the Geoblock endpoint (`https://polymarket.com`).

Reports whether the requesting IP address is geoblocked (`GET /api/geoblock`). The endpoint
lives on `polymarket.com`, not on the API servers. Cheap to clone: clones share one
connection pool.

This is a check only: the SDK never offers a way to route around a geoblock.

```rust
use marcasite::geoblock::GeoblockClient;

let client = GeoblockClient::new()?;
```

**Implements:** `Clone`, `Debug`

#### Associated items

##### <a id="GeoblockClient.constant.DEFAULT_BASE_URL"></a>`DEFAULT_BASE_URL`

```rust
pub const DEFAULT_BASE_URL: &'static str = "https://polymarket.com";
```

The production base URL.

##### <a id="GeoblockClient.fn.new"></a>`new`

```rust
pub fn new() -> Result<Self>
```

Creates a client with the default HTTP settings and base URL.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the HTTP client cannot be built.

##### <a id="GeoblockClient.fn.builder"></a>`builder`

```rust
pub fn builder() -> GeoblockClientBuilder
```

Returns a builder for setting a custom base URL or HTTP client.

##### <a id="GeoblockClient.fn.base_url"></a>`base_url`

```rust
#[must_use]
pub fn base_url(&self) -> &Url
```

The base URL requests are sent to.

##### <a id="GeoblockClient.fn.get_status"></a>`get_status`

```rust
pub async fn get_status(&self) -> Result<GeoblockStatus>
```

Checks the geographic eligibility of the IP address this request is sent from
(`GET /api/geoblock`).

See <https://docs.polymarket.com/api-reference/geoblock>.

```rust
let status = marcasite::geoblock::GeoblockClient::new()?.get_status().await?;
if status.blocked {
    println!("trading not available in {}", status.country);
}
```

###### Errors

[`Error::Api`](marcasite.md#enum.Error) on a non-2xx status,
[`Error::RateLimited`](marcasite.md#enum.Error) on `429`, or any other
[`Error`](marcasite.md#enum.Error) for transport or decoding failures.

### <a id="struct.GeoblockClientBuilder"></a>`struct GeoblockClientBuilder`

```rust
#[must_use]
pub struct GeoblockClientBuilder { /* private fields */ }
```

Builder for [`GeoblockClient`](geoblock.md#struct.GeoblockClient).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="GeoblockClientBuilder.fn.base_url"></a>`base_url`

```rust
pub fn base_url(self, url: impl Into<String>) -> Self
```

Overrides the base URL (default [`GeoblockClient::DEFAULT_BASE_URL`](geoblock.md#GeoblockClient.constant.DEFAULT_BASE_URL)), e.g. to target a
mock server in tests. A path prefix is preserved.

##### <a id="GeoblockClientBuilder.fn.http_client"></a>`http_client`

```rust
pub fn http_client(self, http: HttpClient) -> Self
```

Uses an existing [`HttpClient`](marcasite.md#struct.HttpClient) (and its timeouts, user agent and retry policy).

##### <a id="GeoblockClientBuilder.fn.build"></a>`build`

```rust
pub fn build(self) -> Result<GeoblockClient>
```

Builds the client.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the base URL is invalid or the HTTP
client cannot be built.

### <a id="struct.GeoblockStatus"></a>`struct GeoblockStatus`

```rust
#[non_exhaustive]
pub struct GeoblockStatus {
    /// Whether the address is blocked from placing orders (the docs' wording).
    ///
    /// The docs also describe *close-only* jurisdictions, where existing positions can be
    /// closed but no new ones opened. Whether those report `true` here is not documented
    /// (see `OPEN_QUESTIONS.md`), so treat `true` as "do not open new positions" and do not
    /// read `false` as proof that every action is allowed.
    pub blocked: bool,
    /// The IP address Polymarket detected for the request.
    pub ip: IpAddr,
    /// ISO 3166-1 alpha-2 country code, e.g. `"US"`.
    pub country: String,
    /// Region or state code, e.g. `"OH"`.
    pub region: String,
}
```

The geoblock status of the requesting IP address, returned by
[`GeoblockClient::get_status`](geoblock.md#GeoblockClient.fn.get_status).

This is a check only; see the [module documentation](geoblock.md). Every field is
required, as the docs list them without any optionality. Whether `region` can be absent
for countries without sub-national regions has not been observed (see `OPEN_QUESTIONS.md`).

See <https://docs.polymarket.com/api-reference/geoblock>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`
