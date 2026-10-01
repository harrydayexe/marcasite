//! Bridge API client (`https://bridge.polymarket.com`).
//!
//! The Bridge API moves funds between other chains and a Polymarket wallet: it lists the
//! supported assets, quotes transfers, creates deposit and withdrawal addresses, and reports
//! the transfers seen at those addresses. Start from [`BridgeClient`].
//!
//! | Endpoint | Method |
//! |---|---|
//! | `GET /supported-assets` | [`BridgeClient::get_supported_assets`] |
//! | `POST /quote` | [`BridgeClient::get_quote`] |
//! | `POST /deposit` | [`BridgeClient::create_deposit_addresses`] |
//! | `POST /withdraw` | [`BridgeClient::create_withdrawal_addresses`] |
//! | `GET /status/{address}` | [`BridgeClient::get_transaction_status`] |
//!
//! Every Bridge API endpoint is public; none requires authentication.
//!
//! Token amounts are exchanged as integer strings in the token's **base units** (no
//! decimals, e.g. `"10000000"` for 10 USDC with 6 decimals) and are kept as `String`s so no
//! precision is lost. USD values, fees and percentages are [`Decimal`](crate::Decimal)s.
//!
//! `POST /quote` is read-only and retried like a `GET` under the client's
//! [`RetryPolicy`](crate::RetryPolicy); `POST /deposit` and `POST /withdraw` create addresses
//! and are never retried automatically.
//!
//! See <https://docs.polymarket.com/api-reference/bridge/get-supported-assets>.

mod addresses;
mod assets;
mod client;
mod quote;
mod status;

pub use addresses::{
    BridgeAddresses, ChainAddresses, CreateDepositAddresses, CreateWithdrawalAddresses,
    WithdrawalRequest,
};
pub use assets::{ChainId, SupportedAsset, SupportedAssets, Token};
pub use client::{BridgeClient, BridgeClientBuilder};
pub use quote::{FeeBreakdown, Quote, QuoteId, QuoteRequest};
pub use status::{GetTransactionStatus, Transaction, TransactionStatus, TransactionStatusPage};

/// The optional header attributing deposit/withdrawal requests to an integration. Its value
/// must match `^0x[a-fA-F0-9]{64}$` (`components/parameters/BuilderCodeHeader` in
/// `docs/specs/bridge-openapi.yaml`).
const BUILDER_CODE_HEADER: &str = "X-Builder-Code";
