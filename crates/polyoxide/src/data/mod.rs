//! Data API v2 client (`https://data-api.polymarket.com`).
//!
//! Covers wallet portfolios, trade and activity feeds, market state and leaderboards.
//!
//! Not yet implemented: see `ENDPOINTS.md`.

mod client;

pub use client::{DataClient, DataClientBuilder};
