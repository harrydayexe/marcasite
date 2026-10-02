# Module `marcasite::types`

> Generated from marcasite 0.1.0 (all features) by `just docs-md`. Do not edit.

Identifier newtypes and enums shared by several services.

The service modules re-export the ones they use, so for example `marcasite::gamma::MarketId`
and `marcasite::types::MarketId` are the same type.

## Index

- **Structs:** [`Address`](#struct.Address), [`ConditionId`](#struct.ConditionId), [`EventId`](#struct.EventId), [`MarketId`](#struct.MarketId), [`QuestionId`](#struct.QuestionId), [`TokenId`](#struct.TokenId)
- **Enums:** [`Side`](#enum.Side)

## Structs

### <a id="struct.Address"></a>`struct Address`

```rust
pub struct Address(/* private fields */);
```

An EVM address (`0x` followed by 40 hex characters), e.g. a user's proxy wallet.

The value is kept exactly as given or received; no checksum validation or case
normalisation is applied.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&Address>`, `From<&String>`, `From<&str>`, `From<Address>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="Address.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="Address.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="Address.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.ConditionId"></a>`struct ConditionId`

```rust
pub struct ConditionId(/* private fields */);
```

An on-chain condition id: `0x` followed by 64 hex characters. The CLOB API calls it
`market`, the Data API `condition`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&ConditionId>`, `From<&String>`, `From<&str>`, `From<ConditionId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="ConditionId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="ConditionId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="ConditionId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.EventId"></a>`struct EventId`

```rust
pub struct EventId(/* private fields */);
```

A Gamma event id: Polymarket's own identifier for an event (a group of markets),
e.g. `"16167"`.

The Gamma API sends it as a **string** in responses (an event's `id`) and takes it as
an **integer** in paths and filters (e.g. `GET /events/{id}`, `?id=`); the newtype
keeps the digits as a string and also accepts a JSON integer. The Data API
(`event_id`), the CLOB rewards endpoints (`event_id`) and the CLOB market WebSocket
channel (`event_message.id` of a `new_market` or `market_resolved` message) carry
the same Gamma ids.

Not a [`ConditionId`](types.md#struct.ConditionId): an event groups one or more markets, each with its own
condition id.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&EventId>`, `From<&String>`, `From<&str>`, `From<EventId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="EventId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="EventId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="EventId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.MarketId"></a>`struct MarketId`

```rust
pub struct MarketId(/* private fields */);
```

A Gamma market id: Polymarket's own identifier for a market, e.g. `"239826"`.

The Gamma API sends it as a **string** in responses (a market's `id`) and takes it as
an **integer** in paths and filters (e.g. `GET /markets/{id}`, `?id=`); the newtype
keeps the digits as a string and also accepts a JSON integer. The Data API
(`market_id`), the CLOB rewards endpoints (`market_id`) and the CLOB market
WebSocket channel (`id` of a `new_market` or `market_resolved` message) carry the
same Gamma ids.

Not the on-chain condition id: that is a [`ConditionId`](types.md#struct.ConditionId) (which the CLOB API calls
`market`).

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&MarketId>`, `From<&String>`, `From<&str>`, `From<MarketId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="MarketId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="MarketId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="MarketId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.QuestionId"></a>`struct QuestionId`

```rust
pub struct QuestionId(/* private fields */);
```

A UMA question id: the identifier of the question a market resolves through.

The Data API documents it as `0x` plus 64 hexadecimal characters (`question_id`);
the Gamma API types it as a plain string (a market's `questionID`, filtered with
`question_ids`). The value is kept exactly as given or received; it is not
validated.

Not a [`ConditionId`](types.md#struct.ConditionId), although both have the same shape.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&QuestionId>`, `From<&String>`, `From<&str>`, `From<QuestionId>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="QuestionId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="QuestionId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="QuestionId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

### <a id="struct.TokenId"></a>`struct TokenId`

```rust
pub struct TokenId(/* private fields */);
```

A CLOB outcome token id (also called asset id): a decimal-encoded `uint256`, e.g.
`"71321045679252212594626385532706912750332728571942532289631379312455583992563"`.

**Implements:** `AsRef<str>`, `Borrow<str>`, `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&String>`, `From<&TokenId>`, `From<&str>`, `From<String>`, `From<TokenId>`, `FromStr<Err = never>`, `Hash`, `Ord`, `PartialEq`, `PartialEq<&str>`, `PartialEq<str>`, `PartialOrd`, `Serialize`

#### Methods

##### <a id="TokenId.fn.new"></a>`new`

```rust
#[must_use]
pub fn new(id: impl Into<String>) -> Self
```

Creates the identifier from any string-like value.

##### <a id="TokenId.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The identifier as a string slice.

##### <a id="TokenId.fn.into_inner"></a>`into_inner`

```rust
#[must_use]
pub fn into_inner(self) -> String
```

Consumes the identifier, returning the inner string.

## Enums

### <a id="enum.Side"></a>`enum Side`

```rust
#[non_exhaustive]
pub enum Side {
    /// A buy.
    Buy,
    /// A sell.
    Sell,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The side of an order or trade.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="Side.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="Side.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.
