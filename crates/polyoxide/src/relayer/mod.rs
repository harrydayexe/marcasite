//! Relayer API client (`https://relayer-v2.polymarket.com`).
//!
//! The Relayer submits and tracks gasless transactions for Polymarket wallets. This module
//! covers its public (unauthenticated) endpoints. Start from [`RelayerClient`].
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /transaction` | [`RelayerClient::get_transaction`] |
//! | `GET /nonce` | [`RelayerClient::get_nonce`] |
//! | `GET /relay-payload` | [`RelayerClient::get_relay_payload`] |
//! | `GET /deployed` | [`RelayerClient::check_deployed`] |
//!
//! Endpoints that require Builder API key or Relayer API key authentication
//! (`POST /submit`, `GET /transactions`, `GET /relayer/api/keys`) are not supported yet.
//!
//! Addresses passed to these endpoints are checked client-side against the spec's
//! `Address` pattern (`0x` followed by 40 hex digits) and rejected with
//! [`Error::Validation`](crate::Error::Validation) before any request is sent.
//!
//! See <https://docs.polymarket.com/api-reference/relayer/get-a-transaction-by-id>.

mod client;
mod nonce;
mod transactions;
mod wallets;

pub use client::{RelayerClient, RelayerClientBuilder};
pub use nonce::{Nonce, NonceType, RelayPayload};
pub use transactions::{
    RelayerTransaction, TransactionHash, TransactionId, TransactionState, TransactionType,
};
pub use wallets::{CheckDeployed, DeploymentStatus, WalletType};
