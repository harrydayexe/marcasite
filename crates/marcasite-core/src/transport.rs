//! Request execution for a single service: URL building, retries, logging, error-body
//! parsing and response decoding.

use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use http::{HeaderMap, Method, StatusCode, header};
use serde::{Serialize, de::DeserializeOwned};
use tracing::Instrument as _;
use url::Url;

use crate::{
    HttpClient, Query,
    error::{
        ApiError, ConfigError, DecodeError, Error, MAX_ERROR_BODY_BYTES, Result, Service,
        TransportError, ValidationError, truncate,
    },
};

/// The response header carrying a server-side trace id (documented for Data API v2).
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// Bytes of context shown on each side of a decode failure.
const SNIPPET_RADIUS: usize = 120;

/// Bytes of a response body logged at `TRACE` level.
const MAX_LOGGED_BODY_BYTES: usize = 2048;

/// Parses and validates a base URL for a service.
///
/// The URL must be absolute, use `http` or `https`, and be able to carry a path.
///
/// # Errors
///
/// Returns [`Error::Config`] if the URL is invalid.
pub fn parse_base_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)
        .map_err(|e| ConfigError::with_source(format!("invalid base URL `{url}`"), e))?;
    validate_base_url(parsed)
}

/// Validates an already-parsed base URL (see [`parse_base_url`]).
///
/// # Errors
///
/// Returns [`Error::Config`] if the URL cannot be used as a base URL.
pub fn validate_base_url(url: Url) -> Result<Url> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ConfigError::new(format!(
            "base URL `{url}` must use http or https, not `{}`",
            url.scheme()
        ))
        .into());
    }
    if url.cannot_be_a_base() {
        return Err(ConfigError::new(format!("`{url}` cannot be used as a base URL")).into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(ConfigError::new(format!(
            "base URL `{url}` must not contain a query string or fragment"
        ))
        .into());
    }
    Ok(url)
}

/// Executes requests against one service (one base URL).
///
/// Cheap to clone: it shares the underlying [`HttpClient`] connection pool.
#[derive(Debug, Clone)]
pub struct Transport {
    http: HttpClient,
    service: Service,
    base_url: Url,
}

impl Transport {
    /// Creates a transport for `service` rooted at `base_url`.
    ///
    /// Use [`parse_base_url`] / [`validate_base_url`] to validate user input first.
    #[must_use]
    pub fn new(http: HttpClient, service: Service, base_url: Url) -> Self {
        Self {
            http,
            service,
            base_url,
        }
    }

    /// The service this transport talks to.
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// The base URL requests are resolved against.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// The shared HTTP client.
    #[must_use]
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Starts a `GET` request to the path made of `segments`.
    ///
    /// Each segment is percent-encoded, so user-supplied ids and slugs are safe to pass
    /// as-is: `transport.get(&["events", "slug", slug])`. A segment that would change the
    /// request path instead of being sent verbatim (an empty segment, `.` or `..`) makes
    /// the request fail with [`Error::Validation`] (parameter `"path"`) when it is sent,
    /// without anything being sent.
    pub fn get(&self, segments: &[&str]) -> Request<'_> {
        self.request(Method::GET, segments)
    }

    /// Starts a `POST` request to the path made of `segments` (see [`Transport::get`]).
    ///
    /// `POST` requests are not retried unless marked with [`Request::idempotent`].
    pub fn post(&self, segments: &[&str]) -> Request<'_> {
        self.request(Method::POST, segments)
    }

    /// Starts a request with an arbitrary method (see [`Transport::get`]).
    pub fn request(&self, method: Method, segments: &[&str]) -> Request<'_> {
        let idempotent = matches!(method, Method::GET | Method::HEAD);
        Request {
            transport: self,
            url: self.url_for(segments),
            method,
            query: Query::new(),
            headers: Vec::new(),
            body: None,
            idempotent,
        }
    }

    fn url_for(&self, segments: &[&str]) -> Result<Url> {
        check_segments(segments)?;
        let mut url = self.base_url.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| {
                ConfigError::new(format!("`{}` cannot be used as a base URL", self.base_url))
            })?;
            path.pop_if_empty();
            path.extend(segments);
        }
        Ok(url)
    }
}

/// Rejects path segments that the URL would not carry verbatim: an empty segment (usually
/// an empty id or slug) produces a different path (`/markets/` instead of `/markets/{id}`),
/// and the `url` crate resolves `.` and `..` away (`/events/..` becomes `/`), so the request
/// would reach another endpoint.
fn check_segments(segments: &[&str]) -> Result<()> {
    for (index, segment) in segments.iter().enumerate() {
        let message = match *segment {
            "" => "is empty",
            "." | ".." => "is a dot segment, which would change the request path",
            _ => continue,
        };
        let path = segments.get(..=index).unwrap_or(segments).join("/");
        return Err(ValidationError::new(
            "path",
            format!("path segment `{segment}` in `/{path}` {message}"),
        )
        .into());
    }
    Ok(())
}

/// A request being built. Created by [`Transport::get`] / [`Transport::post`].
#[derive(Debug)]
#[must_use = "a request does nothing until it is sent"]
pub struct Request<'a> {
    transport: &'a Transport,
    url: Result<Url>,
    method: Method,
    query: Query,
    headers: Vec<(&'static str, String)>,
    body: Option<std::result::Result<Vec<u8>, serde_json::Error>>,
    idempotent: bool,
}

impl Request<'_> {
    /// Adds a request header (e.g. an optional documented header such as
    /// `X-Builder-Code`). Invalid header values are reported as [`Error::Validation`] when
    /// the request is sent.
    pub fn header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    /// Sets the query string, replacing any previous one.
    pub fn query(mut self, query: Query) -> Self {
        self.query = query;
        self
    }

    /// Sets a JSON request body.
    pub fn json<B: Serialize + ?Sized>(mut self, body: &B) -> Self {
        self.body = Some(serde_json::to_vec(body));
        self
    }

    /// Marks the request as safe to retry (for read-only `POST` endpoints).
    pub fn idempotent(mut self, idempotent: bool) -> Self {
        self.idempotent = idempotent;
        self
    }

    /// Sends the request and deserializes a JSON response body into `T`.
    ///
    /// # Errors
    ///
    /// - [`Error::Api`] / [`Error::RateLimited`] for non-success statuses,
    /// - [`Error::Timeout`] / [`Error::Transport`] for network failures,
    /// - [`Error::Decode`] if the body does not match `T`,
    /// - [`Error::Validation`] if the request body could not be serialized,
    /// - [`Error::Config`] if the base URL cannot carry a path.
    pub async fn send<T: DeserializeOwned>(self) -> Result<T> {
        let response = self.send_raw().await?;
        response.json()
    }

    /// Sends the request and returns the raw successful response.
    ///
    /// # Errors
    ///
    /// As [`Request::send`], except that no decoding is attempted.
    pub async fn send_raw(self) -> Result<RawResponse> {
        let transport = self.transport;
        let service = transport.service;
        let mut url = self.url?;
        if !self.query.is_empty() {
            url.query_pairs_mut().extend_pairs(self.query.iter());
        }
        let body = match self.body {
            Some(Ok(body)) => Some(body),
            Some(Err(e)) => {
                return Err(ValidationError::new(
                    "body",
                    format!("failed to serialize the request body: {e}"),
                )
                .into());
            }
            None => None,
        };
        let mut headers = HeaderMap::new();
        for (name, value) in self.headers {
            let name = header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| ValidationError::new(name, "invalid header name"))?;
            let value = header::HeaderValue::from_str(&value).map_err(|_| {
                ValidationError::new(name.as_str().to_owned(), "invalid header value")
            })?;
            headers.append(name, value);
        }
        let method = self.method;
        let span = tracing::debug_span!(
            "marcasite.request",
            service = %service,
            method = %method,
            path = %url.path(),
            status = tracing::field::Empty,
            attempts = tracing::field::Empty,
            elapsed_ms = tracing::field::Empty,
            trace_id = tracing::field::Empty,
        );
        execute(transport, method, url, headers, body, self.idempotent)
            .instrument(span)
            .await
    }
}

/// A successful (2xx) HTTP response whose body has been read.
#[derive(Debug, Clone)]
pub struct RawResponse {
    service: Service,
    method: Method,
    url: Url,
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl RawResponse {
    /// The HTTP status.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The response headers.
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// The final request URL.
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// The raw body bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// The `x-trace-id` header, if present.
    #[must_use]
    pub fn trace_id(&self) -> Option<&str> {
        header_str(&self.headers, TRACE_ID_HEADER)
    }

    /// The body as UTF-8 text (lossy).
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// Deserializes the JSON body into `T`, reporting the failing field path on error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Decode`] if the body is not valid JSON for `T`.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        let mut deserializer = serde_json::Deserializer::from_slice(&self.body);
        let value = serde_path_to_error::deserialize::<_, T>(&mut deserializer).map_err(|e| {
            let path = e.path().to_string();
            self.decode_error(path, e.into_inner())
        })?;
        deserializer
            .end()
            .map_err(|e| self.decode_error(".".to_owned(), e))?;
        Ok(value)
    }

    fn decode_error(&self, path: String, source: serde_json::Error) -> Error {
        let snippet = snippet_around(&self.body, &source);
        // Returned to the caller, so not logged above DEBUG.
        tracing::debug!(
            service = %self.service,
            path = %path,
            error = %source,
            "failed to decode response body"
        );
        Error::Decode(Box::new(DecodeError {
            service: self.service,
            method: self.method.clone(),
            url: self.url.to_string(),
            status: self.status,
            path,
            trace_id: self.trace_id().map(str::to_owned),
            snippet,
            source,
        }))
    }
}

async fn execute(
    transport: &Transport,
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    idempotent: bool,
) -> Result<RawResponse> {
    let service = transport.service;
    let retry = transport.http.retry;
    let mut attempt: u32 = 0;
    let first_started = Instant::now();
    loop {
        let started = Instant::now();
        let mut builder = transport
            .http
            .inner
            .request(method.clone(), url.clone())
            .headers(headers.clone());
        if let Some(body) = &body {
            builder = builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(body.clone());
        }
        tracing::debug!(attempt, url = %url, "sending request");

        let outcome = match builder.send().await {
            Ok(response) => {
                read_response(service, &method, response, transport.http.max_response_size).await
            }
            Err(e) => Err(transport_error(service, &method, &url, e)),
        };
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

        let err = match outcome {
            Ok(response) => {
                tracing::debug!(
                    status = response.status.as_u16(),
                    elapsed_ms,
                    bytes = response.body.len(),
                    trace_id = response.trace_id(),
                    "received response"
                );
                tracing::trace!(body = %log_body(&response.body), "response body");
                record_outcome(
                    first_started,
                    attempt,
                    Some(response.status),
                    response.trace_id(),
                );
                return Ok(response);
            }
            Err(err) => err,
        };

        // Only idempotent requests are retried: a failed non-idempotent request may still
        // have been processed. An explicit `"retryable": false` in the body is honoured.
        let retryable = idempotent && err.is_retryable();
        if let Some(delay) = retryable
            .then(|| retry.delay_for(attempt, err.retry_after()))
            .flatten()
        {
            tracing::debug!(
                attempt,
                elapsed_ms,
                delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                error = %err,
                "request failed, retrying"
            );
            tokio::time::sleep(delay).await;
            attempt = attempt.saturating_add(1);
            continue;
        }

        // Returned to the caller, so not logged above DEBUG.
        tracing::debug!(elapsed_ms, error = %err, "request failed");
        record_outcome(first_started, attempt, err.status(), err.trace_id());
        return Err(err);
    }
}

/// Records the final outcome of a request on the current `marcasite.request` span, so span
/// based subscribers (e.g. OpenTelemetry exporters) see it as span attributes.
fn record_outcome(
    started: Instant,
    retries: u32,
    status: Option<StatusCode>,
    trace_id: Option<&str>,
) {
    let span = tracing::Span::current();
    span.record(
        "elapsed_ms",
        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    );
    span.record("attempts", u64::from(retries).saturating_add(1));
    if let Some(status) = status {
        span.record("status", status.as_u16());
    }
    if let Some(trace_id) = trace_id {
        span.record("trace_id", trace_id);
    }
}

async fn read_response(
    service: Service,
    method: &Method,
    response: reqwest::Response,
    max_size: usize,
) -> Result<RawResponse> {
    let status = response.status();
    let headers = response.headers().clone();
    let url = response.url().clone();
    let body = read_body(
        service,
        method,
        &url,
        response,
        max_size,
        status.is_success(),
    )
    .await?;
    if status.is_success() {
        return Ok(RawResponse {
            service,
            method: method.clone(),
            url,
            status,
            headers,
            body,
        });
    }
    let api = parse_api_error(service, method.clone(), &url, status, &headers, &body);
    if status == StatusCode::TOO_MANY_REQUESTS {
        Err(Error::RateLimited(Box::new(api)))
    } else {
        Err(Error::Api(Box::new(api)))
    }
}

/// Reads a response body chunk by chunk, never holding more than `max_size` bytes.
///
/// A successful response (`strict`) whose body exceeds `max_size` fails with
/// [`Error::Transport`] (source [`ResponseTooLarge`]), rejected up front when its
/// `Content-Length` says so. An error response's body is only needed for the error message,
/// so it is cut at `min(max_size, MAX_ERROR_BODY_READ_BYTES)` instead, keeping the status.
async fn read_body(
    service: Service,
    method: &Method,
    url: &Url,
    mut response: reqwest::Response,
    max_size: usize,
    strict: bool,
) -> Result<Vec<u8>> {
    let too_large = || {
        Error::Transport(Box::new(TransportError {
            service,
            method: method.clone(),
            url: url.to_string(),
            is_connect: false,
            is_timeout: false,
            source: Box::new(ResponseTooLarge { limit: max_size }),
        }))
    };
    let limit = if strict {
        max_size
    } else {
        max_size.min(MAX_ERROR_BODY_READ_BYTES)
    };
    if strict
        && response
            .content_length()
            .is_some_and(|length| length > u64::try_from(limit).unwrap_or(u64::MAX))
    {
        return Err(too_large());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| transport_error(service, method, url, e))?
    {
        let room = limit.saturating_sub(body.len());
        if chunk.len() > room {
            if strict {
                return Err(too_large());
            }
            body.extend_from_slice(chunk.get(..room).unwrap_or_default());
            break;
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Bytes of an error response's body read at most (they are only used for the message).
const MAX_ERROR_BODY_READ_BYTES: usize = 64 * 1024;

/// A successful response body was larger than [`HttpClientBuilder::max_response_size`]
/// allows.
///
/// [`HttpClientBuilder::max_response_size`]: crate::HttpClientBuilder::max_response_size
#[derive(Debug)]
pub(crate) struct ResponseTooLarge {
    limit: usize,
}

impl std::fmt::Display for ResponseTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the response body exceeds the maximum size of {} bytes \
             (see `HttpClientBuilder::max_response_size`)",
            self.limit
        )
    }
}

impl std::error::Error for ResponseTooLarge {}

fn transport_error(service: Service, method: &Method, url: &Url, err: reqwest::Error) -> Error {
    let is_timeout = err.is_timeout();
    let inner = Box::new(TransportError {
        service,
        method: method.clone(),
        url: url.to_string(),
        is_connect: err.is_connect(),
        is_timeout,
        source: Box::new(err.without_url()),
    });
    if is_timeout {
        Error::Timeout(inner)
    } else {
        Error::Transport(inner)
    }
}

/// The union of every documented error-body field across the services.
#[derive(Debug, Default, serde::Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: Option<serde_json::Value>,
    #[serde(default)]
    code: Option<serde_json::Value>,
    #[serde(default, rename = "type")]
    error_type: Option<String>,
    #[serde(default)]
    retryable: Option<bool>,
    #[serde(default)]
    parameter: Option<String>,
    #[serde(default)]
    trace_id: Option<String>,
    #[serde(default)]
    retry_after_seconds: Option<u64>,
}

fn value_to_string(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(s) => Some(s),
        other => Some(other.to_string()),
    }
}

/// Builds an [`ApiError`] from a non-success response.
pub(crate) fn parse_api_error(
    service: Service,
    method: Method,
    url: &Url,
    status: StatusCode,
    headers: &HeaderMap,
    body: &[u8],
) -> ApiError {
    let text = String::from_utf8_lossy(body).into_owned();
    let parsed = serde_json::from_slice::<ErrorBody>(body).ok();
    let (message, code, error_type, retryable, parameter, body_trace_id, body_retry_after) =
        match parsed {
            Some(b) => (
                b.error.and_then(value_to_string),
                b.code.and_then(value_to_string),
                b.error_type,
                b.retryable,
                b.parameter,
                b.trace_id,
                b.retry_after_seconds,
            ),
            None => {
                // Not a JSON object: surface short plain-text bodies (not HTML pages).
                let trimmed = text.trim();
                let message =
                    (!trimmed.is_empty() && trimmed.len() <= 512 && !trimmed.starts_with('<'))
                        .then(|| trimmed.to_owned());
                (message, None, None, None, None, None, None)
            }
        };
    let retry_after = header_str(headers, header::RETRY_AFTER.as_str())
        .and_then(|value| parse_retry_after(value, SystemTime::now().into()))
        .or(body_retry_after.map(Duration::from_secs));
    let trace_id = header_str(headers, TRACE_ID_HEADER)
        .map(str::to_owned)
        .or(body_trace_id);
    ApiError {
        service,
        method,
        url: url.to_string(),
        status,
        message,
        code,
        error_type,
        retryable,
        parameter,
        trace_id,
        retry_after,
        body: truncate(text, MAX_ERROR_BODY_BYTES),
    }
}

/// Parses a `Retry-After` header value (RFC 9110): either a number of seconds (`120`) or an
/// HTTP date (`Wed, 21 Oct 2015 07:28:00 GMT`), which is turned into the delay from `now`
/// (zero if the date has passed). Returns `None` for anything else.
pub(crate) fn parse_retry_after(value: &str, now: DateTime<Utc>) -> Option<Duration> {
    let value = value.trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = DateTime::parse_from_rfc2822(value).ok()?;
    Some(
        at.with_timezone(&Utc)
            .signed_duration_since(now)
            .to_std()
            .unwrap_or(Duration::ZERO),
    )
}

fn header_str<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn log_body(body: &[u8]) -> String {
    truncate(
        String::from_utf8_lossy(body).into_owned(),
        MAX_LOGGED_BODY_BYTES,
    )
}

/// Extracts a short excerpt of `body` around the position reported by `err`.
fn snippet_around(body: &[u8], err: &serde_json::Error) -> String {
    let text = String::from_utf8_lossy(body);
    if text.is_empty() {
        return String::new();
    }
    // serde_json reports 1-based lines and columns (column counted in bytes).
    let mut offset = 0usize;
    for (index, line) in text.split('\n').enumerate() {
        if index + 1 == err.line() {
            offset += err.column().min(line.len());
            break;
        }
        offset += line.len() + 1;
    }
    let mut start = offset.saturating_sub(SNIPPET_RADIUS);
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = offset.saturating_add(SNIPPET_RADIUS).min(text.len());
    while end < text.len() && !text.is_char_boundary(end) {
        end += 1;
    }
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
    }
    snippet.push_str(text.get(start..end).unwrap_or_default());
    if end < text.len() {
        snippet.push('…');
    }
    snippet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url() -> Url {
        Url::parse("https://example.com/x?y=1").unwrap()
    }

    #[test]
    fn parses_data_api_error_body() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", http::HeaderValue::from_static("7"));
        headers.insert(TRACE_ID_HEADER, http::HeaderValue::from_static("hdr-trace"));
        let body = br#"{"error":"slow down","code":"rate_limited","retryable":true,"trace_id":"body-trace"}"#;
        let err = parse_api_error(
            Service::Data,
            Method::GET,
            &url(),
            StatusCode::TOO_MANY_REQUESTS,
            &headers,
            body,
        );
        assert_eq!(err.message(), Some("slow down"));
        assert_eq!(err.code(), Some("rate_limited"));
        assert_eq!(err.retryable(), Some(true));
        assert_eq!(err.trace_id(), Some("hdr-trace"));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(7)));
    }

    #[test]
    fn retry_after_accepts_seconds_and_http_dates() {
        let now = DateTime::parse_from_rfc3339("2015-10-21T07:27:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            parse_retry_after("120", now),
            Some(Duration::from_secs(120))
        );
        assert_eq!(parse_retry_after(" 7 ", now), Some(Duration::from_secs(7)));
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT", now),
            Some(Duration::from_secs(60))
        );
        // A date in the past means "now".
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:00:00 GMT", now),
            Some(Duration::ZERO)
        );
        for invalid in ["", "-5", "1.5", "soon", "2015-10-21T07:28:00Z", "€"] {
            assert_eq!(parse_retry_after(invalid, now), None, "{invalid:?}");
        }

        // A header date in the future is honoured by API errors.
        let mut headers = HeaderMap::new();
        let later = (DateTime::<Utc>::from(SystemTime::now()) + chrono::TimeDelta::seconds(3600))
            .to_rfc2822();
        headers.insert("retry-after", http::HeaderValue::from_str(&later).unwrap());
        let err = parse_api_error(
            Service::Clob,
            Method::GET,
            &url(),
            StatusCode::TOO_MANY_REQUESTS,
            &headers,
            b"{}",
        );
        let delay = err.retry_after().unwrap();
        assert!(
            delay > Duration::from_secs(3500) && delay <= Duration::from_secs(3600),
            "{delay:?}"
        );
    }

    #[test]
    fn parses_clob_and_gamma_error_bodies() {
        let headers = HeaderMap::new();
        let clob = parse_api_error(
            Service::Clob,
            Method::GET,
            &url(),
            StatusCode::SERVICE_UNAVAILABLE,
            &headers,
            br#"{"error":"busy","retry_after_seconds":3}"#,
        );
        assert_eq!(clob.retry_after(), Some(Duration::from_secs(3)));
        let gamma = parse_api_error(
            Service::Gamma,
            Method::GET,
            &url(),
            StatusCode::UNPROCESSABLE_ENTITY,
            &headers,
            br#"{"type":"validation error","error":"offset is not allowed on keyset endpoints"}"#,
        );
        assert_eq!(gamma.error_type(), Some("validation error"));
        assert_eq!(
            gamma.message(),
            Some("offset is not allowed on keyset endpoints")
        );
    }

    #[test]
    fn plain_text_and_html_bodies() {
        let headers = HeaderMap::new();
        let text = parse_api_error(
            Service::Gamma,
            Method::GET,
            &url(),
            StatusCode::NOT_FOUND,
            &headers,
            b"not found",
        );
        assert_eq!(text.message(), Some("not found"));
        let html = parse_api_error(
            Service::Gamma,
            Method::GET,
            &url(),
            StatusCode::BAD_GATEWAY,
            &headers,
            b"<html>bad gateway</html>",
        );
        assert_eq!(html.message(), None);
        assert_eq!(html.body(), "<html>bad gateway</html>");
    }

    #[test]
    fn base_url_validation() {
        assert!(parse_base_url("https://gamma-api.polymarket.com").is_ok());
        assert!(parse_base_url("ftp://example.com").is_err());
        assert!(parse_base_url("mailto:me@example.com").is_err());
        assert!(parse_base_url("https://example.com/?q=1").is_err());
        assert!(parse_base_url("not a url").is_err());
    }

    #[test]
    fn url_segments_are_encoded_and_prefix_kept() -> Result<()> {
        let http = HttpClient::new()?;
        let transport = Transport::new(
            http,
            Service::Gamma,
            parse_base_url("http://localhost:1234/prefix/")?,
        );
        let url = transport.url_for(&["events", "slug", "a b/c?d"])?;
        assert_eq!(
            url.as_str(),
            "http://localhost:1234/prefix/events/slug/a%20b%2Fc%3Fd"
        );
        Ok(())
    }

    #[tokio::test]
    async fn unsafe_path_segments_are_rejected_before_sending() -> Result<()> {
        // Nothing listens on port 9: a request that was sent would be a transport error.
        let transport = Transport::new(
            HttpClient::new()?,
            Service::Gamma,
            parse_base_url("http://127.0.0.1:9/prefix")?,
        );
        let cases: [(&[&str], &str); 5] = [
            (&["markets", ""], "path segment `` in `/markets/` is empty"),
            (&["", "markets"], "path segment `` in `/` is empty"),
            (
                &["events", ".."],
                "path segment `..` in `/events/..` is a dot segment, which would change the request path",
            ),
            (
                &["events", "slug", "."],
                "path segment `.` in `/events/slug/.` is a dot segment, which would change the request path",
            ),
            (
                &["events", "..", "1"],
                "path segment `..` in `/events/..` is a dot segment, which would change the request path",
            ),
        ];
        for (segments, message) in cases {
            for request in [transport.get(segments), transport.post(segments)] {
                let err = request.send_raw().await.unwrap_err();
                let Error::Validation(v) = &err else {
                    panic!("expected a validation error for {segments:?}, got {err:?}");
                };
                assert_eq!(v.parameter(), "path");
                assert_eq!(v.message(), message);
            }
        }
        // Segments that merely contain dots are fine and sent verbatim.
        let url = transport.url_for(&["events", "slug", "...", ".a", "a.b"])?;
        assert_eq!(
            url.as_str(),
            "http://127.0.0.1:9/prefix/events/slug/.../.a/a.b"
        );
        Ok(())
    }

    #[tokio::test]
    async fn invalid_header_is_a_validation_error() -> Result<()> {
        let transport = Transport::new(
            HttpClient::new()?,
            Service::Bridge,
            parse_base_url("http://127.0.0.1:9")?,
        );
        let err = transport
            .post(&["deposit"])
            .header("X-Builder-Code", "bad\nvalue")
            .send_raw()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "{err:?}");
        Ok(())
    }

    /// A transport against `server` that retries up to `retries` times without waiting.
    #[tokio::test]
    async fn request_span_records_outcome() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::path("/flaky"))
            .respond_with(ResponseTemplate::new(503).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(matchers::path("/flaky"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header(TRACE_ID_HEADER, "t-1")
                    .set_body_raw("{}", "application/json"),
            )
            .mount(&server)
            .await;

        let (captured, _guard) = crate::test_tracing::capture();
        let transport = retrying_transport(&server, 2).unwrap();
        transport.get(&["flaky"]).send_raw().await.unwrap();

        let captured = captured.lock().unwrap();
        let span = captured.span("marcasite.request").expect("request span");
        assert_eq!(span.get("service").map(String::as_str), Some("data"));
        assert_eq!(span.get("method").map(String::as_str), Some("GET"));
        assert_eq!(span.get("path").map(String::as_str), Some("/flaky"));
        assert_eq!(span.get("status").map(String::as_str), Some("200"));
        assert_eq!(span.get("attempts").map(String::as_str), Some("2"));
        assert_eq!(span.get("trace_id").map(String::as_str), Some("t-1"));
        assert!(span.contains_key("elapsed_ms"));
        assert_eq!(
            captured.event_scope("received response"),
            Some(&["marcasite.request".to_owned()][..])
        );
    }

    #[tokio::test]
    async fn request_span_records_failure_status() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let (captured, _guard) = crate::test_tracing::capture();
        let transport = retrying_transport(&server, 2).unwrap();
        transport.get(&["missing"]).send_raw().await.unwrap_err();

        let captured = captured.lock().unwrap();
        let span = captured.span("marcasite.request").expect("request span");
        assert_eq!(span.get("status").map(String::as_str), Some("404"));
        assert_eq!(span.get("attempts").map(String::as_str), Some("1"));
    }

    fn retrying_transport(server: &wiremock::MockServer, retries: u32) -> Result<Transport> {
        let http = HttpClient::builder()
            .retry_policy(
                crate::RetryPolicy::new(retries).with_initial_backoff(Duration::from_millis(1)),
            )
            .build()?;
        Ok(Transport::new(
            http,
            Service::Data,
            parse_base_url(&server.uri())?,
        ))
    }

    #[tokio::test]
    async fn explicit_retryable_false_is_not_retried() -> Result<()> {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::path("/v2/status"))
            .respond_with(ResponseTemplate::new(503).set_body_raw(
                r#"{"error":"down","code":"dependency_unavailable","retryable":false,"trace_id":"t"}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(matchers::path("/v2/trades"))
            .respond_with(ResponseTemplate::new(429).set_body_raw(
                r#"{"error":"no","code":"rate_limited","retryable":false}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&server)
            .await;

        let transport = retrying_transport(&server, 3)?;
        let err = transport
            .get(&["v2", "status"])
            .send_raw()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Api(_)), "{err:?}");
        assert!(!err.is_retryable());
        let err = transport
            .get(&["v2", "trades"])
            .send_raw()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
        assert!(!err.is_retryable());
        Ok(())
    }

    #[tokio::test]
    async fn transient_failures_are_retried() -> Result<()> {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        // No flag: the status decides.
        Mock::given(matchers::path("/a"))
            .respond_with(ResponseTemplate::new(503))
            .expect(3)
            .mount(&server)
            .await;
        // An explicit `retryable: true` is retried whatever the status.
        Mock::given(matchers::path("/b"))
            .respond_with(ResponseTemplate::new(500).set_body_raw(
                r#"{"error":"try again","retryable":true}"#,
                "application/json",
            ))
            .expect(3)
            .mount(&server)
            .await;
        // Non-idempotent requests are never retried.
        Mock::given(matchers::path("/c"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;

        let transport = retrying_transport(&server, 2)?;
        let err = transport.get(&["a"]).send_raw().await.unwrap_err();
        assert!(err.is_retryable());
        let err = transport.get(&["b"]).send_raw().await.unwrap_err();
        assert!(err.is_retryable());
        let err = transport.post(&["c"]).send_raw().await.unwrap_err();
        assert!(err.is_retryable());
        Ok(())
    }

    #[tokio::test]
    async fn response_size_is_capped() -> Result<()> {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        let body = format!("\"{}\"", "x".repeat(2046));
        // Not retried.
        Mock::given(matchers::path("/big"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body.clone(), "application/json"))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(matchers::path("/fits"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(body[..1024].to_owned(), "text/plain"),
            )
            .mount(&server)
            .await;
        Mock::given(matchers::path("/error"))
            .respond_with(
                ResponseTemplate::new(503).set_body_raw("e".repeat(100_000), "text/plain"),
            )
            // The first attempt and two retries.
            .expect(3)
            .mount(&server)
            .await;
        let http = HttpClient::builder()
            .max_response_size(1024)
            .retry_policy(crate::RetryPolicy::new(2).with_initial_backoff(Duration::from_millis(1)))
            .build()?;
        assert_eq!(http.max_response_size(), 1024);
        let transport = Transport::new(http, Service::Data, parse_base_url(&server.uri())?);

        let err = transport.get(&["big"]).send_raw().await.unwrap_err();
        let Error::Transport(inner) = &err else {
            panic!("expected a transport error, got {err:?}");
        };
        assert!(!inner.is_connect());
        assert!(!err.is_retryable());
        assert!(
            err.to_string()
                .ends_with("failed (response body too large)"),
            "{err}"
        );
        let cause = std::error::Error::source(&err).unwrap().to_string();
        assert!(
            cause.contains("exceeds the maximum size of 1024 bytes"),
            "{cause}"
        );

        assert_eq!(
            transport.get(&["fits"]).send_raw().await?.body().len(),
            1024
        );

        // An oversized error body is cut, and the status is kept.
        let err = transport.get(&["error"]).send_raw().await.unwrap_err();
        let api = err.api_error().unwrap();
        assert_eq!(api.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            api.body().len() <= MAX_ERROR_BODY_BYTES + 4,
            "{}",
            api.body().len()
        );

        assert_eq!(
            HttpClient::new()?.max_response_size(),
            crate::config::DEFAULT_MAX_RESPONSE_SIZE
        );
        Ok(())
    }

    #[tokio::test]
    async fn chunked_bodies_without_a_length_are_capped_too() -> Result<()> {
        use std::io::{Read as _, Write as _};

        // A server that streams 10 000 bytes with chunked encoding (no `Content-Length`).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut request = [0u8; 4096];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n",
            );
            for _ in 0..10 {
                let _ = stream.write_all(format!("3e8\r\n{}\r\n", "1".repeat(1000)).as_bytes());
            }
            let _ = stream.write_all(b"0\r\n\r\n");
        });

        let http = HttpClient::builder().max_response_size(4096).build()?;
        let transport = Transport::new(
            http,
            Service::Gamma,
            parse_base_url(&format!("http://{addr}"))?,
        );
        let err = transport.get(&["stream"]).send_raw().await.unwrap_err();
        assert!(matches!(err, Error::Transport(_)), "{err:?}");
        assert!(
            err.to_string().ends_with("(response body too large)"),
            "{err}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn slow_responses_are_timeout_errors() -> Result<()> {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::path("/slow"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
            .mount(&server)
            .await;
        let http = HttpClient::builder()
            .timeout(Duration::from_millis(100))
            .build()?;
        let transport = Transport::new(http, Service::Gamma, parse_base_url(&server.uri())?);
        let err = transport.get(&["slow"]).send_raw().await.unwrap_err();
        let Error::Timeout(inner) = &err else {
            panic!("expected a timeout, got {err:?}");
        };
        assert_eq!(inner.service(), Service::Gamma);
        assert_eq!(inner.method(), &Method::GET);
        assert!(err.is_retryable());
        assert!(err.to_string().ends_with("failed (timed out)"), "{err}");
        assert!(std::error::Error::source(&err).is_some());
        Ok(())
    }

    #[tokio::test]
    async fn refused_connections_are_connect_errors() -> Result<()> {
        // Bind then drop a listener to get a port nothing listens on.
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let transport = Transport::new(
            HttpClient::new()?,
            Service::Clob,
            parse_base_url(&format!("http://127.0.0.1:{port}"))?,
        );
        let err = transport.get(&["ok"]).send_raw().await.unwrap_err();
        let Error::Transport(inner) = &err else {
            panic!("expected a transport error, got {err:?}");
        };
        assert!(inner.is_connect());
        assert_eq!(inner.url(), format!("http://127.0.0.1:{port}/ok"));
        assert!(err.is_retryable());
        assert!(
            err.to_string().ends_with("failed (could not connect)"),
            "{err}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn sends_the_configured_user_agent() -> Result<()> {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::path("/default"))
            .and(matchers::header(
                "user-agent",
                crate::config::DEFAULT_USER_AGENT,
            ))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(matchers::path("/custom"))
            .and(matchers::header("user-agent", "my-app/1.0"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let base = parse_base_url(&server.uri())?;
        let default = Transport::new(HttpClient::new()?, Service::Data, base.clone());
        default.get(&["default"]).send_raw().await?;
        let http = HttpClient::builder().user_agent("my-app/1.0").build()?;
        let custom = Transport::new(http, Service::Data, base);
        custom.get(&["custom"]).send_raw().await?;
        assert!(crate::config::DEFAULT_USER_AGENT.starts_with("marcasite/"));
        Ok(())
    }

    #[test]
    fn snippet_points_at_error() {
        let body = br#"{"a":1,"b":"oops"}"#;
        let err =
            serde_json::from_slice::<std::collections::HashMap<String, u8>>(body).unwrap_err();
        let snippet = snippet_around(body, &err);
        assert!(snippet.contains("oops"), "{snippet}");
    }
}
