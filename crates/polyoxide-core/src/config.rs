//! HTTP client configuration shared by every service.

use std::time::Duration;

use crate::error::{ConfigError, Result};

/// The default `User-Agent` header sent with every request.
pub const DEFAULT_USER_AGENT: &str = concat!("polyoxide/", env!("CARGO_PKG_VERSION"));

/// The default overall request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The default connection timeout.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// When and how to retry failed requests automatically.
///
/// Only requests that are safe to repeat are retried: `GET` requests, and read-only `POST`
/// requests that the service clients mark as idempotent (e.g. batch price lookups).
/// Requests that create server-side state are never retried. A request is retried when
/// [`Error::is_retryable`](crate::Error::is_retryable) is `true` for its failure: when the
/// error body carries an explicit `retryable` flag (Data API v2), that flag decides
/// (`"retryable": false` is never retried); otherwise `429 Too Many Requests`, `502`,
/// `503`, `504`, a timeout, or a connection error.
///
/// The delay before retry `n` (starting at 0) is the server's `Retry-After` value when
/// present, otherwise `initial_backoff * 2^n`, capped at `max_backoff`. If the server asks
/// for a longer delay than `max_backoff`, the error is returned instead of waiting.
///
/// Retries are **disabled by default** ([`RetryPolicy::none`]).
///
/// ```
/// use std::time::Duration;
/// use polyoxide_core::RetryPolicy;
///
/// let policy = RetryPolicy::new(3)
///     .with_initial_backoff(Duration::from_millis(250))
///     .with_max_backoff(Duration::from_secs(5));
/// assert_eq!(policy.max_retries(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl RetryPolicy {
    /// A policy that never retries.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            max_retries: 0,
            initial_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(10),
        }
    }

    /// A policy that retries up to `max_retries` times with the default backoff
    /// (200 ms initial, 10 s maximum).
    #[must_use]
    pub const fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Self::none()
        }
    }

    /// Sets the delay before the first retry when the server gives no `Retry-After`.
    #[must_use]
    pub const fn with_initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    /// Sets the longest delay the client will wait before a retry.
    #[must_use]
    pub const fn with_max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    /// The maximum number of retries after the first attempt.
    #[must_use]
    pub const fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// The delay before the first retry when the server gives no `Retry-After`.
    #[must_use]
    pub const fn initial_backoff(&self) -> Duration {
        self.initial_backoff
    }

    /// The longest delay the client will wait before a retry.
    #[must_use]
    pub const fn max_backoff(&self) -> Duration {
        self.max_backoff
    }

    /// The delay before retry number `attempt` (0-based), or `None` if no retry should
    /// happen.
    pub(crate) fn delay_for(
        &self,
        attempt: u32,
        retry_after: Option<Duration>,
    ) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }
        match retry_after {
            Some(delay) if delay > self.max_backoff => None,
            Some(delay) => Some(delay),
            None => {
                let factor = 2u32.saturating_pow(attempt);
                Some(
                    self.initial_backoff
                        .saturating_mul(factor)
                        .min(self.max_backoff),
                )
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::none()
    }
}

/// A connection-pooled HTTP client shared by all service clients.
///
/// Cloning is cheap (the connection pool is reference counted), so one `HttpClient` can
/// back every service client in an application.
#[derive(Debug, Clone)]
pub struct HttpClient {
    pub(crate) inner: reqwest::Client,
    pub(crate) retry: RetryPolicy,
}

impl HttpClient {
    /// Creates a client with the default settings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the TLS backend cannot be
    /// initialised.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns a builder for customising timeouts, the user agent and retries.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }

    /// The retry policy in effect.
    #[must_use]
    pub fn retry_policy(&self) -> RetryPolicy {
        self.retry
    }
}

/// Builder for [`HttpClient`].
///
/// ```
/// use std::time::Duration;
/// use polyoxide_core::{HttpClient, RetryPolicy};
///
/// # fn main() -> polyoxide_core::Result<()> {
/// let http = HttpClient::builder()
///     .timeout(Duration::from_secs(10))
///     .user_agent("my-app/1.0")
///     .retry_policy(RetryPolicy::new(2))
///     .build()?;
/// # let _ = http;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use]
pub struct HttpClientBuilder {
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    user_agent: String,
    retry: RetryPolicy,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Some(DEFAULT_TIMEOUT),
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: RetryPolicy::none(),
        }
    }
}

impl HttpClientBuilder {
    /// Sets the overall per-request timeout (default 30 s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Disables the overall per-request timeout.
    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Sets the connection timeout (default 10 s).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Sets the `User-Agent` header (default `polyoxide/<version>`).
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets the automatic retry policy (default: no retries).
    pub fn retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the user agent is not a valid
    /// header value or the TLS backend cannot be initialised.
    pub fn build(self) -> Result<HttpClient> {
        let mut builder = reqwest::Client::builder()
            .user_agent(self.user_agent)
            .gzip(true);
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(timeout) = self.connect_timeout {
            builder = builder.connect_timeout(timeout);
        }
        let inner = builder
            .build()
            .map_err(|e| ConfigError::with_source("failed to build the HTTP client", e))?;
        Ok(HttpClient {
            inner,
            retry: self.retry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_backoff() {
        let policy = RetryPolicy::new(3)
            .with_initial_backoff(Duration::from_millis(100))
            .with_max_backoff(Duration::from_millis(300));
        assert_eq!(policy.delay_for(0, None), Some(Duration::from_millis(100)));
        assert_eq!(policy.delay_for(1, None), Some(Duration::from_millis(200)));
        assert_eq!(policy.delay_for(2, None), Some(Duration::from_millis(300)));
        assert_eq!(policy.delay_for(3, None), None);
        assert_eq!(
            policy.delay_for(0, Some(Duration::from_millis(50))),
            Some(Duration::from_millis(50))
        );
        assert_eq!(policy.delay_for(0, Some(Duration::from_secs(1))), None);
    }

    #[test]
    fn no_retries_by_default() {
        assert_eq!(RetryPolicy::default().delay_for(0, None), None);
    }
}
