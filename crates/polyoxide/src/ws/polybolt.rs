//! PolyBolt live data: `wss://ws-live-v2.polymarket.com/ws`, public `price.polymarket`
//! channel only.
//!
//! Spec: `docs/specs/polybolt-asyncapi.json`; pages: `docs/api-reference/wss/polybolt.md`,
//! `docs/api-reference/live-data/overview.md`,
//! `docs/api-reference/websockets/live-data-channel.md`.

use std::{
    collections::{HashSet, VecDeque},
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use futures_core::{Stream, stream::FusedStream};
use polyoxide_core::{
    Error, Result, Service, ValidationError, WebSocketError, WebSocketErrorKind, serde_util,
    types::{ConditionId, TokenId},
    ws::{WsConnection, WsSender},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::frame::{ConnectOptions, EventStream, IdleTimeout, str_field};

/// Documented limit of active `(channel, filter)` subscriptions per connection.
const MAX_ACTIVE_SUBSCRIPTIONS: usize = 64;
/// Documented limit of subscribe/unsubscribe frames per second.
const MAX_SUBSCRIPTION_FRAMES_PER_SECOND: usize = 20;
/// Documented frame size limit ("64 KB"), read as the stricter 64 000 bytes.
const MAX_FRAME_BYTES: usize = 64_000;
/// Documented maximum number of digits of an `asset_id` filter (`^[0-9]{1,78}$`).
const MAX_ASSET_ID_DIGITS: usize = 78;

polyoxide_core::string_enum! {
    /// A PolyBolt channel name.
    ///
    /// Only [`PricePolymarket`](Self::PricePolymarket) is public; the other channels
    /// require CLOB API credentials and are not supported by this crate yet. They are
    /// listed because acks and error acks may name them.
    ///
    /// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
    pub enum PolyBoltChannelName {
        /// `price.polymarket`: best bid and ask per outcome token (public).
        PricePolymarket => "price.polymarket",
        /// `price.crypto`: crypto reference prices (gated).
        PriceCrypto => "price.crypto",
        /// `price.equity`: equity, ETF, forex, metal and commodity reference prices (gated).
        PriceEquity => "price.equity",
        /// `price.crypto.twap`: crypto time-weighted average prices (gated).
        PriceCryptoTwap => "price.crypto.twap",
        /// `price.equity.twap`: forex time-weighted average prices (gated).
        PriceEquityTwap => "price.equity.twap",
    }
}

polyoxide_core::string_enum! {
    /// The price vendor named in a `subscribed` ack (`provider`). Only sent for the gated
    /// vendor channels while the server's provider selector is enabled.
    ///
    /// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
    pub enum PriceProvider {
        /// Pyth.
        Pyth => "pyth",
        /// Massive.
        Massive => "massive",
        /// Chainlink.
        Chainlink => "chainlink",
    }
}

polyoxide_core::string_enum! {
    /// The reason in an `error` ack (`code`).
    ///
    /// Error acks leave the connection open, except [`SubLimit`](Self::SubLimit),
    /// [`RateLimited`](Self::RateLimited) and [`AuthAttempts`](Self::AuthAttempts), which
    /// are followed by close code `4008` (see [`Self::closes_connection`]).
    ///
    /// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
    pub enum PolyBoltErrorCode {
        /// `bad_op`: malformed frame.
        BadOp => "bad_op",
        /// `bad_channel`: unknown channel.
        BadChannel => "bad_channel",
        /// `bad_filter`: missing or invalid filter.
        BadFilter => "bad_filter",
        /// `sub_limit`: a 65th subscription (followed by close `4008`).
        SubLimit => "sub_limit",
        /// `rate_limited`: more than 20 subscribe or unsubscribe frames per second
        /// (followed by close `4008`).
        RateLimited => "rate_limited",
        /// `auth_required`: a gated channel was subscribed before authenticating.
        AuthRequired => "auth_required",
        /// `auth_expired`: consumed or unknown ticket (web app only).
        AuthExpired => "auth_expired",
        /// `auth_unavailable`: the credential verifier is unreachable; retry later.
        AuthUnavailable => "auth_unavailable",
        /// `auth_invalid`: credentials refused.
        AuthInvalid => "auth_invalid",
        /// `auth_attempts`: a ninth auth frame (followed by close `4008`).
        AuthAttempts => "auth_attempts",
    }
}

impl PolyBoltErrorCode {
    /// `true` for the codes the documentation says are followed by close code `4008`.
    #[must_use]
    pub fn closes_connection(&self) -> bool {
        matches!(
            self,
            Self::SubLimit | Self::RateLimited | Self::AuthAttempts
        )
    }
}

/// A documented PolyBolt close code, for choosing a reconnection strategy.
///
/// See <https://docs.polymarket.com/api-reference/live-data/overview>.
///
/// ```
/// use polyoxide::ws::PolyBoltCloseCode;
///
/// assert_eq!(PolyBoltCloseCode::from_code(4003), PolyBoltCloseCode::Draining);
/// assert_eq!(PolyBoltCloseCode::Draining.code(), 4003);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl PolyBoltCloseCode {
    /// Classifies a close code.
    #[must_use]
    pub const fn from_code(code: u16) -> Self {
        match code {
            4001 => Self::AuthenticationFailed,
            4002 => Self::SlowConsumer,
            4003 => Self::Draining,
            4008 => Self::PolicyViolation,
            other => Self::Other(other),
        }
    }

    /// The numeric close code.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::AuthenticationFailed => 4001,
            Self::SlowConsumer => 4002,
            Self::Draining => 4003,
            Self::PolicyViolation => 4008,
            Self::Other(code) => code,
        }
    }

    /// The close code of an [`Error::WebSocket`] caused by the server closing the
    /// connection; `None` for any other error.
    #[must_use]
    pub fn from_error(error: &Error) -> Option<Self> {
        match error {
            Error::WebSocket(ws) => ws.close_code().map(Self::from_code),
            _ => None,
        }
    }
}

/// The `asset_id` filter of a `price.polymarket` subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AssetIdFilter {
    asset_id: TokenId,
}

/// One `(channel, filter)` pair (`SubscriptionItem`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SubscriptionItem {
    channel: PolyBoltChannelName,
    filter: AssetIdFilter,
}

impl SubscriptionItem {
    /// Checks the documented `asset_id` pattern `^[0-9]{1,78}$`.
    fn validate(&self) -> Result<()> {
        let id = self.filter.asset_id.as_str();
        if id.is_empty()
            || id.len() > MAX_ASSET_ID_DIGITS
            || !id.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ValidationError::new(
                "asset_id",
                format!("`{id}` is not an outcome token id (1 to 78 decimal digits)"),
            )
            .into());
        }
        Ok(())
    }

    /// The server's identity for this subscription: leading zeros of the asset id are
    /// normalized away.
    fn key(&self) -> (String, String) {
        let id = self.filter.asset_id.as_str().trim_start_matches('0');
        let id = if id.is_empty() { "0" } else { id };
        (self.channel.as_str().to_owned(), id.to_owned())
    }
}

/// A set of PolyBolt `(channel, filter)` subscriptions, sent as one batch frame by
/// [`PolyBoltChannel::subscribe`] or [`PolyBoltChannel::unsubscribe`].
///
/// Only the public `price.polymarket` channel is supported.
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
///
/// ```
/// use polyoxide::ws::PolyBoltSubscription;
///
/// let subscription = PolyBoltSubscription::price_polymarket([
///     "21742633143463906290569050155826241533067272736897614950488156847949938836455",
/// ])
/// .rid("s1");
/// # let _ = subscription;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct PolyBoltSubscription {
    items: Vec<SubscriptionItem>,
    rid: Option<String>,
}

impl PolyBoltSubscription {
    /// `price.polymarket` subscriptions for the given outcome token ids: best bid and ask
    /// per token.
    ///
    /// Each id must be 1 to 78 decimal digits; this is checked when the subscription is
    /// sent.
    pub fn price_polymarket<I>(asset_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        Self {
            items: asset_ids
                .into_iter()
                .map(|id| SubscriptionItem {
                    channel: PolyBoltChannelName::PricePolymarket,
                    filter: AssetIdFilter {
                        asset_id: id.into(),
                    },
                })
                .collect(),
            rid: None,
        }
    }

    /// A client request id (`rid`), echoed on every ack this frame produces.
    pub fn rid(mut self, rid: impl Into<String>) -> Self {
        self.rid = Some(rid.into());
        self
    }

    fn validate(&self) -> Result<()> {
        if self.items.is_empty() {
            return Err(ValidationError::new(
                "subscriptions",
                "at least one subscription is required",
            )
            .into());
        }
        self.items.iter().try_for_each(SubscriptionItem::validate)
    }

    fn to_json(&self, op: &'static str) -> Result<String> {
        #[derive(Serialize)]
        struct Wire<'a> {
            op: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            rid: Option<&'a str>,
            subscriptions: &'a [SubscriptionItem],
        }
        encode(&Wire {
            op,
            rid: self.rid.as_deref(),
            subscriptions: &self.items,
        })
    }
}

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|e| {
        Error::WebSocket(Box::new(
            WebSocketError::new(
                Service::PolyBolt,
                WebSocketErrorKind::Send,
                "failed to encode request",
            )
            .with_source(e),
        ))
    })
}

/// A live connection to the PolyBolt live data socket, for the public `price.polymarket`
/// channel (best bid and ask per outcome token).
///
/// The channel is a [`Stream`] of [`PolyBoltEvent`]s: acks for each request, then one
/// snapshot per new subscription followed by live updates. The gated channels
/// (`price.crypto`, `price.equity`, `price.crypto.twap`, `price.equity.twap`) and the
/// `auth` operation are not supported.
///
/// The server keeps the connection alive with protocol-level pings every 25 seconds, which
/// are answered automatically. [`ping`](Self::ping) sends the optional application-level
/// ping. If nothing at all arrives for [`DEFAULT_IDLE_TIMEOUT`](Self::DEFAULT_IDLE_TIMEOUT)
/// (configurable with [`PolyBoltChannelBuilder::idle_timeout`]), the connection is
/// considered dead and the stream ends with an error.
///
/// The documented per-connection limits are enforced client-side, returning
/// [`Error::Validation`] instead of letting the server close the connection with `4008`:
/// at most 64 active subscriptions (counted from the requests sent on this connection), at
/// most 20 subscribe or unsubscribe frames per second (best effort: network jitter can
/// still bunch frames up; batch many ids into one frame instead), and at most 64 000 bytes
/// per frame.
///
/// To subscribe or unsubscribe from another task while the stream is consumed, use a
/// [`PolyBoltChannelHandle`] from [`handle`](Self::handle); the channel and its handles
/// share the limit accounting.
///
/// Note: the PolyBolt overview page says subscriptions require CLOB API credentials,
/// while the AsyncAPI spec marks `price.polymarket` as public (no auth). This type follows
/// the spec.
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt> and the [module
/// documentation](super) for error handling and reconnection.
///
/// ```no_run
/// # async fn run() -> polyoxide::Result<()> {
/// use futures_util::StreamExt as _;
/// use polyoxide::ws::{PolyBoltChannel, PolyBoltEvent, PolyBoltSubscription};
///
/// let mut channel = PolyBoltChannel::connect().await?;
/// channel.subscribe(PolyBoltSubscription::price_polymarket([
///     "21742633143463906290569050155826241533067272736897614950488156847949938836455",
/// ]))?;
/// while let Some(event) = channel.next().await {
///     match event? {
///         PolyBoltEvent::PricePolymarket(envelope) => {
///             if let Some(quote) = envelope.payload {
///                 println!("{} / {}", quote.best_bid, quote.best_ask);
///             }
///         }
///         PolyBoltEvent::Error(error) => eprintln!("request failed: {:?}", error.code),
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Stream items
///
/// Each item is an `Ok(`[`PolyBoltEvent`]`)` or an
/// `Err(`[`Error::WebSocket`](crate::Error::WebSocket)`)`. Check the error's
/// [`kind`](crate::WebSocketError::kind):
///
/// - [`WebSocketErrorKind::Decode`](crate::WebSocketErrorKind::Decode): a message did not
///   match its documented schema. **Not fatal**: the stream continues with the next
///   message.
/// - Any other kind is terminal: the connection failed or was closed abnormally, and the
///   stream ends after this item.
///
/// A normal close ends the stream without an error.
///
/// [`Stream`]: futures_core::Stream
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct PolyBoltChannel {
    events: EventStream<PolyBoltEvent>,
    handle: PolyBoltChannelHandle,
}

impl PolyBoltChannel {
    /// The production URL.
    pub const DEFAULT_URL: &'static str = "wss://ws-live-v2.polymarket.com/ws";

    /// The default idle timeout: three times the documented 25-second interval of the
    /// server's protocol-level pings (see [`PolyBoltChannelBuilder::idle_timeout`]).
    pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(75);

    /// Connects to the production URL. Nothing is subscribed until
    /// [`subscribe`](Self::subscribe) is called.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the connection cannot be established.
    pub async fn connect() -> Result<Self> {
        Self::builder().connect().await
    }

    /// Returns a builder for a custom URL, buffer size or timeouts.
    pub fn builder() -> PolyBoltChannelBuilder {
        PolyBoltChannelBuilder::default()
    }

    /// A cloneable handle to subscribe, unsubscribe, ping or close from other tasks while
    /// this channel is consumed as a stream.
    ///
    /// The channel and all its handles share one count of active subscriptions and one
    /// frame rate budget, so the documented per-connection limits hold however the
    /// requests are spread across tasks.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use futures_util::StreamExt as _;
    /// use polyoxide::ws::{PolyBoltChannel, PolyBoltSubscription};
    ///
    /// let mut channel = PolyBoltChannel::connect().await?;
    /// let handle = channel.handle();
    /// tokio::spawn(async move {
    ///     // For example, in response to user input:
    ///     let asset = "21742633143463906290569050155826241533067272736897614950488156847949938836455";
    ///     if let Err(err) = handle.subscribe(PolyBoltSubscription::price_polymarket([asset])) {
    ///         eprintln!("subscribe failed: {err}");
    ///     }
    /// });
    /// while let Some(event) = channel.next().await {
    ///     println!("{:?}", event?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn handle(&self) -> PolyBoltChannelHandle {
        self.handle.clone()
    }

    /// Adds subscriptions (`op: "subscribe"`, batch form).
    ///
    /// Each newly added subscription is acknowledged with
    /// [`PolyBoltEvent::Subscribed`], then receives one snapshot envelope, then live
    /// updates. Subscribing again to an active subscription is acknowledged again without
    /// a new snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the subscription is empty, an asset id is not 1 to
    /// 78 decimal digits, the connection would exceed 64 active subscriptions, the frame
    /// would exceed 64 000 bytes, or 20 subscribe/unsubscribe frames were already sent in
    /// the last second; [`Error::WebSocket`] if the connection has terminated.
    pub fn subscribe(&self, subscription: PolyBoltSubscription) -> Result<()> {
        self.handle.subscribe(subscription)
    }

    /// Removes subscriptions (`op: "unsubscribe"`, batch form); acknowledged with
    /// [`PolyBoltEvent::Unsubscribed`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the subscription is empty, an asset id is not 1 to
    /// 78 decimal digits, the frame would exceed 64 000 bytes, or 20 subscribe/unsubscribe
    /// frames were already sent in the last second; [`Error::WebSocket`] if the connection
    /// has terminated.
    pub fn unsubscribe(&self, subscription: PolyBoltSubscription) -> Result<()> {
        self.handle.unsubscribe(subscription)
    }

    /// Sends the optional application-level ping (`op: "ping"`); answered with
    /// [`PolyBoltEvent::Pong`]. Not needed for keepalive.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the connection has terminated.
    pub fn ping(&self) -> Result<()> {
        self.handle.ping()
    }

    /// Like [`ping`](Self::ping), with a request id (`rid`) echoed on the pong.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the frame would exceed 64 000 bytes, or
    /// [`Error::WebSocket`] if the connection has terminated.
    pub fn ping_with_rid(&self, rid: impl Into<String>) -> Result<()> {
        self.handle.ping_with_rid(rid)
    }

    /// The number of distinct subscriptions requested and not unsubscribed on this
    /// connection (through the channel or any of its handles), as counted for the
    /// 64-subscription limit.
    ///
    /// This is an upper bound of what the server holds: subscriptions it rejected (see
    /// [`PolyBoltEvent::Error`]) are still counted until unsubscribed.
    #[must_use]
    pub fn active_subscriptions(&self) -> usize {
        self.handle.active_subscriptions()
    }

    /// Closes the connection gracefully; the stream then ends after any events already
    /// received.
    pub fn close(&self) {
        self.handle.close();
    }
}

/// A cloneable handle to a [`PolyBoltChannel`], obtained from [`PolyBoltChannel::handle`]:
/// subscribes, unsubscribes, pings or closes the connection from any task while the
/// channel is consumed as a stream.
///
/// The channel and all its handles share the client-side accounting of the documented
/// per-connection limits (64 active subscriptions, 20 subscribe or unsubscribe frames per
/// second), so concurrent requests from several tasks cannot exceed them together.
///
/// The handle does not keep the connection open: once the [`PolyBoltChannel`] is dropped
/// or the connection has ended, sending fails with an [`Error::WebSocket`] of kind
/// [`Closed`](crate::WebSocketErrorKind::Closed).
#[derive(Debug, Clone)]
pub struct PolyBoltChannelHandle {
    sender: WsSender,
    limits: Arc<Mutex<Limits>>,
}

impl PolyBoltChannelHandle {
    /// Adds subscriptions; see [`PolyBoltChannel::subscribe`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the subscription is empty, an asset id is not 1 to
    /// 78 decimal digits, the connection would exceed 64 active subscriptions, the frame
    /// would exceed 64 000 bytes, or 20 subscribe/unsubscribe frames were already sent in
    /// the last second; [`Error::WebSocket`] if the connection has terminated.
    pub fn subscribe(&self, subscription: PolyBoltSubscription) -> Result<()> {
        subscription.validate()?;
        let frame = subscription.to_json("subscribe")?;
        lock(&self.limits).subscribe(&subscription, frame, |frame| self.sender.send_text(frame))
    }

    /// Removes subscriptions; see [`PolyBoltChannel::unsubscribe`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the subscription is empty, an asset id is not 1 to
    /// 78 decimal digits, the frame would exceed 64 000 bytes, or 20 subscribe/unsubscribe
    /// frames were already sent in the last second; [`Error::WebSocket`] if the connection
    /// has terminated.
    pub fn unsubscribe(&self, subscription: PolyBoltSubscription) -> Result<()> {
        subscription.validate()?;
        let frame = subscription.to_json("unsubscribe")?;
        lock(&self.limits).unsubscribe(&subscription, frame, |frame| self.sender.send_text(frame))
    }

    /// Sends the optional application-level ping; see [`PolyBoltChannel::ping`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the connection has terminated.
    pub fn ping(&self) -> Result<()> {
        self.sender.send_text(r#"{"op":"ping"}"#)
    }

    /// Like [`ping`](Self::ping), with a request id (`rid`) echoed on the pong.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the frame would exceed 64 000 bytes, or
    /// [`Error::WebSocket`] if the connection has terminated.
    pub fn ping_with_rid(&self, rid: impl Into<String>) -> Result<()> {
        #[derive(Serialize)]
        struct Wire {
            op: &'static str,
            rid: String,
        }
        let frame = encode(&Wire {
            op: "ping",
            rid: rid.into(),
        })?;
        check_frame_size(&frame)?;
        self.sender.send_text(frame)
    }

    /// The number of active subscriptions counted for the connection; see
    /// [`PolyBoltChannel::active_subscriptions`].
    #[must_use]
    pub fn active_subscriptions(&self) -> usize {
        lock(&self.limits).active.len()
    }

    /// Closes the connection gracefully; the channel's stream then ends after any events
    /// already received.
    pub fn close(&self) {
        self.sender.close();
    }
}

/// The client-side accounting of the documented per-connection limits, shared by a
/// [`PolyBoltChannel`] and its handles.
#[derive(Debug, Default)]
struct Limits {
    /// Subscription keys (see [`SubscriptionItem::key`]) requested and not unsubscribed.
    active: HashSet<(String, String)>,
    /// When the subscribe/unsubscribe frames of the last second were sent.
    recent_frames: VecDeque<Instant>,
}

impl Limits {
    /// Enforces the subscription count, then sends `frame` (see
    /// [`send_subscription_frame`](Self::send_subscription_frame)) and records the
    /// subscriptions.
    fn subscribe(
        &mut self,
        subscription: &PolyBoltSubscription,
        frame: String,
        send: impl FnOnce(String) -> Result<()>,
    ) -> Result<()> {
        let keys: Vec<_> = subscription
            .items
            .iter()
            .map(SubscriptionItem::key)
            .collect();
        let added: HashSet<_> = keys.iter().filter(|k| !self.active.contains(*k)).collect();
        let after = self.active.len().saturating_add(added.len());
        if after > MAX_ACTIVE_SUBSCRIPTIONS {
            return Err(ValidationError::new(
                "subscriptions",
                format!(
                    "a connection allows at most {MAX_ACTIVE_SUBSCRIPTIONS} active subscriptions; \
                     this request would make {after}"
                ),
            )
            .into());
        }
        self.send_subscription_frame(frame, send)?;
        self.active.extend(keys);
        Ok(())
    }

    /// Sends `frame` (see [`send_subscription_frame`](Self::send_subscription_frame)) and
    /// forgets the subscriptions.
    fn unsubscribe(
        &mut self,
        subscription: &PolyBoltSubscription,
        frame: String,
        send: impl FnOnce(String) -> Result<()>,
    ) -> Result<()> {
        self.send_subscription_frame(frame, send)?;
        for key in subscription.items.iter().map(SubscriptionItem::key) {
            self.active.remove(&key);
        }
        Ok(())
    }

    /// Enforces the frame size and rate limits, then sends a subscribe/unsubscribe frame
    /// with `send` and records it for the rate limit.
    fn send_subscription_frame(
        &mut self,
        frame: String,
        send: impl FnOnce(String) -> Result<()>,
    ) -> Result<()> {
        check_frame_size(&frame)?;
        let now = Instant::now();
        while self
            .recent_frames
            .front()
            .is_some_and(|sent| now.duration_since(*sent) >= Duration::from_secs(1))
        {
            self.recent_frames.pop_front();
        }
        if self.recent_frames.len() >= MAX_SUBSCRIPTION_FRAMES_PER_SECOND {
            return Err(ValidationError::new(
                "op",
                format!(
                    "at most {MAX_SUBSCRIPTION_FRAMES_PER_SECOND} subscribe or unsubscribe frames \
                     may be sent per second; batch subscriptions or retry later"
                ),
            )
            .into());
        }
        send(frame)?;
        self.recent_frames.push_back(now);
        Ok(())
    }
}

/// Locks the shared limits. A poisoned lock (a panic in another thread while it held the
/// lock) is recovered rather than propagated: the accounting is only updated after each
/// check and send succeeds, so it is never left half-updated.
fn lock(limits: &Mutex<Limits>) -> MutexGuard<'_, Limits> {
    limits.lock().unwrap_or_else(PoisonError::into_inner)
}

fn check_frame_size(frame: &str) -> Result<()> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err(ValidationError::new(
            "frame",
            format!(
                "the request is {} bytes; frames are limited to {MAX_FRAME_BYTES} bytes",
                frame.len()
            ),
        )
        .into());
    }
    Ok(())
}

impl Stream for PolyBoltChannel {
    type Item = Result<PolyBoltEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().events.poll_next(cx)
    }
}

impl FusedStream for PolyBoltChannel {
    fn is_terminated(&self) -> bool {
        self.events.is_terminated()
    }
}

/// Builder for [`PolyBoltChannel`].
#[derive(Debug, Clone)]
#[must_use]
pub struct PolyBoltChannelBuilder {
    options: ConnectOptions,
}

impl Default for PolyBoltChannelBuilder {
    fn default() -> Self {
        Self {
            options: ConnectOptions::new(),
        }
    }
}

impl PolyBoltChannelBuilder {
    /// Overrides the URL (default [`PolyBoltChannel::DEFAULT_URL`]), e.g. to target a mock
    /// server in tests. Must use `ws://` or `wss://`.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.options.url = Some(url.into());
        self
    }

    /// Sets how many received frames may be buffered before the connection stops reading
    /// from the socket (default [`DEFAULT_BUFFER`](super::DEFAULT_BUFFER); see
    /// [back-pressure](super#back-pressure)).
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.options.buffer = Some(buffer);
        self
    }

    /// Sets the handshake timeout (default
    /// [`DEFAULT_CONNECT_TIMEOUT`](super::DEFAULT_CONNECT_TIMEOUT)).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.options.connect_timeout = Some(timeout);
        self
    }

    /// Sets the idle timeout (default [`PolyBoltChannel::DEFAULT_IDLE_TIMEOUT`]).
    ///
    /// If no frame of any kind (data, heartbeat or heartbeat reply) arrives for this long,
    /// the stream yields a final [`Error::WebSocket`](crate::Error::WebSocket) of kind
    /// [`Timeout`](crate::WebSocketErrorKind::Timeout) and ends, so that a connection that
    /// died without closing is noticed. The timer does not run while the receive buffer is
    /// full (see [back-pressure](super#back-pressure)). A zero timeout makes `connect` fail
    /// with [`Error::Config`](crate::Error::Config).
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.options.idle_timeout = IdleTimeout::After(timeout);
        self
    }

    /// Disables the idle timeout. A connection that dies without closing then leaves the
    /// stream pending until the operating system reports the connection as broken, which
    /// may never happen.
    pub fn no_idle_timeout(mut self) -> Self {
        self.options.idle_timeout = IdleTimeout::Disabled;
        self
    }

    /// Connects.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the URL is invalid or the idle timeout is zero, or
    /// [`Error::WebSocket`] if the connection cannot be established. For a refused
    /// handshake (e.g. HTTP `429` or `503`), the error's
    /// [`http_status`](crate::WebSocketError::http_status) and
    /// [`retry_after`](crate::WebSocketError::retry_after) (also
    /// [`Error::retry_after`]) give the status and the `Retry-After` delay.
    pub async fn connect(self) -> Result<PolyBoltChannel> {
        let config = self.options.config(
            Service::PolyBolt,
            PolyBoltChannel::DEFAULT_URL,
            PolyBoltChannel::DEFAULT_IDLE_TIMEOUT,
        )?;
        let conn = WsConnection::connect(config).await?;
        let handle = PolyBoltChannelHandle {
            sender: conn.sender(),
            limits: Arc::default(),
        };
        Ok(PolyBoltChannel {
            events: EventStream::new(conn),
            handle,
        })
    }
}

/// A message received from PolyBolt.
///
/// Acks are recognised by `op`, data envelopes by `channel`. Anything else, including acks
/// for operations this crate does not send (`authed`, `challenge`), envelopes of the gated
/// channels and frames that are not JSON, becomes [`PolyBoltEvent::Unknown`] so that new
/// server messages never break the stream.
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[non_exhaustive]
pub enum PolyBoltEvent {
    /// `subscribed`: a subscription was accepted.
    Subscribed(ChannelAck),
    /// `unsubscribed`: an unsubscribe request was accepted.
    Unsubscribed(ChannelAck),
    /// `pong`: the answer to an application-level ping.
    Pong(PongAck),
    /// `error`: a request failed. The connection stays open unless
    /// [`PolyBoltErrorCode::closes_connection`].
    Error(ErrorAck),
    /// A `price.polymarket` data envelope (snapshot or live update).
    #[serde(untagged)]
    PricePolymarket(PricePolymarketEnvelope),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    #[serde(untagged)]
    Unknown(Value),
}

impl<'de> Deserialize<'de> for PolyBoltEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        fn parse<T: DeserializeOwned, E: serde::de::Error>(
            kind: &str,
            value: &Value,
        ) -> std::result::Result<T, E> {
            T::deserialize(value).map_err(|e| E::custom(format!("invalid `{kind}` message: {e}")))
        }

        let value = Value::deserialize(deserializer)?;
        let event = match (str_field(&value, "op"), str_field(&value, "channel")) {
            (Some(kind @ "subscribed"), _) => Self::Subscribed(parse(kind, &value)?),
            (Some(kind @ "unsubscribed"), _) => Self::Unsubscribed(parse(kind, &value)?),
            (Some(kind @ "pong"), _) => Self::Pong(parse(kind, &value)?),
            (Some(kind @ "error"), _) => Self::Error(parse(kind, &value)?),
            (None, Some(kind @ "price.polymarket")) => Self::PricePolymarket(parse(kind, &value)?),
            _ => Self::Unknown(value),
        };
        Ok(event)
    }
}

/// A `subscribed` or `unsubscribed` ack (`ChannelAck`).
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// A `pong` ack (`SimpleAck`).
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PongAck {
    /// The request id, echoed when the ping carried one.
    pub rid: Option<String>,
}

/// An `error` ack (`ErrorAck`).
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// A `price.polymarket` data envelope (`PricePolymarketEnvelope`).
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PricePolymarketEnvelope {
    /// Envelope version (`1`).
    pub v: u32,
    /// The channel (`price.polymarket`).
    pub channel: PolyBoltChannelName,
    /// Dense per-connection, per-channel sequence number, assigned at delivery. Snapshot
    /// frames consume one; it restarts on every reconnect.
    pub seq: u64,
    /// Producer event time (sent as Unix milliseconds).
    #[serde(with = "serde_util::timestamp_millis")]
    pub ts: DateTime<Utc>,
    /// `Some(true)` on the one snapshot frame sent after each new subscription; absent on
    /// live frames. See [`Self::is_snapshot`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
    /// Number of frames dropped on this channel since the last delivered frame because the
    /// client fell behind. Absent when zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropped: Option<u64>,
    /// The best bid and ask. `None` for a snapshot of a token that has not traded within
    /// the server's warm window (sent as an empty array `[]`).
    #[serde(with = "cold_payload")]
    pub payload: Option<BestBidAsk>,
}

impl PricePolymarketEnvelope {
    /// `true` for the snapshot frame that follows a new subscription.
    #[must_use]
    pub fn is_snapshot(&self) -> bool {
        self.snapshot == Some(true)
    }
}

/// The best bid and ask of an outcome token (`BestBidAsk`).
///
/// See <https://docs.polymarket.com/api-reference/wss/polybolt>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BestBidAsk {
    /// Condition id of the market.
    pub market: ConditionId,
    /// Outcome token id.
    pub asset_id: TokenId,
    /// Best bid price.
    pub best_bid: Decimal,
    /// Best ask price.
    pub best_ask: Decimal,
    /// Order book hash at this change.
    pub hash: String,
    /// Order book time (sent as Unix milliseconds).
    #[serde(with = "serde_util::timestamp_millis")]
    pub timestamp: DateTime<Utc>,
}

/// A [`BestBidAsk`] object, or an empty array for a cold snapshot.
mod cold_payload {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
    use serde_json::Value;

    use super::BestBidAsk;

    pub(super) fn serialize<S: Serializer>(
        value: &Option<BestBidAsk>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(quote) => quote.serialize(serializer),
            None => serializer.collect_seq(std::iter::empty::<u8>()),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<BestBidAsk>, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Array(items) if items.is_empty() => Ok(None),
            Value::Array(_) => Err(D::Error::custom(
                "expected a best bid/ask object or an empty array, found a non-empty array",
            )),
            other => BestBidAsk::deserialize(other)
                .map(Some)
                .map_err(D::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSET: &str =
        "21742633143463906290569050155826241533067272736897614950488156847949938836455";

    fn event(json: &str) -> PolyBoltEvent {
        serde_json::from_str(json).unwrap()
    }

    fn ids(range: std::ops::RangeInclusive<u32>) -> PolyBoltSubscription {
        PolyBoltSubscription::price_polymarket(range.map(|i| i.to_string()))
    }

    #[test]
    fn limits_count_subscriptions_and_frames() {
        let mut limits = Limits::default();
        let mut sent = Vec::new();
        let mut send = |frame: String| {
            sent.push(frame);
            Ok(())
        };
        limits
            .subscribe(&ids(1..=64), "a".to_owned(), &mut send)
            .unwrap();
        assert_eq!(limits.active.len(), 64);
        let err = limits
            .subscribe(&ids(65..=65), "b".to_owned(), &mut send)
            .unwrap_err();
        assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "subscriptions"));
        limits
            .unsubscribe(&ids(1..=1), "c".to_owned(), &mut send)
            .unwrap();
        limits
            .subscribe(&ids(65..=65), "d".to_owned(), &mut send)
            .unwrap();
        assert_eq!(limits.active.len(), 64);
        // A frame that fails to send is not counted (`2` is already active, so only the send
        // can fail).
        let err = limits
            .subscribe(&ids(2..=2), "e".to_owned(), |_| {
                Err(Error::Validation(ValidationError::new("x", "send failed")))
            })
            .unwrap_err();
        assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "x"));
        assert_eq!(limits.recent_frames.len(), 3);
        // Nor are the subscriptions of a frame that fails to send.
        limits
            .unsubscribe(&ids(2..=2), "f".to_owned(), &mut send)
            .unwrap();
        let err = limits
            .subscribe(&ids(2..=2), "g".to_owned(), |_| {
                Err(Error::Validation(ValidationError::new("x", "send failed")))
            })
            .unwrap_err();
        assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "x"));
        assert_eq!(limits.active.len(), 63);
        assert_eq!(limits.recent_frames.len(), 4);
        // Rejected requests are not sent.
        assert_eq!(sent, ["a", "c", "d", "f"]);
    }

    #[test]
    fn limits_survive_a_poisoned_lock() {
        let limits = Arc::new(Mutex::new(Limits::default()));
        let poisoner = Arc::clone(&limits);
        let panicked = std::thread::spawn(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("poisoning the lock on purpose");
        })
        .join();
        assert!(panicked.is_err());
        assert!(limits.is_poisoned());

        lock(&limits)
            .subscribe(&ids(1..=2), "frame".to_owned(), |_| Ok(()))
            .unwrap();
        assert_eq!(lock(&limits).active.len(), 2);
    }

    // Requests: `components/messages/{subscribe,unsubscribe,ping}` examples in
    // `docs/specs/polybolt-asyncapi.json` (the price.polymarket item of the batch example).

    #[test]
    fn serializes_batch_subscribe() {
        let frame = PolyBoltSubscription::price_polymarket([ASSET])
            .rid("s1")
            .to_json("subscribe")
            .unwrap();
        assert_eq!(
            frame,
            format!(
                r#"{{"op":"subscribe","rid":"s1","subscriptions":[{{"channel":"price.polymarket","filter":{{"asset_id":"{ASSET}"}}}}]}}"#
            )
        );
        let frame = PolyBoltSubscription::price_polymarket([ASSET])
            .to_json("unsubscribe")
            .unwrap();
        assert_eq!(
            frame,
            format!(
                r#"{{"op":"unsubscribe","subscriptions":[{{"channel":"price.polymarket","filter":{{"asset_id":"{ASSET}"}}}}]}}"#
            )
        );
    }

    #[test]
    fn validates_subscriptions() {
        assert!(matches!(
            PolyBoltSubscription::price_polymarket(Vec::<TokenId>::new()).validate(),
            Err(Error::Validation(_))
        ));
        for bad in ["", "0x12", "12a", &"1".repeat(79)] {
            assert!(
                matches!(
                    PolyBoltSubscription::price_polymarket([bad]).validate(),
                    Err(Error::Validation(_))
                ),
                "{bad:?}"
            );
        }
        assert!(
            PolyBoltSubscription::price_polymarket([ASSET, "0", &"9".repeat(78)])
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn keys_normalize_leading_zeros() {
        let items = PolyBoltSubscription::price_polymarket(["007", "7", "000", "0"]).items;
        let keys: HashSet<_> = items.iter().map(SubscriptionItem::key).collect();
        assert_eq!(keys.len(), 2);
    }

    // Server messages: `components/messages/*` examples in
    // `docs/specs/polybolt-asyncapi.json`.

    #[test]
    fn deserializes_acks() {
        assert_eq!(
            event(r#"{"op":"subscribed","channel":"price.crypto","rid":"s1"}"#),
            PolyBoltEvent::Subscribed(ChannelAck {
                channel: PolyBoltChannelName::PriceCrypto,
                rid: Some("s1".to_owned()),
                provider: None,
            })
        );
        assert_eq!(
            event(
                r#"{"op":"subscribed","channel":"price.equity","rid":"s2","provider":"chainlink"}"#
            ),
            PolyBoltEvent::Subscribed(ChannelAck {
                channel: PolyBoltChannelName::PriceEquity,
                rid: Some("s2".to_owned()),
                provider: Some(PriceProvider::Chainlink),
            })
        );
        assert_eq!(
            event(r#"{"op":"unsubscribed","channel":"price.crypto"}"#),
            PolyBoltEvent::Unsubscribed(ChannelAck {
                channel: PolyBoltChannelName::PriceCrypto,
                rid: None,
                provider: None,
            })
        );
        assert_eq!(
            event(r#"{"op":"pong","rid":"p1"}"#),
            PolyBoltEvent::Pong(PongAck {
                rid: Some("p1".to_owned())
            })
        );
    }

    #[test]
    fn deserializes_error_acks() {
        let PolyBoltEvent::Error(err) =
            event(r#"{"op":"error","code":"auth_required","channel":"price.crypto","rid":"s1"}"#)
        else {
            panic!("expected an error ack");
        };
        assert_eq!(err.code, PolyBoltErrorCode::AuthRequired);
        assert!(!err.code.closes_connection());
        assert_eq!(err.channel, Some(PolyBoltChannelName::PriceCrypto));
        assert_eq!(err.rid.as_deref(), Some("s1"));

        let PolyBoltEvent::Error(err) =
            event(r#"{"op":"error","code":"bad_filter","channel":"price.polymarket","rid":"s2"}"#)
        else {
            panic!("expected an error ack");
        };
        assert_eq!(err.code, PolyBoltErrorCode::BadFilter);

        let PolyBoltEvent::Error(err) = event(r#"{"op":"error","code":"sub_limit"}"#) else {
            panic!("expected an error ack");
        };
        assert!(err.code.closes_connection());
        assert_eq!(err.channel, None);

        let PolyBoltEvent::Error(err) = event(r#"{"op":"error","code":"brand_new"}"#) else {
            panic!("expected an error ack");
        };
        assert_eq!(err.code, PolyBoltErrorCode::Unknown("brand_new".to_owned()));
    }

    #[test]
    fn deserializes_price_polymarket_envelopes() {
        let PolyBoltEvent::PricePolymarket(snapshot) = event(
            r#"{
              "v": 1,
              "channel": "price.polymarket",
              "seq": 1,
              "ts": 1788973000123,
              "snapshot": true,
              "payload": {
                "market": "0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c",
                "asset_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
                "best_bid": "0.51",
                "best_ask": "0.53",
                "hash": "3f9c1e7a",
                "timestamp": 1788972999871
              }
            }"#,
        ) else {
            panic!("expected an envelope");
        };
        assert_eq!(snapshot.v, 1);
        assert_eq!(snapshot.channel, PolyBoltChannelName::PricePolymarket);
        assert_eq!(snapshot.seq, 1);
        assert_eq!(snapshot.ts.timestamp_millis(), 1_788_973_000_123);
        assert!(snapshot.is_snapshot());
        assert_eq!(snapshot.dropped, None);
        let quote = snapshot.payload.unwrap();
        assert_eq!(
            quote.market,
            "0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c"
        );
        assert_eq!(quote.asset_id, ASSET);
        assert_eq!(quote.best_bid, Decimal::new(51, 2));
        assert_eq!(quote.best_ask, Decimal::new(53, 2));
        assert_eq!(quote.hash, "3f9c1e7a");
        assert_eq!(quote.timestamp.timestamp_millis(), 1_788_972_999_871);

        let PolyBoltEvent::PricePolymarket(cold) = event(
            r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1788973000123,"snapshot":true,"payload":[]}"#,
        ) else {
            panic!("expected an envelope");
        };
        assert!(cold.is_snapshot());
        assert_eq!(cold.payload, None);

        let PolyBoltEvent::PricePolymarket(live) = event(
            r#"{"v":1,"channel":"price.polymarket","seq":2,"ts":1788973004501,"payload":{"market":"0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c","asset_id":"21742633143463906290569050155826241533067272736897614950488156847949938836455","best_bid":"0.52","best_ask":"0.53","hash":"a81d02c4","timestamp":1788973004498}}"#,
        ) else {
            panic!("expected an envelope");
        };
        assert!(!live.is_snapshot());
        assert_eq!(live.snapshot, None);
        assert_eq!(live.payload.unwrap().best_bid, Decimal::new(52, 2));

        let PolyBoltEvent::PricePolymarket(dropped) = event(
            r#"{"v":1,"channel":"price.polymarket","seq":9,"ts":1,"dropped":3,"payload":[]}"#,
        ) else {
            panic!("expected an envelope");
        };
        assert_eq!(dropped.dropped, Some(3));
    }

    #[test]
    fn rejects_malformed_envelopes() {
        for bad in [
            r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1,"payload":[1]}"#,
            r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1}"#,
            r#"{"op":"error"}"#,
        ] {
            assert!(serde_json::from_str::<PolyBoltEvent>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn other_messages_are_unknown() {
        for raw in [
            r#"{"op":"authed","rid":"a1"}"#,
            r#"{"op":"challenge","nonce":"3f2b6c1e-9a4d-4c2b-8e1f-0a1b2c3d4e5f","rid":"c1"}"#,
            r#"{"v":1,"channel":"price.crypto","seq":2,"ts":1788973001000,"payload":{"symbol":"btcusd"}}"#,
            r#"{"something":"else"}"#,
        ] {
            assert_eq!(
                event(raw),
                PolyBoltEvent::Unknown(serde_json::from_str(raw).unwrap()),
                "{raw}"
            );
        }
    }

    #[test]
    fn serializes_events_back_to_the_wire_shape() {
        for raw in [
            r#"{"op":"subscribed","channel":"price.polymarket","rid":"s1","provider":null}"#,
            r#"{"op":"pong","rid":null}"#,
            r#"{"op":"error","code":"bad_filter","channel":"price.polymarket","rid":"s2"}"#,
            r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1788973000123,"snapshot":true,"payload":[]}"#,
            r#"{"v":1,"channel":"price.polymarket","seq":2,"ts":1788973004501,"payload":{"market":"0x9d","asset_id":"1","best_bid":"0.52","best_ask":"0.53","hash":"a81d02c4","timestamp":1788973004498}}"#,
            r#"{"op":"authed","rid":"a1"}"#,
        ] {
            assert_eq!(serde_json::to_string(&event(raw)).unwrap(), raw);
        }
    }

    #[test]
    fn close_codes() {
        for (code, expected) in [
            (4001, PolyBoltCloseCode::AuthenticationFailed),
            (4002, PolyBoltCloseCode::SlowConsumer),
            (4003, PolyBoltCloseCode::Draining),
            (4008, PolyBoltCloseCode::PolicyViolation),
            (1011, PolyBoltCloseCode::Other(1011)),
        ] {
            assert_eq!(PolyBoltCloseCode::from_code(code), expected);
            assert_eq!(expected.code(), code);
        }
        let err = Error::WebSocket(Box::new(
            WebSocketError::new(Service::PolyBolt, WebSocketErrorKind::Closed, "closed")
                .with_close(4008, "policy"),
        ));
        assert_eq!(
            PolyBoltCloseCode::from_error(&err),
            Some(PolyBoltCloseCode::PolicyViolation)
        );
        let err = Error::Validation(ValidationError::new("x", "y"));
        assert_eq!(PolyBoltCloseCode::from_error(&err), None);
    }
}
