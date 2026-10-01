//! Data API v2 integration tests.

mod boards;
mod errors;
mod feeds;
mod fixtures;
mod markets;
mod service;
mod wallet;

use wiremock::MockServer;

/// The query string of every request the mock server received, in order.
pub async fn received_queries(server: &MockServer) -> Vec<String> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| r.url.query().unwrap_or_default().to_owned())
        .collect()
}
