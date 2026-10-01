//! CLOB API integration tests.
//!
//! Response bodies are taken from the examples in `docs/specs/clob-openapi.yaml` (cited per
//! test) or built from the documented schemas where the spec has no example.

mod market_data;
mod markets;
mod prices_history;
mod rebates;
mod rewards;
mod time;
mod trades;

use polyoxide::clob::ClobClient;
use wiremock::{MockServer, ResponseTemplate};

use crate::common;

/// A CLOB client pointed at `server`.
fn clob(server: &MockServer) -> ClobClient {
    common::polymarket(server).clob().clone()
}

/// A `200 OK` response with a JSON body.
fn json(body: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.to_owned(), "application/json")
}

/// An error response with the documented CLOB error body (`components/schemas/ErrorResponse`).
fn api_error(status: u16, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(serde_json::json!({ "error": message }))
}
