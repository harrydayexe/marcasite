//! Public (unauthenticated) WebSocket channels.
//!
//! | Channel | Type | URL | Docs |
//! |---|---|---|---|
//! | CLOB market channel: order books, price level deltas, trades, tick sizes, market lifecycle | [`MarketChannel`] | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | <https://docs.polymarket.com/api-reference/wss/market> |
//! | Sports results: live scores and periods | [`SportsChannel`] | `wss://sports-api.polymarket.com/ws` | <https://docs.polymarket.com/api-reference/wss/sports> |
//! | PolyBolt live data, `price.polymarket` only: best bid and ask per token | [`PolyBoltChannel`] | `wss://ws-live-v2.polymarket.com/ws` | <https://docs.polymarket.com/api-reference/wss/polybolt> |
//!
//! Each channel type is a [`Stream`] of typed events (`Item = polyoxide::Result<Event>`).
//! Connect with defaults via `connect`, or use `builder()` to override the URL (e.g. for a
//! mock server), the receive buffer size, the handshake timeout and the idle timeout.
//! Connecting must happen inside a Tokio runtime with the I/O and time drivers enabled (as
//! `#[tokio::main]` does); outside a runtime, `connect` fails with an error of kind
//! [`Connect`].
//!
//! To change subscriptions from another task while the stream is consumed, take a
//! cloneable handle with [`MarketChannel::handle`] ([`MarketChannelHandle`]) or
//! [`PolyBoltChannel::handle`] ([`PolyBoltChannelHandle`]).
//!
//! ```no_run
//! # async fn run() -> polyoxide::Result<()> {
//! use std::time::Duration;
//!
//! use futures_util::StreamExt as _;
//! use polyoxide::ws::{MarketChannel, MarketSubscription};
//!
//! let mut channel = MarketChannel::builder()
//!     .connect_timeout(Duration::from_secs(5))
//!     .connect(MarketSubscription::new([
//!         "65818619657568813474341868652308942079804919287380422192892211131408793125422",
//!     ]))
//!     .await?;
//! while let Some(event) = channel.next().await {
//!     println!("{:?}", event?);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Heartbeats
//!
//! Handled automatically, as documented for each channel; heartbeat frames are never
//! yielded as events.
//!
//! - **Market channel**: the client sends `PING` every 10 seconds and the server answers
//!   `PONG` (configurable with [`MarketChannelBuilder::heartbeat_interval`]).
//! - **Sports channel**: the spec says the server sends a text `ping` every 5 seconds and the
//!   client must answer `pong` within 10 seconds. Live, the server sends protocol-level ping
//!   frames every 15 seconds instead (never a text `ping`); both are answered
//!   automatically.
//! - **PolyBolt**: the server sends protocol-level ping frames every 25 seconds, answered
//!   automatically (two missed pongs close the connection with `4002`). The optional
//!   application-level ping is [`PolyBoltChannel::ping`].
//!
//! # Liveness and timeouts
//!
//! A connection can die without being closed (the server vanishes, a NAT mapping is
//! dropped), which would leave a stream pending forever. Every channel therefore has an
//! **idle timeout**: if no frame of any kind (data, heartbeat or heartbeat reply) arrives
//! for that long, the stream yields a final error of kind [`Timeout`] and ends. The
//! defaults are three times the heartbeat cadence (as documented, or as observed live where it
//! differs):
//!
//! | Channel | Heartbeat | Default idle timeout |
//! |---|---|---|
//! | Market | client `PING` every 10 s, answered with `PONG` | [`MarketChannel::DEFAULT_IDLE_TIMEOUT`] (30 s; three heartbeat intervals if a longer interval is configured) |
//! | Sports | documented: server text `ping` every 5 s; live: protocol ping every 15 s | [`SportsChannel::DEFAULT_IDLE_TIMEOUT`] (45 s) |
//! | PolyBolt | server protocol ping every 25 s | [`PolyBoltChannel::DEFAULT_IDLE_TIMEOUT`] (75 s) |
//!
//! Change it with the builders' `idle_timeout`, or disable it with `no_idle_timeout`.
//!
//! Writing a frame to the socket may take at most 10 seconds; a write that does not
//! complete in time (e.g. because the peer of a half-open connection stopped reading) also
//! ends the stream with an error of kind [`Timeout`]. When either side closes the
//! connection, the close handshake is completed, waiting at most 2 seconds for the other
//! side.
//!
//! # Back-pressure
//!
//! Received frames wait in a bounded buffer (the builders' `buffer`, default
//! [`DEFAULT_BUFFER`]) until the stream is polled. When the buffer is full, the connection
//! stops reading from the socket until there is room again, but keeps sending: heartbeats,
//! subscription changes and a close still go out. Server pings that arrive meanwhile are
//! only answered once the stream is polled again, so a consumer that stalls for longer
//! than the server tolerates (a `pong` within 10 s for the sports `ping`; two missed pongs
//! for PolyBolt, which then closes with `4002`) is disconnected by the server. The idle
//! timeout does not run while reading is paused.
//!
//! # Events and errors
//!
//! - Every documented server message has a typed variant. A message the library does not
//!   recognise (new message type, or a frame that is not JSON) is yielded as the
//!   `Unknown(serde_json::Value)` variant, so server-side additions never break the stream.
//! - A frame holding a JSON array is flattened into one event per element. (None of the
//!   channels documents array frames; they are accepted defensively.)
//! - A recognised message that does not match its documented schema is yielded as an
//!   `Err(`[`Error::WebSocket`]`)` of kind [`WebSocketErrorKind::Decode`] whose message
//!   quotes the offending JSON and says why it did not decode. This is **not** fatal: the
//!   stream continues with the next message.
//! - A connection failure is yielded as one final `Err(`[`Error::WebSocket`]`)`, after
//!   which the stream ends. Its [`kind`](crate::WebSocketError::kind) is [`Closed`] for an
//!   abnormal close (with the server's close code and reason when there is one) or a
//!   connection that dropped, [`Protocol`] for a socket or protocol error, [`Send`] if a
//!   frame could not be written, or [`Timeout`] for an idle or stalled connection (see
//!   [Liveness and timeouts](#liveness-and-timeouts)). A normal close ends the stream
//!   without an error. Every channel implements [`FusedStream`]: `is_terminated()` reports
//!   whether the stream has ended.
//! - `connect` fails with kind [`Connect`] if the connection cannot be established. If the
//!   server refused the handshake with an HTTP status, the error carries it
//!   ([`WebSocketError::http_status`](crate::WebSocketError::http_status), also
//!   [`Error::status`](crate::Error::status)) and the `Retry-After` delay
//!   ([`Error::retry_after`](crate::Error::retry_after)). Sending on a connection that has
//!   ended (e.g. [`MarketChannel::subscribe`]) fails with kind [`Closed`].
//! - Requests that break a documented constraint (for example an empty PolyBolt
//!   subscription, or more than 64 active PolyBolt subscriptions) fail with
//!   [`Error::Validation`] before anything is sent.
//!
//! ```no_run
//! # async fn run() -> polyoxide::Result<()> {
//! use futures_util::StreamExt as _;
//! use polyoxide::{
//!     Error, WebSocketErrorKind,
//!     ws::{SportsChannel, SportsEvent},
//! };
//!
//! let mut channel = SportsChannel::connect().await?;
//! while let Some(item) = channel.next().await {
//!     match item {
//!         Ok(SportsEvent::Update(result)) => println!("{}: {}", result.league_abbreviation, result.score),
//!         Ok(_) => {}
//!         // Not fatal: skip the malformed message.
//!         Err(Error::WebSocket(err)) if err.kind() == WebSocketErrorKind::Decode => {
//!             eprintln!("skipping a message: {err}");
//!         }
//!         // Terminal: the stream ends after this item.
//!         Err(err) => return Err(err),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Reconnecting
//!
//! Connections are not re-established automatically. When a stream ends (with or without a
//! final error), connect again and re-send the subscriptions. Subscription state and
//! sequence numbers (PolyBolt `seq`) do not survive a reconnect; re-seed local state from
//! the new snapshots.
//!
//! The PolyBolt documentation specifies a policy per close code (see
//! [`PolyBoltCloseCode`]): `4003` (draining) reconnect after a random 0 to 10 s delay;
//! `4002` (slow consumer) and abnormal disconnects retry with exponential backoff and
//! jitter (1 s up to 30 s); `4001` and `4008` need a client fix before reconnecting. If the
//! handshake is refused with HTTP `429` or `503`, wait at least the `Retry-After` delay
//! ([`Error::retry_after`](crate::Error::retry_after)).
//! The market and sports channel documentation specifies no reconnection policy;
//! exponential backoff with jitter is a reasonable default.
//!
//! ```no_run
//! # async fn run() -> polyoxide::Result<()> {
//! use std::time::Duration;
//!
//! use futures_util::StreamExt as _;
//! use polyoxide::ws::{PolyBoltChannel, PolyBoltCloseCode, PolyBoltSubscription};
//!
//! let assets = ["21742633143463906290569050155826241533067272736897614950488156847949938836455"];
//! let mut backoff = Duration::from_secs(1);
//! loop {
//!     let mut delay = backoff;
//!     match PolyBoltChannel::connect().await {
//!         Ok(mut channel) => {
//!             channel.subscribe(PolyBoltSubscription::price_polymarket(assets))?;
//!             while let Some(event) = channel.next().await {
//!                 match event {
//!                     Ok(event) => {
//!                         backoff = Duration::from_secs(1);
//!                         println!("{event:?}");
//!                     }
//!                     Err(err) => match PolyBoltCloseCode::from_error(&err) {
//!                         // Reconnecting cannot help until the client is fixed.
//!                         Some(
//!                             PolyBoltCloseCode::AuthenticationFailed
//!                             | PolyBoltCloseCode::PolicyViolation,
//!                         ) => return Err(err),
//!                         // Documented: a uniformly random 0 to 10 s delay.
//!                         Some(PolyBoltCloseCode::Draining) => delay = Duration::from_secs(5),
//!                         // A decode error (the stream goes on) or a connection failure.
//!                         _ => eprintln!("{err}"),
//!                     },
//!                 }
//!             }
//!         }
//!         Err(err) => {
//!             eprintln!("failed to connect: {err}");
//!             // A handshake refused with HTTP 429 or 503 may say how long to wait.
//!             if let Some(retry_after) = err.retry_after() {
//!                 delay = delay.max(retry_after);
//!             }
//!         }
//!     }
//!     // Add random jitter to `delay` in real code.
//!     tokio::time::sleep(delay).await;
//!     backoff = (backoff * 2).min(Duration::from_secs(30));
//! }
//! # }
//! ```
//!
//! # Not supported yet
//!
//! The authenticated channels are not implemented: the CLOB user channel
//! (`/ws/user`), the RFQ quoter gateway, PolyBolt authentication (`auth`, `challenge`) and
//! the gated PolyBolt channels (`price.crypto`, `price.equity`, `price.crypto.twap`,
//! `price.equity.twap`).
//!
//! [`Stream`]: futures_core::Stream
//! [`FusedStream`]: futures_core::stream::FusedStream
//! [`Error::WebSocket`]: crate::Error::WebSocket
//! [`Error::Validation`]: crate::Error::Validation
//! [`WebSocketErrorKind::Decode`]: crate::WebSocketErrorKind::Decode
//! [`Closed`]: crate::WebSocketErrorKind::Closed
//! [`Protocol`]: crate::WebSocketErrorKind::Protocol
//! [`Send`]: crate::WebSocketErrorKind::Send
//! [`Connect`]: crate::WebSocketErrorKind::Connect
//! [`Timeout`]: crate::WebSocketErrorKind::Timeout

mod frame;
mod market;
mod polybolt;
mod sports;

pub use polyoxide_core::ws::{DEFAULT_BUFFER, DEFAULT_CONNECT_TIMEOUT};

pub use market::{
    BestBidAskEvent, BookEvent, EventMessage, FeeSchedule, LastTradePriceEvent, MarketChannel,
    MarketChannelBuilder, MarketChannelHandle, MarketEvent, MarketResolvedEvent,
    MarketSubscription, MarketSubscriptionUpdate, NewMarketEvent, OrderSummary, PriceChange,
    PriceChangeEvent, SubscriptionLevel, TickSizeChangeEvent,
};
pub use polybolt::{
    BestBidAsk, ChannelAck, ErrorAck, PolyBoltChannel, PolyBoltChannelBuilder,
    PolyBoltChannelHandle, PolyBoltChannelName, PolyBoltCloseCode, PolyBoltErrorCode,
    PolyBoltEvent, PolyBoltSubscription, PongAck, PricePolymarketEnvelope, PriceProvider,
};
pub use sports::{SportResult, SportsChannel, SportsChannelBuilder, SportsEvent};

/// Shared identifiers, re-exported here for discoverability (also available from
/// [`crate::types`]).
pub use crate::types::{EventId, MarketId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_types_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MarketChannel>();
        assert_send_sync::<SportsChannel>();
        assert_send_sync::<PolyBoltChannel>();
        assert_send_sync::<MarketEvent>();
        assert_send_sync::<SportsEvent>();
        assert_send_sync::<PolyBoltEvent>();
        assert_send_sync::<MarketChannelBuilder>();
        assert_send_sync::<PolyBoltSubscription>();
        assert_send_sync::<MarketChannelHandle>();
        assert_send_sync::<PolyBoltChannelHandle>();
    }
}
