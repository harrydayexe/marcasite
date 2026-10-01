//! A WebSocket connection driver for the streaming channels.
//!
//! [`WsConnection::connect`] performs the handshake, then spawns a background task (on the
//! current Tokio runtime) that owns the socket. The task:
//!
//! - forwards incoming text frames to the [`WsConnection`] handle (which is a [`Stream`]),
//! - sends outgoing text frames queued with [`WsConnection::send_text`],
//! - sends an application-level heartbeat on a fixed interval, if configured,
//! - answers server text pings (e.g. `ping` → `pong`) and drops heartbeat replies
//!   (e.g. `PONG`), if configured,
//! - answers protocol-level ping frames,
//! - reports an abnormal close or socket error as one final [`Error::WebSocket`] item.
//!
//! Dropping the handle closes the connection. The driver does not reconnect: when the
//! stream ends, create a new connection (and re-subscribe).
//!
//! [`Stream`]: futures_core::Stream

use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    Connector,
    tungstenite::{
        self, Message,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use url::Url;

use crate::error::{ConfigError, Error, Result, Service, WebSocketError};

/// Default capacity of the incoming-message buffer.
pub const DEFAULT_BUFFER: usize = 1024;

/// Default handshake timeout.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// An application-level heartbeat sent by the client on a fixed interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heartbeat {
    /// How often to send the heartbeat.
    pub interval: Duration,
    /// The text frame to send (e.g. `PING`).
    pub message: String,
}

/// Settings for a WebSocket connection.
#[derive(Debug, Clone)]
#[must_use]
pub struct WsConfig {
    service: Service,
    url: Url,
    heartbeat: Option<Heartbeat>,
    auto_replies: Vec<(String, String)>,
    ignored: Vec<String>,
    initial_messages: Vec<String>,
    buffer: usize,
    connect_timeout: Duration,
}

impl WsConfig {
    /// Creates a configuration for `service` at `url` (`ws://` or `wss://`).
    pub fn new(service: Service, url: Url) -> Self {
        Self {
            service,
            url,
            heartbeat: None,
            auto_replies: Vec::new(),
            ignored: Vec::new(),
            initial_messages: Vec::new(),
            buffer: DEFAULT_BUFFER,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
        }
    }

    /// Sends `message` every `interval` to keep the connection alive.
    pub fn heartbeat(mut self, interval: Duration, message: impl Into<String>) -> Self {
        self.heartbeat = Some(Heartbeat {
            interval,
            message: message.into(),
        });
        self
    }

    /// Replies with `reply` whenever the server sends exactly `received` (e.g. `ping` →
    /// `pong`). Matched messages are not forwarded.
    pub fn auto_reply(mut self, received: impl Into<String>, reply: impl Into<String>) -> Self {
        self.auto_replies.push((received.into(), reply.into()));
        self
    }

    /// Drops incoming text frames equal to `message` (e.g. heartbeat replies such as
    /// `PONG`) instead of forwarding them.
    pub fn ignore(mut self, message: impl Into<String>) -> Self {
        self.ignored.push(message.into());
        self
    }

    /// Sends `message` immediately after connecting (e.g. a subscription request).
    pub fn initial_message(mut self, message: impl Into<String>) -> Self {
        self.initial_messages.push(message.into());
        self
    }

    /// Sets how many incoming messages may be buffered before the reader applies
    /// back-pressure (default [`DEFAULT_BUFFER`]).
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer.max(1);
        self
    }

    /// Sets the handshake timeout (default [`DEFAULT_CONNECT_TIMEOUT`]).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// The channel being connected to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// The URL being connected to.
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// Parses and validates a WebSocket URL (`ws://` or `wss://`).
///
/// # Errors
///
/// Returns [`Error::Config`] if the URL is invalid or uses another scheme.
pub fn parse_ws_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)
        .map_err(|e| ConfigError::with_source(format!("invalid WebSocket URL `{url}`"), e))?;
    if !matches!(parsed.scheme(), "ws" | "wss") {
        return Err(ConfigError::new(format!(
            "WebSocket URL `{url}` must use ws or wss, not `{}`",
            parsed.scheme()
        ))
        .into());
    }
    Ok(parsed)
}

enum Command {
    Send(String),
    Close,
}

/// A live WebSocket connection: a [`Stream`] of incoming text frames plus a sender.
///
/// The stream yields `Ok(text)` per message, at most one `Err` describing an abnormal
/// termination, then ends.
///
/// [`Stream`]: futures_core::Stream
#[derive(Debug)]
pub struct WsConnection {
    service: Service,
    commands: mpsc::UnboundedSender<Command>,
    incoming: mpsc::Receiver<Result<String>>,
}

impl WsConnection {
    /// Connects, sends the configured initial messages and starts the driver task.
    ///
    /// Must be called from within a Tokio runtime.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the handshake fails or times out, or
    /// [`Error::Config`] if TLS cannot be configured.
    pub async fn connect(config: WsConfig) -> Result<Self> {
        let service = config.service;
        let connector = if config.url.scheme() == "wss" {
            Some(tls_connector(service)?)
        } else {
            None
        };
        tracing::debug!(service = %service, url = %config.url, "connecting websocket");
        let handshake = tokio_tungstenite::connect_async_tls_with_config(
            config.url.as_str(),
            None,
            false,
            connector,
        );
        let (mut socket, response) =
            match tokio::time::timeout(config.connect_timeout, handshake).await {
                Ok(Ok(ok)) => ok,
                Ok(Err(e)) => return Err(ws_error(service, "failed to connect", Some(e))),
                Err(_) => {
                    return Err(ws_error(
                        service,
                        format!(
                            "handshake timed out after {}s",
                            config.connect_timeout.as_secs()
                        ),
                        None,
                    ));
                }
            };
        tracing::debug!(
            service = %service,
            status = response.status().as_u16(),
            "websocket connected"
        );
        for message in &config.initial_messages {
            tracing::trace!(service = %service, message = %message, "sending initial message");
            socket
                .send(Message::text(message.as_str()))
                .await
                .map_err(|e| ws_error(service, "failed to send initial message", Some(e)))?;
        }

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::channel(config.buffer);
        tokio::spawn(drive(socket, config, command_rx, incoming_tx));
        Ok(Self {
            service,
            commands: command_tx,
            incoming: incoming_rx,
        })
    }

    /// The channel this connection belongs to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// Queues a text frame to be sent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] if the connection has already terminated.
    pub fn send_text(&self, text: impl Into<String>) -> Result<()> {
        self.commands
            .send(Command::Send(text.into()))
            .map_err(|_| ws_error(self.service, "connection is closed", None))
    }

    /// Receives the next incoming text frame; `None` once the connection has ended.
    pub async fn recv(&mut self) -> Option<Result<String>> {
        self.incoming.recv().await
    }

    /// Closes the connection gracefully. Messages already received can still be read.
    pub fn close(&self) {
        let _ = self.commands.send(Command::Close);
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Close);
    }
}

impl Stream for WsConnection {
    type Item = Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.incoming.poll_recv(cx)
    }
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn drive(
    mut socket: Socket,
    config: WsConfig,
    mut commands: mpsc::UnboundedReceiver<Command>,
    incoming: mpsc::Sender<Result<String>>,
) {
    let service = config.service;
    let mut heartbeat = config.heartbeat.as_ref().map(|hb| {
        let mut interval =
            tokio::time::interval_at(tokio::time::Instant::now() + hb.interval, hb.interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        (interval, hb.message.clone())
    });

    let failure: Option<Error> = loop {
        let tick = async {
            match heartbeat.as_mut() {
                Some((interval, _)) => {
                    interval.tick().await;
                }
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            command = commands.recv() => match command {
                Some(Command::Send(text)) => {
                    tracing::trace!(service = %service, message = %text, "sending message");
                    if let Err(e) = socket.send(Message::text(text)).await {
                        break Some(ws_error(service, "failed to send message", Some(e)));
                    }
                }
                Some(Command::Close) | None => {
                    tracing::debug!(service = %service, "closing websocket");
                    let _ = socket.close(None).await;
                    break None;
                }
            },
            () = tick => {
                if let Some((_, message)) = &heartbeat {
                    tracing::trace!(service = %service, "sending heartbeat");
                    if let Err(e) = socket.send(Message::text(message.as_str())).await {
                        break Some(ws_error(service, "failed to send heartbeat", Some(e)));
                    }
                }
            },
            frame = socket.next() => match frame {
                Some(Ok(Message::Text(text))) => {
                    let text = text.as_str();
                    if let Some((_, reply)) = config.auto_replies.iter().find(|(r, _)| r == text) {
                        tracing::trace!(service = %service, received = %text, "answering server heartbeat");
                        if let Err(e) = socket.send(Message::text(reply.as_str())).await {
                            break Some(ws_error(service, "failed to answer heartbeat", Some(e)));
                        }
                        continue;
                    }
                    if config.ignored.iter().any(|m| m == text) {
                        continue;
                    }
                    tracing::trace!(service = %service, message = %text, "received message");
                    if incoming.send(Ok(text.to_owned())).await.is_err() {
                        // The handle was dropped.
                        let _ = socket.close(None).await;
                        break None;
                    }
                }
                Some(Ok(Message::Binary(bytes))) => match String::from_utf8(bytes.to_vec()) {
                    Ok(text) => {
                        if incoming.send(Ok(text)).await.is_err() {
                            let _ = socket.close(None).await;
                            break None;
                        }
                    }
                    Err(_) => tracing::debug!(service = %service, "ignoring non-UTF-8 binary frame"),
                },
                Some(Ok(Message::Ping(_))) => {
                    // tungstenite queues the pong; flush so it is sent promptly.
                    if let Err(e) = socket.flush().await {
                        break Some(ws_error(service, "failed to answer ping", Some(e)));
                    }
                }
                Some(Ok(Message::Pong(_) | Message::Frame(_))) => {}
                Some(Ok(Message::Close(frame))) => break close_error(service, frame),
                Some(Err(e)) => break Some(ws_error(service, "connection error", Some(e))),
                None => break Some(ws_error(service, "connection closed without a close frame", None)),
            },
        }
    };

    if let Some(err) = failure {
        tracing::warn!(service = %service, error = %err, "websocket terminated");
        let _ = incoming.send(Err(err)).await;
    } else {
        tracing::debug!(service = %service, "websocket closed");
    }
}

fn close_error(service: Service, frame: Option<CloseFrame>) -> Option<Error> {
    match frame {
        None => {
            tracing::debug!(service = %service, "server closed the connection");
            None
        }
        Some(frame) if frame.code == CloseCode::Normal => {
            tracing::debug!(service = %service, reason = %frame.reason, "server closed the connection");
            None
        }
        Some(frame) => Some(Error::WebSocket(Box::new(
            WebSocketError::new(service, "server closed the connection")
                .with_close(u16::from(frame.code), frame.reason.as_str()),
        ))),
    }
}

fn ws_error(
    service: Service,
    message: impl Into<std::borrow::Cow<'static, str>>,
    source: Option<tungstenite::Error>,
) -> Error {
    let mut err = WebSocketError::new(service, message);
    if let Some(source) = source {
        err = err.with_source(source);
    }
    Error::WebSocket(Box::new(err))
}

/// Builds a rustls connector with an explicit crypto provider (so that no process-wide
/// default provider is required) and the platform's native root certificates.
fn tls_connector(service: Service) -> Result<Connector> {
    let mut roots = rustls::RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    for error in &native.errors {
        tracing::debug!(service = %service, error = %error, "failed to load a native root certificate");
    }
    let (added, ignored) = roots.add_parsable_certificates(native.certs);
    tracing::trace!(service = %service, added, ignored, "loaded native root certificates");
    if roots.is_empty() {
        return Err(ConfigError::new("no native root certificates could be loaded for TLS").into());
    }
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| ConfigError::with_source("failed to configure TLS", e))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Connector::Rustls(Arc::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_validation() {
        assert!(parse_ws_url("wss://ws-subscriptions-clob.polymarket.com/ws/market").is_ok());
        assert!(parse_ws_url("https://example.com").is_err());
        assert!(parse_ws_url("nope").is_err());
    }
}
