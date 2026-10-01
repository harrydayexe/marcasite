//! A WebSocket connection driver for the streaming channels.
//!
//! [`WsConnection::connect`] performs the handshake, then spawns a background task (on the
//! Tokio runtime it is called from) that owns the socket. The task:
//!
//! - forwards incoming text frames to the [`WsConnection`] handle (which is a [`Stream`]),
//! - sends outgoing text frames queued with [`WsConnection::send_text`] or a cloneable
//!   [`WsSender`] (from [`WsConnection::sender`], usable from other tasks),
//! - sends an application-level heartbeat on a fixed interval, if configured,
//! - answers server text pings (e.g. `ping` → `pong`) and drops heartbeat replies
//!   (e.g. `PONG`), if configured,
//! - answers protocol-level ping frames,
//! - ends the connection when no frame arrives within the idle timeout, if configured,
//! - completes the close handshake, whichever side closes,
//! - reports an abnormal close, a timeout or a socket error as one final
//!   [`Error::WebSocket`] item.
//!
//! Every [`WebSocketError`] it produces carries a [`WebSocketErrorKind`]: `Connect` for
//! handshake failures (returned by [`WsConnection::connect`]; a handshake the server
//! refused with an HTTP status carries [`WebSocketError::http_status`] and
//! [`WebSocketError::retry_after`]), `Closed` for an abnormal close, a connection that
//! ended without a close frame, or a send on a terminated connection, `Protocol` for
//! socket and protocol errors, `Send` for failed writes, and `Timeout` for an idle or
//! stalled connection.
//!
//! Dropping the [`WsConnection`] closes the connection, even while [`WsSender`]s exist. The
//! driver does not reconnect: when the stream ends, create a new connection (and
//! re-subscribe).
//!
//! # Back-pressure
//!
//! Incoming messages are buffered (see [`WsConfig::buffer`]). When the buffer is full
//! because the consumer does not keep up, the driver stops **reading** from the socket
//! until there is room again, so memory use stays bounded. It keeps **sending** meanwhile:
//! heartbeats, queued frames (e.g. subscription changes) and a requested close go out as
//! usual. Frames the server sends in the meantime, including server pings, are only read
//! (and answered) once the consumer catches up, so a consumer that stalls for longer than
//! the server tolerates unanswered pings is disconnected by the server. The idle timeout
//! does not run while reading is paused.
//!
//! # Timeouts
//!
//! - **Handshake**: [`WsConfig::connect_timeout`] (default [`DEFAULT_CONNECT_TIMEOUT`]).
//! - **Writes**: writing one frame to the socket may take at most
//!   [`WsConfig::write_timeout`] (default [`DEFAULT_WRITE_TIMEOUT`]); a write that does
//!   not complete in time (e.g. because the peer of a half-open connection stopped reading)
//!   ends the connection with an error of kind `Timeout`.
//! - **Liveness**: [`WsConfig::idle_timeout`] (off by default) ends the connection with an
//!   error of kind `Timeout` when no frame of any kind arrives for that long.
//! - **Close handshake**: at most [`CLOSE_TIMEOUT`], after which the socket is dropped.
//!
//! # Runtime
//!
//! [`WsConnection::connect`] must be called from within a Tokio runtime whose I/O and time
//! drivers are enabled (`Builder::enable_all`, as `#[tokio::main]` and `#[tokio::test]`
//! do); the driver task is spawned on that runtime. Outside a runtime, `connect` fails with
//! an error of kind `Connect`. A runtime without the I/O or time driver cannot be detected
//! and makes Tokio panic.
//!
//! # Logging
//!
//! Outgoing frames are logged at `TRACE` with their length only, never their content,
//! because they may carry credentials. Incoming frames are logged at `TRACE` with their
//! content.
//!
//! [`Stream`]: futures_core::Stream

use std::{
    borrow::Cow,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;
use futures_util::{SinkExt as _, StreamExt as _};
use http::header;
use tokio::{
    sync::mpsc::{self, error::TrySendError},
    time::Instant,
};
use tokio_tungstenite::{
    Connector,
    tungstenite::{
        self, Message,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use url::Url;

use crate::error::{ConfigError, Error, Result, Service, WebSocketError, WebSocketErrorKind};

/// Default capacity of the incoming-message buffer.
pub const DEFAULT_BUFFER: usize = 1024;

/// The shortest heartbeat interval accepted by [`WsConfig::heartbeat`].
pub const MIN_HEARTBEAT_INTERVAL: Duration = Duration::from_millis(10);

/// Default handshake timeout.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Default limit for writing one frame to the socket (see [`WsConfig::write_timeout`]).
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the driver waits for the close handshake to complete (sending its close frame
/// and receiving the server's reply, or replying to the server's close frame) before it
/// drops the socket.
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);

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
    write_timeout: Duration,
    idle_timeout: Option<Duration>,
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
            write_timeout: DEFAULT_WRITE_TIMEOUT,
            idle_timeout: None,
        }
    }

    /// Sends `message` every `interval` to keep the connection alive.
    ///
    /// Intervals shorter than [`MIN_HEARTBEAT_INTERVAL`] (including zero) are raised to it.
    pub fn heartbeat(mut self, interval: Duration, message: impl Into<String>) -> Self {
        self.heartbeat = Some(Heartbeat {
            interval: interval.max(MIN_HEARTBEAT_INTERVAL),
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

    /// Sets how many incoming messages may be buffered (default [`DEFAULT_BUFFER`], at
    /// least 1).
    ///
    /// When the buffer is full, the driver stops reading from the socket until the
    /// consumer makes room, but keeps sending heartbeats and queued frames; see
    /// [Back-pressure](self#back-pressure).
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer.max(1);
        self
    }

    /// Sets the handshake timeout (default [`DEFAULT_CONNECT_TIMEOUT`]).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets how long writing one frame to the socket (a queued message, a heartbeat, a
    /// reply to a server ping, or an initial message) may take (default
    /// [`DEFAULT_WRITE_TIMEOUT`]).
    ///
    /// A write that does not complete in time, for example because the peer of a
    /// half-open connection stopped reading, ends the connection with an
    /// [`Error::WebSocket`] of kind [`WebSocketErrorKind::Timeout`] (kind
    /// [`WebSocketErrorKind::Connect`] for initial messages).
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Ends the connection if no frame of any kind arrives for `timeout`: no data, no text
    /// heartbeat or heartbeat reply (even those that are answered or ignored), no
    /// protocol-level ping or pong. Off by default.
    ///
    /// This detects a connection that died silently (a peer gone without closing, a
    /// dropped NAT mapping), which would otherwise leave the stream pending forever. The
    /// stream then yields a final [`Error::WebSocket`] of kind
    /// [`WebSocketErrorKind::Timeout`] and ends. Choose a timeout comfortably longer than
    /// the interval at which the server is known to send something (e.g. its heartbeat,
    /// or its reply to the client's heartbeat). The timer does not run while reading is
    /// paused by [back-pressure](self#back-pressure).
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
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

/// A cloneable handle that queues frames on a [`WsConnection`], obtained from
/// [`WsConnection::sender`].
///
/// Use it to send from other tasks (e.g. to change a subscription) while the connection is
/// consumed as a stream. It does not keep the connection open: once the [`WsConnection`] is
/// dropped or closed, or the connection terminates, sending fails with an
/// [`Error::WebSocket`] of kind [`WebSocketErrorKind::Closed`]. A frame queued just before
/// the connection terminates may be dropped without an error.
#[derive(Debug, Clone)]
pub struct WsSender {
    service: Service,
    commands: mpsc::UnboundedSender<Command>,
}

impl WsSender {
    /// The channel this sender belongs to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// Queues a text frame to be sent.
    ///
    /// Frames are sent even while reading is paused by
    /// [back-pressure](self#back-pressure).
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] of kind [`WebSocketErrorKind::Closed`] if the
    /// connection has already terminated.
    pub fn send_text(&self, text: impl Into<String>) -> Result<()> {
        self.commands.send(Command::Send(text.into())).map_err(|_| {
            ws_error(
                self.service,
                WebSocketErrorKind::Closed,
                "connection is closed",
            )
        })
    }

    /// Closes the connection gracefully (for every holder of the connection and its
    /// senders). Messages already received can still be read from the [`WsConnection`].
    /// Does nothing if the connection has already terminated.
    pub fn close(&self) {
        let _ = self.commands.send(Command::Close);
    }
}

/// A live WebSocket connection: a [`Stream`] of incoming text frames plus a sender.
///
/// The stream yields `Ok(text)` per message, at most one `Err` describing an abnormal
/// termination, then ends. Use [`sender`](Self::sender) to send from other tasks while the
/// stream is consumed.
///
/// [`Stream`]: futures_core::Stream
#[derive(Debug)]
pub struct WsConnection {
    sender: WsSender,
    incoming: mpsc::Receiver<Result<String>>,
}

impl WsConnection {
    /// Connects, sends the configured initial messages and starts the driver task.
    ///
    /// Must be called from within a Tokio runtime with the I/O and time drivers enabled;
    /// the driver task is spawned on that runtime (see [Runtime](self#runtime)).
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] of kind [`WebSocketErrorKind::Connect`] if called
    /// outside a Tokio runtime, or if the handshake fails or times out (for a handshake
    /// refused with an HTTP status, see [`WebSocketError::http_status`] and
    /// [`WebSocketError::retry_after`]), or [`Error::Config`] if TLS cannot be configured.
    pub async fn connect(config: WsConfig) -> Result<Self> {
        let service = config.service;
        let runtime = tokio::runtime::Handle::try_current().map_err(|e| {
            ws_error_from(
                service,
                WebSocketErrorKind::Connect,
                "a WebSocket connection must be opened from within a Tokio runtime",
                e,
            )
        })?;
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
                Ok(Err(e)) => return Err(handshake_error(service, e)),
                Err(_) => {
                    return Err(ws_error(
                        service,
                        WebSocketErrorKind::Connect,
                        format!("handshake timed out after {:?}", config.connect_timeout),
                    ));
                }
            };
        tracing::debug!(
            service = %service,
            status = response.status().as_u16(),
            "websocket connected"
        );
        for message in &config.initial_messages {
            tracing::trace!(service = %service, bytes = message.len(), "sending initial message");
            let send = socket.send(Message::text(message.as_str()));
            match tokio::time::timeout(config.write_timeout, send).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    return Err(ws_error_from(
                        service,
                        WebSocketErrorKind::Connect,
                        "failed to send initial message",
                        e,
                    ));
                }
                Err(_) => {
                    return Err(ws_error(
                        service,
                        WebSocketErrorKind::Connect,
                        format!(
                            "timed out after {:?} sending an initial message",
                            config.write_timeout
                        ),
                    ));
                }
            }
        }

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::channel(config.buffer);
        runtime.spawn(drive(socket, config, command_rx, incoming_tx));
        Ok(Self {
            sender: WsSender {
                service,
                commands: command_tx,
            },
            incoming: incoming_rx,
        })
    }

    /// The channel this connection belongs to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.sender.service
    }

    /// A cloneable sender for this connection, to send frames from other tasks while the
    /// connection is consumed as a stream.
    #[must_use]
    pub fn sender(&self) -> WsSender {
        self.sender.clone()
    }

    /// Queues a text frame to be sent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocket`] of kind [`WebSocketErrorKind::Closed`] if the
    /// connection has already terminated.
    pub fn send_text(&self, text: impl Into<String>) -> Result<()> {
        self.sender.send_text(text)
    }

    /// Receives the next incoming text frame; `None` once the connection has ended.
    pub async fn recv(&mut self) -> Option<Result<String>> {
        self.incoming.recv().await
    }

    /// Closes the connection gracefully. Messages already received can still be read.
    pub fn close(&self) {
        self.sender.close();
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        self.sender.close();
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

/// What the driver does with the socket once the connection ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shutdown {
    /// Closing from this side: send a close frame and wait for the server's reply.
    Close,
    /// The server sent a close frame: send the reply that tungstenite queued.
    Reply,
    /// The connection is broken or unresponsive: drop the socket.
    Abandon,
}

async fn drive(
    mut socket: Socket,
    config: WsConfig,
    mut commands: mpsc::UnboundedReceiver<Command>,
    incoming: mpsc::Sender<Result<String>>,
) {
    let service = config.service;
    let write_timeout = config.write_timeout;
    // A first tick so far in the future that it overflows means no heartbeat at all.
    let mut heartbeat = config.heartbeat.as_ref().and_then(|hb| {
        let start = Instant::now().checked_add(hb.interval)?;
        let mut interval = tokio::time::interval_at(start, hb.interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        Some((interval, hb.message.as_str()))
    });
    // The idle timer is re-armed lazily: frames only record `last_activity`, and when the
    // timer fires it is pushed back to `last_activity + timeout` unless that has passed.
    let mut idle_timeout = config.idle_timeout;
    let mut last_activity = Instant::now();
    let idle = tokio::time::sleep(idle_timeout.unwrap_or(Duration::MAX));
    tokio::pin!(idle);
    // A received message waiting for room in the full `incoming` buffer. While it is set,
    // the socket is not read (back-pressure), but commands and heartbeats are still sent.
    let mut pending: Option<Result<String>> = None;

    let (failure, shutdown) = loop {
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
                    tracing::trace!(service = %service, bytes = text.len(), "sending message");
                    let frame = Some(Message::text(text));
                    if let Err(err) = write(&mut socket, frame, service, write_timeout, "send a message").await {
                        break (Some(err), Shutdown::Abandon);
                    }
                }
                Some(Command::Close) | None => {
                    tracing::debug!(service = %service, "closing websocket");
                    break (None, Shutdown::Close);
                }
            },
            () = tick => {
                if let Some((_, message)) = &heartbeat {
                    tracing::trace!(service = %service, "sending heartbeat");
                    let frame = Some(Message::text(*message));
                    if let Err(err) = write(&mut socket, frame, service, write_timeout, "send a heartbeat").await {
                        break (Some(err), Shutdown::Abandon);
                    }
                }
            },
            permit = incoming.reserve(), if pending.is_some() => match permit {
                Ok(permit) => {
                    if let Some(item) = pending.take() {
                        permit.send(item);
                    }
                    tracing::trace!(service = %service, "receive buffer has room again, resuming reads");
                    // Frames were not read while paused: do not count that time as idle.
                    last_activity = Instant::now();
                }
                // The handle was dropped.
                Err(_) => break (None, Shutdown::Close),
            },
            () = &mut idle, if pending.is_none() && idle_timeout.is_some() => {
                if let Some(timeout) = idle_timeout {
                    match last_activity.checked_add(timeout) {
                        Some(deadline) if deadline > Instant::now() => idle.as_mut().reset(deadline),
                        Some(_) => break (Some(idle_error(service, timeout)), Shutdown::Abandon),
                        // Too far in the future to ever fire.
                        None => idle_timeout = None,
                    }
                }
            },
            frame = socket.next(), if pending.is_none() => {
                last_activity = Instant::now();
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        let text = text.as_str();
                        if let Some((_, reply)) = config.auto_replies.iter().find(|(r, _)| r == text) {
                            tracing::trace!(service = %service, received = %text, "answering server heartbeat");
                            let frame = Some(Message::text(reply.as_str()));
                            if let Err(err) = write(&mut socket, frame, service, write_timeout, "answer a server heartbeat").await {
                                break (Some(err), Shutdown::Abandon);
                            }
                            continue;
                        }
                        if config.ignored.iter().any(|m| m == text) {
                            continue;
                        }
                        tracing::trace!(service = %service, message = %text, "received message");
                        if !deliver(&incoming, &mut pending, text.to_owned(), service) {
                            break (None, Shutdown::Close);
                        }
                    }
                    Some(Ok(Message::Binary(bytes))) => match String::from_utf8(bytes.to_vec()) {
                        Ok(text) => {
                            if !deliver(&incoming, &mut pending, text, service) {
                                break (None, Shutdown::Close);
                            }
                        }
                        Err(_) => tracing::debug!(service = %service, "ignoring non-UTF-8 binary frame"),
                    },
                    Some(Ok(Message::Ping(_))) => {
                        // tungstenite queues the pong; flush so it is sent promptly.
                        if let Err(err) = write(&mut socket, None, service, write_timeout, "answer a ping").await {
                            break (Some(err), Shutdown::Abandon);
                        }
                    }
                    Some(Ok(Message::Pong(_) | Message::Frame(_))) => {}
                    Some(Ok(Message::Close(frame))) => break (close_error(service, frame), Shutdown::Reply),
                    Some(Err(e)) => break (Some(read_error(service, e)), Shutdown::Abandon),
                    None => break (
                        Some(ws_error(
                            service,
                            WebSocketErrorKind::Closed,
                            "connection closed without a close frame",
                        )),
                        Shutdown::Abandon,
                    ),
                }
            },
        }
    };

    shut_down(&mut socket, shutdown, service).await;
    drop(socket);
    // Hand over what was received before the end, then the error (if any).
    if let Some(item) = pending.take() {
        let _ = incoming.send(item).await;
    }
    if let Some(err) = failure {
        tracing::warn!(service = %service, error = %err, "websocket terminated");
        let _ = incoming.send(Err(err)).await;
    } else {
        tracing::debug!(service = %service, "websocket closed");
    }
}

/// Hands a received message to the consumer without waiting. If the buffer is full, the
/// message is parked in `pending`, which pauses reading. Returns `false` if the
/// [`WsConnection`] has been dropped.
fn deliver(
    incoming: &mpsc::Sender<Result<String>>,
    pending: &mut Option<Result<String>>,
    text: String,
    service: Service,
) -> bool {
    match incoming.try_send(Ok(text)) {
        Ok(()) => true,
        Err(TrySendError::Full(item)) => {
            tracing::trace!(service = %service, "receive buffer full, pausing reads");
            *pending = Some(item);
            true
        }
        Err(TrySendError::Closed(_)) => false,
    }
}

/// Writes `message` (or, with `None`, flushes frames tungstenite queued itself, such as a
/// pong) within `limit`.
async fn write(
    socket: &mut Socket,
    message: Option<Message>,
    service: Service,
    limit: Duration,
    action: &'static str,
) -> Result<()> {
    let write = async {
        match message {
            Some(message) => socket.send(message).await,
            None => socket.flush().await,
        }
    };
    match tokio::time::timeout(limit, write).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(ws_error_from(
            service,
            WebSocketErrorKind::Send,
            format!("failed to {action}"),
            e,
        )),
        Err(_) => Err(ws_error(
            service,
            WebSocketErrorKind::Timeout,
            format!("timed out after {limit:?} trying to {action}"),
        )),
    }
}

/// Completes the close handshake as far as `how` asks, within [`CLOSE_TIMEOUT`].
async fn shut_down(socket: &mut Socket, how: Shutdown, service: Service) {
    let handshake = async {
        match how {
            Shutdown::Close => {
                // Send a close frame, then read until the server's reply or the end of the
                // connection (frames received in between are discarded).
                futures_util::SinkExt::close(&mut *socket).await?;
                while let Some(Ok(_)) = socket.next().await {}
            }
            // tungstenite queued the reply to the server's close frame when reading it;
            // closing the sink flushes it.
            Shutdown::Reply => futures_util::SinkExt::close(&mut *socket).await?,
            Shutdown::Abandon => {}
        }
        Ok::<(), tungstenite::Error>(())
    };
    match tokio::time::timeout(CLOSE_TIMEOUT, handshake).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => tracing::debug!(service = %service, error = %e, "close handshake failed"),
        Err(_) => tracing::debug!(service = %service, "close handshake timed out"),
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
            WebSocketError::new(
                service,
                WebSocketErrorKind::Closed,
                "server closed the connection",
            )
            .with_close(u16::from(frame.code), frame.reason.as_str()),
        ))),
    }
}

/// A failed handshake ([`WebSocketErrorKind::Connect`]); a refusal with an HTTP status
/// keeps the status and the `Retry-After` delay.
fn handshake_error(service: Service, error: tungstenite::Error) -> Error {
    let tungstenite::Error::Http(response) = &error else {
        return ws_error_from(
            service,
            WebSocketErrorKind::Connect,
            "failed to connect",
            error,
        );
    };
    let status = response.status();
    let retry_after = response
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_secs);
    tracing::debug!(
        service = %service,
        status = status.as_u16(),
        retry_after_secs = retry_after.map(|d| d.as_secs()),
        "websocket handshake refused"
    );
    let mut err = WebSocketError::new(service, WebSocketErrorKind::Connect, "handshake refused");
    err.http_status = Some(status);
    err.retry_after = retry_after;
    Error::WebSocket(Box::new(err))
}

/// No frame arrived within the idle timeout ([`WebSocketErrorKind::Timeout`]).
fn idle_error(service: Service, timeout: Duration) -> Error {
    ws_error(
        service,
        WebSocketErrorKind::Timeout,
        format!("no frame received for {timeout:?} (idle timeout)"),
    )
}

/// An error reading from an established connection: the peer going away is
/// [`WebSocketErrorKind::Closed`], anything else [`WebSocketErrorKind::Protocol`].
fn read_error(service: Service, error: tungstenite::Error) -> Error {
    let kind = match &error {
        tungstenite::Error::ConnectionClosed
        | tungstenite::Error::AlreadyClosed
        | tungstenite::Error::Protocol(
            tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
        ) => WebSocketErrorKind::Closed,
        _ => WebSocketErrorKind::Protocol,
    };
    ws_error_from(service, kind, "connection error", error)
}

fn ws_error(
    service: Service,
    kind: WebSocketErrorKind,
    message: impl Into<Cow<'static, str>>,
) -> Error {
    Error::WebSocket(Box::new(WebSocketError::new(service, kind, message)))
}

fn ws_error_from(
    service: Service,
    kind: WebSocketErrorKind,
    message: impl Into<Cow<'static, str>>,
    source: impl std::error::Error + Send + Sync + 'static,
) -> Error {
    Error::WebSocket(Box::new(
        WebSocketError::new(service, kind, message).with_source(source),
    ))
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
    use std::future::Future;

    use tokio::net::TcpListener;
    use tokio_tungstenite::{WebSocketStream, accept_async};

    use super::*;

    type ServerSocket = WebSocketStream<tokio::net::TcpStream>;

    /// How long a test waits for any single message before failing.
    const TIMEOUT: Duration = Duration::from_secs(5);

    /// Accepts one connection on a random local port and runs `handler` on it; returns the
    /// `ws://` URL and the server task.
    async fn serve<F, Fut>(handler: F) -> (Url, tokio::task::JoinHandle<()>)
    where
        F: FnOnce(ServerSocket) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handler(accept_async(stream).await.unwrap()).await;
        });
        (parse_ws_url(&format!("ws://{addr}")).unwrap(), task)
    }

    /// The next text frame sent by the client, or `None` once it closes.
    async fn recv_text(socket: &mut ServerSocket) -> Option<String> {
        loop {
            let message = tokio::time::timeout(TIMEOUT, socket.next())
                .await
                .expect("timed out waiting for a client frame")?
                .ok()?;
            match message {
                Message::Text(text) => return Some(text.as_str().to_owned()),
                Message::Close(_) => return None,
                _ => {}
            }
        }
    }

    async fn next(conn: &mut WsConnection) -> Option<Result<String>> {
        tokio::time::timeout(TIMEOUT, conn.recv())
            .await
            .expect("timed out waiting for a message")
    }

    /// Waits until sending on `sender` fails (the driver has exited) and checks the kind.
    async fn assert_closed(sender: &WsSender) {
        let deadline = tokio::time::Instant::now() + TIMEOUT;
        loop {
            match sender.send_text("late") {
                Err(err) => return assert_eq!(kind(&err), WebSocketErrorKind::Closed),
                Ok(()) => {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "the connection did not terminate"
                    );
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
        }
    }

    fn kind(err: &Error) -> WebSocketErrorKind {
        match err {
            Error::WebSocket(ws) => ws.kind(),
            other => panic!("expected a websocket error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sender_sends_from_other_tasks_and_closes() {
        let (url, server) = serve(|mut socket| async move {
            assert_eq!(recv_text(&mut socket).await.as_deref(), Some("hello"));
            socket.send(Message::text("world")).await.unwrap();
            assert_eq!(recv_text(&mut socket).await.as_deref(), Some("again"));
            // The client closes after `again`.
            assert_eq!(recv_text(&mut socket).await, None);
        })
        .await;
        let mut conn = WsConnection::connect(WsConfig::new(Service::MarketChannel, url))
            .await
            .unwrap();
        let sender = conn.sender();
        assert_eq!(sender.service(), Service::MarketChannel);
        let other = sender.clone();
        tokio::spawn(async move { other.send_text("hello") })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(next(&mut conn).await.unwrap().unwrap(), "world");
        conn.send_text("again").unwrap();

        // Closing through a sender ends the connection for everyone.
        sender.close();
        assert!(next(&mut conn).await.is_none());
        assert_closed(&sender).await;
        let err = conn.send_text("late").unwrap_err();
        assert_eq!(kind(&err), WebSocketErrorKind::Closed);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn dropping_the_connection_closes_it_despite_senders() {
        let (url, server) = serve(|mut socket| async move {
            assert_eq!(recv_text(&mut socket).await, None);
        })
        .await;
        let conn = WsConnection::connect(WsConfig::new(Service::SportsChannel, url))
            .await
            .unwrap();
        let sender = conn.sender();
        drop(conn);
        server.await.unwrap();
        assert_closed(&sender).await;
    }

    #[tokio::test]
    async fn abnormal_close_is_a_closed_error() {
        let (url, server) = serve(|mut socket| async move {
            socket
                .close(Some(CloseFrame {
                    code: CloseCode::from(4008),
                    reason: "policy".into(),
                }))
                .await
                .unwrap();
        })
        .await;
        let mut conn = WsConnection::connect(WsConfig::new(Service::PolyBolt, url))
            .await
            .unwrap();
        let err = next(&mut conn).await.unwrap().unwrap_err();
        let Error::WebSocket(ws) = &err else {
            panic!("expected a websocket error, got {err:?}");
        };
        assert_eq!(ws.kind(), WebSocketErrorKind::Closed);
        assert_eq!(ws.close_code(), Some(4008));
        assert_eq!(ws.close_reason(), Some("policy"));
        assert!(next(&mut conn).await.is_none());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn refused_connection_is_a_connect_error() {
        // Bind then drop a listener to get a port nothing listens on.
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            listener.local_addr().unwrap().port()
        };
        let url = parse_ws_url(&format!("ws://127.0.0.1:{port}")).unwrap();
        let err = WsConnection::connect(WsConfig::new(Service::MarketChannel, url))
            .await
            .unwrap_err();
        assert_eq!(kind(&err), WebSocketErrorKind::Connect);
    }

    #[tokio::test]
    async fn keeps_sending_heartbeats_and_commands_under_back_pressure() {
        const FLOOD: usize = 50;
        let (flooded_tx, flooded_rx) = tokio::sync::oneshot::channel();
        let (url, server) = serve(|mut socket| async move {
            for i in 0..FLOOD {
                socket.send(Message::text(format!("m{i}"))).await.unwrap();
            }
            // The client does not read, so its one-message buffer is full and it has
            // stopped reading the socket; its heartbeats must still arrive.
            for _ in 0..3 {
                assert_eq!(recv_text(&mut socket).await.as_deref(), Some("PING"));
            }
            flooded_tx.send(()).unwrap();
            // ... and so must frames queued now.
            loop {
                match recv_text(&mut socket).await.as_deref() {
                    Some("PING") => {}
                    Some("subscribe") => break,
                    other => panic!("unexpected client frame {other:?}"),
                }
            }
            // Stay silent for longer than the idle timeout: it must not run while the
            // client is not reading.
            tokio::time::sleep(Duration::from_millis(300)).await;
            socket.close(None).await.unwrap();
        })
        .await;
        let config = WsConfig::new(Service::MarketChannel, url)
            .buffer(1)
            .heartbeat(Duration::from_millis(20), "PING")
            .idle_timeout(Duration::from_millis(100));
        let mut conn = WsConnection::connect(config).await.unwrap();
        tokio::time::timeout(TIMEOUT, flooded_rx)
            .await
            .expect("the server did not receive heartbeats under back-pressure")
            .unwrap();
        conn.send_text("subscribe").unwrap();
        tokio::time::timeout(TIMEOUT, server)
            .await
            .expect("the server did not receive the queued frame under back-pressure")
            .unwrap();

        // Nothing was lost while reading was paused, and the normal close ends the stream
        // without an error.
        for i in 0..FLOOD {
            assert_eq!(next(&mut conn).await.unwrap().unwrap(), format!("m{i}"));
        }
        assert!(next(&mut conn).await.is_none());
    }

    #[tokio::test]
    async fn idle_timeout_ends_a_silent_connection() {
        let (url, server) = serve(|mut socket| async move {
            // Ignored heartbeat replies count as activity.
            for _ in 0..6 {
                socket.send(Message::text("PONG")).await.unwrap();
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            socket.send(Message::text("data")).await.unwrap();
            // Then silence, until the client gives up and goes away.
            while let Some(Ok(_)) = socket.next().await {}
        })
        .await;
        let config = WsConfig::new(Service::SportsChannel, url)
            .ignore("PONG")
            .idle_timeout(Duration::from_millis(150));
        let mut conn = WsConnection::connect(config).await.unwrap();
        assert_eq!(next(&mut conn).await.unwrap().unwrap(), "data");
        let err = next(&mut conn).await.unwrap().unwrap_err();
        assert_eq!(kind(&err), WebSocketErrorKind::Timeout, "{err}");
        assert!(err.to_string().contains("idle timeout"), "{err}");
        assert!(next(&mut conn).await.is_none());
        tokio::time::timeout(TIMEOUT, server)
            .await
            .expect("the client did not drop the connection")
            .unwrap();
    }

    #[tokio::test]
    async fn answers_pings_and_completes_the_servers_close_handshake() {
        let (url, server) = serve(|mut socket| async move {
            socket.send(Message::Ping("x".into())).await.unwrap();
            let pong = tokio::time::timeout(TIMEOUT, socket.next()).await.unwrap();
            assert!(
                matches!(&pong, Some(Ok(Message::Pong(payload))) if payload.as_ref() == b"x"),
                "{pong:?}"
            );
            socket
                .close(Some(CloseFrame {
                    code: CloseCode::from(4003),
                    reason: "draining".into(),
                }))
                .await
                .unwrap();
            // The client echoes the close frame before dropping the connection.
            let reply = tokio::time::timeout(TIMEOUT, socket.next()).await.unwrap();
            assert!(matches!(reply, Some(Ok(Message::Close(_)))), "{reply:?}");
        })
        .await;
        let mut conn = WsConnection::connect(WsConfig::new(Service::PolyBolt, url))
            .await
            .unwrap();
        let err = next(&mut conn).await.unwrap().unwrap_err();
        let Error::WebSocket(ws) = &err else {
            panic!("expected a websocket error, got {err:?}");
        };
        assert_eq!(ws.kind(), WebSocketErrorKind::Closed);
        assert_eq!(ws.close_code(), Some(4003));
        assert!(next(&mut conn).await.is_none());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn refused_handshake_exposes_status_and_retry_after() {
        use tokio_tungstenite::{
            accept_hdr_async,
            tungstenite::handshake::server::{ErrorResponse, Request, Response},
        };

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            // The signature is tungstenite's server callback.
            #[allow(clippy::result_large_err)]
            let refuse =
                |_: &Request, _: Response| -> std::result::Result<Response, ErrorResponse> {
                    let mut response = ErrorResponse::new(Some("slow down".to_owned()));
                    *response.status_mut() = http::StatusCode::TOO_MANY_REQUESTS;
                    response
                        .headers_mut()
                        .insert(header::RETRY_AFTER, http::HeaderValue::from_static("7"));
                    Err(response)
                };
            assert!(accept_hdr_async(stream, refuse).await.is_err());
        });
        let url = parse_ws_url(&format!("ws://{addr}")).unwrap();
        let err = WsConnection::connect(WsConfig::new(Service::PolyBolt, url))
            .await
            .unwrap_err();
        let Error::WebSocket(ws) = &err else {
            panic!("expected a websocket error, got {err:?}");
        };
        assert_eq!(ws.kind(), WebSocketErrorKind::Connect);
        assert_eq!(ws.http_status(), Some(http::StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(ws.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(err.status(), Some(http::StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(
            err.to_string(),
            "polybolt websocket error: handshake refused (HTTP 429 Too Many Requests) (retry after 7s)"
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn stalled_writes_time_out() {
        let (url, server) = serve(|socket| async move {
            // Never read: the client's writes eventually block on full TCP buffers.
            tokio::time::sleep(TIMEOUT).await;
            drop(socket);
        })
        .await;
        let config =
            WsConfig::new(Service::MarketChannel, url).write_timeout(Duration::from_millis(300));
        let mut conn = WsConnection::connect(config).await.unwrap();
        // Far larger than the kernel's socket buffers.
        conn.send_text("x".repeat(16 << 20)).unwrap();
        let err = next(&mut conn).await.unwrap().unwrap_err();
        assert_eq!(kind(&err), WebSocketErrorKind::Timeout, "{err}");
        assert!(next(&mut conn).await.is_none());
        server.abort();
    }

    #[tokio::test]
    async fn huge_durations_do_not_overflow() {
        let (url, server) = serve(|mut socket| async move {
            assert_eq!(recv_text(&mut socket).await.as_deref(), Some("hello"));
            socket.send(Message::text("world")).await.unwrap();
            assert_eq!(recv_text(&mut socket).await, None);
        })
        .await;
        let config = WsConfig::new(Service::MarketChannel, url)
            .heartbeat(Duration::MAX, "PING")
            .idle_timeout(Duration::MAX)
            .write_timeout(Duration::MAX)
            .connect_timeout(Duration::MAX);
        let mut conn = WsConnection::connect(config).await.unwrap();
        conn.send_text("hello").unwrap();
        assert_eq!(next(&mut conn).await.unwrap().unwrap(), "world");
        conn.close();
        assert!(next(&mut conn).await.is_none());
        server.await.unwrap();
    }

    #[test]
    fn connecting_outside_a_runtime_is_an_error() {
        use futures_util::FutureExt as _;

        let url = parse_ws_url("ws://127.0.0.1:1").unwrap();
        let err = WsConnection::connect(WsConfig::new(Service::MarketChannel, url))
            .now_or_never()
            .expect("fails without waiting")
            .unwrap_err();
        assert_eq!(kind(&err), WebSocketErrorKind::Connect);
        assert!(err.to_string().contains("Tokio runtime"), "{err}");
    }

    #[test]
    fn sender_is_send_sync_and_clone() {
        fn assert_traits<T: Send + Sync + Clone + std::fmt::Debug>() {}
        assert_traits::<WsSender>();
    }

    #[test]
    fn zero_heartbeat_is_clamped() {
        let url = parse_ws_url("ws://127.0.0.1:1").unwrap();
        let config = WsConfig::new(Service::MarketChannel, url).heartbeat(Duration::ZERO, "PING");
        assert_eq!(
            config.heartbeat.map(|hb| hb.interval),
            Some(MIN_HEARTBEAT_INTERVAL)
        );
    }

    #[test]
    fn ws_url_validation() {
        assert!(parse_ws_url("wss://ws-subscriptions-clob.polymarket.com/ws/market").is_ok());
        assert!(parse_ws_url("https://example.com").is_err());
        assert!(parse_ws_url("nope").is_err());
    }
}
