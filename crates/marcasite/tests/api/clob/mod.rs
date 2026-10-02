//! CLOB API integration tests.
//!
//! Response bodies are taken from the examples in `docs/polymarket/specs/clob-openapi.yaml` (cited per
//! test) or built from the documented schemas where the spec has no example.

mod market_data;
mod markets;
mod prices_history;
mod rebates;
mod rewards;
mod time;
mod trades;

use std::time::Duration;

use marcasite::{HttpClient, RetryPolicy, clob::ClobClient};
use wiremock::{MockServer, ResponseTemplate};

use crate::common;

/// A CLOB client pointed at `server`.
fn clob(server: &MockServer) -> ClobClient {
    common::polymarket(server).clob().clone()
}

/// A CLOB client pointed at `server` that retries once, almost immediately.
fn retrying_clob(server: &MockServer) -> ClobClient {
    let http = HttpClient::builder()
        .retry_policy(RetryPolicy::new(1).with_initial_backoff(Duration::from_millis(1)))
        .build()
        .unwrap();
    ClobClient::builder()
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap()
}

/// A `200 OK` response with a JSON body.
fn json(body: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.to_owned(), "application/json")
}

/// An error response with the documented CLOB error body (`components/schemas/ErrorResponse`).
fn api_error(status: u16, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(serde_json::json!({ "error": message }))
}
