//! Sports results channel: `wss://sports-api.polymarket.com/ws`.
//!
//! Spec: `docs/specs/asyncapi-sports.json`; page: `docs/api-reference/wss/sports.md`.

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use chrono::{DateTime, Utc};
use futures_core::{Stream, stream::FusedStream};
use polyoxide_core::{Result, Service, serde_util, ws::WsConnection};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::frame::{ConnectOptions, EventStream};

/// The server's heartbeat text frame.
const PING: &str = "ping";
/// The client's reply to [`PING`].
const PONG: &str = "pong";

/// A live connection to the sports results channel: score, period and status updates for
/// every active sports event.
///
/// No subscription is needed: the server broadcasts every update to every client. The
/// server sends `ping` every 5 seconds and closes connections that do not answer `pong`
/// within 10 seconds; this channel answers automatically and does not surface the
/// heartbeats. See the [module documentation](super) for error handling and reconnection.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
///
/// ```no_run
/// # async fn run() -> polyoxide::Result<()> {
/// use futures_util::StreamExt as _;
/// use polyoxide::ws::{SportsChannel, SportsEvent};
///
/// let mut channel = SportsChannel::connect().await?;
/// while let Some(event) = channel.next().await {
///     if let SportsEvent::Update(result) = event? {
///         println!("{}: {:?} ({:?})", result.slug, result.score, result.period);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Stream items
///
/// Each item is an `Ok(`[`SportsEvent`]`)` or an
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
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct SportsChannel {
    events: EventStream<SportsEvent>,
}

impl SportsChannel {
    /// The production URL.
    pub const DEFAULT_URL: &'static str = "wss://sports-api.polymarket.com/ws";

    /// Connects to the production URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`](crate::Error::WebSocket) if the connection cannot be
    /// established.
    pub async fn connect() -> Result<Self> {
        Self::builder().connect().await
    }

    /// Returns a builder for a custom URL, buffer size or timeout.
    pub fn builder() -> SportsChannelBuilder {
        SportsChannelBuilder::default()
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

impl Stream for SportsChannel {
    type Item = Result<SportsEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().events.poll_next(cx)
    }
}

impl FusedStream for SportsChannel {
    fn is_terminated(&self) -> bool {
        self.events.is_terminated()
    }
}

/// Builder for [`SportsChannel`].
#[derive(Debug, Clone)]
#[must_use]
pub struct SportsChannelBuilder {
    options: ConnectOptions,
}

impl Default for SportsChannelBuilder {
    fn default() -> Self {
        Self {
            options: ConnectOptions::new(),
        }
    }
}

impl SportsChannelBuilder {
    /// Overrides the URL (default [`SportsChannel::DEFAULT_URL`]), e.g. to target a mock
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

    /// Connects.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the URL is invalid, or
    /// [`Error::WebSocket`](crate::Error::WebSocket) if the connection cannot be
    /// established.
    pub async fn connect(self) -> Result<SportsChannel> {
        let config = self
            .options
            .config(Service::SportsChannel, SportsChannel::DEFAULT_URL)?
            .auto_reply(PING, PONG);
        let conn = WsConnection::connect(config).await?;
        Ok(SportsChannel {
            events: EventStream::new(conn),
        })
    }
}

/// A message received on the sports channel.
///
/// The channel documents a single message type, the match update ([`SportResult`]),
/// recognised by its required `slug` field. Anything else (including a frame that is not
/// JSON) becomes [`SportsEvent::Unknown`] so that new server messages never break the
/// stream.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SportsEvent {
    /// A live match update: score change, period change, match started or ended.
    Update(SportResult),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    Unknown(Value),
}

impl<'de> Deserialize<'de> for SportsEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.get("slug").is_some() {
            SportResult::deserialize(&value)
                .map(Self::Update)
                .map_err(|e| serde::de::Error::custom(format!("invalid sports result: {e}")))
        } else {
            Ok(Self::Unknown(value))
        }
    }
}

/// A real-time sports match update (`SportResult`).
///
/// Only `slug` is required; every other field may be omitted when not applicable. Covers
/// NFL, soccer, NBA, MLB, NHL and cricket.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SportResult {
    /// Unique match identifier (e.g. `"mci-liv-2025-02-03"`).
    pub slug: String,
    /// Whether the match is in progress.
    pub live: Option<bool>,
    /// Whether the match has ended.
    pub ended: Option<bool>,
    /// Current score (e.g. `"2-1"` for soccer, `"14-7"` for football).
    pub score: Option<String>,
    /// Current period, as sent. Documented values: soccer `1H`, `2H`, `HT`, `FT`, `PEN`;
    /// NFL and NBA/CBB `Q1`–`Q4`, `HT`, `OT`, `FT`; MLB `Top 1st`, `Bot 1st`, ...; ice
    /// hockey `P1`, `P2`, `P3`, `OT`, `PEN`, `FT`; cricket `1H`, `1A`, `2H`, `2A`, `SO`,
    /// `FT`; other `CAN`, `POST`, `INT`, `AB`.
    pub period: Option<String>,
    /// Elapsed time in the current period (`MM:SS`), or an empty string when not
    /// applicable. Kept as sent.
    pub elapsed: Option<String>,
    /// Time of the last update.
    #[serde(default, with = "serde_util::datetime_option")]
    pub last_update: Option<DateTime<Utc>>,
    /// Time the match ended. Only present for ended matches.
    #[serde(default, with = "serde_util::datetime_option")]
    pub finished_timestamp: Option<DateTime<Utc>>,
    /// Abbreviation of the team in possession. NFL only.
    pub turn: Option<String>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    fn result(json: &str) -> SportResult {
        match serde_json::from_str(json).unwrap() {
            SportsEvent::Update(result) => result,
            other => panic!("expected a sports result, got {other:?}"),
        }
    }

    // Examples: `components/messages/sportsUpdate` in `docs/specs/asyncapi-sports.json`.

    #[test]
    fn deserializes_soccer_update() {
        let update = result(
            r#"{"slug":"mci-liv-2025-02-03","live":true,"ended":false,"score":"1-0","period":"1H","elapsed":"32:15","last_update":"2025-02-03T19:50:16.939Z"}"#,
        );
        assert_eq!(update.slug, "mci-liv-2025-02-03");
        assert_eq!(update.live, Some(true));
        assert_eq!(update.ended, Some(false));
        assert_eq!(update.score.as_deref(), Some("1-0"));
        assert_eq!(update.period.as_deref(), Some("1H"));
        assert_eq!(update.elapsed.as_deref(), Some("32:15"));
        assert_eq!(
            update.last_update.map(|t| t.timestamp_millis()),
            Some(1_738_612_216_939)
        );
        assert_eq!(update.finished_timestamp, None);
        assert_eq!(update.turn, None);
    }

    #[test]
    fn deserializes_nfl_update_with_possession() {
        let update = result(
            r#"{"slug":"sea-sf-2025-02-03","live":true,"ended":false,"score":"14-7","period":"Q2","elapsed":"08:45","last_update":"2025-02-03T20:15:30.123Z","turn":"sea"}"#,
        );
        assert_eq!(update.turn.as_deref(), Some("sea"));
        assert_eq!(update.period.as_deref(), Some("Q2"));
    }

    #[test]
    fn deserializes_other_sports() {
        for json in [
            r#"{"slug":"lal-gsw-2025-02-03","live":true,"ended":false,"score":"78-82","period":"Q3","elapsed":"05:30","last_update":"2025-02-03T21:30:45.789Z"}"#,
            r#"{"slug":"nyy-bos-2025-06-15","live":true,"ended":false,"score":"3-2","period":"Top 5th","elapsed":"","last_update":"2025-06-15T23:30:00.000Z"}"#,
            r#"{"slug":"tor-mtl-2025-02-03","live":true,"ended":false,"score":"2-2","period":"P2","elapsed":"12:30","last_update":"2025-02-03T20:45:00.000Z"}"#,
        ] {
            let update = result(json);
            assert!(update.last_update.is_some());
            assert_eq!(update.live, Some(true));
        }
    }

    #[test]
    fn deserializes_finished_match() {
        let update = result(
            r#"{"slug":"ars-che-2025-02-03","live":false,"ended":true,"score":"2-1","period":"FT","elapsed":"","last_update":"2025-02-03T22:00:00.000Z","finished_timestamp":"2025-02-03T21:55:00.000Z"}"#,
        );
        assert_eq!(update.ended, Some(true));
        assert_eq!(update.elapsed.as_deref(), Some(""));
        assert_eq!(
            update.finished_timestamp,
            Some(Utc.with_ymd_and_hms(2025, 2, 3, 21, 55, 0).unwrap())
        );
    }

    #[test]
    fn only_slug_is_required() {
        let update = result(r#"{"slug":"x"}"#);
        assert_eq!(update.live, None);
        assert_eq!(update.last_update, None);
    }

    #[test]
    fn other_messages_are_unknown() {
        let event: SportsEvent = serde_json::from_str(r#"{"type":"hello"}"#).unwrap();
        assert_eq!(
            event,
            SportsEvent::Unknown(serde_json::json!({"type": "hello"}))
        );
        assert!(serde_json::from_str::<SportsEvent>(r#"{"slug":1}"#).is_err());
    }

    #[test]
    fn serializes_transparently() {
        let json = r#"{"slug":"x","live":true,"ended":null,"score":null,"period":null,"elapsed":null,"last_update":"2025-02-03T22:00:00Z","finished_timestamp":null,"turn":null}"#;
        let event: SportsEvent = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_string(&event).unwrap(), json);
    }
}
