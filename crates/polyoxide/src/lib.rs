//! Unofficial Rust SDK for the Polymarket Predictions APIs.
//!
//! This project is not affiliated with or endorsed by Polymarket.
//!
//! # Logging
//!
//! This crate emits diagnostics via [`tracing`]. It never installs a subscriber; install one
//! in your application (e.g. with `tracing-subscriber`) to see them.

pub use polyoxide_core as core;
