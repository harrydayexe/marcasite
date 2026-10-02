//! Sports channel (`/ws`) against a mock server.
//!
//! Message bodies are frames captured from `wss://sports-api.polymarket.com/ws` on
//! 2026-10-02 (the spec's `slug` shape is not sent live).

use std::time::Duration;

use futures_core::stream::FusedStream as _;
use futures_util::SinkExt as _;
use marcasite::{
    Error, WebSocketErrorKind,
    ws::{SportsChannel, SportsEvent},
};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

use super::mock::{drain, next, recv_text, send, serve};

const TENNIS: &str = r#"{"gameId":6365478,"leagueAbbreviation":"wta challenger","homeTeam":"Alexandra Shubladze","awayTeam":"Sijia Wei","status":"inprogress","score":"6-7(3-7), 6-3, 2-1","period":"S3","live":true,"ended":false}"#;
const ESPORTS: &str = r#"{"gameId":1697663,"leagueAbbreviation":"lol","homeTeam":"Solary","awayTeam":"T1 Academy","status":"running","score":"000-000|0-1|Bo5","period":"2/5","live":true,"ended":false}"#;
const FINISHED: &str = r#"{"metadataGameId":"id2704888975110644","leagueAbbreviation":"cricket","score":"123-125","period":"FT","live":false,"ended":true,"finishedTimestamp":"2026-10-02T09:46:11.137533661Z"}"#;
/// The spec's shape (`slug`, `last_update`), which the live channel does not send.
const SPEC_SHAPE: &str = r#"{"slug":"mci-liv-2025-02-03","live":true,"ended":false,"score":"1-0","period":"1H","elapsed":"32:15","last_update":"2025-02-03T19:50:16.939Z"}"#;

#[tokio::test]
async fn answers_ping_and_decodes_updates() {
    let (url, server) = serve("/ws", |mut socket| async move {
        // No subscription is sent: the client's first frame is the answer to `ping`.
        send(&mut socket, "ping").await;
        assert_eq!(recv_text(&mut socket).await, "pong");
        send(&mut socket, TENNIS).await;
        send(&mut socket, "ping").await;
        assert_eq!(recv_text(&mut socket).await, "pong");
        send(&mut socket, &format!("[{ESPORTS},{FINISHED}]")).await;
        send(&mut socket, r#"{"type":"announcement"}"#).await;
        // Drop the TCP connection without a close frame.
    })
    .await;

    let mut channel = SportsChannel::builder().url(&url).connect().await.unwrap();

    let Some(Ok(SportsEvent::Update(tennis))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert_eq!(tennis.game_id, Some(6_365_478));
    assert_eq!(tennis.score, "6-7(3-7), 6-3, 2-1");

    let Some(Ok(SportsEvent::Update(esports))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert_eq!(esports.league_abbreviation, "lol");
    assert!(esports.status_is("running"));

    let Some(Ok(SportsEvent::Update(finished))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert!(finished.ended);
    assert_eq!(
        finished.metadata_game_id.as_deref(),
        Some("id2704888975110644")
    );
    assert!(finished.finished_timestamp.is_some());

    let Some(Ok(SportsEvent::Unknown(unknown))) = next(&mut channel).await else {
        panic!("expected an unknown message");
    };
    assert_eq!(unknown, json!({"type": "announcement"}));

    // The server vanished without a close frame: one final error, then the end.
    let Some(Err(Error::WebSocket(err))) = next(&mut channel).await else {
        panic!("expected a connection error");
    };
    assert_eq!(err.service(), marcasite::Service::SportsChannel);
    assert_eq!(err.kind(), WebSocketErrorKind::Closed, "{err}");
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn idle_timeout_ends_a_silent_connection() {
    let (url, server) = serve("/ws", |mut socket| async move {
        send(&mut socket, TENNIS).await;
        // Then nothing at all: no data and no `ping`.
        drain(&mut socket).await;
    })
    .await;

    let mut channel = SportsChannel::builder()
        .url(&url)
        .idle_timeout(Duration::from_millis(500))
        .connect()
        .await
        .unwrap();
    let Some(Ok(SportsEvent::Update(_))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    let Some(Err(Error::WebSocket(err))) = next(&mut channel).await else {
        panic!("expected an idle timeout");
    };
    assert_eq!(err.kind(), WebSocketErrorKind::Timeout, "{err}");
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn zero_idle_timeout_is_a_config_error() {
    let err = SportsChannel::builder()
        .url("ws://127.0.0.1:1/ws")
        .idle_timeout(Duration::ZERO)
        .connect()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    // Disabling it is fine (the connection then fails because nothing listens).
    let err = SportsChannel::builder()
        .url("ws://127.0.0.1:1/ws")
        .no_idle_timeout()
        .connect()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::WebSocket(_)), "{err:?}");
}

/// The spec's `slug` shape is not recognised any more: it is kept as `Unknown`.
#[tokio::test]
async fn spec_shape_is_unknown() {
    let (url, server) = serve("/ws", |mut socket| async move {
        send(&mut socket, SPEC_SHAPE).await;
        drain(&mut socket).await;
    })
    .await;
    let mut channel = SportsChannel::builder().url(&url).connect().await.unwrap();
    let Some(Ok(SportsEvent::Unknown(value))) = next(&mut channel).await else {
        panic!("expected an unknown message");
    };
    assert_eq!(value["slug"], "mci-liv-2025-02-03");
    channel.close();
    server.await.unwrap();
}

/// Live (2026-10-02) the server keeps the connection alive with protocol-level ping frames
/// (every 15 s), not the text `ping` of the spec. They must be answered and must reset the
/// idle timeout, so a quiet channel stays open.
#[tokio::test]
async fn protocol_pings_keep_a_quiet_channel_alive() {
    let (url, server) = serve("/ws", |mut socket| async move {
        // Pings for longer than the client's idle timeout, with no data in between.
        for _ in 0..8 {
            socket.send(Message::Ping(Vec::new().into())).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        send(&mut socket, TENNIS).await;
        drain(&mut socket).await;
    })
    .await;

    let mut channel = SportsChannel::builder()
        .url(&url)
        .idle_timeout(Duration::from_millis(400))
        .connect()
        .await
        .unwrap();
    let Some(Ok(SportsEvent::Update(update))) = next(&mut channel).await else {
        panic!("the idle timeout fired although pings kept arriving");
    };
    assert_eq!(update.game_id, Some(6_365_478));
    channel.close();
    server.await.unwrap();
}

#[test]
fn default_idle_timeout_outlasts_the_live_ping_interval() {
    // The live server pings every 15 s; the default must be a comfortable multiple.
    assert!(SportsChannel::DEFAULT_IDLE_TIMEOUT >= Duration::from_secs(45));
}
