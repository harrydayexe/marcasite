//! Geoblock check (`https://polymarket.com/api/geoblock`).
//!
//! Polymarket rejects orders from some jurisdictions. This module asks Polymarket whether the
//! IP address the request comes from is blocked, so an application can warn its users before
//! they try to trade. Start from [`GeoblockClient`].
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /api/geoblock` | [`GeoblockClient::get_status`] |
//!
//! The endpoint is on `polymarket.com`, not on the API servers, and needs no authentication.
//! It reports the status of the address the request was sent **from**: it is a check only.
//! marcasite never provides, and will not add, any way to bypass a geoblock.
//!
//! See <https://docs.polymarket.com/api-reference/geoblock>.

mod client;
mod status;

pub use client::{GeoblockClient, GeoblockClientBuilder};
pub use status::GeoblockStatus;
