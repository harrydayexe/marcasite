//! Combos / RFQ REST API client (`https://combos-rfq-api.polymarket.com`).
//!
//! Covers the public endpoints: combo-eligible markets.
//!
//! Not yet implemented: see `ENDPOINTS.md`.

mod client;

pub use client::{CombosClient, CombosClientBuilder};
