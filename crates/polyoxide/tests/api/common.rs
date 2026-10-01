//! Shared test helpers.

#![allow(
    dead_code,
    reason = "not every helper is used by every feature combination"
)]

use wiremock::MockServer;

/// Starts a mock server.
pub async fn server() -> MockServer {
    MockServer::start().await
}

/// A Polymarket client whose every REST service points at `server`.
pub fn polymarket(server: &MockServer) -> polyoxide::Polymarket {
    let uri = server.uri();
    #[allow(unused_mut, reason = "unused when no REST feature is enabled")]
    let mut builder = polyoxide::Polymarket::builder();
    #[cfg(feature = "gamma")]
    {
        builder = builder.gamma_base_url(&uri);
    }
    #[cfg(feature = "clob")]
    {
        builder = builder.clob_base_url(&uri);
    }
    #[cfg(feature = "data")]
    {
        builder = builder.data_base_url(&uri);
    }
    #[cfg(feature = "relayer")]
    {
        builder = builder.relayer_base_url(&uri);
    }
    #[cfg(feature = "bridge")]
    {
        builder = builder.bridge_base_url(&uri);
    }
    #[cfg(feature = "combos")]
    {
        builder = builder.combos_base_url(&uri);
    }
    let _ = &uri;
    builder.build().expect("client builds")
}
