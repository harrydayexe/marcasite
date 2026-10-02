//! The [`ClobClient`] and its builder.

use polyoxide_core::{HttpClient, Result, Service, Transport, Url, transport::parse_base_url};

/// Client for the CLOB API (`https://clob.polymarket.com`).
///
/// Covers the public endpoints: order books, prices, spreads, markets, price history,
/// rewards, rebates, builder trades and the server time (see the [module docs](crate::clob)
/// for the full list). Cheap to clone: clones share one connection pool.
///
/// ```no_run
/// # async fn run() -> polyoxide::Result<()> {
/// use polyoxide::clob::ClobClient;
///
/// let client = ClobClient::new()?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ClobClient {
    pub(crate) transport: Transport,
}

impl ClobClient {
    /// The production base URL.
    pub const DEFAULT_BASE_URL: &'static str = "https://clob.polymarket.com";

    /// Creates a client with the default HTTP settings and base URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the HTTP client cannot be built.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns a builder for setting a custom base URL or HTTP client.
    pub fn builder() -> ClobClientBuilder {
        ClobClientBuilder::default()
    }

    /// The base URL requests are sent to.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.transport.base_url()
    }
}

/// Builder for [`ClobClient`].
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct ClobClientBuilder {
    base_url: Option<String>,
    http: Option<HttpClient>,
}

impl ClobClientBuilder {
    /// Overrides the base URL (default [`ClobClient::DEFAULT_BASE_URL`]), e.g. to target a
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
    pub fn build(self) -> Result<ClobClient> {
        let base_url = parse_base_url(
            self.base_url
                .as_deref()
                .unwrap_or(ClobClient::DEFAULT_BASE_URL),
        )?;
        let http = match self.http {
            Some(http) => http,
            None => HttpClient::new()?,
        };
        Ok(ClobClient {
            transport: Transport::new(http, Service::Clob, base_url),
        })
    }
}
