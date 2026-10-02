//! Live tests against the production Polymarket APIs.
//!
//! Every test is `#[ignore]`d (so default `cargo test` stays offline) and read-only. Run them
//! with `just test-live`. See `common` for the drift report.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::print_stderr,
    unreachable_pub,
    reason = "test code may panic on unexpected results and prints the drift report"
)]

mod common;

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
