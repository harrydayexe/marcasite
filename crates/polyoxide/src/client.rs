//! The [`Polymarket`] umbrella client.

use std::time::Duration;

use polyoxide_core::{HttpClient, HttpClientBuilder, Result, RetryPolicy};

/// One handle to every enabled Polymarket service, sharing a single HTTP connection pool.
///
/// Cheap to clone. Use [`Polymarket::new`] for the defaults or [`Polymarket::builder`] to
/// configure timeouts, retries, the user agent or per-service base URLs.
///
/// ```no_run
/// # async fn run() -> polyoxide::Result<()> {
/// use std::time::Duration;
/// use polyoxide::{Polymarket, RetryPolicy};
///
/// let pm = Polymarket::builder()
///     .timeout(Duration::from_secs(10))
///     .retry_policy(RetryPolicy::new(3))
///     .build()?;
/// let tag = pm.gamma().list_tags().limit(1).send().await?;
/// # let _ = tag;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Polymarket {
    #[cfg(feature = "gamma")]
    gamma: crate::gamma::GammaClient,
    #[cfg(feature = "clob")]
    clob: crate::clob::ClobClient,
    #[cfg(feature = "data")]
    data: crate::data::DataClient,
    #[cfg(feature = "relayer")]
    relayer: crate::relayer::RelayerClient,
    #[cfg(feature = "bridge")]
    bridge: crate::bridge::BridgeClient,
    #[cfg(feature = "combos")]
    combos: crate::combos::CombosClient,
    http: HttpClient,
}

impl Polymarket {
    /// Creates a client for every enabled service with default settings and base URLs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if the HTTP client cannot be built.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns a builder for customising the shared HTTP client and base URLs.
    pub fn builder() -> PolymarketBuilder {
        PolymarketBuilder::default()
    }

    /// The shared HTTP client.
    #[must_use]
    pub fn http_client(&self) -> &HttpClient {
        &self.http
    }

    /// The Gamma API client (events, markets, tags, series, comments, sports, search,
    /// profiles).
    #[cfg(feature = "gamma")]
    #[cfg_attr(docsrs, doc(cfg(feature = "gamma")))]
    #[must_use]
    pub fn gamma(&self) -> &crate::gamma::GammaClient {
        &self.gamma
    }

    /// The CLOB API client (public market data).
    #[cfg(feature = "clob")]
    #[cfg_attr(docsrs, doc(cfg(feature = "clob")))]
    #[must_use]
    pub fn clob(&self) -> &crate::clob::ClobClient {
        &self.clob
    }

    /// The Data API v2 client (wallets, feeds, boards, market state).
    #[cfg(feature = "data")]
    #[cfg_attr(docsrs, doc(cfg(feature = "data")))]
    #[must_use]
    pub fn data(&self) -> &crate::data::DataClient {
        &self.data
    }

    /// The Relayer API client (public endpoints).
    #[cfg(feature = "relayer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "relayer")))]
    #[must_use]
    pub fn relayer(&self) -> &crate::relayer::RelayerClient {
        &self.relayer
    }

    /// The Bridge API client.
    #[cfg(feature = "bridge")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bridge")))]
    #[must_use]
    pub fn bridge(&self) -> &crate::bridge::BridgeClient {
        &self.bridge
    }

    /// The Combos / RFQ REST client (public endpoints).
    #[cfg(feature = "combos")]
    #[cfg_attr(docsrs, doc(cfg(feature = "combos")))]
    #[must_use]
    pub fn combos(&self) -> &crate::combos::CombosClient {
        &self.combos
    }
}

/// Builder for [`Polymarket`].
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct PolymarketBuilder {
    http: Option<HttpClient>,
    http_builder: HttpClientBuilder,
    #[cfg(feature = "gamma")]
    gamma_base_url: Option<String>,
    #[cfg(feature = "clob")]
    clob_base_url: Option<String>,
    #[cfg(feature = "data")]
    data_base_url: Option<String>,
    #[cfg(feature = "relayer")]
    relayer_base_url: Option<String>,
    #[cfg(feature = "bridge")]
    bridge_base_url: Option<String>,
    #[cfg(feature = "combos")]
    combos_base_url: Option<String>,
}

impl PolymarketBuilder {
    /// Uses an existing [`HttpClient`]; the timeout, user agent and retry settings of this
    /// builder are then ignored.
    pub fn http_client(mut self, http: HttpClient) -> Self {
        self.http = Some(http);
        self
    }

    /// Sets the overall per-request timeout (default 30 s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http_builder = self.http_builder.timeout(timeout);
        self
    }

    /// Sets the connection timeout (default 10 s).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.http_builder = self.http_builder.connect_timeout(timeout);
        self
    }

    /// Sets the `User-Agent` header (default `polyoxide/<version>`).
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.http_builder = self.http_builder.user_agent(user_agent);
        self
    }

    /// Sets the automatic retry policy (default: no retries).
    pub fn retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.http_builder = self.http_builder.retry_policy(retry);
        self
    }

    /// Overrides the Gamma API base URL.
    #[cfg(feature = "gamma")]
    #[cfg_attr(docsrs, doc(cfg(feature = "gamma")))]
    pub fn gamma_base_url(mut self, url: impl Into<String>) -> Self {
        self.gamma_base_url = Some(url.into());
        self
    }

    /// Overrides the CLOB API base URL.
    #[cfg(feature = "clob")]
    #[cfg_attr(docsrs, doc(cfg(feature = "clob")))]
    pub fn clob_base_url(mut self, url: impl Into<String>) -> Self {
        self.clob_base_url = Some(url.into());
        self
    }

    /// Overrides the Data API base URL.
    #[cfg(feature = "data")]
    #[cfg_attr(docsrs, doc(cfg(feature = "data")))]
    pub fn data_base_url(mut self, url: impl Into<String>) -> Self {
        self.data_base_url = Some(url.into());
        self
    }

    /// Overrides the Relayer API base URL.
    #[cfg(feature = "relayer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "relayer")))]
    pub fn relayer_base_url(mut self, url: impl Into<String>) -> Self {
        self.relayer_base_url = Some(url.into());
        self
    }

    /// Overrides the Bridge API base URL.
    #[cfg(feature = "bridge")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bridge")))]
    pub fn bridge_base_url(mut self, url: impl Into<String>) -> Self {
        self.bridge_base_url = Some(url.into());
        self
    }

    /// Overrides the Combos / RFQ REST base URL.
    #[cfg(feature = "combos")]
    #[cfg_attr(docsrs, doc(cfg(feature = "combos")))]
    pub fn combos_base_url(mut self, url: impl Into<String>) -> Self {
        self.combos_base_url = Some(url.into());
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) if a base URL is invalid or the HTTP
    /// client cannot be built.
    pub fn build(self) -> Result<Polymarket> {
        let http = match self.http {
            Some(http) => http,
            None => self.http_builder.build()?,
        };
        Ok(Polymarket {
            #[cfg(feature = "gamma")]
            gamma: with_base_url(
                crate::gamma::GammaClient::builder().http_client(http.clone()),
                self.gamma_base_url,
                crate::gamma::GammaClientBuilder::base_url,
            )
            .build()?,
            #[cfg(feature = "clob")]
            clob: with_base_url(
                crate::clob::ClobClient::builder().http_client(http.clone()),
                self.clob_base_url,
                crate::clob::ClobClientBuilder::base_url,
            )
            .build()?,
            #[cfg(feature = "data")]
            data: with_base_url(
                crate::data::DataClient::builder().http_client(http.clone()),
                self.data_base_url,
                crate::data::DataClientBuilder::base_url,
            )
            .build()?,
            #[cfg(feature = "relayer")]
            relayer: with_base_url(
                crate::relayer::RelayerClient::builder().http_client(http.clone()),
                self.relayer_base_url,
                crate::relayer::RelayerClientBuilder::base_url,
            )
            .build()?,
            #[cfg(feature = "bridge")]
            bridge: with_base_url(
                crate::bridge::BridgeClient::builder().http_client(http.clone()),
                self.bridge_base_url,
                crate::bridge::BridgeClientBuilder::base_url,
            )
            .build()?,
            #[cfg(feature = "combos")]
            combos: with_base_url(
                crate::combos::CombosClient::builder().http_client(http.clone()),
                self.combos_base_url,
                crate::combos::CombosClientBuilder::base_url,
            )
            .build()?,
            http,
        })
    }
}

#[allow(
    dead_code,
    reason = "unused when every REST service feature is disabled"
)]
fn with_base_url<B>(builder: B, url: Option<String>, set: fn(B, String) -> B) -> B {
    match url {
        Some(url) => set(builder, url),
        None => builder,
    }
}
