//! Sports results channel: `wss://sports-api.polymarket.com/ws`.
//!
//! Spec: `docs/specs/asyncapi-sports.json`; page: `docs/api-reference/wss/sports.md`.
//!
//! The live channel does not match the spec: see [`SportResult`] and `SPEC_DEVIATIONS.md`.

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use chrono::{DateTime, Utc};
use futures_core::{Stream, stream::FusedStream};
use marcasite_core::{Result, Service, serde_util, ws::WsConnection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::frame::{
    ChannelEvent, ConnectOptions, EventStream, IdleTimeout, Rejected, deserialize_via_from_value,
};

/// The text heartbeat the spec documents (`ping` every 5 s). The live server does not send
/// it (it sends protocol-level ping frames instead, which are answered automatically), but
/// it is still answered in case it does.
const PING: &str = "ping";
/// The client's reply to [`PING`].
const PONG: &str = "pong";

/// A live connection to the sports results channel: score, period and status updates for
/// every active sports event.
///
/// No subscription is needed: the server broadcasts every update to every client.
///
/// **Heartbeats.** The spec says the server sends a text `ping` every 5 seconds and closes
/// connections that do not answer `pong` within 10 seconds. Live (2026-10-02) the server
/// instead sends a WebSocket protocol-level ping frame every 15 seconds and never a text
/// `ping`. Both are answered automatically and neither is surfaced as an event. If nothing
/// at all arrives for [`DEFAULT_IDLE_TIMEOUT`](Self::DEFAULT_IDLE_TIMEOUT) (three times the
/// observed 15-second ping; configurable with [`SportsChannelBuilder::idle_timeout`]), the
/// connection is considered dead and the stream ends with an error. See the
/// [module documentation](super) for error handling and reconnection.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
///
/// ```no_run
/// # async fn run() -> marcasite::Result<()> {
/// use futures_util::StreamExt as _;
/// use marcasite::ws::{SportsChannel, SportsEvent};
///
/// let mut channel = SportsChannel::connect().await?;
/// while let Some(event) = channel.next().await {
///     if let SportsEvent::Update(result) = event? {
///         println!(
///             "{} {:?} v {:?}: {} ({})",
///             result.league_abbreviation, result.home_team, result.away_team, result.score,
///             result.period,
///         );
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

    /// The default idle timeout: three times the 15-second interval at which the live
    /// server sends its protocol-level ping (the spec documents a text `ping` every 5
    /// seconds, which live does not send; see [`SportsChannelBuilder::idle_timeout`]).
    pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(45);

    /// Connects to the production URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`](crate::Error::WebSocket) if the connection cannot be
    /// established.
    pub async fn connect() -> Result<Self> {
        Self::builder().connect().await
    }

    /// Returns a builder for a custom URL, buffer size or timeouts.
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

    /// Sets the idle timeout (default [`SportsChannel::DEFAULT_IDLE_TIMEOUT`]).
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
    /// Returns [`Error::Config`](crate::Error::Config) if the URL is invalid or the idle
    /// timeout is zero, or [`Error::WebSocket`](crate::Error::WebSocket) if the connection
    /// cannot be established.
    pub async fn connect(self) -> Result<SportsChannel> {
        let config = self
            .options
            .config(
                Service::SportsChannel,
                SportsChannel::DEFAULT_URL,
                SportsChannel::DEFAULT_IDLE_TIMEOUT,
            )?
            .auto_reply(PING, PONG);
        let conn = WsConnection::connect(config).await?;
        Ok(SportsChannel {
            events: EventStream::new(conn),
        })
    }
}

/// A message received on the sports channel.
///
/// The channel carries a single message type, the match update ([`SportResult`]),
/// recognised by its `gameId` or `metadataGameId` field. Anything else (including a frame
/// that is not JSON, and the spec's `slug`-keyed shape, which the live channel does not
/// send) becomes [`SportsEvent::Unknown`] so that new server messages never break the
/// stream.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SportsEvent {
    /// A live match update: score change, period change, match started or ended. Boxed
    /// because it is much larger than [`Unknown`](Self::Unknown).
    Update(Box<SportResult>),
    /// A message this version of the library does not recognise, as raw JSON (a frame that
    /// is not JSON is kept as a JSON string).
    Unknown(Value),
}

impl ChannelEvent for SportsEvent {
    fn from_value(value: Value) -> std::result::Result<Self, Rejected> {
        if value.get("gameId").is_none() && value.get("metadataGameId").is_none() {
            return Ok(Self::Unknown(value));
        }
        match SportResult::deserialize(&value) {
            Ok(result) => Ok(Self::Update(Box::new(result))),
            Err(e) => Err(Rejected {
                reason: format!("invalid sports result: {e}"),
                value,
            }),
        }
    }
}

deserialize_via_from_value!(SportsEvent);

/// A string that the API may send as a JSON string or a JSON integer.
mod string_or_integer_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
    use serde_json::Value;

    pub(super) fn serialize<S: Serializer>(
        value: &Option<String>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<String>, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            Value::Number(n) if n.is_u64() || n.is_i64() => Ok(Some(n.to_string())),
            other => Err(D::Error::custom(format!(
                "expected a string or an integer, got {other}"
            ))),
        }
    }
}

/// A real-time sports match update, as the live channel sends it.
///
/// **This is the live shape, not the spec's.** `docs/specs/asyncapi-sports.json` describes
/// a `SportResult` keyed by `slug` with snake_case fields (`last_update`,
/// `finished_timestamp`); the live channel (observed 2026-10-02 across tennis, cricket,
/// esports and MLBB, 155 frames) sends camelCase fields, no `slug` and no `last_update`.
/// The spec-only fields were dropped, and the live ones are modelled; see
/// `SPEC_DEVIATIONS.md`.
///
/// Live frames come in three flavours:
///
/// - a running game: `gameId`, `leagueAbbreviation`, `homeTeam`, `awayTeam`, `status`,
///   `score`, `period`, `live`, `ended` (and, on some games, `elapsed`, `turn`,
///   `turnProviderId` and `sportradarGameId`);
/// - a game that has just ended: `metadataGameId` (a string such as `"id2704888975110644"`)
///   instead of `gameId`, `finishedTimestamp`, `live: false`, `ended: true`, `period`
///   `"FT"`, and no teams or status;
/// - the same without `finishedTimestamp`.
///
/// So exactly one of [`game_id`](Self::game_id) and
/// [`metadata_game_id`](Self::metadata_game_id) is normally present. They are different
/// id spaces (an integer and a string) and are not known to be related, so they are two
/// fields rather than one.
///
/// See <https://docs.polymarket.com/api-reference/wss/sports>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SportResult {
    /// Numeric game id (`gameId`), e.g. `6365478`. Present on running-game updates.
    pub game_id: Option<u64>,
    /// String game id (`metadataGameId`), e.g. `"id2704888975110644"`. Present on the
    /// updates that end a game, in place of [`game_id`](Self::game_id).
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
    /// [`status_is`](Self::status_is) to compare without regard to case. Absent on the
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
    #[serde(default, with = "string_or_integer_option")]
    pub turn_provider_id: Option<String>,
    /// Sportradar game id (`sportradarGameId`). Observed on some earlier frames; kept as a
    /// string (a JSON string or integer on the wire).
    #[serde(default, with = "string_or_integer_option")]
    pub sportradar_game_id: Option<String>,
    /// When the game ended (`finishedTimestamp`, RFC 3339 with up to nanosecond precision,
    /// e.g. `2026-10-02T02:36:43.364683616Z`). Only on the updates that end a game, and
    /// not on all of them.
    #[serde(default, with = "serde_util::datetime_option")]
    pub finished_timestamp: Option<DateTime<Utc>>,
}

impl SportResult {
    /// Whether [`status`](Self::status) equals `status`, ignoring ASCII case (live sends
    /// both `inprogress` and `InProgress`).
    #[must_use]
    pub fn status_is(&self, status: &str) -> bool {
        self.status
            .as_deref()
            .is_some_and(|own| own.eq_ignore_ascii_case(status))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    fn result(json: &str) -> SportResult {
        match serde_json::from_str(json).unwrap() {
            SportsEvent::Update(result) => *result,
            other => panic!("expected a sports result, got {other:?}"),
        }
    }

    // Fixtures: frames captured from `wss://sports-api.polymarket.com/ws` on 2026-10-02
    // (the spec's `slug` shape in `docs/specs/asyncapi-sports.json` is not sent live).

    const TENNIS: &str = r#"{"gameId":6365478,"leagueAbbreviation":"wta challenger","homeTeam":"Alexandra Shubladze","awayTeam":"Sijia Wei","status":"inprogress","score":"6-7(3-7), 6-3, 2-1","period":"S3","live":true,"ended":false}"#;
    const ESPORTS: &str = r#"{"gameId":1697663,"leagueAbbreviation":"lol","homeTeam":"Solary","awayTeam":"T1 Academy","status":"running","score":"000-000|0-1|Bo5","period":"2/5","live":true,"ended":false}"#;
    const FINISHED: &str = r#"{"metadataGameId":"id2704888975110644","leagueAbbreviation":"cricket","score":"123-125","period":"FT","live":false,"ended":true,"finishedTimestamp":"2026-10-02T09:46:11.137533661Z"}"#;
    const FINISHED_WITHOUT_TIME: &str = r#"{"metadataGameId":"id2704888975110644","leagueAbbreviation":"cricket","score":"0-1","period":"FT","live":false,"ended":true}"#;

    #[test]
    fn deserializes_running_games() {
        let tennis = result(TENNIS);
        assert_eq!(tennis.game_id, Some(6_365_478));
        assert_eq!(tennis.metadata_game_id, None);
        assert_eq!(tennis.league_abbreviation, "wta challenger");
        assert_eq!(tennis.home_team.as_deref(), Some("Alexandra Shubladze"));
        assert_eq!(tennis.away_team.as_deref(), Some("Sijia Wei"));
        assert_eq!(tennis.score, "6-7(3-7), 6-3, 2-1");
        assert_eq!(tennis.period, "S3");
        assert!(tennis.live && !tennis.ended);
        assert_eq!(tennis.elapsed, None);
        assert_eq!(tennis.turn, None);
        assert_eq!(tennis.finished_timestamp, None);
        assert!(tennis.status_is("InProgress") && tennis.status_is("inprogress"));
        assert!(!tennis.status_is("running"));

        let esports = result(ESPORTS);
        assert_eq!(esports.score, "000-000|0-1|Bo5");
        assert_eq!(esports.period, "2/5");
        assert!(esports.status_is("running"));
    }

    #[test]
    fn deserializes_finished_game_with_nanosecond_timestamp() {
        let finished = result(FINISHED);
        assert_eq!(finished.game_id, None);
        assert_eq!(
            finished.metadata_game_id.as_deref(),
            Some("id2704888975110644")
        );
        assert_eq!(finished.home_team, None);
        assert_eq!(finished.status, None);
        assert!(!finished.live && finished.ended);
        assert_eq!(finished.period, "FT");
        let time = finished.finished_timestamp.unwrap();
        assert_eq!(
            time.date_naive(),
            Utc.with_ymd_and_hms(2026, 10, 2, 0, 0, 0)
                .unwrap()
                .date_naive()
        );
        assert_eq!(time.timestamp_subsec_nanos(), 137_533_661);

        assert_eq!(result(FINISHED_WITHOUT_TIME).finished_timestamp, None);
    }

    #[test]
    fn mixed_case_status_is_kept_raw() {
        let update = result(
            r#"{"gameId":1,"leagueAbbreviation":"nba","homeTeam":"a","awayTeam":"b","status":"InProgress","score":"1-0","period":"Q1","live":true,"ended":false}"#,
        );
        assert_eq!(update.status.as_deref(), Some("InProgress"));
        assert!(update.status_is("inprogress"));
    }

    #[test]
    fn optional_extras_are_modelled() {
        let update = result(
            r#"{"gameId":2,"leagueAbbreviation":"nfl","homeTeam":"a","awayTeam":"b","status":"inprogress","score":"14-7","period":"Q2","elapsed":"08:45","turn":"sea","turnProviderId":"77","sportradarGameId":"sr:match:1","live":true,"ended":false}"#,
        );
        assert_eq!(update.elapsed.as_deref(), Some("08:45"));
        assert_eq!(update.turn.as_deref(), Some("sea"));
        assert_eq!(update.turn_provider_id.as_deref(), Some("77"));
        assert_eq!(update.sportradar_game_id.as_deref(), Some("sr:match:1"));

        // Integers are accepted for the provider ids and kept as strings.
        let update = result(
            r#"{"gameId":2,"leagueAbbreviation":"nfl","score":"0-0","period":"Q1","turnProviderId":77,"sportradarGameId":12345,"live":true,"ended":false}"#,
        );
        assert_eq!(update.turn_provider_id.as_deref(), Some("77"));
        assert_eq!(update.sportradar_game_id.as_deref(), Some("12345"));
    }

    #[test]
    fn spec_shape_and_other_messages_are_unknown() {
        // The spec's `slug`-keyed shape is not what the live channel sends.
        let spec = r#"{"slug":"mci-liv-2025-02-03","live":true,"ended":false,"score":"1-0","period":"1H"}"#;
        let event: SportsEvent = serde_json::from_str(spec).unwrap();
        assert!(matches!(event, SportsEvent::Unknown(_)), "{event:?}");

        let event: SportsEvent = serde_json::from_str(r#"{"type":"hello"}"#).unwrap();
        assert_eq!(
            event,
            SportsEvent::Unknown(serde_json::json!({"type": "hello"}))
        );
        // A recognised message with a wrong shape is an error, not `Unknown`.
        assert!(serde_json::from_str::<SportsEvent>(r#"{"gameId":"x"}"#).is_err());
        assert!(serde_json::from_str::<SportsEvent>(r#"{"gameId":1}"#).is_err());
    }

    #[test]
    fn serializes_back_to_the_live_shape() {
        let event: SportsEvent = serde_json::from_str(TENNIS).unwrap();
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["gameId"], 6_365_478);
        assert_eq!(value["leagueAbbreviation"], "wta challenger");
        assert_eq!(value["homeTeam"], "Alexandra Shubladze");
        assert!(value.get("slug").is_none());
        let again: SportsEvent = serde_json::from_value(value).unwrap();
        assert_eq!(again, event);

        let finished: SportsEvent = serde_json::from_str(FINISHED).unwrap();
        let value = serde_json::to_value(&finished).unwrap();
        assert_eq!(value["finishedTimestamp"], "2026-10-02T09:46:11.137533661Z");
    }
}
