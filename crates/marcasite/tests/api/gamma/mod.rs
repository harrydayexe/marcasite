//! Gamma API integration tests.
//!
//! The Gamma docs publish no example response bodies, so `fixtures/*.json` are generated
//! from `components/schemas` in `docs/polymarket/specs/gamma-openapi.yaml`: every documented property
//! is present with a placeholder value of its documented type (nested `Market`, `Event` and
//! `Series` objects are reduced to `{"id": "9"}`). The exception is the amounts the spec
//! types as plain `string` (`Market` `liquidity`, `volume`, `fee`, `umaBond`, `umaReward`
//! and `CommentPosition.positionSize`): they hold numeric strings (`"1.5"`), because the
//! crate decodes them as decimals. Whether the server always sends numeric text there is
//! an open question; `""` and `null` decode as `None`. The list-in-a-string fields (`Market`
//! `outcomes`, `outcomePrices`, `clobTokenIds`, `umaResolutionStatuses`) hold JSON-encoded
//! lists, as the live API sends them.
//!
//! `fixtures/live/*.json` are responses captured from the live API; see `live`.

mod comments;
mod events;
mod live;
mod markets;
mod models;
mod profiles;
mod search;
mod series;
mod sports;
mod status;
mod tags;
mod validation;

use serde_json::Value;
use wiremock::{MockServer, ResponseTemplate};

/// A `200 OK` response with a JSON body.
pub fn json(body: impl Into<String>) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.into(), "application/json")
}

/// A `200 OK` response whose body is `value`.
pub fn json_value(value: &Value) -> ResponseTemplate {
    json(value.to_string())
}

/// Loads `fixtures/{name}.json` (named after the spec schema).
pub fn fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/api/gamma/fixtures/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap()
}

/// Loads `fixtures/live/{name}.json`, a response captured from the live API, and returns its
/// `body` (the file also records the `_captured` route and date).
pub fn live_fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/api/gamma/fixtures/live/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut wrapper: Value = serde_json::from_str(&text).unwrap();
    assert!(wrapper.get("_captured").is_some(), "{path}: no `_captured`");
    wrapper["body"].take()
}

/// Every request the server received, in order.
pub async fn requests(server: &MockServer) -> Vec<wiremock::Request> {
    server
        .received_requests()
        .await
        .expect("request recording is enabled")
}

/// The decoded query pairs of the `index`-th request the server received.
pub async fn query_of(server: &MockServer, index: usize) -> Vec<(String, String)> {
    let requests = requests(server).await;
    let request = requests
        .get(index)
        .unwrap_or_else(|| panic!("only {} requests received", requests.len()));
    request
        .url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

/// Owned query pairs, for comparison with [`query_of`].
pub fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// Removes `null` object members recursively (the response types serialize absent fields
/// as `null`).
pub fn strip_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

/// A Gamma client for `server` that never retries, so `5xx` tests do not back off.
pub fn no_retry_gamma(server: &MockServer) -> marcasite::gamma::GammaClient {
    let http = marcasite::HttpClient::builder()
        .retry_policy(marcasite::RetryPolicy::none())
        .build()
        .unwrap();
    marcasite::gamma::GammaClient::builder()
        .base_url(server.uri())
        .http_client(http)
        .build()
        .unwrap()
}

/// A `500` Gamma internal error (`components/schemas/InternalError`, using the spec's
/// example values).
pub fn internal_error() -> ResponseTemplate {
    ResponseTemplate::new(500).set_body_raw(
        r#"{"type":"internal error","error":"cannot get the information"}"#,
        "application/json",
    )
}

/// A `503` Gamma service-unavailable error (`components/schemas/ServiceUnavailableError`,
/// using the spec's example values).
pub fn service_unavailable() -> ResponseTemplate {
    ResponseTemplate::new(503).set_body_raw(
        r#"{"type":"service unavailable","error":"keyset pagination is not configured"}"#,
        "application/json",
    )
}

/// A `422` Gamma validation error (`components/schemas/ValidationError`, using the spec's
/// example values).
pub fn validation_error() -> ResponseTemplate {
    ResponseTemplate::new(422).set_body_raw(
        r#"{"type":"validation error","error":"offset is not allowed on keyset endpoints"}"#,
        "application/json",
    )
}
