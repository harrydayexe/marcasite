//! Unofficial Rust SDK for the Polymarket Predictions APIs.
//!
//! This project is not affiliated with or endorsed by Polymarket.
//!
//! polyoxide gives typed, async access to the **public (unauthenticated)** Polymarket
//! Predictions APIs:
//!
//! | Service | Client | Feature |
//! |---|---|---|
//! | Gamma API: events, markets, tags, series, comments, sports, search, profiles | [`gamma::GammaClient`] | `gamma` |
//! | CLOB API: order books, prices, spreads, markets, price history, rewards, rebates, builder trades | [`clob::ClobClient`] | `clob` |
//! | Data API v2: positions, PnL, trades, activity, leaderboards, holders | [`data::DataClient`] | `data` |
//! | Relayer API: transaction status, nonces, wallet deployment | [`relayer::RelayerClient`] | `relayer` |
//! | Bridge API: supported assets, quotes, deposit/withdrawal addresses | [`bridge::BridgeClient`] | `bridge` |
//! | Combos / RFQ API: combo-eligible markets | [`combos::CombosClient`] | `combos` |
//! | WebSockets: market channel, sports results, PolyBolt public prices | [`ws`] | `ws` |
//!
//! All features are enabled by default; disable default features and pick the ones you need
//! to reduce compile time. [`ENDPOINTS.md`] lists every endpoint and its implementation status.
//!
//! # Quick start
//!
//! ```no_run
//! # #[cfg(feature = "gamma")]
//! # async fn run() -> polyoxide::Result<()> {
//! use polyoxide::Polymarket;
//!
//! let pm = Polymarket::new()?;
//! let tags = pm.gamma().list_tags().limit(5).send().await?;
//! for tag in tags {
//!     println!("{:?}: {:?}", tag.id, tag.label);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Each service client can also be created on its own, e.g. [`gamma::GammaClient::new`], and
//! every client accepts a custom base URL (for example to point tests at a mock server).
//!
//! # Requests
//!
//! Endpoints with only required parameters are plain `async fn`s. Endpoints with optional
//! parameters return a request builder: chain setters, then call `.send().await`. Required
//! parameters are always arguments of the client method, never builder setters.
//!
//! # Pagination
//!
//! Paginated endpoints return one page from `.send()` (including the cursor or offset needed
//! for the next page), and every item of every page from `.into_stream()`, as a
//! [`Paginated`] [`Stream`] that fetches the pages lazily. It ends after the last page, or
//! right after yielding the first error. [`Paginated`] is `Unpin`, so a plain `while let`
//! loop works:
//!
//! ```no_run
//! # #[cfg(feature = "gamma")]
//! # async fn run() -> polyoxide::Result<()> {
//! use futures_util::StreamExt as _;
//!
//! let pm = polyoxide::Polymarket::new()?;
//! let mut tags = pm.gamma().list_tags().limit(100).into_stream();
//! while let Some(tag) = tags.next().await {
//!     println!("{:?}", tag?.label);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The usual stream adapters apply, for example to collect a bounded number of items:
//!
//! ```no_run
//! # #[cfg(feature = "gamma")]
//! # async fn run() -> polyoxide::Result<()> {
//! use futures_util::{StreamExt as _, TryStreamExt as _};
//!
//! let pm = polyoxide::Polymarket::new()?;
//! let first_200: Vec<_> = pm
//!     .gamma()
//!     .list_tags()
//!     .limit(100)
//!     .into_stream()
//!     .take(200)
//!     .try_collect()
//!     .await?;
//! # let _ = first_200;
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! Every fallible operation returns [`Result<T>`], whose error type [`Error`] distinguishes
//! API errors (with the parsed error body and trace id), rate limiting (with `Retry-After`),
//! timeouts, transport failures, decoding failures (with the JSON path of the bad field),
//! client-side validation failures and configuration errors. See [`Error`].
//!
//! # Types
//!
//! Prices, sizes and amounts are [`Decimal`]s, never floating point. Values the API sends as
//! JSON strings are parsed exactly. Values it sends as JSON numbers are exact if they are
//! integers (within `i64`/`u64`) or have at most 15 significant digits; longer numbers pass
//! through `f64` while being parsed and may be rounded (see [`Decimal`]). Timestamps are
//! [`chrono::DateTime<Utc>`](chrono::DateTime). Identifiers are newtypes (see [`types`]) so
//! that, for example, a token id cannot be passed where a condition id is expected. String
//! enums are `#[non_exhaustive]` and have an `Unknown(String)` variant, so values added by the
//! server later never break deserialization.
//!
//! # Dependencies in the public API
//!
//! Types from these crates appear in polyoxide's public API, so upgrading any of them to a
//! new major (or, before 1.0, minor) version is a breaking change of polyoxide:
//! `http` 1 ([`StatusCode`], [`Method`]), `url` 2 ([`Url`]), `serde_json` 1
//! ([`serde_json`]), `chrono` 0.4 ([`chrono`]), `rust_decimal` 1 ([`Decimal`]) and
//! `futures-core` 0.3 ([`Stream`]). The types are re-exported, so you do not need to
//! depend on these crates yourself to name them:
//!
//! ```
//! use polyoxide::{Method, Url, chrono::Utc, serde_json::Value};
//!
//! let _ = (Method::GET, Url::parse("https://example.com"), Value::Null, Utc::now());
//! ```
//!
//! # Logging
//!
//! This crate emits diagnostics via [`tracing`]. It never installs a subscriber; install one
//! in your application (e.g. with `tracing-subscriber`) to see them. Requests, retries and
//! failures are logged at `DEBUG`, response bodies and WebSocket frames at `TRACE`. Errors
//! that are returned to you are never logged above `DEBUG`, so they are not reported twice.
//!
//! [`ENDPOINTS.md`]: https://github.com/harrydayexe/polyoxide/blob/main/ENDPOINTS.md
//! [`Stream`]: futures_core::Stream
#![cfg_attr(docsrs, feature(doc_cfg))]

mod client;

#[cfg(feature = "bridge")]
#[cfg_attr(docsrs, doc(cfg(feature = "bridge")))]
pub mod bridge;
#[cfg(feature = "clob")]
#[cfg_attr(docsrs, doc(cfg(feature = "clob")))]
pub mod clob;
#[cfg(feature = "combos")]
#[cfg_attr(docsrs, doc(cfg(feature = "combos")))]
pub mod combos;
#[cfg(feature = "data")]
#[cfg_attr(docsrs, doc(cfg(feature = "data")))]
pub mod data;
#[cfg(feature = "gamma")]
#[cfg_attr(docsrs, doc(cfg(feature = "gamma")))]
pub mod gamma;
#[cfg(feature = "relayer")]
#[cfg_attr(docsrs, doc(cfg(feature = "relayer")))]
pub mod relayer;
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub mod ws;

pub use client::{Polymarket, PolymarketBuilder};
pub use polyoxide_core::pagination::Paginated;
pub use polyoxide_core::{
    ApiError, ConfigError, DecodeError, Error, HttpClient, HttpClientBuilder, Method, Result,
    RetryPolicy, Service, StatusCode, TransportError, Url, ValidationError,
};
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub use polyoxide_core::{WebSocketError, WebSocketErrorKind};

/// Re-export of [`rust_decimal::Decimal`], used for all prices, sizes and amounts.
///
/// A decimal sent by the API as a JSON string is parsed exactly. One sent as a JSON number
/// is exact if it is an integer within `i64`/`u64` or has at most 15 significant digits;
/// a longer number passes through `f64` while being parsed and is rounded to about 15 to
/// 17 significant digits (e.g. `12345678901.123456` becomes `12345678901.123455`).
pub use rust_decimal::Decimal;

/// Re-export of the [`chrono`] crate, whose `DateTime<Utc>` is used for timestamps. Its
/// `now` feature is enabled, so `polyoxide::chrono::Utc::now()` works.
pub use chrono;

/// Re-export of the [`serde_json`] crate, whose `Value` appears in the public API (e.g. the
/// `Unknown` variants of the WebSocket event enums).
pub use serde_json;

/// Identifier newtypes and enums shared by several services.
///
/// The service modules re-export the ones they use, so for example `polyoxide::gamma::MarketId`
/// and `polyoxide::types::MarketId` are the same type.
pub mod types {
    pub use polyoxide_core::types::{
        Address, ConditionId, EventId, MarketId, QuestionId, Side, TokenId,
    };
}
