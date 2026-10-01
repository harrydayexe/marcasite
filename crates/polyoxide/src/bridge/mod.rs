//! Bridge API client (`https://bridge.polymarket.com`).
//!
//! Covers supported assets, quotes, deposit/withdrawal addresses and transfer status.
//!
//! Not yet implemented: see `ENDPOINTS.md`.

mod client;

pub use client::{BridgeClient, BridgeClientBuilder};
