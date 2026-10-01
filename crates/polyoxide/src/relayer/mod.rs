//! Relayer API client (`https://relayer-v2.polymarket.com`).
//!
//! Covers the public endpoints: transaction status, nonces, relay payloads and wallet deployment checks.
//!
//! Not yet implemented: see `ENDPOINTS.md`.

mod client;

pub use client::{RelayerClient, RelayerClientBuilder};
