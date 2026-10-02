//! The [`RelayerClient`] and its builder.

use marcasite_core::{HttpClient, Result, Service, Transport, Url, transport::parse_base_url};

/// Client for the Relayer API (`https://relayer-v2.polymarket.com`).
///
/// Covers the public endpoints: transaction status, nonces, relay payloads and wallet deployment checks. Cheap to clone: clones share one connection pool.
///
/// ```no_run
/// # async fn run() -> marcasite::Result<()> {
/// use marcasite::relayer::RelayerClient;
///
/// let client = RelayerClient::new()?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RelayerClient {
    pub(crate) transport: Transport,
}

impl RelayerClient {
    /// The production base URL.
    pub const DEFAULT_BASE_URL: &'static str = "https://relayer-v2.polymarket.com";

    /// Creates a client with the default HTTP settings and base URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the HTTP client cannot be built.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns a builder for setting a custom base URL or HTTP client.
    pub fn builder() -> RelayerClientBuilder {
        RelayerClientBuilder::default()
    }

    /// The base URL requests are sent to.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.transport.base_url()
    }
}

/// Builder for [`RelayerClient`].
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct RelayerClientBuilder {
    base_url: Option<String>,
    http: Option<HttpClient>,
}

impl RelayerClientBuilder {
    /// Overrides the base URL (default [`RelayerClient::DEFAULT_BASE_URL`]), e.g. to target a
    /// mock server in tests. A path prefix is preserved.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Uses an existing [`HttpClient`] (and its timeouts, user agent and retry policy).
    pub fn http_client(mut self, http: HttpClient) -> Self {
        self.http = Some(http);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the base URL is invalid or the HTTP
    /// client cannot be built.
    pub fn build(self) -> Result<RelayerClient> {
        let base_url = parse_base_url(
            self.base_url
                .as_deref()
                .unwrap_or(RelayerClient::DEFAULT_BASE_URL),
        )?;
        let http = match self.http {
            Some(http) => http,
            None => HttpClient::new()?,
        };
        Ok(RelayerClient {
            transport: Transport::new(http, Service::Relayer, base_url),
        })
    }
}
