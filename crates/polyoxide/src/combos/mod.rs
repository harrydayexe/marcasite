//! Combos / RFQ REST API client (`https://combos-rfq-api.polymarket.com`).
//!
//! The combinatorial RFQ (request for quote) system lets traders combine several markets
//! ("legs") into one combo position. This module covers its public catalog endpoint. Start
//! from [`CombosClient`].
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /v1/rfq/combo-markets` | [`CombosClient::list_combo_markets`] |
//!
//! The authenticated maker (quoter) commands (`POST /v1/maker/quotes`,
//! `POST /v1/maker/quotes/cancel`, `POST /v1/maker/confirmations`) and the RFQ Quoter
//! Gateway WebSocket are not supported yet.
//!
//! See <https://docs.polymarket.com/api-reference/combo-markets/get-combo-markets>.

mod client;
mod markets;

pub use client::{CombosClient, CombosClientBuilder};
pub use markets::{ComboMarket, ComboMarketId, ComboMarketsPage, ListComboMarkets, PositionId};
