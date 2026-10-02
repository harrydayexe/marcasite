//! Integration tests against a mock HTTP server (no live network).
//!
//! One test binary for all services (faster to build and link than one binary per file).
//! Each service has its own module directory; shared helpers live in `common`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "test code may panic on unexpected results"
)]

mod common;

#[cfg(feature = "gamma")]
mod client;

#[cfg(feature = "bridge")]
mod bridge;
#[cfg(feature = "clob")]
mod clob;
#[cfg(feature = "combos")]
mod combos;
#[cfg(feature = "data")]
mod data;
#[cfg(feature = "gamma")]
mod gamma;
#[cfg(feature = "relayer")]
mod relayer;
#[cfg(feature = "ws")]
mod ws;
