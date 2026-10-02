//! The error type shared by every polyoxide client.
//!
//! All fallible operations return [`Result<T>`], whose error is the single [`Error`] enum.
//! Each variant carries a boxed, structured payload with accessor methods, so callers can
//! branch on the failure kind (`match`) and still get the full context (service, URL, HTTP
//! status, parsed API error body, trace id, ...).
//!
//! ```
//! use polyoxide_core::Error;
//!
//! fn describe(err: &Error) -> String {
//!     match err {
//!         Error::RateLimited(api) => format!("slow down, retry after {:?}", api.retry_after()),
//!         Error::Api(api) if api.status().as_u16() == 404 => "not found".to_owned(),
//!         Error::Validation(v) => format!("bad input: {v}"),
//!         other => other.to_string(),
//!     }
//! }
//! ```

use std::{borrow::Cow, fmt, time::Duration};

use http::{Method, StatusCode};

/// Convenience alias for `Result<T, polyoxide_core::Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Maximum number of bytes of a response body retained in errors.
pub(crate) const MAX_ERROR_BODY_BYTES: usize = 4096;

/// Maximum number of bytes of a deserializer's message shown by [`DecodeError`].
const MAX_DECODE_REASON_BYTES: usize = 512;

/// A Polymarket service (REST API or WebSocket channel).
///
/// Used in errors and logs to say which service a failure relates to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Service {
    /// Gamma API (`gamma-api.polymarket.com`).
    Gamma,
    /// CLOB API (`clob.polymarket.com`).
    Clob,
    /// Data API v2 (`data-api.polymarket.com`).
    Data,
    /// Relayer API (`relayer-v2.polymarket.com`).
    Relayer,
    /// Bridge API (`bridge.polymarket.com`).
    Bridge,
    /// Combos / RFQ REST API (`combos-rfq-api.polymarket.com`).
    Combos,
    /// CLOB market WebSocket channel (`ws-subscriptions-clob.polymarket.com/ws/market`).
    MarketChannel,
    /// Sports results WebSocket channel (`sports-api.polymarket.com/ws`).
    SportsChannel,
    /// PolyBolt live data WebSocket (`ws-live-v2.polymarket.com/ws`).
    PolyBolt,
}

impl Service {
    /// A short, stable, human-readable name for the service (e.g. `"gamma"`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Gamma => "gamma",
            Self::Clob => "clob",
            Self::Data => "data",
            Self::Relayer => "relayer",
            Self::Bridge => "bridge",
            Self::Combos => "combos",
            Self::MarketChannel => "market-channel",
            Self::SportsChannel => "sports-channel",
            Self::PolyBolt => "polybolt",
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The error type for every polyoxide operation.
///
/// The enum is `#[non_exhaustive]`: new failure kinds may be added in minor releases, so
/// include a wildcard arm when matching.
///
/// # Display and cause chain
///
/// `Display` describes this failure with its context (service, method, URL without the
/// query string, status, ...) but does not repeat the underlying cause, such as the HTTP
/// client's or the WebSocket library's error. The cause is available through
/// [`std::error::Error::source`], so a reporter that walks the chain shows each message
/// once: for example `anyhow`'s `{:#}`, or a loop over `source()`:
///
/// ```
/// fn report(err: &polyoxide_core::Error) -> String {
///     let mut text = err.to_string();
///     let mut cause = std::error::Error::source(err);
///     while let Some(inner) = cause {
///         text.push_str(&format!(": {inner}"));
///         cause = inner.source();
///     }
///     text
/// }
/// # let _ = report;
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The server answered with a non-success HTTP status (other than `429`).
    ///
    /// The payload holds the status, the parsed error body and the trace id, if any.
    #[error(transparent)]
    Api(Box<ApiError>),

    /// The server answered `429 Too Many Requests`.
    ///
    /// See [`ApiError::retry_after`] for the server's suggested delay, when provided.
    #[error(transparent)]
    RateLimited(Box<ApiError>),

    /// The request did not complete within the configured timeout.
    #[error(transparent)]
    Timeout(Box<TransportError>),

    /// The request could not be sent or the response could not be read (DNS, connection,
    /// TLS, I/O, ...).
    #[error(transparent)]
    Transport(Box<TransportError>),

    /// A successful response body did not match the expected shape.
    #[error(transparent)]
    Decode(Box<DecodeError>),

    /// The request was rejected client-side before being sent, because a parameter
    /// violates a documented constraint (e.g. too many ids in a batch).
    #[error(transparent)]
    Validation(ValidationError),

    /// The client was misconfigured (e.g. an invalid base URL).
    #[error(transparent)]
    Config(ConfigError),

    /// A WebSocket connection failed, closed unexpectedly or received an invalid frame.
    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    #[error(transparent)]
    WebSocket(Box<WebSocketError>),
}

impl Error {
    /// The service the failing request was sent to, when known.
    #[must_use]
    pub fn service(&self) -> Option<Service> {
        match self {
            Self::Api(e) | Self::RateLimited(e) => Some(e.service),
            Self::Timeout(e) | Self::Transport(e) => Some(e.service),
            Self::Decode(e) => Some(e.service),
            Self::Validation(_) | Self::Config(_) => None,
            #[cfg(feature = "ws")]
            Self::WebSocket(e) => Some(e.service),
        }
    }

    /// The HTTP status code, for [`Error::Api`], [`Error::RateLimited`] and
    /// [`Error::Decode`], and for an `Error::WebSocket` whose handshake the server refused
    /// with an HTTP status (see `WebSocketError::http_status`).
    #[must_use]
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Api(e) | Self::RateLimited(e) => Some(e.status),
            Self::Decode(e) => Some(e.status),
            #[cfg(feature = "ws")]
            Self::WebSocket(e) => e.http_status(),
            _ => None,
        }
    }

    /// The parsed API error, for [`Error::Api`] and [`Error::RateLimited`].
    #[must_use]
    pub fn api_error(&self) -> Option<&ApiError> {
        match self {
            Self::Api(e) | Self::RateLimited(e) => Some(e),
            _ => None,
        }
    }

    /// How long the server asked the client to wait before retrying, when provided: the
    /// `Retry-After` header or a documented body field of an API error, or the
    /// `Retry-After` header of a refused WebSocket handshake (see
    /// `WebSocketError::retry_after`).
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Api(e) | Self::RateLimited(e) => e.retry_after(),
            #[cfg(feature = "ws")]
            Self::WebSocket(e) => e.retry_after(),
            _ => None,
        }
    }

    /// The server-side trace id, when the service provides one (e.g. Data API v2's
    /// `x-trace-id` header). Include it when reporting a failure to Polymarket.
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        match self {
            Self::Api(e) | Self::RateLimited(e) => e.trace_id(),
            Self::Decode(e) => e.trace_id(),
            _ => None,
        }
    }

    /// `true` if the request failed with HTTP `404 Not Found`.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(StatusCode::NOT_FOUND)
    }

    /// `true` if the failure is transient: the same request may succeed if sent again
    /// later.
    ///
    /// When the API error body carries an explicit `retryable` flag (as Data API v2 errors
    /// do), the flag decides, whatever the status: `"retryable": false` is never
    /// retryable, not even for a `429` or `503`. Without the flag, this covers rate
    /// limiting (`429`), `502`/`503`/`504` responses, timeouts and connection failures.
    /// The automatic retries of a [`RetryPolicy`](crate::RetryPolicy) use the same rule.
    ///
    /// This describes the *failure*, not the *request*: it does **not** mean that the
    /// request is safe to repeat. A request that creates server-side state (for example
    /// creating Bridge deposit or withdrawal addresses) may have been processed even though
    /// it failed with a timeout or a `5xx`, and repeating it may do so twice. Such requests
    /// are never retried automatically; the caller must decide whether repeating them is
    /// safe.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api(e) | Self::RateLimited(e) => e.is_transient(),
            Self::Timeout(_) => true,
            Self::Transport(e) => e.is_connect(),
            _ => false,
        }
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Self::Validation(err)
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Self::Config(err)
    }
}

/// A non-success HTTP response from a Polymarket API.
///
/// Every service documents a JSON error body with an `error` message; some add more
/// fields (Data API v2: `code`, `retryable`, `trace_id`, `parameter`; Gamma: `type`;
/// CLOB: `code`, `retry_after_seconds`). All documented fields are parsed when present and
/// exposed through the accessors below; the raw body is kept (truncated) for anything else.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub(crate) service: Service,
    pub(crate) method: Method,
    pub(crate) url: String,
    pub(crate) status: StatusCode,
    pub(crate) message: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) error_type: Option<String>,
    pub(crate) retryable: Option<bool>,
    pub(crate) parameter: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) retry_after: Option<Duration>,
    pub(crate) body: String,
}

impl ApiError {
    /// The service that returned the error.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// The HTTP method of the failing request.
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The full URL of the failing request, including the query string.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The HTTP status code.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The human-readable error message (`error` field of the body), when present.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The machine-readable error code (`code` field), when the service provides one.
    ///
    /// For example Data API v2 returns codes such as `invalid_request` or `rate_limited`.
    #[must_use]
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// The error classification (`type` field), when the service provides one (Gamma).
    #[must_use]
    pub fn error_type(&self) -> Option<&str> {
        self.error_type.as_deref()
    }

    /// Whether the server marked the failure as retryable (`retryable` field), when
    /// provided. When present, it overrides the status-based rule of
    /// [`Error::is_retryable`] and of the automatic retries.
    #[must_use]
    pub fn retryable(&self) -> Option<bool> {
        self.retryable
    }

    /// The name of the offending request parameter (`parameter` field), when provided.
    #[must_use]
    pub fn parameter(&self) -> Option<&str> {
        self.parameter.as_deref()
    }

    /// The server-side trace id (`x-trace-id` header or `trace_id` body field).
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// How long to wait before retrying (`Retry-After` header, in seconds or as an HTTP
    /// date, or the CLOB `retry_after_seconds` body field), when provided.
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }

    /// The raw response body, truncated to a few KiB.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Whether the failure is transient: the body's `retryable` flag when present,
    /// otherwise `true` for `429`, `502`, `503` and `504`.
    pub(crate) fn is_transient(&self) -> bool {
        self.retryable.unwrap_or(matches!(
            self.status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        ))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} API returned {} for {} {}",
            self.service,
            self.status,
            self.method,
            redact_query(&self.url)
        )?;
        if let Some(message) = &self.message {
            write!(f, ": {message}")?;
        }
        if let Some(code) = &self.code {
            write!(f, " (code: {code})")?;
        }
        if let Some(parameter) = &self.parameter {
            write!(f, " (parameter: {parameter})")?;
        }
        if let Some(retry_after) = self.retry_after {
            write!(f, " (retry after {}s)", retry_after.as_secs())?;
        }
        if let Some(trace_id) = &self.trace_id {
            write!(f, " (trace id: {trace_id})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

/// The request could not be completed at the transport level.
///
/// The underlying cause (from the HTTP client) is available through
/// [`std::error::Error::source`]; `Display` does not repeat it.
#[derive(Debug)]
pub struct TransportError {
    pub(crate) service: Service,
    pub(crate) method: Method,
    pub(crate) url: String,
    pub(crate) is_connect: bool,
    pub(crate) is_timeout: bool,
    pub(crate) source: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl TransportError {
    /// The service the request was sent to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// The HTTP method of the failing request.
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The full URL of the failing request.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// `true` if the connection could not be established (DNS, refused, TLS handshake).
    #[must_use]
    pub fn is_connect(&self) -> bool {
        self.is_connect
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} request {} {} failed",
            self.service,
            self.method,
            redact_query(&self.url),
        )?;
        if self.is_timeout {
            f.write_str(" (timed out)")?;
        } else if self.is_connect {
            f.write_str(" (could not connect)")?;
        } else if self.source.is::<crate::transport::ResponseTooLarge>() {
            f.write_str(" (response body too large)")?;
        }
        Ok(())
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// A successful response whose body could not be deserialized into the expected type.
///
/// This usually means the API changed shape or returned a value the documentation does not
/// describe. [`DecodeError::path`] points at the offending field and
/// [`DecodeError::body_snippet`] shows the surrounding JSON. `Display` includes the
/// deserializer's message; there is no further [`source`](std::error::Error::source).
#[derive(Debug)]
pub struct DecodeError {
    pub(crate) service: Service,
    pub(crate) method: Method,
    pub(crate) url: String,
    pub(crate) status: StatusCode,
    pub(crate) path: String,
    pub(crate) trace_id: Option<String>,
    pub(crate) snippet: String,
    pub(crate) source: serde_json::Error,
}

impl DecodeError {
    /// The service that returned the response.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// The HTTP method of the request.
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The full URL of the request.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The HTTP status of the response.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The path of the field that failed to decode, e.g. `data[3].outcome_index`
    /// (`.` when the failure is at the root).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The server-side trace id, when provided.
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// A short excerpt of the response body around the failure position.
    #[must_use]
    pub fn body_snippet(&self) -> &str {
        &self.snippet
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The deserializer's message can quote a whole (long) string value.
        write!(
            f,
            "failed to decode {} response from {} {} at `{}`: {}",
            self.service,
            self.method,
            redact_query(&self.url),
            self.path,
            truncate(self.source.to_string(), MAX_DECODE_REASON_BYTES)
        )?;
        if !self.snippet.is_empty() {
            write!(f, " (near: `{}`)", self.snippet)?;
        }
        Ok(())
    }
}

impl std::error::Error for DecodeError {}

/// A request parameter violates a documented constraint; the request was not sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    parameter: Cow<'static, str>,
    message: Cow<'static, str>,
}

impl ValidationError {
    /// Creates a validation error for `parameter` with an explanatory `message`.
    #[must_use]
    pub fn new(
        parameter: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// The name of the offending parameter.
    #[must_use]
    pub fn parameter(&self) -> &str {
        &self.parameter
    }

    /// Why the value was rejected.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid `{}`: {}", self.parameter, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// The client configuration is invalid (e.g. a base URL that cannot be used).
#[derive(Debug)]
pub struct ConfigError {
    message: Cow<'static, str>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ConfigError {
    /// Creates a configuration error with a message.
    #[must_use]
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a configuration error with a message and an underlying cause.
    #[must_use]
    pub fn with_source(
        message: impl Into<Cow<'static, str>>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// The error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid client configuration: {}", self.message)
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// What went wrong on a WebSocket connection; see [`WebSocketError::kind`].
///
/// Every kind except [`Decode`](Self::Decode) means the connection is unusable (or was
/// never established): a stream yields at most one such error, as its last item.
/// [`Decode`](Self::Decode) errors are **not fatal**: the stream continues with the next
/// message.
///
/// The enum is `#[non_exhaustive]`: include a wildcard arm when matching.
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WebSocketErrorKind {
    /// The connection could not be established: the TCP connection, the TLS or WebSocket
    /// handshake failed (including a handshake refused with an HTTP status such as `429`
    /// or `503`, see [`WebSocketError::http_status`] and [`WebSocketError::retry_after`]),
    /// the handshake timed out, the messages to send right after connecting (e.g. a
    /// subscription) could not be sent, or the connection was opened outside a Tokio
    /// runtime.
    Connect,
    /// The connection is closed: the server closed it with a close code other than `1000`
    /// (normal), see [`WebSocketError::close_code`]; it ended without a close frame; or a
    /// frame was queued after the connection had already terminated.
    Closed,
    /// The established connection failed at the socket or protocol level (an I/O or TLS
    /// error, or a WebSocket protocol violation).
    Protocol,
    /// A frame could not be sent: writing it to the socket failed (which ends the
    /// connection), or the request could not be encoded.
    Send,
    /// The connection stopped responding and was abandoned: no frame of any kind arrived
    /// within the configured idle timeout, or writing a frame to the socket did not
    /// complete within the write timeout (e.g. a half-open connection).
    Timeout,
    /// A received message could not be decoded into the expected type. Not fatal: the
    /// connection stays open and the stream continues with the next message.
    Decode,
}

#[cfg(feature = "ws")]
impl WebSocketErrorKind {
    /// A short, stable, human-readable name for the kind (e.g. `"decode"`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Closed => "closed",
            Self::Protocol => "protocol",
            Self::Send => "send",
            Self::Timeout => "timeout",
            Self::Decode => "decode",
        }
    }
}

#[cfg(feature = "ws")]
impl fmt::Display for WebSocketErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A WebSocket connection failed, closed unexpectedly, or received a message that could
/// not be decoded. [`WebSocketError::kind`] tells these apart.
///
/// The underlying cause, if any (e.g. the WebSocket library's error), is available through
/// [`std::error::Error::source`]; `Display` does not repeat it.
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
#[derive(Debug)]
pub struct WebSocketError {
    pub(crate) service: Service,
    pub(crate) kind: WebSocketErrorKind,
    pub(crate) message: Cow<'static, str>,
    pub(crate) close_code: Option<u16>,
    pub(crate) close_reason: Option<String>,
    pub(crate) http_status: Option<StatusCode>,
    pub(crate) retry_after: Option<Duration>,
    pub(crate) source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

#[cfg(feature = "ws")]
impl WebSocketError {
    /// Creates a WebSocket error of the given `kind` for `service` with a message.
    #[must_use]
    pub fn new(
        service: Service,
        kind: WebSocketErrorKind,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            service,
            kind,
            message: message.into(),
            close_code: None,
            close_reason: None,
            http_status: None,
            retry_after: None,
            source: None,
        }
    }

    /// Attaches the underlying cause.
    #[must_use]
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Attaches the close code and reason sent by the server.
    #[must_use]
    pub fn with_close(mut self, code: u16, reason: impl Into<String>) -> Self {
        self.close_code = Some(code);
        self.close_reason = Some(reason.into());
        self
    }

    /// The channel the error relates to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// What kind of failure this is. In particular, [`WebSocketErrorKind::Decode`] marks a
    /// non-fatal decode error, after which the stream continues; every other kind is
    /// terminal.
    #[must_use]
    pub fn kind(&self) -> WebSocketErrorKind {
        self.kind
    }

    /// A description of what went wrong.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The close code sent by the server, if the connection was closed by the server.
    #[must_use]
    pub fn close_code(&self) -> Option<u16> {
        self.close_code
    }

    /// The close reason sent by the server, if any.
    #[must_use]
    pub fn close_reason(&self) -> Option<&str> {
        self.close_reason.as_deref()
    }

    /// The HTTP status the server answered the WebSocket handshake with, when it refused
    /// the upgrade (e.g. `429 Too Many Requests` or `503 Service Unavailable`). Only set on
    /// errors of kind [`WebSocketErrorKind::Connect`].
    #[must_use]
    pub fn http_status(&self) -> Option<StatusCode> {
        self.http_status
    }

    /// How long the server asked the client to wait before connecting again: the
    /// `Retry-After` header (in seconds or as an HTTP date) of a refused handshake, when
    /// present. Only set on
    /// errors of kind [`WebSocketErrorKind::Connect`]. Also available as
    /// [`Error::retry_after`].
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }
}

#[cfg(feature = "ws")]
impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} websocket error: {}", self.service, self.message)?;
        if let Some(status) = self.http_status {
            write!(f, " (HTTP {status})")?;
        }
        if let Some(retry_after) = self.retry_after {
            write!(f, " (retry after {}s)", retry_after.as_secs())?;
        }
        if let Some(code) = self.close_code {
            write!(f, " (close code {code}")?;
            match self.close_reason.as_deref() {
                Some(reason) if !reason.is_empty() => write!(f, ": {reason})")?,
                _ => f.write_str(")")?,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "ws")]
impl std::error::Error for WebSocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// Strips the query string from a URL for display purposes, keeping messages short.
/// The full URL stays available through the error accessors.
fn redact_query(url: &str) -> &str {
    url.split_once('?').map_or(url, |(base, _)| base)
}

/// Truncates `s` to at most `max` bytes on a char boundary, appending `…` if truncated.
pub(crate) fn truncate(mut s: String, max: usize) -> String {
    if s.len() > max {
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
        s.push('…');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_respects_char_boundaries() {
        assert_eq!(truncate("héllo".to_owned(), 2), "h…");
        assert_eq!(truncate("hello".to_owned(), 10), "hello");
    }

    #[test]
    fn api_error_display_includes_context() {
        let err = ApiError {
            service: Service::Data,
            method: Method::GET,
            url: "https://data-api.polymarket.com/v2/positions?user=0x1".to_owned(),
            status: StatusCode::BAD_REQUEST,
            message: Some("offset is not supported".to_owned()),
            code: Some("invalid_request".to_owned()),
            error_type: None,
            retryable: Some(false),
            parameter: Some("offset".to_owned()),
            trace_id: Some("abc".to_owned()),
            retry_after: None,
            body: String::new(),
        };
        assert_eq!(
            err.to_string(),
            "data API returned 400 Bad Request for GET https://data-api.polymarket.com/v2/positions: \
             offset is not supported (code: invalid_request) (parameter: offset) (trace id: abc)"
        );
    }

    #[cfg(feature = "ws")]
    #[test]
    fn websocket_error_exposes_kind_and_close() {
        let err = WebSocketError::new(
            Service::PolyBolt,
            WebSocketErrorKind::Closed,
            "server closed the connection",
        )
        .with_close(4008, "policy violation");
        assert_eq!(err.kind(), WebSocketErrorKind::Closed);
        assert_eq!(err.service(), Service::PolyBolt);
        assert_eq!(err.close_code(), Some(4008));
        assert_eq!(
            err.to_string(),
            "polybolt websocket error: server closed the connection (close code 4008: policy violation)"
        );
        assert_eq!(WebSocketErrorKind::Decode.to_string(), "decode");
        let err = Error::WebSocket(Box::new(WebSocketError::new(
            Service::MarketChannel,
            WebSocketErrorKind::Decode,
            "failed to decode message",
        )));
        assert_eq!(err.service(), Some(Service::MarketChannel));
        assert_eq!(err.status(), None);
        assert_eq!(err.retry_after(), None);
    }

    #[cfg(feature = "ws")]
    #[test]
    fn websocket_error_exposes_a_refused_handshake() {
        let mut ws = WebSocketError::new(
            Service::PolyBolt,
            WebSocketErrorKind::Connect,
            "handshake refused",
        );
        ws.http_status = Some(StatusCode::TOO_MANY_REQUESTS);
        ws.retry_after = Some(Duration::from_secs(7));
        assert_eq!(ws.http_status(), Some(StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(ws.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(
            ws.to_string(),
            "polybolt websocket error: handshake refused (HTTP 429 Too Many Requests) (retry after 7s)"
        );
        let err = Error::WebSocket(Box::new(ws));
        assert_eq!(err.status(), Some(StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
        assert!(!err.is_not_found());
        assert_eq!(WebSocketErrorKind::Timeout.to_string(), "timeout");
    }

    fn api_error(status: StatusCode, retryable: Option<bool>) -> ApiError {
        ApiError {
            service: Service::Data,
            method: Method::GET,
            url: "https://data-api.polymarket.com/v2/status".to_owned(),
            status,
            message: None,
            code: None,
            error_type: None,
            retryable,
            parameter: None,
            trace_id: None,
            retry_after: None,
            body: String::new(),
        }
    }

    #[test]
    fn explicit_retryable_flag_overrides_the_status() {
        let cases = [
            (StatusCode::SERVICE_UNAVAILABLE, None, true),
            (StatusCode::BAD_GATEWAY, None, true),
            (StatusCode::GATEWAY_TIMEOUT, None, true),
            (StatusCode::INTERNAL_SERVER_ERROR, None, false),
            (StatusCode::BAD_REQUEST, None, false),
            (StatusCode::SERVICE_UNAVAILABLE, Some(false), false),
            (StatusCode::GATEWAY_TIMEOUT, Some(false), false),
            (StatusCode::INTERNAL_SERVER_ERROR, Some(true), true),
        ];
        for (status, flag, expected) in cases {
            let err = Error::Api(Box::new(api_error(status, flag)));
            assert_eq!(err.is_retryable(), expected, "{status} {flag:?}");
        }
        let limited =
            |flag| Error::RateLimited(Box::new(api_error(StatusCode::TOO_MANY_REQUESTS, flag)));
        assert!(limited(None).is_retryable());
        assert!(limited(Some(true)).is_retryable());
        assert!(!limited(Some(false)).is_retryable());
        assert!(!Error::from(ValidationError::new("x", "y")).is_retryable());
    }

    /// Each message in the cause chain, as a chain-walking reporter prints them.
    fn chain(err: &(dyn std::error::Error + 'static)) -> Vec<String> {
        let mut out = vec![err.to_string()];
        let mut cause = err.source();
        while let Some(inner) = cause {
            out.push(inner.to_string());
            cause = inner.source();
        }
        out
    }

    #[derive(Debug, thiserror::Error)]
    #[error("connection refused by peer")]
    struct Cause;

    #[test]
    fn causes_are_not_repeated_in_display() {
        let transport = |is_connect, is_timeout| {
            Error::Transport(Box::new(TransportError {
                service: Service::Gamma,
                method: Method::GET,
                url: "https://gamma-api.polymarket.com/tags?limit=1".to_owned(),
                is_connect,
                is_timeout,
                source: Box::new(Cause),
            }))
        };
        assert_eq!(
            chain(&transport(true, false)),
            [
                "gamma request GET https://gamma-api.polymarket.com/tags failed (could not connect)",
                "connection refused by peer",
            ]
        );
        assert_eq!(
            transport(false, true).to_string(),
            "gamma request GET https://gamma-api.polymarket.com/tags failed (timed out)"
        );
        assert_eq!(
            transport(false, false).to_string(),
            "gamma request GET https://gamma-api.polymarket.com/tags failed"
        );

        #[cfg(feature = "ws")]
        {
            let ws = Error::WebSocket(Box::new(
                WebSocketError::new(
                    Service::PolyBolt,
                    WebSocketErrorKind::Protocol,
                    "connection error",
                )
                .with_source(Cause),
            ));
            assert_eq!(
                chain(&ws),
                [
                    "polybolt websocket error: connection error",
                    "connection refused by peer"
                ]
            );
        }

        let source = serde_json::from_str::<u8>("300").unwrap_err();
        let decode = Error::Decode(Box::new(DecodeError {
            service: Service::Data,
            method: Method::GET,
            url: "https://data-api.polymarket.com/v2/trades".to_owned(),
            status: StatusCode::OK,
            path: "data[0].size".to_owned(),
            trace_id: None,
            snippet: String::new(),
            source,
        }));
        let messages = chain(&decode);
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(
            messages[0].contains("at `data[0].size`: invalid value: integer `300`"),
            "{messages:?}"
        );
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Error>();
    }
}
