//! A minimal one-connection WebSocket mock server.

use std::{future::Future, time::Duration};

use futures_util::{SinkExt as _, Stream, StreamExt as _};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_tungstenite::{
    WebSocketStream, accept_async,
    tungstenite::{
        Message,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};

/// The server side of a mock connection.
pub type ServerSocket = WebSocketStream<tokio::net::TcpStream>;

/// How long a test waits for any single message before failing.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Accepts one connection on a random local port and runs `handler` on it.
///
/// Returns the `ws://` URL (with `path`) and the server task; await the task at the end of
/// the test so that assertion failures inside the handler fail the test.
pub async fn serve<F, Fut>(path: &str, handler: F) -> (String, JoinHandle<()>)
where
    F: FnOnce(ServerSocket) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let socket = accept_async(stream).await.unwrap();
        handler(socket).await;
    });
    (format!("ws://{addr}{path}"), task)
}

/// Receives the next text frame sent by the client (skipping protocol pings/pongs).
pub async fn recv_text(socket: &mut ServerSocket) -> String {
    loop {
        let message = tokio::time::timeout(TIMEOUT, socket.next())
            .await
            .expect("timed out waiting for a client frame")
            .expect("client disconnected")
            .expect("socket error");
        match message {
            Message::Text(text) => return text.as_str().to_owned(),
            Message::Ping(_) | Message::Pong(_) => {}
            other => panic!("unexpected client frame {other:?}"),
        }
    }
}

/// Receives the next frame of any kind sent by the client; `None` once the connection has
/// ended.
pub async fn recv_message(socket: &mut ServerSocket) -> Option<Message> {
    tokio::time::timeout(TIMEOUT, socket.next())
        .await
        .expect("timed out waiting for a client frame")?
        .ok()
}

/// Receives the next text frame and parses it as JSON.
pub async fn recv_json(socket: &mut ServerSocket) -> serde_json::Value {
    let text = recv_text(socket).await;
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("client sent invalid JSON {text:?}: {e}"))
}

/// Sends a text frame.
pub async fn send(socket: &mut ServerSocket, text: &str) {
    socket.send(Message::text(text)).await.unwrap();
}

/// Closes the connection with `code` and `reason`.
pub async fn close(socket: &mut ServerSocket, code: u16, reason: &str) {
    socket
        .close(Some(CloseFrame {
            code: CloseCode::from(code),
            reason: reason.into(),
        }))
        .await
        .unwrap();
}

/// Reads and discards client frames until the client goes away.
pub async fn drain(socket: &mut ServerSocket) {
    while let Some(Ok(message)) = socket.next().await {
        if message.is_close() {
            break;
        }
    }
}

/// The next item of a client stream, failing the test after a timeout.
pub async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
    tokio::time::timeout(TIMEOUT, stream.next())
        .await
        .expect("timed out waiting for an event")
}

/// Calls `f` until it fails (e.g. a send after the connection has terminated, which the
/// driver task notices asynchronously), failing the test after a timeout.
pub async fn eventually_err<T>(mut f: impl FnMut() -> marcasite::Result<T>) -> marcasite::Error {
    let deadline = tokio::time::Instant::now() + TIMEOUT;
    loop {
        match f() {
            Err(err) => return err,
            Ok(_) => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "timed out waiting for an error"
                );
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }
    }
}
