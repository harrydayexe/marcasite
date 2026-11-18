//! An opt-in, auto-reconnecting wrapper around the CLOB market channel.
//!
//! Spec: `docs/polymarket/specs/asyncapi.json`; page: `docs/polymarket/api-reference/wss/market.md`.
//! The market channel documentation specifies no reconnection policy; this is a client-side
//! convenience layered on [`MarketChannel`].

use std::{
    collections::BTreeSet,
    future::Future,
    num::NonZeroU32,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    task::{Context, Poll, Waker},
    time::Duration,
};

use futures_core::{Stream, stream::FusedStream};
use marcasite_core::{
    ConfigError, Error, Result, Service, ValidationError, WebSocketError, WebSocketErrorKind,
    types::TokenId,
};
use tokio::time::{Instant, Sleep};

use super::market::{
    MarketChannel, MarketChannelBuilder, MarketChannelHandle, MarketEvent, MarketSubscription,
    MarketSubscriptionUpdate, Operation, SubscriptionLevel,
};

/// How a [`ReconnectingMarketChannel`] waits between connection attempts.
///
/// The delay before each attempt grows exponentially (doubling) from
/// [`initial_backoff`](Self::initial_backoff) up to [`max_backoff`](Self::max_backoff). It is
/// deterministic: there is no jitter.
///
/// # Attempts and stability
///
/// The policy counts connection *attempts* since the last *stable* connection. A connection
/// is stable if it stayed up for at least [`stable_after`](Self::stable_after)
/// (default [`DEFAULT_STABLE_AFTER`](Self::DEFAULT_STABLE_AFTER)); when a stable connection is
/// lost the count and the backoff start over. A connection that is lost sooner (a server that
/// accepts and immediately drops, for example) does **not** reset them, so a flapping
/// server is backed off from like an unreachable one. Receiving messages does not make a
/// connection stable: the server sends the initial snapshots right away. Stability is
/// measured up to the moment the loss is *observed* (the stream is polled and sees the
/// connection end). A connection ended by a `Timeout` (the idle timeout: nothing, not even a
/// heartbeat reply, arrived for that long) is only credited with the time up to the start of
/// that silence, so a server that accepts connections and then goes silent is backed off from
/// and counts against [`max_attempts`](Self::max_attempts).
///
/// [`max_attempts`](Self::max_attempts), if set, bounds the number of attempts without a
/// stable connection in between; by default the channel retries forever.
///
/// # Refused handshakes
///
/// If the server refuses the handshake with a `Retry-After` delay (HTTP `429` or `503`, see
/// [`Error::retry_after`]), the next attempt waits for the longer of the backoff and that delay,
/// but never more than [`MAX_RETRY_AFTER`](Self::MAX_RETRY_AFTER) (one hour), so that a
/// bogus header cannot park the channel indefinitely.
///
/// ```
/// use std::{num::NonZeroU32, time::Duration};
///
/// use marcasite::ws::ReconnectPolicy;
///
/// # fn run() -> marcasite::Result<()> {
/// let policy = ReconnectPolicy::new(Duration::from_millis(500), Duration::from_secs(20))?
///     .max_attempts(NonZeroU32::MIN.saturating_add(9));
/// assert_eq!(policy.max_backoff(), Duration::from_secs(20));
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct ReconnectPolicy {
    initial_backoff: Duration,
    max_backoff: Duration,
    max_attempts: Option<NonZeroU32>,
    stable_after: Duration,
    retry_client_errors: bool,
}

impl ReconnectPolicy {
    /// The default delay before the first reconnection attempt: 1 second.
    pub const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_secs(1);
    /// The default upper bound of the delay: 30 seconds.
    pub const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(30);
    /// The default time a connection must stay up to count as stable: 30 seconds (the
    /// default [idle timeout](MarketChannel::DEFAULT_IDLE_TIMEOUT) of the market channel).
    pub const DEFAULT_STABLE_AFTER: Duration = Duration::from_secs(30);
    /// The longest `Retry-After` delay that is honoured: one hour.
    pub const MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);

    /// A policy with the given initial and maximum backoff, retrying forever, with the
    /// default [`stable_after`](Self::stable_after).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `initial_backoff` is zero or `max_backoff` is
    /// shorter than `initial_backoff`.
    pub fn new(initial_backoff: Duration, max_backoff: Duration) -> Result<Self> {
        if initial_backoff.is_zero() {
            return Err(ValidationError::new(
                "initial_backoff",
                "the initial backoff must be greater than zero",
            )
            .into());
        }
        if max_backoff < initial_backoff {
            return Err(ValidationError::new(
                "max_backoff",
                "the maximum backoff must not be shorter than the initial backoff",
            )
            .into());
        }
        Ok(Self {
            initial_backoff,
            max_backoff,
            max_attempts: None,
            stable_after: Self::DEFAULT_STABLE_AFTER,
            retry_client_errors: false,
        })
    }

    /// Gives up after this many connection attempts without a stable connection in
    /// between (default: retry forever). See [`ReconnectingMarketChannel`] for what the
    /// stream yields then.
    pub fn max_attempts(mut self, max_attempts: NonZeroU32) -> Self {
        self.max_attempts = Some(max_attempts);
        self
    }

    /// Retries forever (the default).
    pub fn unlimited_attempts(mut self) -> Self {
        self.max_attempts = None;
        self
    }

    /// How long a connection must stay up to count as stable, which resets the attempt count
    /// and the backoff when it is lost (default
    /// [`DEFAULT_STABLE_AFTER`](Self::DEFAULT_STABLE_AFTER)). Zero makes every connection
    /// stable.
    pub fn stable_after(mut self, stable_after: Duration) -> Self {
        self.stable_after = stable_after;
        self
    }

    /// Whether to keep retrying when the server refuses the handshake with an HTTP `4xx`
    /// status other than `408` and `429` (default `false`).
    ///
    /// Such a refusal (`401`, `403` for a geoblock, `404`, ...) is not fixed by trying again,
    /// so by default the stream yields the error and ends. `408`, `429` and every `5xx` are
    /// always retried. Set this to `true` to retry regardless, e.g. behind a flaky proxy.
    pub fn retry_client_errors(mut self, retry: bool) -> Self {
        self.retry_client_errors = retry;
        self
    }

    /// The delay before the first reconnection attempt.
    #[must_use]
    pub fn initial_backoff(&self) -> Duration {
        self.initial_backoff
    }

    /// The upper bound of the delay between attempts.
    #[must_use]
    pub fn max_backoff(&self) -> Duration {
        self.max_backoff
    }

    /// The attempt limit, if any.
    #[must_use]
    pub fn attempt_limit(&self) -> Option<NonZeroU32> {
        self.max_attempts
    }

    /// Whether handshakes refused with a `4xx` status are retried.
    #[must_use]
    pub fn retries_client_errors(&self) -> bool {
        self.retry_client_errors
    }

    /// How long a connection must stay up to count as stable.
    #[must_use]
    pub fn stable_period(&self) -> Duration {
        self.stable_after
    }

    /// The delay before the attempt that follows `attempts` attempts: the initial backoff
    /// after zero or one, then doubling, capped at the maximum.
    fn backoff(&self, attempts: u32) -> Duration {
        // `2^31` is far beyond any sensible ratio; the shift cannot overflow.
        let factor = 1u32 << attempts.saturating_sub(1).min(31);
        self.initial_backoff
            .saturating_mul(factor)
            .min(self.max_backoff)
    }

    /// Whether `attempts` attempts have used up the limit.
    fn exhausted(&self, attempts: u32) -> bool {
        self.max_attempts.is_some_and(|max| attempts >= max.get())
    }
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial_backoff: Self::DEFAULT_INITIAL_BACKOFF,
            max_backoff: Self::DEFAULT_MAX_BACKOFF,
            max_attempts: None,
            stable_after: Self::DEFAULT_STABLE_AFTER,
            retry_client_errors: false,
        }
    }
}

/// An item of a [`ReconnectingMarketChannel`]: a connection lifecycle signal or a market
/// event.
///
/// The `asset_ids` of the lifecycle events are the subscription the channel tracks at that
/// moment (sorted), including changes made through the
/// [handle](ReconnectingMarketChannelHandle) while disconnected.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReconnectEvent {
    /// The channel is connected and subscribed: on the first connection and after every
    /// reconnection. The server sends a fresh `book` snapshot per asset next (unless
    /// [`MarketSubscription::initial_dump`] was set to `false`).
    Connected {
        /// The assets subscribed on the new connection.
        asset_ids: Vec<TokenId>,
        /// Which connection attempt since the last stable connection this was (`1` for the
        /// first connection; see [`ReconnectPolicy`]).
        attempt: u32,
    },
    /// The connection was lost: the stream ended, with the error that ended it (or `None`
    /// for a close the server finished normally). Emitted exactly once per lost connection,
    /// after every message received before the loss and before any reconnection attempt.
    /// Nothing is received for these assets until the next [`Connected`](Self::Connected).
    Disconnected {
        /// The tracked subscription when the loss was observed.
        asset_ids: Vec<TokenId>,
        /// Why the connection ended, if it ended with an error.
        error: Option<Error>,
    },
    /// A connection attempt failed; the channel waits `retry_in` and tries again. Not
    /// emitted for the last attempt when [`ReconnectPolicy::max_attempts`] is exhausted: the
    /// stream yields that error as an `Err` item instead.
    ConnectFailed {
        /// The failed attempt, counted as in [`Connected`](Self::Connected).
        attempt: u32,
        /// Why it failed.
        error: Error,
        /// How long the channel waits before the next attempt.
        retry_in: Duration,
    },
    /// An event of the market channel.
    Market(MarketEvent),
}

/// The subscription tracked across connections, shared between the stream and its handles.
#[derive(Debug)]
struct Tracked {
    assets: BTreeSet<TokenId>,
    initial_dump: Option<bool>,
    level: Option<SubscriptionLevel>,
    custom_feature_enabled: Option<bool>,
    /// Bumped by every subscription change, to detect changes made while connecting.
    version: u64,
    /// Set by `close` and when the stream is dropped; never reset.
    closed: bool,
    /// The live connection, if any.
    live: Option<MarketChannelHandle>,
    /// The stream's waker, to wake it when it waits and the channel is closed.
    waker: Option<Waker>,
}

type Shared = Arc<Mutex<Tracked>>;

fn lock(shared: &Shared) -> MutexGuard<'_, Tracked> {
    // Nothing panics while holding the lock; if it were poisoned the data is still valid.
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Tracked {
    fn sorted_ids(&self) -> Vec<TokenId> {
        self.assets.iter().cloned().collect()
    }

    fn subscription(&self) -> MarketSubscription {
        MarketSubscription {
            asset_ids: self.sorted_ids(),
            initial_dump: self.initial_dump,
            level: self.level,
            custom_feature_enabled: self.custom_feature_enabled,
        }
    }
}

fn closed_error() -> Error {
    Error::WebSocket(Box::new(WebSocketError::new(
        Service::MarketChannel,
        WebSocketErrorKind::Closed,
        "the reconnecting channel is closed",
    )))
}

/// A cloneable handle to a [`ReconnectingMarketChannel`], from
/// [`ReconnectingMarketChannel::handle`]: changes the subscription or closes the channel
/// from any task while the channel is consumed as a stream.
///
/// Changes are recorded in the tracked subscription, sent on the live connection if there
/// is one, and otherwise applied when the channel reconnects. The handle does not keep the
/// channel alive: once the channel is closed or dropped its methods fail with an
/// [`Error::WebSocket`] of kind [`Closed`](crate::WebSocketErrorKind::Closed).
#[derive(Debug, Clone)]
pub struct ReconnectingMarketChannelHandle {
    shared: Shared,
}

impl ReconnectingMarketChannelHandle {
    /// Starts following more asset ids (on the live connection now, or after reconnecting).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// channel is closed.
    pub fn subscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.update_subscription(MarketSubscriptionUpdate::subscribe(asset_ids))
    }

    /// Stops following some asset ids (on the live connection now, or after reconnecting).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// channel is closed.
    pub fn unsubscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.update_subscription(MarketSubscriptionUpdate::unsubscribe(asset_ids))
    }

    /// Applies a subscription change to the tracked subscription and sends it on the live
    /// connection, if any. A `level` or `custom_feature_enabled` set on the update is kept
    /// for reconnections too.
    ///
    /// Changes are never lost to a connection that fails while the update is sent: they
    /// are in the tracked subscription that the next connection starts from.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the update has no asset ids, or [`Error::WebSocket`]
    /// if the channel is closed.
    pub fn update_subscription(&self, update: MarketSubscriptionUpdate) -> Result<()> {
        if update.asset_ids.is_empty() {
            return Err(ValidationError::new(
                "assets_ids",
                "a subscription update needs at least one asset id",
            )
            .into());
        }
        let mut tracked = lock(&self.shared);
        if tracked.closed {
            return Err(closed_error());
        }
        match update.operation {
            Operation::Subscribe => tracked.assets.extend(update.asset_ids.iter().cloned()),
            Operation::Unsubscribe => {
                for id in &update.asset_ids {
                    tracked.assets.remove(id);
                }
            }
        }
        // Settings only take effect with a subscribe; an unsubscribe does not change what the
        // remaining assets are subscribed with.
        if update.operation == Operation::Subscribe {
            if update.level.is_some() {
                tracked.level = update.level;
            }
            if update.custom_feature_enabled.is_some() {
                tracked.custom_feature_enabled = update.custom_feature_enabled;
            }
        }
        tracked.version = tracked.version.wrapping_add(1);
        if let Some(live) = &tracked.live {
            // A failure means the connection has just ended; the change is tracked and the
            // next connection starts from it.
            if let Err(err) = live.update_subscription(update) {
                tracing::debug!(error = %err, "subscription change not sent; applied on reconnect");
            }
        }
        Ok(())
    }

    /// The asset ids currently tracked (sorted): what the channel is subscribed to, or will
    /// be after the next reconnection.
    #[must_use]
    pub fn asset_ids(&self) -> Vec<TokenId> {
        lock(&self.shared).sorted_ids()
    }

    /// Closes the channel for good: the live connection is closed gracefully, the stream
    /// ends after the events already received (without a
    /// [`Disconnected`](ReconnectEvent::Disconnected)), and it never reconnects. Also ends
    /// a stream that is waiting to reconnect.
    pub fn close(&self) {
        let (live, waker) = {
            let mut tracked = lock(&self.shared);
            tracked.closed = true;
            (tracked.live.take(), tracked.waker.take())
        };
        if let Some(live) = live {
            live.close();
        }
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

/// A future that is `Send` but not necessarily `Sync`, made `Sync` by only ever accessing
/// it through `&mut` (via `Mutex::get_mut`, which never locks).
struct SyncFuture(Mutex<Pin<Box<dyn Future<Output = Result<MarketChannel>> + Send>>>);

impl std::fmt::Debug for SyncFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SyncFuture(..)")
    }
}

impl SyncFuture {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<MarketChannel>> {
        self.0
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .as_mut()
            .poll(cx)
    }
}

/// The subscription an attempt sent as its initial message.
#[derive(Debug)]
struct Sent {
    assets: BTreeSet<TokenId>,
    level: Option<SubscriptionLevel>,
    custom_feature_enabled: Option<bool>,
}

#[derive(Debug)]
enum State {
    /// Built but not yet polled: the first attempt starts at the first poll, so that it sends
    /// the subscription as it is then (including changes made through a handle before).
    Idle,
    /// Connecting; `sent` is the tracked subscription the attempt sent, `version` its version.
    Connecting {
        connect: SyncFuture,
        sent: Sent,
        version: u64,
    },
    Connected {
        channel: MarketChannel,
        since: Instant,
    },
    /// Waiting out the backoff before the next attempt.
    Waiting(Pin<Box<Sleep>>),
    /// Out of attempts: yield this error, then end.
    GaveUp(Error),
    Done,
}

/// A [`MarketChannel`] that reconnects by itself, with explicit connection lifecycle events.
///
/// The channel is a [`Stream`] of [`ReconnectEvent`]s. It connects when first polled and,
/// whenever the connection is lost, waits per its [`ReconnectPolicy`], connects again and
/// re-sends the subscription: the tracked asset ids (the initial ones, changed by the
/// [handle](ReconnectingMarketChannelHandle)), `level`, `initial_dump` and
/// `custom_feature_enabled`. If you set `initial_dump(false)`, the server does not send
/// `book` snapshots after a reconnection either.
///
/// `level` and `custom_feature_enabled` are tracked per channel, not per asset: the last
/// value set by the initial subscription or a *subscribe* update is sent for **every** tracked
/// asset on reconnection (the initial message carries one value for all of them). If you
/// subscribe assets with different settings, they converge on the latest after a reconnect.
/// Whether the server applies these settings per asset or per connection is undocumented
/// (see `OPEN_QUESTIONS.md`).
///
/// Created with [`new`](Self::new) or [`MarketChannelBuilder::reconnecting`]; it does not
/// connect before it is polled, so nothing blocks and a Tokio runtime is only needed from the
/// first poll on.
///
/// # Stream items
///
/// - `Ok(`[`ReconnectEvent::Connected`]`)`: connected and subscribed (first connection and
///   every reconnection).
/// - `Ok(`[`ReconnectEvent::Market`]`)`: an event of the live connection.
/// - `Ok(`[`ReconnectEvent::Disconnected`]`)`: the connection was lost, exactly once per lost
///   connection, with the assets that were subscribed. Every message that arrived before the
///   loss is yielded before it. A connection counts as lost when the stream of the inner
///   [`MarketChannel`] ends for any reason: an error (kinds `Closed`, `Protocol`, `Send` or
///   `Timeout`, including the idle timeout) or a close the server finished **normally**, which is
///   also followed by a reconnection (a market channel has no reason to end on purpose).
/// - `Ok(`[`ReconnectEvent::ConnectFailed`]`)`: an attempt to (re)connect failed (any error of
///   kind `Connect`, including a handshake refused with `408`, `429` or a `5xx` status).
///   A refusal with another `4xx` status (`401`, `403` for a geoblock, `404`) is **not**
///   retried unless [`ReconnectPolicy::retry_client_errors`] is set: it ends the stream as an
///   `Err`. Use [`ReconnectPolicy::max_attempts`] or
///   [`close`](ReconnectingMarketChannelHandle::close) to stop earlier.
/// - `Err(`[`Error::WebSocket`]`)` of kind
///   [`Decode`](crate::WebSocketErrorKind::Decode): one message did not decode. Not fatal,
///   exactly as on [`MarketChannel`]: the connection stays up and nothing is reconnected.
/// - Any other `Err` is the last item: the policy ran out of attempts (the last connection
///   error; if the last attempt was a connection that was then lost, an error of kind
///   `Closed` after its `Disconnected`), or a connection error that retrying cannot fix
///   ([`Error::Config`], e.g. TLS could not be set up, or polling outside a Tokio runtime;
///   or a refused handshake with a permanent `4xx` status). The stream then ends and the
///   handles fail with `Closed`.
///
/// # Tracking books
///
/// A consumer that keeps local order books should mark the assets of every `Disconnected`
/// untrusted and trust an asset again at its first `book` after the following `Connected`.
///
/// A message that fails to decode may be a lost delta, and it does not end the connection,
/// so no `Disconnected` or fresh `book` follows on its own. Mark every tracked asset
/// untrusted on a decode error until its next `book`. A reconnection sends fresh snapshots
/// (unless `initial_dump(false)`); whether a subscribe update through the handle does is
/// undocumented.
///
/// ```no_run
/// # async fn run() -> marcasite::Result<()> {
/// use std::{collections::HashSet, time::Duration};
///
/// use futures_util::StreamExt as _;
/// use marcasite::{
///     Error, WebSocketErrorKind,
///     ws::{MarketChannel, MarketEvent, MarketSubscription, ReconnectEvent, ReconnectPolicy},
/// };
///
/// let policy = ReconnectPolicy::new(Duration::from_millis(500), Duration::from_secs(30))?;
/// let mut channel = MarketChannel::builder()
///     .reconnecting()
///     .policy(policy)
///     .build(MarketSubscription::new([
///         "65818619657568813474341868652308942079804919287380422192892211131408793125422",
///     ]))?;
/// let handle = channel.handle(); // e.g. for `handle.subscribe(..)` from other tasks
///
/// let mut untrusted = HashSet::new();
/// while let Some(item) = channel.next().await {
///     let event = match item {
///         Ok(event) => event,
///         // One message did not decode: not fatal, the stream continues, but it may have
///         // been a lost delta. Distrust every book until its next snapshot.
///         Err(Error::WebSocket(err)) if err.kind() == WebSocketErrorKind::Decode => {
///             untrusted.extend(handle.asset_ids());
///             continue;
///         }
///         // The policy gave up, or the error cannot be fixed by retrying.
///         Err(err) => return Err(err),
///     };
///     match event {
///         ReconnectEvent::Connected { asset_ids, attempt } => {
///             println!("connected (attempt {attempt}); waiting for books of {asset_ids:?}");
///             untrusted.extend(asset_ids);
///         }
///         ReconnectEvent::Disconnected { asset_ids, error } => {
///             eprintln!("lost the connection: {error:?}");
///             untrusted.extend(asset_ids);
///         }
///         ReconnectEvent::ConnectFailed { attempt, error, retry_in } => {
///             eprintln!("attempt {attempt} failed ({error}); retrying in {retry_in:?}");
///         }
///         ReconnectEvent::Market(MarketEvent::Book(book)) => {
///             untrusted.remove(&book.asset_id); // a fresh snapshot: trust the book again
///             // ... replace the local book ...
///         }
///         ReconnectEvent::Market(_) => { /* apply deltas to books that are not untrusted */ }
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Ending
///
/// [`ReconnectingMarketChannelHandle::close`] ends the stream for good (after the events
/// already received, without a `Disconnected`); dropping the stream closes the connection and
/// makes the handles fail. No background task is spawned by this type itself (the live
/// connection has its own driver task, which ends with it), and all buffers are bounded: the
/// receive buffer of the underlying channel is the only queue fed by the network.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
///
/// [`Stream`]: futures_core::Stream
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct ReconnectingMarketChannel {
    builder: MarketChannelBuilder,
    policy: ReconnectPolicy,
    shared: Shared,
    state: State,
    /// Connection attempts since the last stable connection.
    attempts: u32,
}

impl ReconnectingMarketChannel {
    /// A channel with the default endpoint, options and [`ReconnectPolicy`]; it connects when
    /// first polled.
    ///
    /// # Errors
    ///
    /// Never fails for the default settings; the signature matches
    /// [`ReconnectingMarketChannelBuilder::build`].
    pub fn new(subscription: MarketSubscription) -> Result<Self> {
        MarketChannelBuilder::default()
            .reconnecting()
            .build(subscription)
    }

    /// Returns a builder for a custom URL, timeouts or policy; same as
    /// `MarketChannel::builder().reconnecting()`.
    pub fn builder() -> ReconnectingMarketChannelBuilder {
        MarketChannelBuilder::default().reconnecting()
    }

    /// A cloneable handle to change the subscription or close the channel from other tasks.
    pub fn handle(&self) -> ReconnectingMarketChannelHandle {
        ReconnectingMarketChannelHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Starts following more asset ids; see [`ReconnectingMarketChannelHandle::subscribe`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// channel is closed.
    pub fn subscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.handle().subscribe(asset_ids)
    }

    /// Stops following some asset ids; see [`ReconnectingMarketChannelHandle::unsubscribe`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// channel is closed.
    pub fn unsubscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.handle().unsubscribe(asset_ids)
    }

    /// Applies a subscription change; see
    /// [`ReconnectingMarketChannelHandle::update_subscription`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the update has no asset ids, or [`Error::WebSocket`]
    /// if the channel is closed.
    pub fn update_subscription(&self, update: MarketSubscriptionUpdate) -> Result<()> {
        self.handle().update_subscription(update)
    }

    /// Closes the channel for good; see [`ReconnectingMarketChannelHandle::close`].
    pub fn close(&self) {
        self.handle().close();
    }

    /// Starts an attempt from the tracked subscription.
    fn begin_connect(&mut self) -> State {
        self.attempts = self.attempts.saturating_add(1);
        let (subscription, sent, version) = {
            let tracked = lock(&self.shared);
            (
                tracked.subscription(),
                Sent {
                    assets: tracked.assets.clone(),
                    level: tracked.level,
                    custom_feature_enabled: tracked.custom_feature_enabled,
                },
                tracked.version,
            )
        };
        State::Connecting {
            connect: SyncFuture(Mutex::new(Box::pin(
                self.builder.clone().connect(subscription),
            ))),
            sent,
            version,
        }
    }

    /// Marks the channel closed for the handles (after a fatal end or giving up).
    fn shut(&self) {
        let mut tracked = lock(&self.shared);
        tracked.closed = true;
        tracked.live = None;
        tracked.waker = None;
    }

    /// Whether the channel was closed; otherwise registers `waker` to be woken on close.
    fn poll_closed(&self, waker: &Waker) -> bool {
        let mut tracked = lock(&self.shared);
        if tracked.closed {
            return true;
        }
        if !tracked.waker.as_ref().is_some_and(|w| w.will_wake(waker)) {
            tracked.waker = Some(waker.clone());
        }
        false
    }

    /// The connection ended with `error`: emits `Disconnected` and schedules what follows.
    fn lost(
        &mut self,
        error: Option<Error>,
        since: Instant,
    ) -> Poll<Option<Result<ReconnectEvent>>> {
        let (asset_ids, closed) = {
            let mut tracked = lock(&self.shared);
            tracked.live = None;
            (tracked.sorted_ids(), tracked.closed)
        };
        if closed {
            self.state = State::Done;
            return Poll::Ready(None);
        }
        // Measured up to the moment the loss is observed; a connection that timed out was
        // silent for (at least) the idle timeout before that.
        let mut up_for = since.elapsed();
        let timed_out = matches!(
            &error,
            Some(Error::WebSocket(ws)) if ws.kind() == WebSocketErrorKind::Timeout
        );
        if timed_out && let Some(idle) = self.builder.effective_idle_timeout() {
            up_for = up_for.saturating_sub(idle);
        }
        if up_for >= self.policy.stable_after {
            self.attempts = 0;
        }
        self.state = if self.policy.exhausted(self.attempts) {
            self.shut();
            State::GaveUp(Error::WebSocket(Box::new(WebSocketError::new(
                Service::MarketChannel,
                WebSocketErrorKind::Closed,
                format!(
                    "gave up reconnecting after {} attempts without a stable connection",
                    self.attempts
                ),
            ))))
        } else {
            State::Waiting(Box::pin(tokio::time::sleep(
                self.policy.backoff(self.attempts),
            )))
        };
        Poll::Ready(Some(Ok(ReconnectEvent::Disconnected { asset_ids, error })))
    }

    /// A connection was established: installs it and brings it up to date with changes made
    /// while connecting.
    fn established(
        &mut self,
        channel: MarketChannel,
        sent: &Sent,
        sent_version: u64,
    ) -> Poll<Option<Result<ReconnectEvent>>> {
        let asset_ids = {
            let mut tracked = lock(&self.shared);
            if tracked.closed {
                self.state = State::Done;
                return Poll::Ready(None);
            }
            let handle = channel.handle();
            if tracked.version != sent_version {
                // Changes made through the handle while this attempt was connecting reached
                // neither the initial message nor a live connection: send the difference.
                // If `level` or `custom_feature_enabled` changed, subscribe again to every
                // tracked asset so that the new settings apply to them.
                let settings_changed = tracked.level != sent.level
                    || tracked.custom_feature_enabled != sent.custom_feature_enabled;
                let to_subscribe: Vec<TokenId> = if settings_changed {
                    tracked.assets.iter().cloned().collect()
                } else {
                    tracked.assets.difference(&sent.assets).cloned().collect()
                };
                let removed: Vec<TokenId> =
                    sent.assets.difference(&tracked.assets).cloned().collect();
                if !to_subscribe.is_empty() {
                    let mut update = MarketSubscriptionUpdate::subscribe(to_subscribe);
                    update.level = tracked.level;
                    update.custom_feature_enabled = tracked.custom_feature_enabled;
                    if let Err(err) = handle.update_subscription(update) {
                        tracing::debug!(error = %err, "catch-up subscription change not sent");
                    }
                }
                if !removed.is_empty() {
                    let update = MarketSubscriptionUpdate::unsubscribe(removed);
                    if let Err(err) = handle.update_subscription(update) {
                        tracing::debug!(error = %err, "catch-up subscription change not sent");
                    }
                }
            }
            tracked.live = Some(handle);
            tracked.sorted_ids()
        };
        self.state = State::Connected {
            channel,
            since: Instant::now(),
        };
        Poll::Ready(Some(Ok(ReconnectEvent::Connected {
            asset_ids,
            attempt: self.attempts,
        })))
    }

    /// An attempt failed with `error`.
    fn failed(&mut self, error: Error) -> Poll<Option<Result<ReconnectEvent>>> {
        let retryable = matches!(
            &error,
            Error::WebSocket(e) if e.kind() == WebSocketErrorKind::Connect
        );
        let refused_for_good = matches!(
            &error,
            Error::WebSocket(e) if e.http_status().is_some_and(|status| {
                let code = status.as_u16();
                (400..500).contains(&code) && code != 408 && code != 429
            })
        ) && !self.policy.retry_client_errors;
        if !retryable || refused_for_good || self.policy.exhausted(self.attempts) {
            self.shut();
            self.state = State::Done;
            return Poll::Ready(Some(Err(error)));
        }
        let mut retry_in = self.policy.backoff(self.attempts);
        if let Some(retry_after) = error.retry_after() {
            retry_in = retry_in.max(retry_after.min(ReconnectPolicy::MAX_RETRY_AFTER));
        }
        tracing::debug!(attempt = self.attempts, ?retry_in, error = %error, "connection attempt failed");
        self.state = State::Waiting(Box::pin(tokio::time::sleep(retry_in)));
        Poll::Ready(Some(Ok(ReconnectEvent::ConnectFailed {
            attempt: self.attempts,
            error,
            retry_in,
        })))
    }
}

impl Stream for ReconnectingMarketChannel {
    type Item = Result<ReconnectEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if !matches!(this.state, State::Done) && tokio::runtime::Handle::try_current().is_err() {
            // Timers and connections need a runtime; fail instead of panicking.
            this.shut();
            this.state = State::Done;
            return Poll::Ready(Some(Err(ConfigError::new(
                "a ReconnectingMarketChannel must be polled within a Tokio runtime",
            )
            .into())));
        }
        loop {
            match std::mem::replace(&mut this.state, State::Done) {
                State::Done => return Poll::Ready(None),
                State::GaveUp(error) => return Poll::Ready(Some(Err(error))),
                State::Idle => this.state = this.begin_connect(),
                State::Waiting(mut sleep) => {
                    if this.poll_closed(cx.waker()) {
                        return Poll::Ready(None);
                    }
                    if sleep.as_mut().poll(cx).is_pending() {
                        this.state = State::Waiting(sleep);
                        return Poll::Pending;
                    }
                    this.state = this.begin_connect();
                }
                State::Connecting {
                    mut connect,
                    sent,
                    version,
                } => {
                    if this.poll_closed(cx.waker()) {
                        return Poll::Ready(None);
                    }
                    match connect.poll(cx) {
                        Poll::Pending => {
                            this.state = State::Connecting {
                                connect,
                                sent,
                                version,
                            };
                            return Poll::Pending;
                        }
                        Poll::Ready(Ok(channel)) => {
                            return this.established(channel, &sent, version);
                        }
                        Poll::Ready(Err(error)) => return this.failed(error),
                    }
                }
                State::Connected { mut channel, since } => {
                    match Pin::new(&mut channel).poll_next(cx) {
                        Poll::Pending => {
                            this.state = State::Connected { channel, since };
                            return Poll::Pending;
                        }
                        Poll::Ready(Some(Ok(event))) => {
                            this.state = State::Connected { channel, since };
                            return Poll::Ready(Some(Ok(ReconnectEvent::Market(event))));
                        }
                        Poll::Ready(Some(Err(Error::WebSocket(e))))
                            if e.kind() == WebSocketErrorKind::Decode =>
                        {
                            this.state = State::Connected { channel, since };
                            return Poll::Ready(Some(Err(Error::WebSocket(e))));
                        }
                        Poll::Ready(Some(Err(error))) => return this.lost(Some(error), since),
                        Poll::Ready(None) => return this.lost(None, since),
                    }
                }
            }
        }
    }
}

impl FusedStream for ReconnectingMarketChannel {
    fn is_terminated(&self) -> bool {
        matches!(self.state, State::Done)
    }
}

impl Drop for ReconnectingMarketChannel {
    fn drop(&mut self) {
        let mut tracked = lock(&self.shared);
        tracked.closed = true;
        tracked.live = None;
        tracked.waker = None;
    }
}

/// Builder for [`ReconnectingMarketChannel`], from [`MarketChannelBuilder::reconnecting`]
/// (which carries over the URL, buffer, timeouts and heartbeat interval set on it).
#[derive(Debug, Clone)]
#[must_use]
pub struct ReconnectingMarketChannelBuilder {
    channel: MarketChannelBuilder,
    policy: ReconnectPolicy,
}

impl MarketChannelBuilder {
    /// Turns this builder into one for a [`ReconnectingMarketChannel`], with the default
    /// [`ReconnectPolicy`]. The settings of this builder apply to every connection.
    pub fn reconnecting(self) -> ReconnectingMarketChannelBuilder {
        ReconnectingMarketChannelBuilder {
            channel: self,
            policy: ReconnectPolicy::default(),
        }
    }
}

impl ReconnectingMarketChannelBuilder {
    /// Sets the [`ReconnectPolicy`] (default: [`ReconnectPolicy::default`]).
    pub fn policy(mut self, policy: ReconnectPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Creates the channel with `subscription` as the initial tracked subscription. It
    /// connects when first polled.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the URL is invalid or the heartbeat interval or idle
    /// timeout is zero, and [`Error::Validation`] if the initial subscription cannot be
    /// encoded, so that these fail here rather than as a first stream item.
    pub fn build(self, subscription: MarketSubscription) -> Result<ReconnectingMarketChannel> {
        let _ = self.channel.config()?;
        subscription.to_json()?;
        let shared = Arc::new(Mutex::new(Tracked {
            assets: subscription.asset_ids.iter().cloned().collect(),
            initial_dump: subscription.initial_dump,
            level: subscription.level,
            custom_feature_enabled: subscription.custom_feature_enabled,
            version: 0,
            closed: false,
            live: None,
            waker: None,
        }));
        let channel = ReconnectingMarketChannel {
            builder: self.channel,
            policy: self.policy,
            shared,
            state: State::Idle,
            attempts: 0,
        };
        Ok(channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_is_validated() {
        let ms = Duration::from_millis;
        assert!(matches!(
            ReconnectPolicy::new(Duration::ZERO, ms(10)),
            Err(Error::Validation(_))
        ));
        assert!(matches!(
            ReconnectPolicy::new(ms(10), ms(9)),
            Err(Error::Validation(_))
        ));
        let policy = ReconnectPolicy::new(ms(10), ms(10)).unwrap();
        assert_eq!(policy.attempt_limit(), None);
        assert_eq!(
            policy.stable_period(),
            ReconnectPolicy::DEFAULT_STABLE_AFTER
        );
    }

    #[test]
    fn backoff_doubles_up_to_the_cap() {
        let policy = ReconnectPolicy::new(Duration::from_secs(1), Duration::from_secs(30)).unwrap();
        let delays: Vec<u64> = [0, 1, 2, 3, 4, 5, 6, 100, u32::MAX]
            .into_iter()
            .map(|n| policy.backoff(n).as_secs())
            .collect();
        assert_eq!(delays, [1, 1, 2, 4, 8, 16, 30, 30, 30]);
        // No overflow with a huge initial backoff.
        let huge = ReconnectPolicy::new(Duration::MAX, Duration::MAX).unwrap();
        assert_eq!(huge.backoff(u32::MAX), Duration::MAX);
    }

    #[test]
    fn attempt_limit() {
        let policy = ReconnectPolicy::default().max_attempts(NonZeroU32::MIN.saturating_add(2));
        assert!(!policy.exhausted(2));
        assert!(policy.exhausted(3));
        assert!(!policy.unlimited_attempts().exhausted(u32::MAX));
    }

    #[test]
    fn public_types_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ReconnectingMarketChannel>();
        assert_send_sync::<ReconnectingMarketChannelHandle>();
        assert_send_sync::<ReconnectingMarketChannelBuilder>();
        assert_send_sync::<ReconnectEvent>();
        assert_send_sync::<ReconnectPolicy>();
    }
}
