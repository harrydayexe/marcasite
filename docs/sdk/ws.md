# Module `marcasite::ws`

> Generated from marcasite 0.1.1 (all features) by `just docs-md`. Do not edit.

Public (unauthenticated) WebSocket channels.

| Channel | Type | URL | Docs |
|---|---|---|---|
| CLOB market channel: order books, price level deltas, trades, tick sizes, market lifecycle | [`MarketChannel`](ws.md#struct.MarketChannel) | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | <https://docs.polymarket.com/api-reference/wss/market> |
| Sports results: live scores and periods | [`SportsChannel`](ws.md#struct.SportsChannel) | `wss://sports-api.polymarket.com/ws` | <https://docs.polymarket.com/api-reference/wss/sports> |
| PolyBolt live data, `price.polymarket` only: best bid and ask per token | [`PolyBoltChannel`](ws.md#struct.PolyBoltChannel) | `wss://ws-live-v2.polymarket.com/ws` | <https://docs.polymarket.com/api-reference/wss/polybolt> |

Each channel type is a `Stream` of typed events (`Item = marcasite::Result<Event>`).
Connect with defaults via `connect`, or use `builder()` to override the URL (e.g. for a
mock server), the receive buffer size, the handshake timeout and the idle timeout.
Connecting must happen inside a Tokio runtime with the I/O and time drivers enabled (as
`#[tokio::main]` does); outside a runtime, `connect` fails with an error of kind
[`Connect`](marcasite.md#enum.WebSocketErrorKind).

To change subscriptions from another task while the stream is consumed, take a
cloneable handle with [`MarketChannel::handle`](ws.md#MarketChannel.fn.handle) ([`MarketChannelHandle`](ws.md#struct.MarketChannelHandle)) or
[`PolyBoltChannel::handle`](ws.md#PolyBoltChannel.fn.handle) ([`PolyBoltChannelHandle`](ws.md#struct.PolyBoltChannelHandle)).

```rust
use std::time::Duration;

use futures_util::StreamExt as _;
use marcasite::ws::{MarketChannel, MarketSubscription};

let mut channel = MarketChannel::builder()
    .connect_timeout(Duration::from_secs(5))
    .connect(MarketSubscription::new([
        "65818619657568813474341868652308942079804919287380422192892211131408793125422",
    ]))
    .await?;
while let Some(event) = channel.next().await {
    println!("{:?}", event?);
}
```

## Heartbeats

Handled automatically, as documented for each channel; heartbeat frames are never
yielded as events.

- **Market channel**: the client sends `PING` every 10 seconds and the server answers
  `PONG` (configurable with [`MarketChannelBuilder::heartbeat_interval`](ws.md#MarketChannelBuilder.fn.heartbeat_interval)).
- **Sports channel**: the spec says the server sends a text `ping` every 5 seconds and the
  client must answer `pong` within 10 seconds. Live, the server sends protocol-level ping
  frames every 15 seconds instead (never a text `ping`); both are answered
  automatically.
- **PolyBolt**: the server sends protocol-level ping frames every 25 seconds, answered
  automatically (two missed pongs close the connection with `4002`). The optional
  application-level ping is [`PolyBoltChannel::ping`](ws.md#PolyBoltChannel.fn.ping).

## Liveness and timeouts

A connection can die without being closed (the server vanishes, a NAT mapping is
dropped), which would leave a stream pending forever. Every channel therefore has an
**idle timeout**: if no frame of any kind (data, heartbeat or heartbeat reply) arrives
for that long, the stream yields a final error of kind [`Timeout`](marcasite.md#enum.WebSocketErrorKind) and ends. The
defaults are three times the heartbeat cadence (as documented, or as observed live where it
differs):

| Channel | Heartbeat | Default idle timeout |
|---|---|---|
| Market | client `PING` every 10 s, answered with `PONG` | [`MarketChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#MarketChannel.constant.DEFAULT_IDLE_TIMEOUT) (30 s; three heartbeat intervals if a longer interval is configured) |
| Sports | documented: server text `ping` every 5 s; live: protocol ping every 15 s | [`SportsChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#SportsChannel.constant.DEFAULT_IDLE_TIMEOUT) (45 s) |
| PolyBolt | server protocol ping every 25 s | [`PolyBoltChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#PolyBoltChannel.constant.DEFAULT_IDLE_TIMEOUT) (75 s) |

Change it with the builders' `idle_timeout`, or disable it with `no_idle_timeout`.

Writing a frame to the socket may take at most 10 seconds; a write that does not
complete in time (e.g. because the peer of a half-open connection stopped reading) also
ends the stream with an error of kind [`Timeout`](marcasite.md#enum.WebSocketErrorKind). When either side closes the
connection, the close handshake is completed, waiting at most 2 seconds for the other
side.

## Back-pressure

Received frames wait in a bounded buffer (the builders' `buffer`, default
[`DEFAULT_BUFFER`](ws.md#constant.DEFAULT_BUFFER)) until the stream is polled. When the buffer is full, the connection
stops reading from the socket until there is room again, but keeps sending: heartbeats,
subscription changes and a close still go out. Server pings that arrive meanwhile are
only answered once the stream is polled again, so a consumer that stalls for longer
than the server tolerates (a `pong` within 10 s for the sports `ping`; two missed pongs
for PolyBolt, which then closes with `4002`) is disconnected by the server. The idle
timeout does not run while reading is paused.

## Events and errors

- Every documented server message has a typed variant. A message the library does not
  recognise (new message type, or a frame that is not JSON) is yielded as the
  `Unknown(serde_json::Value)` variant, so server-side additions never break the stream.
- A frame holding a JSON array is flattened into one event per element. (None of the
  channels documents array frames; they are accepted defensively.)
- A recognised message that does not match its documented schema is yielded as an
  `Err(`[`Error::WebSocket`](marcasite.md#enum.Error)`)` of kind [`WebSocketErrorKind::Decode`](marcasite.md#enum.WebSocketErrorKind) whose message
  quotes the offending JSON and says why it did not decode. This is **not** fatal: the
  stream continues with the next message.
- A connection failure is yielded as one final `Err(`[`Error::WebSocket`](marcasite.md#enum.Error)`)`, after
  which the stream ends. Its [`kind`](marcasite.md#WebSocketError.fn.kind) is [`Closed`](marcasite.md#enum.WebSocketErrorKind) for an
  abnormal close (with the server's close code and reason when there is one) or a
  connection that dropped, [`Protocol`](marcasite.md#enum.WebSocketErrorKind) for a socket or protocol error, [`Send`](marcasite.md#enum.WebSocketErrorKind) if a
  frame could not be written, or [`Timeout`](marcasite.md#enum.WebSocketErrorKind) for an idle or stalled connection (see
  [Liveness and timeouts](#liveness-and-timeouts)). A normal close ends the stream
  without an error. Every channel implements `FusedStream`: `is_terminated()` reports
  whether the stream has ended.
- `connect` fails with kind [`Connect`](marcasite.md#enum.WebSocketErrorKind) if the connection cannot be established. If the
  server refused the handshake with an HTTP status, the error carries it
  ([`WebSocketError::http_status`](marcasite.md#WebSocketError.fn.http_status), also
  [`Error::status`](marcasite.md#Error.fn.status)) and the `Retry-After` delay
  ([`Error::retry_after`](marcasite.md#Error.fn.retry_after)). Sending on a connection that has
  ended (e.g. [`MarketChannel::subscribe`](ws.md#MarketChannel.fn.subscribe)) fails with kind [`Closed`](marcasite.md#enum.WebSocketErrorKind).
- Requests that break a documented constraint (for example an empty PolyBolt
  subscription, or more than 64 active PolyBolt subscriptions) fail with
  [`Error::Validation`](marcasite.md#enum.Error) before anything is sent.

```rust
use futures_util::StreamExt as _;
use marcasite::{
    Error, WebSocketErrorKind,
    ws::{SportsChannel, SportsEvent},
};

let mut channel = SportsChannel::connect().await?;
while let Some(item) = channel.next().await {
    match item {
        Ok(SportsEvent::Update(result)) => println!("{}: {}", result.league_abbreviation, result.score),
        Ok(_) => {}
        // Not fatal: skip the malformed message.
        Err(Error::WebSocket(err)) if err.kind() == WebSocketErrorKind::Decode => {
            eprintln!("skipping a message: {err}");
        }
        // Terminal: the stream ends after this item.
        Err(err) => return Err(err),
    }
}
```

## Reconnecting

Connections are not re-established automatically. When a stream ends (with or without a
final error), connect again and re-send the subscriptions. Subscription state and
sequence numbers (PolyBolt `seq`) do not survive a reconnect; re-seed local state from
the new snapshots.

The PolyBolt documentation specifies a policy per close code (see
[`PolyBoltCloseCode`](ws.md#enum.PolyBoltCloseCode)): `4003` (draining) reconnect after a random 0 to 10 s delay;
`4002` (slow consumer) and abnormal disconnects retry with exponential backoff and
jitter (1 s up to 30 s); `4001` and `4008` need a client fix before reconnecting. If the
handshake is refused with HTTP `429` or `503`, wait at least the `Retry-After` delay
([`Error::retry_after`](marcasite.md#Error.fn.retry_after)).
The market and sports channel documentation specifies no reconnection policy;
exponential backoff with jitter is a reasonable default.

```rust
use std::time::Duration;

use futures_util::StreamExt as _;
use marcasite::ws::{PolyBoltChannel, PolyBoltCloseCode, PolyBoltSubscription};

let assets = ["21742633143463906290569050155826241533067272736897614950488156847949938836455"];
let mut backoff = Duration::from_secs(1);
loop {
    let mut delay = backoff;
    match PolyBoltChannel::connect().await {
        Ok(mut channel) => {
            channel.subscribe(PolyBoltSubscription::price_polymarket(assets))?;
            while let Some(event) = channel.next().await {
                match event {
                    Ok(event) => {
                        backoff = Duration::from_secs(1);
                        println!("{event:?}");
                    }
                    Err(err) => match PolyBoltCloseCode::from_error(&err) {
                        // Reconnecting cannot help until the client is fixed.
                        Some(
                            PolyBoltCloseCode::AuthenticationFailed
                            | PolyBoltCloseCode::PolicyViolation,
                        ) => return Err(err),
                        // Documented: a uniformly random 0 to 10 s delay.
                        Some(PolyBoltCloseCode::Draining) => delay = Duration::from_secs(5),
                        // A decode error (the stream goes on) or a connection failure.
                        _ => eprintln!("{err}"),
                    },
                }
            }
        }
        Err(err) => {
            eprintln!("failed to connect: {err}");
            // A handshake refused with HTTP 429 or 503 may say how long to wait.
            if let Some(retry_after) = err.retry_after() {
                delay = delay.max(retry_after);
            }
        }
    }
    // Add random jitter to `delay` in real code.
    tokio::time::sleep(delay).await;
    backoff = (backoff * 2).min(Duration::from_secs(30));
}
```

## Not supported yet

The authenticated channels are not implemented: the CLOB user channel
(`/ws/user`), the RFQ quoter gateway, PolyBolt authentication (`auth`, `challenge`) and
the gated PolyBolt channels (`price.crypto`, `price.equity`, `price.crypto.twap`,
`price.equity.twap`).

## Index

- **Re-exports:** `EventId`, `MarketId`
- **Structs:** [`BestBidAsk`](#struct.BestBidAsk), [`BestBidAskEvent`](#struct.BestBidAskEvent), [`BookEvent`](#struct.BookEvent), [`ChannelAck`](#struct.ChannelAck), [`ErrorAck`](#struct.ErrorAck), [`EventMessage`](#struct.EventMessage), [`FeeSchedule`](#struct.FeeSchedule), [`LastTradePriceEvent`](#struct.LastTradePriceEvent), [`MarketChannel`](#struct.MarketChannel), [`MarketChannelBuilder`](#struct.MarketChannelBuilder), [`MarketChannelHandle`](#struct.MarketChannelHandle), [`MarketResolvedEvent`](#struct.MarketResolvedEvent), [`MarketSubscription`](#struct.MarketSubscription), [`MarketSubscriptionUpdate`](#struct.MarketSubscriptionUpdate), [`NewMarketEvent`](#struct.NewMarketEvent), [`OrderSummary`](#struct.OrderSummary), [`PolyBoltChannel`](#struct.PolyBoltChannel), [`PolyBoltChannelBuilder`](#struct.PolyBoltChannelBuilder), [`PolyBoltChannelHandle`](#struct.PolyBoltChannelHandle), [`PolyBoltSubscription`](#struct.PolyBoltSubscription), [`PongAck`](#struct.PongAck), [`PriceChange`](#struct.PriceChange), [`PriceChangeEvent`](#struct.PriceChangeEvent), [`PricePolymarketEnvelope`](#struct.PricePolymarketEnvelope), [`SportResult`](#struct.SportResult), [`SportsChannel`](#struct.SportsChannel), [`SportsChannelBuilder`](#struct.SportsChannelBuilder), [`TickSizeChangeEvent`](#struct.TickSizeChangeEvent)
- **Enums:** [`MarketEvent`](#enum.MarketEvent), [`PolyBoltChannelName`](#enum.PolyBoltChannelName), [`PolyBoltCloseCode`](#enum.PolyBoltCloseCode), [`PolyBoltErrorCode`](#enum.PolyBoltErrorCode), [`PolyBoltEvent`](#enum.PolyBoltEvent), [`PriceProvider`](#enum.PriceProvider), [`SportsEvent`](#enum.SportsEvent), [`SubscriptionLevel`](#enum.SubscriptionLevel)
- **Constants:** [`DEFAULT_BUFFER`](#constant.DEFAULT_BUFFER), [`DEFAULT_CONNECT_TIMEOUT`](#constant.DEFAULT_CONNECT_TIMEOUT)

## Re-exports

- `EventId`: re-export of [`marcasite::types::EventId`](types.md#struct.EventId).
- `MarketId`: re-export of [`marcasite::types::MarketId`](types.md#struct.MarketId).

## Structs

### <a id="struct.BestBidAsk"></a>`struct BestBidAsk`

```rust
#[non_exhaustive]
pub struct BestBidAsk {
    /// Condition id of the market.
    pub market: ConditionId,
    /// Outcome token id.
    pub asset_id: TokenId,
    /// Best bid price. The spec types it as a (required) decimal string and does not say
    /// how an empty book side is sent; an empty string decodes to `None` (and
    /// re-serializes as `""`).
    pub best_bid: Option<Decimal>,
    /// Best ask price; `None` for an empty string, as for [`best_bid`](ws.md#struct.BestBidAsk).
    pub best_ask: Option<Decimal>,
    /// Order book hash at this change.
    pub hash: String,
    /// Order book time (sent as Unix milliseconds).
    pub timestamp: DateTime<Utc>,
}
```

The best bid and ask of an outcome token (`BestBidAsk`).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.BestBidAskEvent"></a>`struct BestBidAskEvent`

```rust
#[non_exhaustive]
pub struct BestBidAskEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Best bid price. The spec types it as a (required) string and does not say how an
    /// empty side is sent; an empty string decodes to `None` (and re-serializes as `""`).
    pub best_bid: Option<Decimal>,
    /// Best ask price; `None` for an empty string, as for [`best_bid`](ws.md#struct.BestBidAskEvent).
    pub best_ask: Option<Decimal>,
    /// Spread between best ask and best bid; `None` for an empty string, as for
    /// [`best_bid`](ws.md#struct.BestBidAskEvent).
    pub spread: Option<Decimal>,
    /// Event time, as sent. The spec gives no unit for this field (the documented example
    /// is Unix milliseconds); see [`Self::timestamp_millis`](ws.md#BestBidAskEvent.fn.timestamp_millis).
    pub timestamp: String,
}
```

`best_bid_ask`: the best bid and ask of an asset changed (`BestBidAskEvent`). Requires
[`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="BestBidAskEvent.fn.timestamp_millis"></a>`timestamp_millis`

```rust
#[must_use]
pub fn timestamp_millis(&self) -> Option<DateTime<Utc>>
```

[`Self::timestamp`](ws.md#struct.BestBidAskEvent) interpreted as Unix milliseconds, as in the documented example;
`None` if it is not an integer.

### <a id="struct.BookEvent"></a>`struct BookEvent`

```rust
#[non_exhaustive]
pub struct BookEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Aggregated buy orders by price level. An empty side is an empty array.
    ///
    /// Live (2026-10-02) the levels are **not** sorted best-first: bids arrive ascending
    /// (worst, i.e. lowest, first) and asks descending (worst, i.e. highest, first). Sort
    /// them yourself if the order matters.
    pub bids: Vec<OrderSummary>,
    /// Aggregated sell orders by price level. See [`bids`](ws.md#struct.BookEvent) for the empty case
    /// and the ordering.
    pub asks: Vec<OrderSummary>,
    /// Snapshot time (sent as a string of Unix milliseconds).
    pub timestamp: DateTime<Utc>,
    /// Hash of the order book content.
    pub hash: String,
    /// The market's minimum tick size (e.g. `0.01`). Undocumented; observed live
    /// (2026-10-02) on the initial snapshots sent on subscribe. `None` when absent or empty.
    pub tick_size: Option<Decimal>,
    /// Price of the last trade. Undocumented; observed live (2026-10-02) on the initial
    /// snapshots sent on subscribe, as an empty string for an empty book, which decodes to
    /// `None` (as does an absent field).
    pub last_trade_price: Option<Decimal>,
}
```

`book`: a full aggregated order book snapshot for one asset (`BookEvent`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ChannelAck"></a>`struct ChannelAck`

```rust
#[non_exhaustive]
pub struct ChannelAck {
    /// The channel the request applied to.
    pub channel: PolyBoltChannelName,
    /// The request id, echoed when the request carried one.
    pub rid: Option<String>,
    /// The provider actually served. Only on `subscribed` acks for vendor channels while
    /// the server's provider selector is enabled; never for `price.polymarket`.
    pub provider: Option<PriceProvider>,
}
```

A `subscribed` or `unsubscribed` ack (`ChannelAck`).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.ErrorAck"></a>`struct ErrorAck`

```rust
#[non_exhaustive]
pub struct ErrorAck {
    /// Why the request failed.
    pub code: PolyBoltErrorCode,
    /// The channel the failed request named, when applicable.
    pub channel: Option<PolyBoltChannelName>,
    /// The request id, echoed when the request carried one (absent when the frame could
    /// not be parsed).
    pub rid: Option<String>,
}
```

An `error` ack (`ErrorAck`).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.EventMessage"></a>`struct EventMessage`

```rust
#[non_exhaustive]
pub struct EventMessage {
    /// Event id.
    pub id: Option<EventId>,
    /// Event ticker.
    pub ticker: Option<String>,
    /// Event slug.
    pub slug: Option<String>,
    /// Event title.
    pub title: Option<String>,
    /// Event description.
    pub description: Option<String>,
}
```

Parent event metadata of a market (`EventMessage`). Every field is optional in the
spec.

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.FeeSchedule"></a>`struct FeeSchedule`

```rust
#[non_exhaustive]
pub struct FeeSchedule {
    /// Fee curve exponent (a decimal string on the wire).
    pub exponent: Option<Decimal>,
    /// Fee rate (a decimal string on the wire).
    pub rate: Option<Decimal>,
    /// Whether the fee applies to takers only.
    pub taker_only: Option<bool>,
    /// Rebate rate (a decimal string on the wire).
    pub rebate_rate: Option<Decimal>,
}
```

A market's fee schedule, as sent in [`NewMarketEvent::fee_schedule`](ws.md#struct.NewMarketEvent).

Undocumented; observed live (2026-10-02) as
`{"exponent":"1","rate":"0.07","taker_only":true,"rebate_rate":"0.2"}`. Only the field
types were observed (decimal strings and one boolean), so every field is optional.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.LastTradePriceEvent"></a>`struct LastTradePriceEvent`

```rust
#[non_exhaustive]
pub struct LastTradePriceEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Trade execution price.
    pub price: Decimal,
    /// Trade size.
    pub size: Decimal,
    /// Fee rate in basis points.
    pub fee_rate_bps: Option<Decimal>,
    /// Trade side, from the taker's perspective.
    pub side: Side,
    /// Trade time (sent as a string of Unix milliseconds).
    pub timestamp: DateTime<Utc>,
    /// On-chain transaction hash.
    pub transaction_hash: Option<String>,
}
```

`last_trade_price`: a trade execution (`LastTradePriceEvent`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.MarketChannel"></a>`struct MarketChannel`

```rust
#[must_use = "streams do nothing unless polled"]
pub struct MarketChannel { /* private fields */ }
```

A live connection to the CLOB market channel: order book snapshots and deltas, trades,
tick size changes and market lifecycle events for the subscribed assets.

The channel is a `Stream` of [`MarketEvent`](ws.md#enum.MarketEvent)s. It sends the documented `PING`
heartbeat every 10 seconds and drops the server's `PONG` replies. If nothing at all
arrives for [`DEFAULT_IDLE_TIMEOUT`](ws.md#MarketChannel.constant.DEFAULT_IDLE_TIMEOUT) (configurable with
[`MarketChannelBuilder::idle_timeout`](ws.md#MarketChannelBuilder.fn.idle_timeout)), the connection is considered dead and the
stream ends with an error. See the [module documentation](ws.md) for error handling and
reconnection.

To change the subscription from another task while the stream is consumed, use a
[`MarketChannelHandle`](ws.md#struct.MarketChannelHandle) from [`handle`](ws.md#MarketChannel.fn.handle).

See <https://docs.polymarket.com/api-reference/wss/market>.

```rust
use futures_util::StreamExt as _;
use marcasite::ws::{MarketChannel, MarketEvent, MarketSubscription};

let subscription = MarketSubscription::new([
    "65818619657568813474341868652308942079804919287380422192892211131408793125422",
]);
let mut channel = MarketChannel::connect(subscription).await?;
while let Some(event) = channel.next().await {
    match event? {
        MarketEvent::Book(book) => println!("{} bids, {} asks", book.bids.len(), book.asks.len()),
        MarketEvent::PriceChange(change) => println!("{} level changes", change.price_changes.len()),
        other => println!("{other:?}"),
    }
}
```

#### Stream items

Each item is an `Ok(`[`MarketEvent`](ws.md#enum.MarketEvent)`)` or an
`Err(`[`Error::WebSocket`](marcasite.md#enum.Error)`)`. Check the error's
[`kind`](marcasite.md#WebSocketError.fn.kind):

- [`WebSocketErrorKind::Decode`](marcasite.md#enum.WebSocketErrorKind): a message did not
  match its documented schema. **Not fatal**: the stream continues with the next
  message.
- Any other kind is terminal: the connection failed or was closed abnormally, and the
  stream ends after this item.

A normal close ends the stream without an error.

**Implements:** `Debug`, `FusedStream`, `Stream<Item = Result<MarketEvent, Error>>`

#### Associated items

##### <a id="MarketChannel.constant.DEFAULT_URL"></a>`DEFAULT_URL`

```rust
pub const DEFAULT_URL: &'static str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
```

The production URL.

##### <a id="MarketChannel.constant.DEFAULT_HEARTBEAT_INTERVAL"></a>`DEFAULT_HEARTBEAT_INTERVAL`

```rust
pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = _;
```

The documented heartbeat interval: the client sends `PING` every 10 seconds.

##### <a id="MarketChannel.constant.DEFAULT_IDLE_TIMEOUT"></a>`DEFAULT_IDLE_TIMEOUT`

```rust
pub const DEFAULT_IDLE_TIMEOUT: Duration = _;
```

The default idle timeout: three times the documented 10-second heartbeat interval,
since the server answers every `PING` with `PONG`. When the heartbeat interval is
raised with [`MarketChannelBuilder::heartbeat_interval`](ws.md#MarketChannelBuilder.fn.heartbeat_interval) and no idle timeout is set,
the default becomes three times that interval (it never drops below this value).

##### <a id="MarketChannel.fn.connect"></a>`connect`

```rust
pub async fn connect(subscription: MarketSubscription) -> Result<Self>
```

Connects to the production URL and sends `subscription`.

###### Errors

Returns [`Error::WebSocket`](marcasite.md#enum.Error) if the connection cannot be established.

##### <a id="MarketChannel.fn.builder"></a>`builder`

```rust
pub fn builder() -> MarketChannelBuilder
```

Returns a builder for a custom URL, buffer size, timeout or heartbeat interval.

##### <a id="MarketChannel.fn.handle"></a>`handle`

```rust
pub fn handle(&self) -> MarketChannelHandle
```

A cloneable handle to change the subscription (or close the connection) from other
tasks while this channel is consumed as a stream.

```rust
use futures_util::StreamExt as _;
use marcasite::ws::{MarketChannel, MarketSubscription};

let mut channel = MarketChannel::connect(MarketSubscription::new([
    "65818619657568813474341868652308942079804919287380422192892211131408793125422",
]))
.await?;
let handle = channel.handle();
tokio::spawn(async move {
    // For example, in response to user input:
    let more = ["71321045679252212594626385532706912750332728571942532289631379312455583992563"];
    if let Err(err) = handle.subscribe(more) {
        eprintln!("subscription change failed: {err}");
    }
});
while let Some(event) = channel.next().await {
    println!("{:?}", event?);
}
```

##### <a id="MarketChannel.fn.subscribe"></a>`subscribe`

```rust
pub fn subscribe<I>(&self, asset_ids: I) -> Result<()>
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Starts following more asset ids without reconnecting.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `asset_ids` is empty, or [`Error::WebSocket`](marcasite.md#enum.Error) if the
connection has terminated.

##### <a id="MarketChannel.fn.unsubscribe"></a>`unsubscribe`

```rust
pub fn unsubscribe<I>(&self, asset_ids: I) -> Result<()>
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Stops following some asset ids without reconnecting.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `asset_ids` is empty, or [`Error::WebSocket`](marcasite.md#enum.Error) if the
connection has terminated.

##### <a id="MarketChannel.fn.update_subscription"></a>`update_subscription`

```rust
pub fn update_subscription(&self, update: MarketSubscriptionUpdate) -> Result<()>
```

Sends a subscription change.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the update has no asset ids, or
[`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="MarketChannel.fn.close"></a>`close`

```rust
pub fn close(&self)
```

Closes the connection gracefully; the stream then ends after any events already
received.

### <a id="struct.MarketChannelBuilder"></a>`struct MarketChannelBuilder`

```rust
#[must_use]
pub struct MarketChannelBuilder { /* private fields */ }
```

Builder for [`MarketChannel`](ws.md#struct.MarketChannel).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="MarketChannelBuilder.fn.url"></a>`url`

```rust
pub fn url(self, url: impl Into<String>) -> Self
```

Overrides the URL (default [`MarketChannel::DEFAULT_URL`](ws.md#MarketChannel.constant.DEFAULT_URL)), e.g. to target a mock
server in tests. Must use `ws://` or `wss://`.

##### <a id="MarketChannelBuilder.fn.buffer"></a>`buffer`

```rust
pub fn buffer(self, buffer: usize) -> Self
```

Sets how many received frames may be buffered before the connection stops reading
from the socket (default [`DEFAULT_BUFFER`](ws.md#constant.DEFAULT_BUFFER); see
[back-pressure](ws.md)).

##### <a id="MarketChannelBuilder.fn.connect_timeout"></a>`connect_timeout`

```rust
pub fn connect_timeout(self, timeout: Duration) -> Self
```

Sets the handshake timeout (default
[`DEFAULT_CONNECT_TIMEOUT`](ws.md#constant.DEFAULT_CONNECT_TIMEOUT)).

##### <a id="MarketChannelBuilder.fn.heartbeat_interval"></a>`heartbeat_interval`

```rust
pub fn heartbeat_interval(self, interval: Duration) -> Self
```

Overrides how often `PING` is sent (default
[`MarketChannel::DEFAULT_HEARTBEAT_INTERVAL`](ws.md#MarketChannel.constant.DEFAULT_HEARTBEAT_INTERVAL), as documented).

##### <a id="MarketChannelBuilder.fn.idle_timeout"></a>`idle_timeout`

```rust
pub fn idle_timeout(self, timeout: Duration) -> Self
```

Sets the idle timeout (default: three times the heartbeat interval, and at least
[`MarketChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#MarketChannel.constant.DEFAULT_IDLE_TIMEOUT)).

If no frame of any kind (data, heartbeat or heartbeat reply) arrives for this long,
the stream yields a final [`Error::WebSocket`](marcasite.md#enum.Error) of kind
[`Timeout`](marcasite.md#enum.WebSocketErrorKind) and ends, so that a connection that
died without closing is noticed. The timer does not run while the receive buffer is
full (see [back-pressure](ws.md)). A zero timeout makes `connect` fail
with [`Error::Config`](marcasite.md#enum.Error).

##### <a id="MarketChannelBuilder.fn.no_idle_timeout"></a>`no_idle_timeout`

```rust
pub fn no_idle_timeout(self) -> Self
```

Disables the idle timeout. A connection that dies without closing then leaves the
stream pending until the operating system reports the connection as broken, which
may never happen.

##### <a id="MarketChannelBuilder.fn.connect"></a>`connect`

```rust
pub async fn connect(self, subscription: MarketSubscription) -> Result<MarketChannel>
```

Connects and sends `subscription`.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the URL is invalid or the heartbeat interval or idle
timeout is zero, or [`Error::WebSocket`](marcasite.md#enum.Error) if the connection cannot be established.

### <a id="struct.MarketChannelHandle"></a>`struct MarketChannelHandle`

```rust
pub struct MarketChannelHandle { /* private fields */ }
```

A cloneable handle to a [`MarketChannel`](ws.md#struct.MarketChannel), obtained from [`MarketChannel::handle`](ws.md#MarketChannel.fn.handle):
changes the subscription or closes the connection from any task while the channel is
consumed as a stream.

The handle does not keep the connection open: once the [`MarketChannel`](ws.md#struct.MarketChannel) is dropped or
the connection has ended, its methods fail with an
[`Error::WebSocket`](marcasite.md#enum.Error) of kind [`Closed`](marcasite.md#enum.WebSocketErrorKind).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="MarketChannelHandle.fn.subscribe"></a>`subscribe`

```rust
pub fn subscribe<I>(&self, asset_ids: I) -> Result<()>
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Starts following more asset ids without reconnecting.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `asset_ids` is empty, or [`Error::WebSocket`](marcasite.md#enum.Error) if the
connection has terminated.

##### <a id="MarketChannelHandle.fn.unsubscribe"></a>`unsubscribe`

```rust
pub fn unsubscribe<I>(&self, asset_ids: I) -> Result<()>
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Stops following some asset ids without reconnecting.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `asset_ids` is empty, or [`Error::WebSocket`](marcasite.md#enum.Error) if the
connection has terminated.

##### <a id="MarketChannelHandle.fn.update_subscription"></a>`update_subscription`

```rust
pub fn update_subscription(&self, update: MarketSubscriptionUpdate) -> Result<()>
```

Sends a subscription change.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the update has no asset ids, or
[`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="MarketChannelHandle.fn.close"></a>`close`

```rust
pub fn close(&self)
```

Closes the connection gracefully; the channel's stream then ends after any events
already received.

### <a id="struct.MarketResolvedEvent"></a>`struct MarketResolvedEvent`

```rust
#[non_exhaustive]
pub struct MarketResolvedEvent {
    /// Market id (the spec does not describe this field; the documented example uses the
    /// same value as the market id of `new_market`).
    pub id: MarketId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// The market's asset (token) ids.
    pub assets_ids: Vec<TokenId>,
    /// The winning asset (token) id.
    pub winning_asset_id: TokenId,
    /// The winning outcome label (e.g. `"Yes"`).
    pub winning_outcome: String,
    /// Parent event metadata.
    pub event_message: Option<EventMessage>,
    /// Event time, as sent. The spec gives no unit for this field (the documented example
    /// is Unix milliseconds); see [`Self::timestamp_millis`](ws.md#MarketResolvedEvent.fn.timestamp_millis).
    pub timestamp: String,
    /// Tag slugs (e.g. `["stocks"]`).
    pub tags: Option<Vec<String>>,
}
```

`market_resolved`: a market was resolved (`MarketResolvedEvent`). Requires
[`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="MarketResolvedEvent.fn.timestamp_millis"></a>`timestamp_millis`

```rust
#[must_use]
pub fn timestamp_millis(&self) -> Option<DateTime<Utc>>
```

[`Self::timestamp`](ws.md#struct.MarketResolvedEvent) interpreted as Unix milliseconds, as in the documented example;
`None` if it is not an integer.

### <a id="struct.MarketSubscription"></a>`struct MarketSubscription`

```rust
#[must_use]
pub struct MarketSubscription { /* private fields */ }
```

The subscription request sent right after connecting to the market channel.

Required: the asset (token) ids to follow. Optional settings are left to the server
default unless set.

See <https://docs.polymarket.com/api-reference/wss/market>.

```rust
use marcasite::ws::{MarketSubscription, SubscriptionLevel};

let subscription = MarketSubscription::new([
    "65818619657568813474341868652308942079804919287380422192892211131408793125422",
])
.custom_feature_enabled(true)
.level(SubscriptionLevel::Level2);
assert_eq!(subscription.asset_ids().len(), 1);
```

**Implements:** `Clone`, `Debug`, `Eq`, `PartialEq`

#### Methods

##### <a id="MarketSubscription.fn.new"></a>`new`

```rust
pub fn new<I>(asset_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Subscribes to the given asset (token) ids (`assets_ids`).

The list may be empty: the documentation does not require any asset ids, and assets
can be added later with [`MarketChannel::subscribe`](ws.md#MarketChannel.fn.subscribe).

##### <a id="MarketSubscription.fn.initial_dump"></a>`initial_dump`

```rust
pub fn initial_dump(self, initial_dump: bool) -> Self
```

Whether to receive an initial order book snapshot for each asset (`initial_dump`;
the server defaults to `true`).

##### <a id="MarketSubscription.fn.level"></a>`level`

```rust
pub fn level(self, level: SubscriptionLevel) -> Self
```

The subscription level (`level`; the server defaults to
[`SubscriptionLevel::Level2`](ws.md#enum.SubscriptionLevel)).

##### <a id="MarketSubscription.fn.custom_feature_enabled"></a>`custom_feature_enabled`

```rust
pub fn custom_feature_enabled(self, enabled: bool) -> Self
```

Enables the [`MarketEvent::BestBidAsk`](ws.md#enum.MarketEvent), [`MarketEvent::NewMarket`](ws.md#enum.MarketEvent) and
[`MarketEvent::MarketResolved`](ws.md#enum.MarketEvent) events (`custom_feature_enabled`; the server
defaults to `false`).

##### <a id="MarketSubscription.fn.asset_ids"></a>`asset_ids`

```rust
#[must_use]
pub fn asset_ids(&self) -> &[TokenId]
```

The asset ids to subscribe to.

### <a id="struct.MarketSubscriptionUpdate"></a>`struct MarketSubscriptionUpdate`

```rust
#[must_use]
pub struct MarketSubscriptionUpdate { /* private fields */ }
```

A dynamic subscription change on an open market channel: add or remove asset ids
without reconnecting.

Use [`MarketChannel::subscribe`](ws.md#MarketChannel.fn.subscribe) / [`MarketChannel::unsubscribe`](ws.md#MarketChannel.fn.unsubscribe) for the common case,
or build one of these to also set `level` or `custom_feature_enabled`.

An update needs at least one asset id. The spec does not require any (it has no
`minItems`), but an update without asset ids changes nothing, so it is rejected
client-side with [`Error::Validation`](marcasite.md#enum.Error) as a convenience. The initial
[`MarketSubscription`](ws.md#struct.MarketSubscription) may be empty.

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Eq`, `PartialEq`

#### Methods

##### <a id="MarketSubscriptionUpdate.fn.subscribe"></a>`subscribe`

```rust
pub fn subscribe<I>(asset_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Starts following the given asset ids (`operation: "subscribe"`).

##### <a id="MarketSubscriptionUpdate.fn.unsubscribe"></a>`unsubscribe`

```rust
pub fn unsubscribe<I>(asset_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

Stops following the given asset ids (`operation: "unsubscribe"`).

##### <a id="MarketSubscriptionUpdate.fn.level"></a>`level`

```rust
pub fn level(self, level: SubscriptionLevel) -> Self
```

The subscription level (`level`).

##### <a id="MarketSubscriptionUpdate.fn.custom_feature_enabled"></a>`custom_feature_enabled`

```rust
pub fn custom_feature_enabled(self, enabled: bool) -> Self
```

Enables or disables the custom-feature events (`custom_feature_enabled`).

##### <a id="MarketSubscriptionUpdate.fn.asset_ids"></a>`asset_ids`

```rust
#[must_use]
pub fn asset_ids(&self) -> &[TokenId]
```

The asset ids this update applies to.

### <a id="struct.NewMarketEvent"></a>`struct NewMarketEvent`

```rust
#[non_exhaustive]
pub struct NewMarketEvent {
    /// Market id.
    pub id: MarketId,
    /// The market question.
    pub question: String,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Market slug.
    pub slug: String,
    /// Market description.
    pub description: Option<String>,
    /// The market's asset (token) ids.
    pub assets_ids: Vec<TokenId>,
    /// Outcome labels (e.g. `["Yes", "No"]`).
    pub outcomes: Vec<String>,
    /// Parent event metadata.
    pub event_message: Option<EventMessage>,
    /// Event time, as sent. The spec gives no unit for this field (the documented example
    /// is Unix milliseconds); see [`Self::timestamp_millis`](ws.md#NewMarketEvent.fn.timestamp_millis).
    pub timestamp: String,
    /// Tag slugs (e.g. `["stocks"]`).
    pub tags: Option<Vec<String>>,
    /// Condition id.
    pub condition_id: Option<ConditionId>,
    /// Whether the market is active.
    pub active: Option<bool>,
    /// CLOB token ids of the market.
    pub clob_token_ids: Option<Vec<TokenId>>,
    /// Sports market type, such as spread or moneyline (an empty string in the documented
    /// example of a non-sports market).
    pub sports_market_type: Option<String>,
    /// Betting line value, or an empty string when not applicable. Kept as sent.
    pub line: Option<String>,
    /// Game start time (RFC 3339). `None` when absent or sent as an empty string (not
    /// applicable).
    pub game_start_time: Option<DateTime<Utc>>,
    /// Minimum tick size for order prices.
    pub order_price_min_tick_size: Option<Decimal>,
    /// Display title of the market within its group.
    pub group_item_title: Option<String>,
    /// Taker base fee (e.g. `1000`, as sent in a string). Undocumented; observed live
    /// (2026-10-02). The unit is not documented.
    pub taker_base_fee: Option<Decimal>,
    /// Whether fees are enabled for the market. Undocumented; observed live (2026-10-02).
    pub fees_enabled: Option<bool>,
    /// The market's fee schedule. Undocumented; observed live (2026-10-02).
    pub fee_schedule: Option<FeeSchedule>,
}
```

`new_market`: a market was created (`NewMarketEvent`). Requires
[`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="NewMarketEvent.fn.timestamp_millis"></a>`timestamp_millis`

```rust
#[must_use]
pub fn timestamp_millis(&self) -> Option<DateTime<Utc>>
```

[`Self::timestamp`](ws.md#struct.NewMarketEvent) interpreted as Unix milliseconds, as in the documented example;
`None` if it is not an integer.

### <a id="struct.OrderSummary"></a>`struct OrderSummary`

```rust
#[non_exhaustive]
pub struct OrderSummary {
    /// Price level (e.g. `0.50`).
    pub price: Decimal,
    /// Total size at this price level.
    pub size: Decimal,
}
```

One aggregated price level of an order book (`OrderSummary`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PolyBoltChannel"></a>`struct PolyBoltChannel`

```rust
#[must_use = "streams do nothing unless polled"]
pub struct PolyBoltChannel { /* private fields */ }
```

A live connection to the PolyBolt live data socket, for the public `price.polymarket`
channel (best bid and ask per outcome token).

The channel is a `Stream` of [`PolyBoltEvent`](ws.md#enum.PolyBoltEvent)s: acks for each request, then one
snapshot per new subscription followed by live updates. The gated channels
(`price.crypto`, `price.equity`, `price.crypto.twap`, `price.equity.twap`) and the
`auth` operation are not supported.

The server keeps the connection alive with protocol-level pings every 25 seconds, which
are answered automatically. [`ping`](ws.md#PolyBoltChannel.fn.ping) sends the optional application-level
ping. If nothing at all arrives for [`DEFAULT_IDLE_TIMEOUT`](ws.md#PolyBoltChannel.constant.DEFAULT_IDLE_TIMEOUT)
(configurable with [`PolyBoltChannelBuilder::idle_timeout`](ws.md#PolyBoltChannelBuilder.fn.idle_timeout)), the connection is
considered dead and the stream ends with an error.

The documented per-connection limits are enforced client-side, returning
[`Error::Validation`](marcasite.md#enum.Error) instead of letting the server close the connection with `4008`:
at most 64 active subscriptions (counted from the requests sent on this connection; see
[`active_subscriptions`](ws.md#PolyBoltChannel.fn.active_subscriptions)), at most 20 subscribe or
unsubscribe frames per second, and at most 64 000 bytes per frame.

The frame rate is counted when a frame is queued for sending, not when it reaches the
socket. Queued frames are written promptly (also while the receive buffer is full),
but a slow socket write or network jitter can still bunch frames that were queued in
different seconds into one, so the server may count more than the client did. Stay
well below the limit by batching many ids into one frame.

To subscribe or unsubscribe from another task while the stream is consumed, use a
[`PolyBoltChannelHandle`](ws.md#struct.PolyBoltChannelHandle) from [`handle`](ws.md#PolyBoltChannel.fn.handle); the channel and its handles
share the limit accounting.

Note: the PolyBolt overview page says subscriptions require CLOB API credentials,
while the AsyncAPI spec marks `price.polymarket` as public (no auth). This type follows
the spec.

See <https://docs.polymarket.com/api-reference/wss/polybolt> and the [module
documentation](super) for error handling and reconnection.

```rust
use futures_util::StreamExt as _;
use marcasite::ws::{PolyBoltChannel, PolyBoltEvent, PolyBoltSubscription};

let mut channel = PolyBoltChannel::connect().await?;
channel.subscribe(PolyBoltSubscription::price_polymarket([
    "21742633143463906290569050155826241533067272736897614950488156847949938836455",
]))?;
while let Some(event) = channel.next().await {
    match event? {
        PolyBoltEvent::PricePolymarket(envelope) => {
            if let Some(quote) = envelope.payload {
                println!("{:?} / {:?}", quote.best_bid, quote.best_ask);
            }
        }
        PolyBoltEvent::Error(error) => eprintln!("request failed: {:?}", error.code),
        _ => {}
    }
}
```

#### Stream items

Each item is an `Ok(`[`PolyBoltEvent`](ws.md#enum.PolyBoltEvent)`)` or an
`Err(`[`Error::WebSocket`](marcasite.md#enum.Error)`)`. Check the error's
[`kind`](marcasite.md#WebSocketError.fn.kind):

- [`WebSocketErrorKind::Decode`](marcasite.md#enum.WebSocketErrorKind): a message did not
  match its documented schema. **Not fatal**: the stream continues with the next
  message.
- Any other kind is terminal: the connection failed or was closed abnormally, and the
  stream ends after this item.

A normal close ends the stream without an error.

**Implements:** `Debug`, `FusedStream`, `Stream<Item = Result<PolyBoltEvent, Error>>`

#### Associated items

##### <a id="PolyBoltChannel.constant.DEFAULT_URL"></a>`DEFAULT_URL`

```rust
pub const DEFAULT_URL: &'static str = "wss://ws-live-v2.polymarket.com/ws";
```

The production URL.

##### <a id="PolyBoltChannel.constant.DEFAULT_IDLE_TIMEOUT"></a>`DEFAULT_IDLE_TIMEOUT`

```rust
pub const DEFAULT_IDLE_TIMEOUT: Duration = _;
```

The default idle timeout: three times the documented 25-second interval of the
server's protocol-level pings (see [`PolyBoltChannelBuilder::idle_timeout`](ws.md#PolyBoltChannelBuilder.fn.idle_timeout)).

##### <a id="PolyBoltChannel.fn.connect"></a>`connect`

```rust
pub async fn connect() -> Result<Self>
```

Connects to the production URL. Nothing is subscribed until
[`subscribe`](ws.md#PolyBoltChannel.fn.subscribe) is called.

###### Errors

Returns [`Error::WebSocket`](marcasite.md#enum.Error) if the connection cannot be established.

##### <a id="PolyBoltChannel.fn.builder"></a>`builder`

```rust
pub fn builder() -> PolyBoltChannelBuilder
```

Returns a builder for a custom URL, buffer size or timeouts.

##### <a id="PolyBoltChannel.fn.handle"></a>`handle`

```rust
pub fn handle(&self) -> PolyBoltChannelHandle
```

A cloneable handle to subscribe, unsubscribe, ping or close from other tasks while
this channel is consumed as a stream.

The channel and all its handles share one count of active subscriptions and one
frame rate budget, so the documented per-connection limits hold however the
requests are spread across tasks.

```rust
use futures_util::StreamExt as _;
use marcasite::ws::{PolyBoltChannel, PolyBoltSubscription};

let mut channel = PolyBoltChannel::connect().await?;
let handle = channel.handle();
tokio::spawn(async move {
    // For example, in response to user input:
    let asset = "21742633143463906290569050155826241533067272736897614950488156847949938836455";
    if let Err(err) = handle.subscribe(PolyBoltSubscription::price_polymarket([asset])) {
        eprintln!("subscribe failed: {err}");
    }
});
while let Some(event) = channel.next().await {
    println!("{:?}", event?);
}
```

##### <a id="PolyBoltChannel.fn.subscribe"></a>`subscribe`

```rust
pub fn subscribe(&self, subscription: PolyBoltSubscription) -> Result<()>
```

Adds subscriptions (`op: "subscribe"`, batch form).

Each newly added subscription is acknowledged with
[`PolyBoltEvent::Subscribed`](ws.md#enum.PolyBoltEvent), then receives one snapshot envelope, then live
updates. Subscribing again to an active subscription is acknowledged again without
a new snapshot.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the subscription is empty, an asset id is not 1 to
78 decimal digits, the connection would exceed 64 active subscriptions, the frame
would exceed 64 000 bytes, or 20 subscribe/unsubscribe frames were already sent in
the last second; [`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannel.fn.unsubscribe"></a>`unsubscribe`

```rust
pub fn unsubscribe(&self, subscription: PolyBoltSubscription) -> Result<()>
```

Removes subscriptions (`op: "unsubscribe"`, batch form); acknowledged with
[`PolyBoltEvent::Unsubscribed`](ws.md#enum.PolyBoltEvent).

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the subscription is empty, an asset id is not 1 to
78 decimal digits, the frame would exceed 64 000 bytes, or 20 subscribe/unsubscribe
frames were already sent in the last second; [`Error::WebSocket`](marcasite.md#enum.Error) if the connection
has terminated.

##### <a id="PolyBoltChannel.fn.ping"></a>`ping`

```rust
pub fn ping(&self) -> Result<()>
```

Sends the optional application-level ping (`op: "ping"`); answered with
[`PolyBoltEvent::Pong`](ws.md#enum.PolyBoltEvent). Not needed for keepalive.

###### Errors

Returns [`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannel.fn.ping_with_rid"></a>`ping_with_rid`

```rust
pub fn ping_with_rid(&self, rid: impl Into<String>) -> Result<()>
```

Like [`ping`](ws.md#PolyBoltChannel.fn.ping), with a request id (`rid`) echoed on the pong.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the frame would exceed 64 000 bytes, or
[`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannel.fn.active_subscriptions"></a>`active_subscriptions`

```rust
#[must_use]
pub fn active_subscriptions(&self) -> usize
```

The number of distinct subscriptions requested and not unsubscribed on this
connection (through the channel or any of its handles), as counted for the
64-subscription limit.

An `error` ack ([`PolyBoltEvent::Error`](ws.md#enum.PolyBoltEvent)) names the `rid` of the rejected request
but not its filter, so a rejected subscription stops being counted only when that
can be told exactly: the request was a [`subscribe`](ws.md#PolyBoltChannel.fn.subscribe) of a single
new subscription with a [`rid`](ws.md#PolyBoltSubscription.fn.rid) not used by another
request still awaiting its ack, and no later request named the same asset id. The
slot is released when the error ack is read from this stream. Any other rejected
subscription stays counted until unsubscribed, so the count is an upper bound of
what the server holds. Give single subscriptions unique `rid`s to keep it exact.

##### <a id="PolyBoltChannel.fn.close"></a>`close`

```rust
pub fn close(&self)
```

Closes the connection gracefully; the stream then ends after any events already
received.

### <a id="struct.PolyBoltChannelBuilder"></a>`struct PolyBoltChannelBuilder`

```rust
#[must_use]
pub struct PolyBoltChannelBuilder { /* private fields */ }
```

Builder for [`PolyBoltChannel`](ws.md#struct.PolyBoltChannel).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="PolyBoltChannelBuilder.fn.url"></a>`url`

```rust
pub fn url(self, url: impl Into<String>) -> Self
```

Overrides the URL (default [`PolyBoltChannel::DEFAULT_URL`](ws.md#PolyBoltChannel.constant.DEFAULT_URL)), e.g. to target a mock
server in tests. Must use `ws://` or `wss://`.

##### <a id="PolyBoltChannelBuilder.fn.buffer"></a>`buffer`

```rust
pub fn buffer(self, buffer: usize) -> Self
```

Sets how many received frames may be buffered before the connection stops reading
from the socket (default [`DEFAULT_BUFFER`](ws.md#constant.DEFAULT_BUFFER); see
[back-pressure](ws.md)).

##### <a id="PolyBoltChannelBuilder.fn.connect_timeout"></a>`connect_timeout`

```rust
pub fn connect_timeout(self, timeout: Duration) -> Self
```

Sets the handshake timeout (default
[`DEFAULT_CONNECT_TIMEOUT`](ws.md#constant.DEFAULT_CONNECT_TIMEOUT)).

##### <a id="PolyBoltChannelBuilder.fn.idle_timeout"></a>`idle_timeout`

```rust
pub fn idle_timeout(self, timeout: Duration) -> Self
```

Sets the idle timeout (default [`PolyBoltChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#PolyBoltChannel.constant.DEFAULT_IDLE_TIMEOUT)).

If no frame of any kind (data, heartbeat or heartbeat reply) arrives for this long,
the stream yields a final [`Error::WebSocket`](marcasite.md#enum.Error) of kind
[`Timeout`](marcasite.md#enum.WebSocketErrorKind) and ends, so that a connection that
died without closing is noticed. The timer does not run while the receive buffer is
full (see [back-pressure](ws.md)). A zero timeout makes `connect` fail
with [`Error::Config`](marcasite.md#enum.Error).

##### <a id="PolyBoltChannelBuilder.fn.no_idle_timeout"></a>`no_idle_timeout`

```rust
pub fn no_idle_timeout(self) -> Self
```

Disables the idle timeout. A connection that dies without closing then leaves the
stream pending until the operating system reports the connection as broken, which
may never happen.

##### <a id="PolyBoltChannelBuilder.fn.connect"></a>`connect`

```rust
pub async fn connect(self) -> Result<PolyBoltChannel>
```

Connects.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the URL is invalid or the idle timeout is zero, or
[`Error::WebSocket`](marcasite.md#enum.Error) if the connection cannot be established. For a refused
handshake (e.g. HTTP `429` or `503`), the error's
[`http_status`](marcasite.md#WebSocketError.fn.http_status) and
[`retry_after`](marcasite.md#WebSocketError.fn.retry_after) (also
[`Error::retry_after`](marcasite.md#Error.fn.retry_after)) give the status and the `Retry-After` delay.

### <a id="struct.PolyBoltChannelHandle"></a>`struct PolyBoltChannelHandle`

```rust
pub struct PolyBoltChannelHandle { /* private fields */ }
```

A cloneable handle to a [`PolyBoltChannel`](ws.md#struct.PolyBoltChannel), obtained from [`PolyBoltChannel::handle`](ws.md#PolyBoltChannel.fn.handle):
subscribes, unsubscribes, pings or closes the connection from any task while the
channel is consumed as a stream.

The channel and all its handles share the client-side accounting of the documented
per-connection limits (64 active subscriptions, 20 subscribe or unsubscribe frames per
second), so concurrent requests from several tasks cannot exceed them together.

The handle does not keep the connection open: once the [`PolyBoltChannel`](ws.md#struct.PolyBoltChannel) is dropped
or the connection has ended, sending fails with an [`Error::WebSocket`](marcasite.md#enum.Error) of kind
[`Closed`](marcasite.md#enum.WebSocketErrorKind).

**Implements:** `Clone`, `Debug`

#### Methods

##### <a id="PolyBoltChannelHandle.fn.subscribe"></a>`subscribe`

```rust
pub fn subscribe(&self, subscription: PolyBoltSubscription) -> Result<()>
```

Adds subscriptions; see [`PolyBoltChannel::subscribe`](ws.md#PolyBoltChannel.fn.subscribe).

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the subscription is empty, an asset id is not 1 to
78 decimal digits, the connection would exceed 64 active subscriptions, the frame
would exceed 64 000 bytes, or 20 subscribe/unsubscribe frames were already sent in
the last second; [`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannelHandle.fn.unsubscribe"></a>`unsubscribe`

```rust
pub fn unsubscribe(&self, subscription: PolyBoltSubscription) -> Result<()>
```

Removes subscriptions; see [`PolyBoltChannel::unsubscribe`](ws.md#PolyBoltChannel.fn.unsubscribe).

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the subscription is empty, an asset id is not 1 to
78 decimal digits, the frame would exceed 64 000 bytes, or 20 subscribe/unsubscribe
frames were already sent in the last second; [`Error::WebSocket`](marcasite.md#enum.Error) if the connection
has terminated.

##### <a id="PolyBoltChannelHandle.fn.ping"></a>`ping`

```rust
pub fn ping(&self) -> Result<()>
```

Sends the optional application-level ping; see [`PolyBoltChannel::ping`](ws.md#PolyBoltChannel.fn.ping).

###### Errors

Returns [`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannelHandle.fn.ping_with_rid"></a>`ping_with_rid`

```rust
pub fn ping_with_rid(&self, rid: impl Into<String>) -> Result<()>
```

Like [`ping`](ws.md#PolyBoltChannelHandle.fn.ping), with a request id (`rid`) echoed on the pong.

###### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if the frame would exceed 64 000 bytes, or
[`Error::WebSocket`](marcasite.md#enum.Error) if the connection has terminated.

##### <a id="PolyBoltChannelHandle.fn.active_subscriptions"></a>`active_subscriptions`

```rust
#[must_use]
pub fn active_subscriptions(&self) -> usize
```

The number of active subscriptions counted for the connection; see
[`PolyBoltChannel::active_subscriptions`](ws.md#PolyBoltChannel.fn.active_subscriptions).

##### <a id="PolyBoltChannelHandle.fn.close"></a>`close`

```rust
pub fn close(&self)
```

Closes the connection gracefully; the channel's stream then ends after any events
already received.

### <a id="struct.PolyBoltSubscription"></a>`struct PolyBoltSubscription`

```rust
#[must_use]
pub struct PolyBoltSubscription { /* private fields */ }
```

A set of PolyBolt `(channel, filter)` subscriptions, sent as one batch frame by
[`PolyBoltChannel::subscribe`](ws.md#PolyBoltChannel.fn.subscribe) or [`PolyBoltChannel::unsubscribe`](ws.md#PolyBoltChannel.fn.unsubscribe).

Only the public `price.polymarket` channel is supported.

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

```rust
use marcasite::ws::PolyBoltSubscription;

let subscription = PolyBoltSubscription::price_polymarket([
    "21742633143463906290569050155826241533067272736897614950488156847949938836455",
])
.rid("s1");
```

**Implements:** `Clone`, `Debug`, `Eq`, `PartialEq`

#### Methods

##### <a id="PolyBoltSubscription.fn.price_polymarket"></a>`price_polymarket`

```rust
pub fn price_polymarket<I>(asset_ids: I) -> Self
where
    I: IntoIterator,
    <I as >::Item: Into<TokenId>,
```

`price.polymarket` subscriptions for the given outcome token ids: best bid and ask
per token.

Each id must be 1 to 78 decimal digits; this is checked when the subscription is
sent.

##### <a id="PolyBoltSubscription.fn.rid"></a>`rid`

```rust
pub fn rid(self, rid: impl Into<String>) -> Self
```

A client request id (`rid`), echoed on every ack this frame produces.

### <a id="struct.PongAck"></a>`struct PongAck`

```rust
#[non_exhaustive]
pub struct PongAck {
    /// The request id, echoed when the ping carried one.
    pub rid: Option<String>,
}
```

A `pong` ack (`SimpleAck`).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PriceChange"></a>`struct PriceChange`

```rust
#[non_exhaustive]
pub struct PriceChange {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// The price level affected.
    pub price: Decimal,
    /// The new aggregate size at this level (`0` means the level was removed).
    pub size: Decimal,
    /// The book side of the level.
    pub side: Side,
    /// Hash of the order that caused this change.
    pub hash: String,
    /// Best bid after the change; `None` when absent or an empty string (the spec does not
    /// say how an empty side is sent).
    pub best_bid: Option<Decimal>,
    /// Best ask after the change; `None` when absent or an empty string.
    pub best_ask: Option<Decimal>,
}
```

One price level change within a [`PriceChangeEvent`](ws.md#struct.PriceChangeEvent) (`PriceChangeMessage`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PriceChangeEvent"></a>`struct PriceChangeEvent`

```rust
#[non_exhaustive]
pub struct PriceChangeEvent {
    /// Condition id of the market.
    pub market: ConditionId,
    /// The changed price levels.
    pub price_changes: Vec<PriceChange>,
    /// Update time (sent as a string of Unix milliseconds).
    pub timestamp: DateTime<Utc>,
}
```

`price_change`: one or more order book price level updates (`PriceChangeEvent`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="struct.PricePolymarketEnvelope"></a>`struct PricePolymarketEnvelope`

```rust
#[non_exhaustive]
pub struct PricePolymarketEnvelope {
    /// Envelope version (`1`). An envelope of any other version is not decoded as this
    /// type but yielded as [`PolyBoltEvent::Unknown`](ws.md#enum.PolyBoltEvent), since its shape may differ.
    pub v: u32,
    /// The channel (`price.polymarket`).
    pub channel: PolyBoltChannelName,
    /// Dense per-connection, per-channel sequence number, assigned at delivery. Snapshot
    /// frames consume one; it restarts on every reconnect.
    pub seq: u64,
    /// Producer event time (sent as Unix milliseconds).
    pub ts: DateTime<Utc>,
    /// `Some(true)` on the one snapshot frame sent after each new subscription; absent on
    /// live frames. See [`Self::is_snapshot`](ws.md#PricePolymarketEnvelope.fn.is_snapshot).
    pub snapshot: Option<bool>,
    /// Number of frames dropped on this channel since the last delivered frame because the
    /// client fell behind. Absent when zero.
    pub dropped: Option<u64>,
    /// The best bid and ask. `None` for a snapshot of a token that has not traded within
    /// the server's warm window (sent as an empty array `[]`).
    pub payload: Option<BestBidAsk>,
}
```

A `price.polymarket` data envelope (`PricePolymarketEnvelope`).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PricePolymarketEnvelope.fn.is_snapshot"></a>`is_snapshot`

```rust
#[must_use]
pub fn is_snapshot(&self) -> bool
```

`true` for the snapshot frame that follows a new subscription.

### <a id="struct.SportResult"></a>`struct SportResult`

```rust
#[non_exhaustive]
pub struct SportResult {
    /// Numeric game id (`gameId`), e.g. `6365478`. Present on running-game updates.
    pub game_id: Option<u64>,
    /// String game id (`metadataGameId`), e.g. `"id2704888975110644"`. Present on the
    /// updates that end a game, in place of [`game_id`](ws.md#struct.SportResult).
    pub metadata_game_id: Option<String>,
    /// League abbreviation (`leagueAbbreviation`), lower case: observed `atp`, `wta`,
    /// `wta challenger`, `challenger`, `cricket`, `mlbb`, `lol`, `cs2`, `r6siege`, `val`,
    /// `dota2` and `ow`.
    pub league_abbreviation: String,
    /// Home team or player (`homeTeam`). Absent on the updates that end a game.
    pub home_team: Option<String>,
    /// Away team or player (`awayTeam`). Absent on the updates that end a game.
    pub away_team: Option<String>,
    /// Game status, kept as the raw string because the casing is inconsistent: observed
    /// `"running"` (esports), `"inprogress"` and, earlier, `"InProgress"`. Use
    /// [`status_is`](ws.md#SportResult.fn.status_is) to compare without regard to case. Absent on the
    /// updates that end a game.
    pub status: Option<String>,
    /// Current score, kept as sent. The format depends on the sport: `"0-0"`,
    /// `"6-7(3-7), 6-3, 3-1"` (tennis sets), `"123-125"` (cricket) or
    /// `"000-000|1-1|Bo3"` (esports: round score, map score, series format).
    pub score: String,
    /// Current period, kept as sent: observed `"S1"`-`"S3"` (tennis sets), `"1/3"`-`"3/3"`
    /// and `"2/5"` (esports maps), `"FT"` (final) and `"Live"` / `"LIVE"` (mixed case). The spec's list (`1H`,
    /// `Q1`, `Top 1st`, ...) was not observed on any live frame.
    pub period: String,
    /// Elapsed time in the current period, kept as sent (the spec says `MM:SS` or an empty
    /// string). Absent on every frame of the 2026-10-02 capture.
    pub elapsed: Option<String>,
    /// Whether the game is in progress.
    pub live: bool,
    /// Whether the game has ended.
    pub ended: bool,
    /// Team in possession, as sent (`turn`). Absent on every frame of the 2026-10-02
    /// capture; the spec says NFL only.
    pub turn: Option<String>,
    /// Provider id of the team in possession (`turnProviderId`). Observed on some earlier
    /// frames; kept as a string (a JSON string or integer on the wire).
    pub turn_provider_id: Option<String>,
    /// Sportradar game id (`sportradarGameId`). Observed on some earlier frames; kept as a
    /// string (a JSON string or integer on the wire).
    pub sportradar_game_id: Option<String>,
    /// When the game ended (`finishedTimestamp`, RFC 3339 with up to nanosecond precision,
    /// e.g. `2026-10-02T02:36:43.364683616Z`). Only on the updates that end a game, and
    /// not on all of them.
    pub finished_timestamp: Option<DateTime<Utc>>,
}
```

A real-time sports match update, as the live channel sends it.

**This is the live shape, not the spec's.** `docs/polymarket/specs/asyncapi-sports.json` describes
a `SportResult` keyed by `slug` with snake_case fields (`last_update`,
`finished_timestamp`); the live channel (observed 2026-10-02 across tennis, cricket,
esports and MLBB, 155 frames) sends camelCase fields, no `slug` and no `last_update`.
The spec-only fields were dropped, and the live ones are modelled; see
`SPEC_DEVIATIONS.md`.

Live frames come in three flavours:

- a running game: `gameId`, `leagueAbbreviation`, `homeTeam`, `awayTeam`, `status`,
  `score`, `period`, `live`, `ended` (and, on some games, `elapsed`, `turn`,
  `turnProviderId` and `sportradarGameId`);
- a game that has just ended: `metadataGameId` (a string such as `"id2704888975110644"`)
  instead of `gameId`, `finishedTimestamp`, `live: false`, `ended: true`, `period`
  `"FT"`, and no teams or status;
- the same without `finishedTimestamp`.

So exactly one of [`game_id`](ws.md#struct.SportResult) and
[`metadata_game_id`](ws.md#struct.SportResult) is normally present. They are different
id spaces (an integer and a string) and are not known to be related, so they are two
fields rather than one.

See <https://docs.polymarket.com/api-reference/wss/sports>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SportResult.fn.status_is"></a>`status_is`

```rust
#[must_use]
pub fn status_is(&self, status: &str) -> bool
```

Whether [`status`](ws.md#struct.SportResult) equals `status`, ignoring ASCII case (live sends
both `inprogress` and `InProgress`).

### <a id="struct.SportsChannel"></a>`struct SportsChannel`

```rust
#[must_use = "streams do nothing unless polled"]
pub struct SportsChannel { /* private fields */ }
```

A live connection to the sports results channel: score, period and status updates for
every active sports event.

No subscription is needed: the server broadcasts every update to every client.

**Heartbeats.** The spec says the server sends a text `ping` every 5 seconds and closes
connections that do not answer `pong` within 10 seconds. Live (2026-10-02) the server
instead sends a WebSocket protocol-level ping frame every 15 seconds and never a text
`ping`. Both are answered automatically and neither is surfaced as an event. If nothing
at all arrives for [`DEFAULT_IDLE_TIMEOUT`](ws.md#SportsChannel.constant.DEFAULT_IDLE_TIMEOUT) (three times the
observed 15-second ping; configurable with [`SportsChannelBuilder::idle_timeout`](ws.md#SportsChannelBuilder.fn.idle_timeout)), the
connection is considered dead and the stream ends with an error. See the
[module documentation](ws.md) for error handling and reconnection.

See <https://docs.polymarket.com/api-reference/wss/sports>.

```rust
use futures_util::StreamExt as _;
use marcasite::ws::{SportsChannel, SportsEvent};

let mut channel = SportsChannel::connect().await?;
while let Some(event) = channel.next().await {
    if let SportsEvent::Update(result) = event? {
        println!(
            "{} {:?} v {:?}: {} ({})",
            result.league_abbreviation, result.home_team, result.away_team, result.score,
            result.period,
        );
    }
}
```

#### Stream items

Each item is an `Ok(`[`SportsEvent`](ws.md#enum.SportsEvent)`)` or an
`Err(`[`Error::WebSocket`](marcasite.md#enum.Error)`)`. Check the error's
[`kind`](marcasite.md#WebSocketError.fn.kind):

- [`WebSocketErrorKind::Decode`](marcasite.md#enum.WebSocketErrorKind): a message did not
  match its documented schema. **Not fatal**: the stream continues with the next
  message.
- Any other kind is terminal: the connection failed or was closed abnormally, and the
  stream ends after this item.

A normal close ends the stream without an error.

**Implements:** `Debug`, `FusedStream`, `Stream<Item = Result<SportsEvent, Error>>`

#### Associated items

##### <a id="SportsChannel.constant.DEFAULT_URL"></a>`DEFAULT_URL`

```rust
pub const DEFAULT_URL: &'static str = "wss://sports-api.polymarket.com/ws";
```

The production URL.

##### <a id="SportsChannel.constant.DEFAULT_IDLE_TIMEOUT"></a>`DEFAULT_IDLE_TIMEOUT`

```rust
pub const DEFAULT_IDLE_TIMEOUT: Duration = _;
```

The default idle timeout: three times the 15-second interval at which the live
server sends its protocol-level ping (the spec documents a text `ping` every 5
seconds, which live does not send; see [`SportsChannelBuilder::idle_timeout`](ws.md#SportsChannelBuilder.fn.idle_timeout)).

##### <a id="SportsChannel.fn.connect"></a>`connect`

```rust
pub async fn connect() -> Result<Self>
```

Connects to the production URL.

###### Errors

Returns [`Error::WebSocket`](marcasite.md#enum.Error) if the connection cannot be
established.

##### <a id="SportsChannel.fn.builder"></a>`builder`

```rust
pub fn builder() -> SportsChannelBuilder
```

Returns a builder for a custom URL, buffer size or timeouts.

##### <a id="SportsChannel.fn.close"></a>`close`

```rust
pub fn close(&self)
```

Closes the connection gracefully; the stream then ends after any events already
received.

### <a id="struct.SportsChannelBuilder"></a>`struct SportsChannelBuilder`

```rust
#[must_use]
pub struct SportsChannelBuilder { /* private fields */ }
```

Builder for [`SportsChannel`](ws.md#struct.SportsChannel).

**Implements:** `Clone`, `Debug`, `Default`

#### Methods

##### <a id="SportsChannelBuilder.fn.url"></a>`url`

```rust
pub fn url(self, url: impl Into<String>) -> Self
```

Overrides the URL (default [`SportsChannel::DEFAULT_URL`](ws.md#SportsChannel.constant.DEFAULT_URL)), e.g. to target a mock
server in tests. Must use `ws://` or `wss://`.

##### <a id="SportsChannelBuilder.fn.buffer"></a>`buffer`

```rust
pub fn buffer(self, buffer: usize) -> Self
```

Sets how many received frames may be buffered before the connection stops reading
from the socket (default [`DEFAULT_BUFFER`](ws.md#constant.DEFAULT_BUFFER); see
[back-pressure](ws.md)).

##### <a id="SportsChannelBuilder.fn.connect_timeout"></a>`connect_timeout`

```rust
pub fn connect_timeout(self, timeout: Duration) -> Self
```

Sets the handshake timeout (default
[`DEFAULT_CONNECT_TIMEOUT`](ws.md#constant.DEFAULT_CONNECT_TIMEOUT)).

##### <a id="SportsChannelBuilder.fn.idle_timeout"></a>`idle_timeout`

```rust
pub fn idle_timeout(self, timeout: Duration) -> Self
```

Sets the idle timeout (default [`SportsChannel::DEFAULT_IDLE_TIMEOUT`](ws.md#SportsChannel.constant.DEFAULT_IDLE_TIMEOUT)).

If no frame of any kind (data, heartbeat or heartbeat reply) arrives for this long,
the stream yields a final [`Error::WebSocket`](marcasite.md#enum.Error) of kind
[`Timeout`](marcasite.md#enum.WebSocketErrorKind) and ends, so that a connection that
died without closing is noticed. The timer does not run while the receive buffer is
full (see [back-pressure](ws.md)). A zero timeout makes `connect` fail
with [`Error::Config`](marcasite.md#enum.Error).

##### <a id="SportsChannelBuilder.fn.no_idle_timeout"></a>`no_idle_timeout`

```rust
pub fn no_idle_timeout(self) -> Self
```

Disables the idle timeout. A connection that dies without closing then leaves the
stream pending until the operating system reports the connection as broken, which
may never happen.

##### <a id="SportsChannelBuilder.fn.connect"></a>`connect`

```rust
pub async fn connect(self) -> Result<SportsChannel>
```

Connects.

###### Errors

Returns [`Error::Config`](marcasite.md#enum.Error) if the URL is invalid or the idle
timeout is zero, or [`Error::WebSocket`](marcasite.md#enum.Error) if the connection
cannot be established.

### <a id="struct.TickSizeChangeEvent"></a>`struct TickSizeChangeEvent`

```rust
#[non_exhaustive]
pub struct TickSizeChangeEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// The previous tick size.
    pub old_tick_size: Decimal,
    /// The new tick size.
    pub new_tick_size: Decimal,
    /// Event time, as sent. The spec gives no unit for this field (the documented example
    /// is Unix milliseconds); see [`Self::timestamp_millis`](ws.md#TickSizeChangeEvent.fn.timestamp_millis).
    pub timestamp: String,
}
```

`tick_size_change`: the market's minimum tick size changed (`TickSizeChangeEvent`).

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

#### Methods

##### <a id="TickSizeChangeEvent.fn.timestamp_millis"></a>`timestamp_millis`

```rust
#[must_use]
pub fn timestamp_millis(&self) -> Option<DateTime<Utc>>
```

[`Self::timestamp`](ws.md#struct.TickSizeChangeEvent) interpreted as Unix milliseconds, as in the documented example;
`None` if it is not an integer.

## Enums

### <a id="enum.MarketEvent"></a>`enum MarketEvent`

```rust
#[non_exhaustive]
pub enum MarketEvent {
    /// `book`: a full order book snapshot (on subscribe, and after a trade).
    Book(BookEvent),
    /// `price_change`: order book price level deltas (an order was placed or cancelled).
    PriceChange(PriceChangeEvent),
    /// `last_trade_price`: a trade was executed.
    LastTradePrice(LastTradePriceEvent),
    /// `tick_size_change`: the market's minimum tick size changed.
    TickSizeChange(TickSizeChangeEvent),
    /// `best_bid_ask`: the best bid and ask changed. Requires
    /// [`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled).
    BestBidAsk(BestBidAskEvent),
    /// `new_market`: a market was created. Requires
    /// [`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled). Boxed because it is much larger
    /// than the frequent order book events.
    NewMarket(Box<NewMarketEvent>),
    /// `market_resolved`: a market was resolved. Requires
    /// [`MarketSubscription::custom_feature_enabled`](ws.md#MarketSubscription.fn.custom_feature_enabled). Boxed because it is much larger
    /// than the frequent order book events.
    MarketResolved(Box<MarketResolvedEvent>),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    Unknown(Value),
}
```

A message received on the market channel.

Decoded from the `event_type` field. A message with an unrecognised (or missing)
`event_type`, or a frame that is not JSON, becomes [`MarketEvent::Unknown`](ws.md#enum.MarketEvent) so that
new server messages never break the stream.

#### Timestamps

The spec documents the `timestamp` of `book`, `price_change` and `last_trade_price` as
Unix milliseconds, so those events expose it as a `DateTime<Utc>`. For
`tick_size_change`, `best_bid_ask`, `new_market` and `market_resolved` it gives no unit,
so their `timestamp` is kept as the string sent; each of these events has a
`timestamp_millis()` helper that reads it as Unix milliseconds, as in the documented
examples. Live (2026-10-02) every timestamp on the channel is Unix milliseconds.

#### Empty prices

The best bid and ask (and the spread) are typed as strings and the spec does not say
how an empty book side is sent, so they are `Option<Decimal>` everywhere: an empty
string decodes to `None` instead of failing the whole event.

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="enum.PolyBoltChannelName"></a>`enum PolyBoltChannelName`

```rust
#[non_exhaustive]
pub enum PolyBoltChannelName {
    /// `price.polymarket`: best bid and ask per outcome token (public).
    PricePolymarket,
    /// `price.crypto`: crypto reference prices (gated).
    PriceCrypto,
    /// `price.equity`: equity, ETF, forex, metal and commodity reference prices (gated).
    PriceEquity,
    /// `price.crypto.twap`: crypto time-weighted average prices (gated).
    PriceCryptoTwap,
    /// `price.equity.twap`: forex time-weighted average prices (gated).
    PriceEquityTwap,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

A PolyBolt channel name.

Only [`PricePolymarket`](ws.md#enum.PolyBoltChannelName) is public; the other channels
require CLOB API credentials and are not supported by this crate yet. They are
listed because acks and error acks may name them.

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PolyBoltChannelName.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PolyBoltChannelName.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.PolyBoltCloseCode"></a>`enum PolyBoltCloseCode`

```rust
#[non_exhaustive]
pub enum PolyBoltCloseCode {
    /// `4001`: authentication failed on a presented credential. Fix authentication before
    /// reconnecting.
    AuthenticationFailed,
    /// `4002`: slow consumer or pong timeout. Reconnect with exponential backoff and
    /// jitter; reduce the subscription load if it repeats.
    SlowConsumer,
    /// `4003`: the server is draining. Reconnect once after a uniformly random delay of
    /// 0 to 10 seconds.
    Draining,
    /// `4008`: policy violation (subscription, rate, frame size or auth limits). Fix the
    /// client before reconnecting; do not reconnect blindly.
    PolicyViolation,
    /// Any other close code.
    Other(u16),
}
```

A documented PolyBolt close code, for choosing a reconnection strategy.

See <https://docs.polymarket.com/api-reference/live-data/overview>.

```rust
use marcasite::ws::PolyBoltCloseCode;

assert_eq!(PolyBoltCloseCode::from_code(4003), PolyBoltCloseCode::Draining);
assert_eq!(PolyBoltCloseCode::Draining.code(), 4003);
```

**Implements:** `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `PartialEq`

#### Methods

##### <a id="PolyBoltCloseCode.fn.from_code"></a>`from_code`

```rust
#[must_use]
pub const fn from_code(code: u16) -> Self
```

Classifies a close code.

##### <a id="PolyBoltCloseCode.fn.code"></a>`code`

```rust
#[must_use]
pub const fn code(self) -> u16
```

The numeric close code.

##### <a id="PolyBoltCloseCode.fn.from_error"></a>`from_error`

```rust
#[must_use]
pub fn from_error(error: &Error) -> Option<Self>
```

The close code of an [`Error::WebSocket`](marcasite.md#enum.Error) caused by the server closing the
connection; `None` for any other error.

### <a id="enum.PolyBoltErrorCode"></a>`enum PolyBoltErrorCode`

```rust
#[non_exhaustive]
pub enum PolyBoltErrorCode {
    /// `bad_op`: malformed frame.
    BadOp,
    /// `bad_channel`: unknown channel.
    BadChannel,
    /// `bad_filter`: missing or invalid filter.
    BadFilter,
    /// `sub_limit`: a 65th subscription (followed by close `4008`).
    SubLimit,
    /// `rate_limited`: more than 20 subscribe or unsubscribe frames per second
    /// (followed by close `4008`).
    RateLimited,
    /// `auth_required`: a gated channel was subscribed before authenticating.
    AuthRequired,
    /// `auth_expired`: consumed or unknown ticket (web app only).
    AuthExpired,
    /// `auth_unavailable`: the credential verifier is unreachable; retry later.
    AuthUnavailable,
    /// `auth_invalid`: credentials refused.
    AuthInvalid,
    /// `auth_attempts`: a ninth auth frame (followed by close `4008`).
    AuthAttempts,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The reason in an `error` ack (`code`).

Error acks leave the connection open, except [`SubLimit`](ws.md#enum.PolyBoltErrorCode),
[`RateLimited`](ws.md#enum.PolyBoltErrorCode) and [`AuthAttempts`](ws.md#enum.PolyBoltErrorCode), which
are followed by close code `4008` (see [`Self::closes_connection`](ws.md#PolyBoltErrorCode.fn.closes_connection)).

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PolyBoltErrorCode.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PolyBoltErrorCode.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

##### <a id="PolyBoltErrorCode.fn.closes_connection"></a>`closes_connection`

```rust
#[must_use]
pub fn closes_connection(&self) -> bool
```

`true` for the codes the documentation says are followed by close code `4008`.

### <a id="enum.PolyBoltEvent"></a>`enum PolyBoltEvent`

```rust
#[non_exhaustive]
pub enum PolyBoltEvent {
    /// `subscribed`: a subscription was accepted.
    Subscribed(ChannelAck),
    /// `unsubscribed`: an unsubscribe request was accepted.
    Unsubscribed(ChannelAck),
    /// `pong`: the answer to an application-level ping.
    Pong(PongAck),
    /// `error`: a request failed. The connection stays open unless
    /// [`PolyBoltErrorCode::closes_connection`](ws.md#PolyBoltErrorCode.fn.closes_connection).
    Error(ErrorAck),
    /// A `price.polymarket` data envelope (snapshot or live update).
    PricePolymarket(PricePolymarketEnvelope),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    Unknown(Value),
}
```

A message received from PolyBolt.

Acks are recognised by `op`, data envelopes by `channel`. Anything else, including acks
for operations this crate does not send (`authed`, `challenge`), envelopes of the gated
channels and frames that are not JSON, becomes [`PolyBoltEvent::Unknown`](ws.md#enum.PolyBoltEvent) so that new
server messages never break the stream.

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="enum.PriceProvider"></a>`enum PriceProvider`

```rust
#[non_exhaustive]
pub enum PriceProvider {
    /// Pyth.
    Pyth,
    /// Massive.
    Massive,
    /// Chainlink.
    Chainlink,
    /// A value not known to this version of the library, holding the raw wire
    /// string.
    Unknown(String),
}
```

The price vendor named in a `subscribed` ack (`provider`). Only sent for the gated
vendor channels while the server's provider selector is enabled.

See <https://docs.polymarket.com/api-reference/wss/polybolt>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Display`, `Eq`, `From<&str>`, `From<String>`, `FromStr<Err = never>`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="PriceProvider.fn.as_str"></a>`as_str`

```rust
#[must_use]
pub fn as_str(&self) -> &str
```

The wire spelling of this value.

##### <a id="PriceProvider.fn.is_unknown"></a>`is_unknown`

```rust
#[must_use]
pub fn is_unknown(&self) -> bool
```

`true` if this is the `Unknown` catch-all variant.

### <a id="enum.SportsEvent"></a>`enum SportsEvent`

```rust
#[non_exhaustive]
pub enum SportsEvent {
    /// A live match update: score change, period change, match started or ended. Boxed
    /// because it is much larger than [`Unknown`](ws.md#enum.SportsEvent).
    Update(Box<SportResult>),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    Unknown(Value),
}
```

A message received on the sports channel.

The channel carries a single message type, the match update ([`SportResult`](ws.md#struct.SportResult)),
recognised by its `gameId` or `metadataGameId` field. Anything else (including a frame
that is not JSON, and the spec's `slug`-keyed shape, which the live channel does not
send) becomes [`SportsEvent::Unknown`](ws.md#enum.SportsEvent) so that new server messages never break the
stream.

See <https://docs.polymarket.com/api-reference/wss/sports>.

**Implements:** `Clone`, `Debug`, `Deserialize<'de>`, `Eq`, `PartialEq`, `Serialize`

### <a id="enum.SubscriptionLevel"></a>`enum SubscriptionLevel`

```rust
#[non_exhaustive]
pub enum SubscriptionLevel {
    /// Level `1`.
    Level1,
    /// Level `2` (the server default).
    Level2,
    /// Level `3`.
    Level3,
}
```

The subscription `level` of the market channel (`1`, `2` or `3`; the server defaults
to `2`).

The documentation lists the allowed values but does not describe what each level
changes.

See <https://docs.polymarket.com/api-reference/wss/market>.

**Implements:** `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `PartialEq`, `Serialize`

#### Methods

##### <a id="SubscriptionLevel.fn.as_u8"></a>`as_u8`

```rust
#[must_use]
pub const fn as_u8(self) -> u8
```

The wire value (`1`, `2` or `3`).

## Constants

### <a id="constant.DEFAULT_BUFFER"></a>`const DEFAULT_BUFFER`

```rust
pub const DEFAULT_BUFFER: usize = 1024;
```

Default capacity of the incoming-message buffer.

### <a id="constant.DEFAULT_CONNECT_TIMEOUT"></a>`const DEFAULT_CONNECT_TIMEOUT`

```rust
pub const DEFAULT_CONNECT_TIMEOUT: Duration = _;
```

Default handshake timeout.
