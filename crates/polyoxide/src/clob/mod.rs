//! CLOB API client (`https://clob.polymarket.com`).
//!
//! Covers public market data: order books, prices, spreads, markets, price history, rewards and rebates.
//!
//! Not yet implemented: see `ENDPOINTS.md`.

mod client;

pub use client::{ClobClient, ClobClientBuilder};
