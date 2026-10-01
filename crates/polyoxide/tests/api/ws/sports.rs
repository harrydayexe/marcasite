//! Sports channel (`/ws`) against a mock server.
//!
//! Message bodies are the examples in `docs/specs/asyncapi-sports.json`.

use std::time::Duration;

use futures_core::stream::FusedStream as _;
use polyoxide::{
    Error, WebSocketErrorKind,
    ws::{SportsChannel, SportsEvent},
};
use serde_json::json;

use super::mock::{drain, next, recv_text, send, serve};

const SOCCER: &str = r#"{"slug":"mci-liv-2025-02-03","live":true,"ended":false,"score":"1-0","period":"1H","elapsed":"32:15","last_update":"2025-02-03T19:50:16.939Z"}"#;
const NFL: &str = r#"{"slug":"sea-sf-2025-02-03","live":true,"ended":false,"score":"14-7","period":"Q2","elapsed":"08:45","last_update":"2025-02-03T20:15:30.123Z","turn":"sea"}"#;
const FINISHED: &str = r#"{"slug":"ars-che-2025-02-03","live":false,"ended":true,"score":"2-1","period":"FT","elapsed":"","last_update":"2025-02-03T22:00:00.000Z","finished_timestamp":"2025-02-03T21:55:00.000Z"}"#;

#[tokio::test]
async fn answers_ping_and_decodes_updates() {
    let (url, server) = serve("/ws", |mut socket| async move {
        // No subscription is sent: the client's first frame is the answer to `ping`.
        send(&mut socket, "ping").await;
        assert_eq!(recv_text(&mut socket).await, "pong");
        send(&mut socket, SOCCER).await;
        send(&mut socket, "ping").await;
        assert_eq!(recv_text(&mut socket).await, "pong");
        send(&mut socket, &format!("[{NFL},{FINISHED}]")).await;
        send(&mut socket, r#"{"type":"announcement"}"#).await;
        // Drop the TCP connection without a close frame.
    })
    .await;

    let mut channel = SportsChannel::builder().url(&url).connect().await.unwrap();

    let Some(Ok(SportsEvent::Update(soccer))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert_eq!(soccer.slug, "mci-liv-2025-02-03");
    assert_eq!(soccer.score.as_deref(), Some("1-0"));

    let Some(Ok(SportsEvent::Update(nfl))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert_eq!(nfl.turn.as_deref(), Some("sea"));

    let Some(Ok(SportsEvent::Update(finished))) = next(&mut channel).await else {
        panic!("expected a sports update");
    };
    assert_eq!(finished.ended, Some(true));
    assert!(finished.finished_timestamp.is_some());

    let Some(Ok(SportsEvent::Unknown(unknown))) = next(&mut channel).await else {
        panic!("expected an unknown message");
    };
    assert_eq!(unknown, json!({"type": "announcement"}));

    // The server vanished without a close frame: one final error, then the end.
    let Some(Err(Error::WebSocket(err))) = next(&mut channel).await else {
        panic!("expected a connection error");
    };
    assert_eq!(err.service(), polyoxide::Service::SportsChannel);
    assert_eq!(err.kind(), WebSocketErrorKind::Closed, "{err}");
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn idle_timeout_ends_a_silent_connection() {
    let (url, server) = serve("/ws", |mut socket| async move {
        send(&mut socket, SOCCER).await;
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
