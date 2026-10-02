//! Shared helpers for the live tests.
//!
//! Besides calling the SDK, each test can fetch the same route as raw JSON and pass it to
//! [`check`], which decodes it with the SDK's model type and reports *drift*: keys the live
//! server sends that the model drops, and enum values that fell into an `Unknown(..)`
//! catch-all. Drift is printed (run with `--nocapture`) and appended to the file named by
//! `POLYOXIDE_LIVE_REPORT`, if set. It fails the test only when `POLYOXIDE_LIVE_STRICT=1`,
//! because the live API may add fields at any time.

#![allow(
    dead_code,
    reason = "not every helper is used by every feature combination"
)]

use std::{collections::BTreeSet, fmt::Debug, io::Write as _};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::sync::OnceCell;

/// Gamma API base URL.
pub const GAMMA: &str = "https://gamma-api.polymarket.com";
/// CLOB API base URL.
pub const CLOB: &str = "https://clob.polymarket.com";
/// Data API base URL (routes are under `/v2`).
pub const DATA: &str = "https://data-api.polymarket.com";
/// Relayer API base URL.
pub const RELAYER: &str = "https://relayer-v2.polymarket.com";
/// Bridge API base URL.
pub const BRIDGE: &str = "https://bridge.polymarket.com";
/// Combos / RFQ REST base URL.
pub const COMBOS: &str = "https://combos-rfq-api.polymarket.com";

/// A client for the production APIs.
pub fn pm() -> polyoxide::Polymarket {
    polyoxide::Polymarket::new().expect("client builds")
}

/// A raw live response: the exact body text and its parsed JSON.
#[derive(Debug, Clone)]
pub struct Raw {
    /// The body as sent by the server.
    pub text: String,
    /// The parsed body.
    pub json: Value,
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("polyoxide-live-tests")
        .build()
        .expect("reqwest client builds")
}

async fn finish(label: String, response: reqwest::Response) -> Raw {
    let status = response.status();
    let text = response.text().await.expect("body reads");
    assert!(status.is_success(), "{label}: HTTP {status}: {text}");
    let json = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{label}: not JSON: {e}"));
    Raw { text, json }
}

/// `GET {base}{path}?{query}` as raw JSON. Panics on a non-2xx status.
pub async fn get(base: &str, path: &str, query: &[(&str, &str)]) -> Raw {
    let response = http()
        .get(format!("{base}{path}"))
        .query(query)
        .send()
        .await
        .unwrap_or_else(|e| panic!("GET {path}: {e}"));
    finish(format!("GET {path}"), response).await
}

/// `POST {base}{path}` with a JSON body, as raw JSON. Panics on a non-2xx status.
pub async fn post(base: &str, path: &str, body: &Value) -> Raw {
    let response = http()
        .post(format!("{base}{path}"))
        .json(body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {path}: {e}"));
    finish(format!("POST {path}"), response).await
}

/// Decodes `raw` as `T` (panicking with the JSON path on failure) and reports drift.
pub fn check<T>(label: &str, raw: &Raw) -> T
where
    T: DeserializeOwned + Serialize + Debug,
{
    let de = &mut serde_json::Deserializer::from_str(&raw.text);
    let value: T = serde_path_to_error::deserialize(de).unwrap_or_else(|e| {
        panic!(
            "{label}: live response does not decode at `{}`: {}",
            e.path(),
            e.inner()
        )
    });
    check_value(label, &raw.json, &value);
    value
}

/// Reports drift between a raw JSON value and an already-decoded `value` of it.
pub fn check_value<T: Serialize + Debug>(label: &str, raw: &Value, value: &T) {
    let back = serde_json::to_value(value).expect("model serializes");
    let mut drift = BTreeSet::new();
    unmodelled(raw, &back, "$", &mut drift);
    unknown_enum_values(&format!("{value:?}"), &mut drift);
    report(label, &drift);
}

/// Collects paths of keys present (and non-empty) in `raw` but absent from `back`.
fn unmodelled(raw: &Value, back: &Value, path: &str, out: &mut BTreeSet<String>) {
    match (raw, back) {
        (Value::Object(raw), Value::Object(back)) => {
            for (key, raw_value) in raw {
                // `$schema` is response metadata, not data.
                if is_empty(raw_value) || key == "$schema" {
                    // Serialization may skip `None` / empty values, so absence proves nothing.
                    continue;
                }
                let child = format!("{path}.{key}");
                match back.get(key) {
                    Some(back_value) => unmodelled(raw_value, back_value, &child, out),
                    None => {
                        out.insert(format!("unmodelled key {child}"));
                    }
                }
            }
        }
        (Value::Array(raw), Value::Array(back)) => {
            for (raw_item, back_item) in raw.iter().zip(back) {
                unmodelled(raw_item, back_item, &format!("{path}[]"), out);
            }
        }
        // Scalars, or a JSON-in-string field the model parses: nothing to compare.
        _ => {}
    }
}

fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        Value::Bool(_) | Value::Number(_) => false,
    }
}

/// Finds `Unknown("..")` catch-all enum values in a `Debug` rendering.
fn unknown_enum_values(debug: &str, out: &mut BTreeSet<String>) {
    let mut rest = debug;
    while let Some(start) = rest.find("Unknown(\"") {
        let field_start = rest[..start]
            .rfind(['{', ',', '(', '['])
            .map_or(0, |i| i + 1);
        let tail = &rest[start..];
        let end = tail.find("\")").map_or(tail.len(), |i| i + 2);
        out.insert(format!(
            "unknown enum value {}{}",
            rest[field_start..start].trim(),
            &tail[..end]
        ));
        rest = &tail[end..];
    }
}

fn report(label: &str, drift: &BTreeSet<String>) {
    if drift.is_empty() {
        return;
    }
    let lines: Vec<String> = drift
        .iter()
        .map(|d| format!("DRIFT {label}: {d}"))
        .collect();
    for line in &lines {
        eprintln!("{line}");
    }
    if let Ok(path) = std::env::var("POLYOXIDE_LIVE_REPORT") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        for line in &lines {
            writeln!(file, "{line}").expect("report writes");
        }
    }
    assert!(
        std::env::var("POLYOXIDE_LIVE_STRICT").as_deref() != Ok("1"),
        "{label}: drift from the live API (POLYOXIDE_LIVE_STRICT=1)"
    );
}

/// Real identifiers discovered from the live APIs once per test run, so tests do not
/// hard-code markets that will eventually close. Fetched as raw JSON, so a broken SDK
/// model cannot cascade into every test.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Gamma id of an active, liquid market.
    pub market_id: String,
    /// Slug of that market.
    pub market_slug: String,
    /// Condition id of that market.
    pub condition_id: String,
    /// The market's two CLOB token ids.
    pub token_ids: Vec<String>,
    /// Gamma id of an active event with that market.
    pub event_id: String,
    /// Slug of that event.
    pub event_slug: String,
    /// Id of a series, if the event belongs to one.
    pub series_id: Option<String>,
    /// Slug of that series.
    pub series_slug: Option<String>,
    /// Id of a tag on the event.
    pub tag_id: String,
    /// Slug of that tag.
    pub tag_slug: String,
    /// Wallet address of the top leaderboard user (has positions, trades and activity).
    pub user: String,
}

static SAMPLE: OnceCell<Sample> = OnceCell::const_new();

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("sample: missing string `{key}` in {value}"))
        .to_owned()
}

/// The shared [`Sample`].
pub async fn sample() -> &'static Sample {
    SAMPLE
        .get_or_init(|| async {
            let events = get(
                GAMMA,
                "/events",
                &[
                    ("limit", "1"),
                    ("active", "true"),
                    ("closed", "false"),
                    ("order", "volume24hr"),
                    ("ascending", "false"),
                ],
            )
            .await
            .json;
            let event = &events[0];
            let market = event["markets"]
                .as_array()
                .and_then(|markets| {
                    markets.iter().find(|m| {
                        m["active"] == true
                            && m["closed"] == false
                            && m["enableOrderBook"] == true
                            && m["clobTokenIds"].is_string()
                    })
                })
                .unwrap_or_else(|| panic!("sample: no tradable market in event {}", event["id"]));
            let token_ids: Vec<String> =
                serde_json::from_str(&str_field(market, "clobTokenIds")).expect("token id list");
            let tag = &event["tags"][0];
            let series = event["series"].get(0);
            let leaderboard = get(DATA, "/v2/leaderboard", &[("limit", "1")]).await.json;
            Sample {
                market_id: str_field(market, "id"),
                market_slug: str_field(market, "slug"),
                condition_id: str_field(market, "conditionId"),
                token_ids,
                event_id: str_field(event, "id"),
                event_slug: str_field(event, "slug"),
                series_id: series.map(|s| str_field(s, "id")),
                series_slug: series.map(|s| str_field(s, "slug")),
                tag_id: str_field(tag, "id"),
                tag_slug: str_field(tag, "slug"),
                user: str_field(&leaderboard["data"][0], "user_id"),
            }
        })
        .await
}
