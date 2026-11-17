//! The [`GeoblockClient`] and its builder.

use marcasite_core::{HttpClient, Result, Service, Transport, Url, transport::parse_base_url};

/// Client for the Geoblock endpoint (`https://polymarket.com`).
///
/// Reports whether the requesting IP address is geoblocked (`GET /api/geoblock`). The endpoint
/// lives on `polymarket.com`, not on the API servers. Cheap to clone: clones share one
/// connection pool.
///
/// This is a check only: the SDK never offers a way to route around a geoblock.
///
/// ```no_run
/// # async fn run() -> marcasite::Result<()> {
/// use marcasite::geoblock::GeoblockClient;
///
/// let client = GeoblockClient::new()?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GeoblockClient {
    pub(crate) transport: Transport,
}

impl GeoblockClient {
    /// The production base URL.
    pub const DEFAULT_BASE_URL: &'static str = "https://polymarket.com";

    /// Creates a client with the default HTTP settings and base URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the HTTP client cannot be built.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns a builder for setting a custom base URL or HTTP client.
    pub fn builder() -> GeoblockClientBuilder {
        GeoblockClientBuilder::default()
    }

    /// The base URL requests are sent to.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        self.transport.base_url()
    }
}

/// Builder for [`GeoblockClient`].
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct GeoblockClientBuilder {
    base_url: Option<String>,
    http: Option<HttpClient>,
}

impl GeoblockClientBuilder {
    /// Overrides the base URL (default [`GeoblockClient::DEFAULT_BASE_URL`]), e.g. to target a
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
    pub fn build(self) -> Result<GeoblockClient> {
        let base_url = parse_base_url(
            self.base_url
                .as_deref()
                .unwrap_or(GeoblockClient::DEFAULT_BASE_URL),
        )?;
        let http = match self.http {
            Some(http) => http,
            None => HttpClient::new()?,
        };
        Ok(GeoblockClient {
            transport: Transport::new(http, Service::Geoblock, base_url),
        })
    }
}
