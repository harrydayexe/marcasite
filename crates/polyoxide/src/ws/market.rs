//! CLOB market channel: `wss://ws-subscriptions-clob.polymarket.com/ws/market`.
//!
//! Spec: `docs/specs/asyncapi.json`; page: `docs/api-reference/wss/market.md`.

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use chrono::{DateTime, Utc};
use futures_core::{Stream, stream::FusedStream};
use polyoxide_core::{
    Error, Result, Service, ValidationError, WebSocketError, WebSocketErrorKind, serde_util,
    types::{ConditionId, EventId, MarketId, Side, TokenId},
    ws::WsConnection,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::Value;

use super::frame::{ConnectOptions, EventStream, check_interval, str_field};

/// The text frame the client sends as a heartbeat.
const PING: &str = "PING";
/// The server's reply to [`PING`]; dropped by the driver.
const PONG: &str = "PONG";

/// The subscription `level` of the market channel (`1`, `2` or `3`; the server defaults
/// to `2`).
///
/// The documentation lists the allowed values but does not describe what each level
/// changes.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SubscriptionLevel {
    /// Level `1`.
    Level1,
    /// Level `2` (the server default).
    Level2,
    /// Level `3`.
    Level3,
}

impl SubscriptionLevel {
    /// The wire value (`1`, `2` or `3`).
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Level1 => 1,
            Self::Level2 => 2,
            Self::Level3 => 3,
        }
    }
}

impl Serialize for SubscriptionLevel {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.as_u8())
    }
}

/// The subscription request sent right after connecting to the market channel.
///
/// Required: the asset (token) ids to follow. Optional settings are left to the server
/// default unless set.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
///
/// ```
/// use polyoxide::ws::{MarketSubscription, SubscriptionLevel};
///
/// let subscription = MarketSubscription::new([
///     "65818619657568813474341868652308942079804919287380422192892211131408793125422",
/// ])
/// .custom_feature_enabled(true)
/// .level(SubscriptionLevel::Level2);
/// assert_eq!(subscription.asset_ids().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct MarketSubscription {
    asset_ids: Vec<TokenId>,
    initial_dump: Option<bool>,
    level: Option<SubscriptionLevel>,
    custom_feature_enabled: Option<bool>,
}

impl MarketSubscription {
    /// Subscribes to the given asset (token) ids (`assets_ids`).
    ///
    /// The list may be empty: the documentation does not require any asset ids, and assets
    /// can be added later with [`MarketChannel::subscribe`].
    pub fn new<I>(asset_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        Self {
            asset_ids: asset_ids.into_iter().map(Into::into).collect(),
            initial_dump: None,
            level: None,
            custom_feature_enabled: None,
        }
    }

    /// Whether to receive an initial order book snapshot for each asset (`initial_dump`;
    /// the server defaults to `true`).
    pub fn initial_dump(mut self, initial_dump: bool) -> Self {
        self.initial_dump = Some(initial_dump);
        self
    }

    /// The subscription level (`level`; the server defaults to
    /// [`SubscriptionLevel::Level2`]).
    pub fn level(mut self, level: SubscriptionLevel) -> Self {
        self.level = Some(level);
        self
    }

    /// Enables the [`MarketEvent::BestBidAsk`], [`MarketEvent::NewMarket`] and
    /// [`MarketEvent::MarketResolved`] events (`custom_feature_enabled`; the server
    /// defaults to `false`).
    pub fn custom_feature_enabled(mut self, enabled: bool) -> Self {
        self.custom_feature_enabled = Some(enabled);
        self
    }

    /// The asset ids to subscribe to.
    #[must_use]
    pub fn asset_ids(&self) -> &[TokenId] {
        &self.asset_ids
    }

    fn to_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Wire<'a> {
            assets_ids: &'a [TokenId],
            #[serde(rename = "type")]
            kind: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            initial_dump: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            level: Option<SubscriptionLevel>,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_feature_enabled: Option<bool>,
        }
        encode(&Wire {
            assets_ids: &self.asset_ids,
            kind: "market",
            initial_dump: self.initial_dump,
            level: self.level,
            custom_feature_enabled: self.custom_feature_enabled,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Operation {
    Subscribe,
    Unsubscribe,
}

/// A dynamic subscription change on an open market channel: add or remove asset ids
/// without reconnecting.
///
/// Use [`MarketChannel::subscribe`] / [`MarketChannel::unsubscribe`] for the common case,
/// or build one of these to also set `level` or `custom_feature_enabled`.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct MarketSubscriptionUpdate {
    operation: Operation,
    asset_ids: Vec<TokenId>,
    level: Option<SubscriptionLevel>,
    custom_feature_enabled: Option<bool>,
}

impl MarketSubscriptionUpdate {
    /// Starts following the given asset ids (`operation: "subscribe"`).
    pub fn subscribe<I>(asset_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        Self::new(Operation::Subscribe, asset_ids)
    }

    /// Stops following the given asset ids (`operation: "unsubscribe"`).
    pub fn unsubscribe<I>(asset_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        Self::new(Operation::Unsubscribe, asset_ids)
    }

    fn new<I>(operation: Operation, asset_ids: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        Self {
            operation,
            asset_ids: asset_ids.into_iter().map(Into::into).collect(),
            level: None,
            custom_feature_enabled: None,
        }
    }

    /// The subscription level (`level`).
    pub fn level(mut self, level: SubscriptionLevel) -> Self {
        self.level = Some(level);
        self
    }

    /// Enables or disables the custom-feature events (`custom_feature_enabled`).
    pub fn custom_feature_enabled(mut self, enabled: bool) -> Self {
        self.custom_feature_enabled = Some(enabled);
        self
    }

    /// The asset ids this update applies to.
    #[must_use]
    pub fn asset_ids(&self) -> &[TokenId] {
        &self.asset_ids
    }

    fn to_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Wire<'a> {
            operation: Operation,
            assets_ids: &'a [TokenId],
            #[serde(skip_serializing_if = "Option::is_none")]
            level: Option<SubscriptionLevel>,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_feature_enabled: Option<bool>,
        }
        if self.asset_ids.is_empty() {
            return Err(ValidationError::new(
                "assets_ids",
                "a subscription update needs at least one asset id",
            )
            .into());
        }
        encode(&Wire {
            operation: self.operation,
            assets_ids: &self.asset_ids,
            level: self.level,
            custom_feature_enabled: self.custom_feature_enabled,
        })
    }
}

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|e| {
        Error::WebSocket(Box::new(
            WebSocketError::new(
                Service::MarketChannel,
                WebSocketErrorKind::Send,
                "failed to encode request",
            )
            .with_source(e),
        ))
    })
}

/// A live connection to the CLOB market channel: order book snapshots and deltas, trades,
/// tick size changes and market lifecycle events for the subscribed assets.
///
/// The channel is a [`Stream`] of [`MarketEvent`]s. It sends the documented `PING`
/// heartbeat every 10 seconds and drops the server's `PONG` replies. See the
/// [module documentation](super) for error handling and reconnection.
///
/// # Stream items
///
/// Each item is an `Ok(`[`MarketEvent`]`)` or an
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
/// See <https://docs.polymarket.com/api-reference/wss/market>.
///
/// ```no_run
/// # async fn run() -> polyoxide::Result<()> {
/// use futures_util::StreamExt as _;
/// use polyoxide::ws::{MarketChannel, MarketEvent, MarketSubscription};
///
/// let subscription = MarketSubscription::new([
///     "65818619657568813474341868652308942079804919287380422192892211131408793125422",
/// ]);
/// let mut channel = MarketChannel::connect(subscription).await?;
/// while let Some(event) = channel.next().await {
///     match event? {
///         MarketEvent::Book(book) => println!("{} bids, {} asks", book.bids.len(), book.asks.len()),
///         MarketEvent::PriceChange(change) => println!("{} level changes", change.price_changes.len()),
///         other => println!("{other:?}"),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// [`Stream`]: futures_core::Stream
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct MarketChannel {
    events: EventStream<MarketEvent>,
}

impl MarketChannel {
    /// The production URL.
    pub const DEFAULT_URL: &'static str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

    /// The documented heartbeat interval: the client sends `PING` every 10 seconds.
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

    /// Connects to the production URL and sends `subscription`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the connection cannot be established.
    pub async fn connect(subscription: MarketSubscription) -> Result<Self> {
        Self::builder().connect(subscription).await
    }

    /// Returns a builder for a custom URL, buffer size, timeout or heartbeat interval.
    pub fn builder() -> MarketChannelBuilder {
        MarketChannelBuilder::default()
    }

    /// Starts following more asset ids without reconnecting.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// connection has terminated.
    pub fn subscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.update_subscription(MarketSubscriptionUpdate::subscribe(asset_ids))
    }

    /// Stops following some asset ids without reconnecting.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if `asset_ids` is empty, or [`Error::WebSocket`] if the
    /// connection has terminated.
    pub fn unsubscribe<I>(&self, asset_ids: I) -> Result<()>
    where
        I: IntoIterator,
        I::Item: Into<TokenId>,
    {
        self.update_subscription(MarketSubscriptionUpdate::unsubscribe(asset_ids))
    }

    /// Sends a subscription change.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if the update has no asset ids, or
    /// [`Error::WebSocket`] if the connection has terminated.
    pub fn update_subscription(&self, update: MarketSubscriptionUpdate) -> Result<()> {
        let frame = update.to_json()?;
        self.connection().send_text(frame)
    }

    /// Closes the connection gracefully; the stream then ends after any events already
    /// received.
    pub fn close(&self) {
        self.connection().close();
    }

    fn connection(&self) -> &WsConnection {
        self.events.connection()
    }
}

impl Stream for MarketChannel {
    type Item = Result<MarketEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().events.poll_next(cx)
    }
}

impl FusedStream for MarketChannel {
    fn is_terminated(&self) -> bool {
        self.events.is_terminated()
    }
}

/// Builder for [`MarketChannel`].
#[derive(Debug, Clone)]
#[must_use]
pub struct MarketChannelBuilder {
    options: ConnectOptions,
    heartbeat_interval: Duration,
}

impl Default for MarketChannelBuilder {
    fn default() -> Self {
        Self {
            options: ConnectOptions::new(),
            heartbeat_interval: MarketChannel::DEFAULT_HEARTBEAT_INTERVAL,
        }
    }
}

impl MarketChannelBuilder {
    /// Overrides the URL (default [`MarketChannel::DEFAULT_URL`]), e.g. to target a mock
    /// server in tests. Must use `ws://` or `wss://`.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.options.url = Some(url.into());
        self
    }

    /// Sets how many received frames may be buffered before the connection applies
    /// back-pressure (default
    /// [`DEFAULT_BUFFER`](super::DEFAULT_BUFFER)).
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

    /// Overrides how often `PING` is sent (default
    /// [`MarketChannel::DEFAULT_HEARTBEAT_INTERVAL`], as documented).
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Connects and sends `subscription`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the URL is invalid or the heartbeat interval is zero,
    /// or [`Error::WebSocket`] if the connection cannot be established.
    pub async fn connect(self, subscription: MarketSubscription) -> Result<MarketChannel> {
        check_interval(self.heartbeat_interval)?;
        let config = self
            .options
            .config(Service::MarketChannel, MarketChannel::DEFAULT_URL)?
            .heartbeat(self.heartbeat_interval, PING)
            .ignore(PONG)
            .initial_message(subscription.to_json()?);
        let conn = WsConnection::connect(config).await?;
        Ok(MarketChannel {
            events: EventStream::new(conn),
        })
    }
}

/// A message received on the market channel.
///
/// Decoded from the `event_type` field. A message with an unrecognised (or missing)
/// `event_type`, or a frame that is not JSON, becomes [`MarketEvent::Unknown`] so that
/// new server messages never break the stream.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
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
    /// [`MarketSubscription::custom_feature_enabled`].
    BestBidAsk(BestBidAskEvent),
    /// `new_market`: a market was created. Requires
    /// [`MarketSubscription::custom_feature_enabled`]. Boxed because it is much larger
    /// than the frequent order book events.
    NewMarket(Box<NewMarketEvent>),
    /// `market_resolved`: a market was resolved. Requires
    /// [`MarketSubscription::custom_feature_enabled`]. Boxed because it is much larger
    /// than the frequent order book events.
    MarketResolved(Box<MarketResolvedEvent>),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    #[serde(untagged)]
    Unknown(Value),
}

impl<'de> Deserialize<'de> for MarketEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        fn parse<T: DeserializeOwned, E: serde::de::Error>(
            kind: &str,
            value: &Value,
        ) -> std::result::Result<T, E> {
            T::deserialize(value).map_err(|e| E::custom(format!("invalid `{kind}` event: {e}")))
        }

        let value = Value::deserialize(deserializer)?;
        let event = match str_field(&value, "event_type") {
            Some(kind @ "book") => Self::Book(parse(kind, &value)?),
            Some(kind @ "price_change") => Self::PriceChange(parse(kind, &value)?),
            Some(kind @ "last_trade_price") => Self::LastTradePrice(parse(kind, &value)?),
            Some(kind @ "tick_size_change") => Self::TickSizeChange(parse(kind, &value)?),
            Some(kind @ "best_bid_ask") => Self::BestBidAsk(parse(kind, &value)?),
            Some(kind @ "new_market") => Self::NewMarket(parse(kind, &value)?),
            Some(kind @ "market_resolved") => Self::MarketResolved(parse(kind, &value)?),
            _ => Self::Unknown(value),
        };
        Ok(event)
    }
}

/// Unix milliseconds sent as a decimal string (e.g. `"1757908892351"`).
mod millis_string {
    use chrono::{DateTime, Utc};
    use polyoxide_core::serde_util;
    use serde::{Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&value.timestamp_millis())
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        serde_util::timestamp_millis::deserialize(deserializer)
    }
}

/// Interprets a raw timestamp string as Unix milliseconds.
fn parse_millis(raw: &str) -> Option<DateTime<Utc>> {
    raw.trim()
        .parse::<i64>()
        .ok()
        .and_then(DateTime::from_timestamp_millis)
}

/// One aggregated price level of an order book (`OrderSummary`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderSummary {
    /// Price level (e.g. `0.50`).
    pub price: Decimal,
    /// Total size at this price level.
    pub size: Decimal,
}

/// `book`: a full aggregated order book snapshot for one asset (`BookEvent`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Aggregated buy orders by price level.
    pub bids: Vec<OrderSummary>,
    /// Aggregated sell orders by price level.
    pub asks: Vec<OrderSummary>,
    /// Snapshot time (sent as a string of Unix milliseconds).
    #[serde(with = "millis_string")]
    pub timestamp: DateTime<Utc>,
    /// Hash of the order book content.
    pub hash: String,
}

/// `price_change`: one or more order book price level updates (`PriceChangeEvent`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceChangeEvent {
    /// Condition id of the market.
    pub market: ConditionId,
    /// The changed price levels.
    pub price_changes: Vec<PriceChange>,
    /// Update time (sent as a string of Unix milliseconds).
    #[serde(with = "millis_string")]
    pub timestamp: DateTime<Utc>,
}

/// One price level change within a [`PriceChangeEvent`] (`PriceChangeMessage`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Best bid after the change.
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub best_bid: Option<Decimal>,
    /// Best ask after the change.
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub best_ask: Option<Decimal>,
}

/// `last_trade_price`: a trade execution (`LastTradePriceEvent`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub fee_rate_bps: Option<Decimal>,
    /// Trade side, from the taker's perspective.
    pub side: Side,
    /// Trade time (sent as a string of Unix milliseconds).
    #[serde(with = "millis_string")]
    pub timestamp: DateTime<Utc>,
    /// On-chain transaction hash.
    pub transaction_hash: Option<String>,
}

/// `tick_size_change`: the market's minimum tick size changed (`TickSizeChangeEvent`).
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// is Unix milliseconds); see [`Self::timestamp_millis`].
    pub timestamp: String,
}

impl TickSizeChangeEvent {
    /// [`Self::timestamp`] interpreted as Unix milliseconds, as in the documented example;
    /// `None` if it is not an integer.
    #[must_use]
    pub fn timestamp_millis(&self) -> Option<DateTime<Utc>> {
        parse_millis(&self.timestamp)
    }
}

/// `best_bid_ask`: the best bid and ask of an asset changed (`BestBidAskEvent`). Requires
/// [`MarketSubscription::custom_feature_enabled`].
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BestBidAskEvent {
    /// Asset (token) id.
    pub asset_id: TokenId,
    /// Condition id of the market.
    pub market: ConditionId,
    /// Best bid price.
    pub best_bid: Decimal,
    /// Best ask price.
    pub best_ask: Decimal,
    /// Spread between best ask and best bid.
    pub spread: Decimal,
    /// Event time, as sent. The spec gives no unit for this field (the documented example
    /// is Unix milliseconds); see [`Self::timestamp_millis`].
    pub timestamp: String,
}

impl BestBidAskEvent {
    /// [`Self::timestamp`] interpreted as Unix milliseconds, as in the documented example;
    /// `None` if it is not an integer.
    #[must_use]
    pub fn timestamp_millis(&self) -> Option<DateTime<Utc>> {
        parse_millis(&self.timestamp)
    }
}

/// Parent event metadata of a market (`EventMessage`). Every field is optional in the
/// spec.
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// `new_market`: a market was created (`NewMarketEvent`). Requires
/// [`MarketSubscription::custom_feature_enabled`].
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// is Unix milliseconds); see [`Self::timestamp_millis`].
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
    #[serde(default, with = "serde_util::datetime_option")]
    pub game_start_time: Option<DateTime<Utc>>,
    /// Minimum tick size for order prices.
    #[serde(default, with = "serde_util::string_or_number_option")]
    pub order_price_min_tick_size: Option<Decimal>,
    /// Display title of the market within its group.
    pub group_item_title: Option<String>,
}

impl NewMarketEvent {
    /// [`Self::timestamp`] interpreted as Unix milliseconds, as in the documented example;
    /// `None` if it is not an integer.
    #[must_use]
    pub fn timestamp_millis(&self) -> Option<DateTime<Utc>> {
        parse_millis(&self.timestamp)
    }
}

/// `market_resolved`: a market was resolved (`MarketResolvedEvent`). Requires
/// [`MarketSubscription::custom_feature_enabled`].
///
/// See <https://docs.polymarket.com/api-reference/wss/market>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// is Unix milliseconds); see [`Self::timestamp_millis`].
    pub timestamp: String,
    /// Tag slugs (e.g. `["stocks"]`).
    pub tags: Option<Vec<String>>,
}

impl MarketResolvedEvent {
    /// [`Self::timestamp`] interpreted as Unix milliseconds, as in the documented example;
    /// `None` if it is not an integer.
    #[must_use]
    pub fn timestamp_millis(&self) -> Option<DateTime<Utc>> {
        parse_millis(&self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::ws::frame::decode_frame;

    fn event(json: &str) -> MarketEvent {
        serde_json::from_str(json).unwrap()
    }

    // Requests: `components/messages/subscriptionRequest` and
    // `subscriptionRequestUpdate` examples in `docs/specs/asyncapi.json`.

    #[test]
    fn serializes_subscription_requests() {
        let basic = MarketSubscription::new([
            "65818619657568813474341868652308942079804919287380422192892211131408793125422",
            "52114319501245915516055106046884209969926127482827954674443846427813813222426",
        ]);
        assert_eq!(
            serde_json::from_str::<Value>(&basic.to_json().unwrap()).unwrap(),
            serde_json::json!({
                "assets_ids": [
                    "65818619657568813474341868652308942079804919287380422192892211131408793125422",
                    "52114319501245915516055106046884209969926127482827954674443846427813813222426"
                ],
                "type": "market"
            })
        );

        let custom = MarketSubscription::new([
            "65818619657568813474341868652308942079804919287380422192892211131408793125422",
        ])
        .custom_feature_enabled(true)
        .initial_dump(true)
        .level(SubscriptionLevel::Level3);
        assert_eq!(
            serde_json::from_str::<Value>(&custom.to_json().unwrap()).unwrap(),
            serde_json::json!({
                "assets_ids": [
                    "65818619657568813474341868652308942079804919287380422192892211131408793125422"
                ],
                "type": "market",
                "custom_feature_enabled": true,
                "initial_dump": true,
                "level": 3
            })
        );

        let empty = MarketSubscription::new(Vec::<TokenId>::new());
        assert_eq!(
            empty.to_json().unwrap(),
            r#"{"assets_ids":[],"type":"market"}"#
        );
    }

    #[test]
    fn serializes_subscription_updates() {
        let subscribe = MarketSubscriptionUpdate::subscribe([
            "71321045679252212594626385532706912750332728571942532289631379312455583992563",
        ]);
        assert_eq!(
            subscribe.to_json().unwrap(),
            r#"{"operation":"subscribe","assets_ids":["71321045679252212594626385532706912750332728571942532289631379312455583992563"]}"#
        );
        let unsubscribe = MarketSubscriptionUpdate::unsubscribe([
            "65818619657568813474341868652308942079804919287380422192892211131408793125422",
        ])
        .level(SubscriptionLevel::Level1)
        .custom_feature_enabled(false);
        assert_eq!(
            unsubscribe.to_json().unwrap(),
            r#"{"operation":"unsubscribe","assets_ids":["65818619657568813474341868652308942079804919287380422192892211131408793125422"],"level":1,"custom_feature_enabled":false}"#
        );
        let empty = MarketSubscriptionUpdate::subscribe(Vec::<String>::new());
        assert!(matches!(empty.to_json(), Err(Error::Validation(_))));
    }

    // Events: `components/messages/*` examples in `docs/specs/asyncapi.json`.

    #[test]
    fn deserializes_book() {
        let MarketEvent::Book(book) = event(
            r#"{
              "event_type": "book",
              "asset_id": "65818619657568813474341868652308942079804919287380422192892211131408793125422",
              "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
              "bids": [
                { "price": "0.48", "size": "30" },
                { "price": "0.49", "size": "20" },
                { "price": "0.50", "size": "15" }
              ],
              "asks": [
                { "price": "0.52", "size": "25" },
                { "price": "0.53", "size": "60" },
                { "price": "0.54", "size": "10" }
              ],
              "timestamp": "1757908892351",
              "hash": "0xabc123..."
            }"#,
        ) else {
            panic!("expected a book event");
        };
        assert_eq!(
            book.asset_id,
            "65818619657568813474341868652308942079804919287380422192892211131408793125422"
        );
        assert_eq!(
            book.market,
            "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af"
        );
        assert_eq!(book.bids.len(), 3);
        assert_eq!(book.bids[2].price, Decimal::new(50, 2));
        assert_eq!(book.asks[1].size, Decimal::new(60, 0));
        assert_eq!(book.timestamp.timestamp_millis(), 1_757_908_892_351);
        assert_eq!(book.hash, "0xabc123...");
    }

    #[test]
    fn deserializes_price_change() {
        let MarketEvent::PriceChange(change) = event(
            r#"{
              "event_type": "price_change",
              "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
              "price_changes": [
                {
                  "asset_id": "71321045679252212594626385532706912750332728571942532289631379312455583992563",
                  "price": "0.5",
                  "size": "200",
                  "side": "BUY",
                  "hash": "56621a121a47ed9333273e21c83b660cff37ae50",
                  "best_bid": "0.5",
                  "best_ask": "1"
                }
              ],
              "timestamp": "1757908892351"
            }"#,
        ) else {
            panic!("expected a price_change event");
        };
        assert_eq!(change.timestamp.timestamp_millis(), 1_757_908_892_351);
        let level = &change.price_changes[0];
        assert_eq!(level.price, Decimal::new(5, 1));
        assert_eq!(level.size, Decimal::new(200, 0));
        assert_eq!(level.side, Side::Buy);
        assert_eq!(level.hash, "56621a121a47ed9333273e21c83b660cff37ae50");
        assert_eq!(level.best_bid, Some(Decimal::new(5, 1)));
        assert_eq!(level.best_ask, Some(Decimal::ONE));
    }

    #[test]
    fn deserializes_price_change_without_optional_fields() {
        let MarketEvent::PriceChange(change) = event(
            r#"{"event_type":"price_change","market":"0x1","timestamp":"1","price_changes":[
                {"asset_id":"1","price":"0.1","size":"0","side":"SELL","hash":"h"}]}"#,
        ) else {
            panic!("expected a price_change event");
        };
        assert_eq!(change.price_changes[0].best_bid, None);
        assert_eq!(change.price_changes[0].side, Side::Sell);
    }

    #[test]
    fn deserializes_last_trade_price() {
        let MarketEvent::LastTradePrice(trade) = event(
            r#"{
              "event_type": "last_trade_price",
              "asset_id": "114122071509644379678018727908709560226618148003371446110114509806601493071694",
              "market": "0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347",
              "price": "0.456",
              "size": "219.217767",
              "fee_rate_bps": "0",
              "side": "BUY",
              "timestamp": "1750428146322",
              "transaction_hash": "0xeeefffggghhh"
            }"#,
        ) else {
            panic!("expected a last_trade_price event");
        };
        assert_eq!(trade.price, Decimal::new(456, 3));
        assert_eq!(trade.size, Decimal::new(219_217_767, 6));
        assert_eq!(trade.fee_rate_bps, Some(Decimal::ZERO));
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.timestamp.timestamp_millis(), 1_750_428_146_322);
        assert_eq!(trade.transaction_hash.as_deref(), Some("0xeeefffggghhh"));
    }

    #[test]
    fn deserializes_tick_size_change() {
        let MarketEvent::TickSizeChange(tick) = event(
            r#"{
              "event_type": "tick_size_change",
              "asset_id": "65818619657568813474341868652308942079804919287380422192892211131408793125422",
              "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
              "old_tick_size": "0.01",
              "new_tick_size": "0.001",
              "timestamp": "1757908892351"
            }"#,
        ) else {
            panic!("expected a tick_size_change event");
        };
        assert_eq!(tick.old_tick_size, Decimal::new(1, 2));
        assert_eq!(tick.new_tick_size, Decimal::new(1, 3));
        assert_eq!(tick.timestamp, "1757908892351");
        assert_eq!(
            tick.timestamp_millis().map(|t| t.timestamp_millis()),
            Some(1_757_908_892_351)
        );
    }

    #[test]
    fn deserializes_best_bid_ask() {
        let MarketEvent::BestBidAsk(bba) = event(
            r#"{
              "event_type": "best_bid_ask",
              "market": "0x0005c0d312de0be897668695bae9f32b624b4a1ae8b140c49f08447fcc74f442",
              "asset_id": "85354956062430465315924116860125388538595433819574542752031640332592237464430",
              "best_bid": "0.73",
              "best_ask": "0.77",
              "spread": "0.04",
              "timestamp": "1766789469958"
            }"#,
        ) else {
            panic!("expected a best_bid_ask event");
        };
        assert_eq!(bba.best_bid, Decimal::new(73, 2));
        assert_eq!(bba.best_ask, Decimal::new(77, 2));
        assert_eq!(bba.spread, Decimal::new(4, 2));
        assert_eq!(
            bba.timestamp_millis().map(|t| t.timestamp_millis()),
            Some(1_766_789_469_958)
        );
    }

    #[test]
    fn deserializes_new_market() {
        let MarketEvent::NewMarket(market) = event(
            r#"{
              "event_type": "new_market",
              "id": "1031769",
              "question": "Will NVIDIA (NVDA) close above $240 end of January?",
              "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
              "slug": "nvda-above-240-on-january-30-2026",
              "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\".",
              "assets_ids": [
                "76043073756653678226373981964075571318267289248134717369284518995922789326425",
                "31690934263385727664202099278545688007799199447969475608906331829650099442770"
              ],
              "outcomes": ["Yes", "No"],
              "event_message": {
                "id": "125819",
                "ticker": "nvda-above-in-january-2026",
                "slug": "nvda-above-in-january-2026",
                "title": "Will NVIDIA (NVDA) close above ___ end of January?",
                "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\"."
              },
              "timestamp": "1766790415550",
              "tags": ["stocks"],
              "condition_id": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
              "active": true,
              "clob_token_ids": [
                "76043073756653678226373981964075571318267289248134717369284518995922789326425",
                "31690934263385727664202099278545688007799199447969475608906331829650099442770"
              ],
              "sports_market_type": "",
              "line": "",
              "game_start_time": "",
              "order_price_min_tick_size": "0.01",
              "group_item_title": "NVDA above $240"
            }"#,
        ) else {
            panic!("expected a new_market event");
        };
        assert_eq!(market.id, MarketId::from("1031769"));
        assert_eq!(market.slug, "nvda-above-240-on-january-30-2026");
        assert_eq!(market.assets_ids.len(), 2);
        assert_eq!(market.outcomes, vec!["Yes", "No"]);
        let parent = market.event_message.as_ref().unwrap();
        assert_eq!(parent.id, Some(EventId::from("125819")));
        assert_eq!(parent.ticker.as_deref(), Some("nvda-above-in-january-2026"));
        assert_eq!(
            market.timestamp_millis().map(|t| t.timestamp_millis()),
            Some(1_766_790_415_550)
        );
        assert_eq!(market.tags, Some(vec!["stocks".to_owned()]));
        assert_eq!(market.condition_id, Some(market.market.clone()));
        assert_eq!(market.active, Some(true));
        assert_eq!(
            market.clob_token_ids.as_deref(),
            Some(&market.assets_ids[..])
        );
        assert_eq!(market.sports_market_type.as_deref(), Some(""));
        assert_eq!(market.line.as_deref(), Some(""));
        assert_eq!(market.game_start_time, None);
        assert_eq!(market.order_price_min_tick_size, Some(Decimal::new(1, 2)));
        assert_eq!(market.group_item_title.as_deref(), Some("NVDA above $240"));
    }

    #[test]
    fn deserializes_new_market_game_start_time() {
        let MarketEvent::NewMarket(market) = event(
            r#"{"event_type":"new_market","id":"1","question":"q","market":"0x1","slug":"s",
                "assets_ids":[],"outcomes":[],"timestamp":"1","game_start_time":"2025-02-03T19:00:00Z"}"#,
        ) else {
            panic!("expected a new_market event");
        };
        assert_eq!(
            market.game_start_time.map(|t| t.timestamp()),
            Some(1_738_609_200)
        );
        assert_eq!(market.event_message, None);
        assert_eq!(market.description, None);
    }

    #[test]
    fn deserializes_market_resolved() {
        let MarketEvent::MarketResolved(resolved) = event(
            r#"{
              "event_type": "market_resolved",
              "id": "1031769",
              "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
              "assets_ids": [
                "76043073756653678226373981964075571318267289248134717369284518995922789326425",
                "31690934263385727664202099278545688007799199447969475608906331829650099442770"
              ],
              "winning_asset_id": "76043073756653678226373981964075571318267289248134717369284518995922789326425",
              "winning_outcome": "Yes",
              "timestamp": "1766790415550",
              "tags": ["stocks"]
            }"#,
        ) else {
            panic!("expected a market_resolved event");
        };
        assert_eq!(resolved.id, "1031769");
        assert_eq!(resolved.winning_asset_id, resolved.assets_ids[0]);
        assert_eq!(resolved.winning_outcome, "Yes");
        assert_eq!(resolved.event_message, None);
        assert_eq!(resolved.tags, Some(vec!["stocks".to_owned()]));
    }

    #[test]
    fn unknown_messages_are_preserved() {
        let raw = r#"{"event_type":"brand_new","x":1}"#;
        assert_eq!(
            event(raw),
            MarketEvent::Unknown(serde_json::from_str(raw).unwrap())
        );
        assert_eq!(
            event(r#"{"no_type":true}"#),
            MarketEvent::Unknown(serde_json::json!({"no_type": true}))
        );
    }

    #[test]
    fn known_type_with_bad_shape_fails() {
        let err = serde_json::from_str::<MarketEvent>(r#"{"event_type":"book","market":"0x1"}"#)
            .unwrap_err();
        assert!(err.to_string().contains("invalid `book` event"), "{err}");
    }

    #[test]
    fn serializes_events_with_their_tag() {
        let raw = r#"{"event_type":"tick_size_change","asset_id":"1","market":"0x1","old_tick_size":"0.01","new_tick_size":"0.001","timestamp":"1757908892351"}"#;
        let value = serde_json::to_value(event(raw)).unwrap();
        assert_eq!(value, serde_json::from_str::<Value>(raw).unwrap());

        let book = r#"{"event_type":"book","asset_id":"1","market":"0x1","bids":[{"price":"0.5","size":"1"}],"asks":[],"timestamp":"1757908892351","hash":"h"}"#;
        let value = serde_json::to_value(event(book)).unwrap();
        assert_eq!(value, serde_json::from_str::<Value>(book).unwrap());

        let unknown = serde_json::json!({"event_type": "brand_new"});
        assert_eq!(
            serde_json::to_value(MarketEvent::Unknown(unknown.clone())).unwrap(),
            unknown
        );
    }

    #[test]
    fn frames_with_arrays_decode_to_several_events() {
        let mut out = VecDeque::new();
        decode_frame::<MarketEvent>(
            Service::MarketChannel,
            r#"[{"event_type":"tick_size_change","asset_id":"1","market":"0x1","old_tick_size":"0.01","new_tick_size":"0.001","timestamp":"1"},
                {"event_type":"book"},
                {"event_type":"other"}]"#,
            &mut out,
        );
        assert!(matches!(out[0], Ok(MarketEvent::TickSizeChange(_))));
        assert!(matches!(out[1], Err(Error::WebSocket(_))));
        assert!(matches!(out[2], Ok(MarketEvent::Unknown(_))));
    }
}
