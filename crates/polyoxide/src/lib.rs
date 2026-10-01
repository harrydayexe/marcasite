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
//! | CLOB API: order books, prices, spreads, markets, price history, rewards | [`clob::ClobClient`] | `clob` |
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
//! for the next page), and every page lazily as a [`Stream`] of items from `.into_stream()`:
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
//! Prices, sizes and amounts are [`Decimal`]s, never floating point. Timestamps are
//! [`chrono::DateTime<Utc>`](chrono::DateTime). Identifiers are newtypes (see [`types`]) so
//! that, for example, a token id cannot be passed where a condition id is expected. String
//! enums are `#[non_exhaustive]` and have an `Unknown(String)` variant, so values added by the
//! server later never break deserialization.
//!
//! # Logging
//!
//! This crate emits diagnostics via [`tracing`]. It never installs a subscriber; install one
//! in your application (e.g. with `tracing-subscriber`) to see them. Requests are logged at
//! `DEBUG`, response bodies at `TRACE`.
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
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub use polyoxide_core::WebSocketError;
pub use polyoxide_core::{
    ApiError, ConfigError, DecodeError, Error, HttpClient, HttpClientBuilder, Result, RetryPolicy,
    Service, StatusCode, TransportError, ValidationError,
};

/// Re-export of [`rust_decimal::Decimal`], used for all prices, sizes and amounts.
pub use rust_decimal::Decimal;

/// Re-export of the [`chrono`] crate, whose `DateTime<Utc>` is used for timestamps.
pub use chrono;

/// Identifier newtypes and enums shared by several services.
pub mod types {
    pub use polyoxide_core::types::{Address, ConditionId, Side, TokenId};
}
