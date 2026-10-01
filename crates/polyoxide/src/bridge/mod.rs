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

use polyoxide_core::{Result, ValidationError};

/// `true` if `value` is `0x` followed by exactly `digits` hex digits.
fn is_prefixed_hex(value: &str, digits: usize) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == digits && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Checks an EVM address against the spec's `Address` pattern `^0x[a-fA-F0-9]{40}$`
/// (`components/schemas/Address` in `docs/specs/bridge-openapi.yaml`).
fn validate_address(parameter: &'static str, address: &str) -> Result<()> {
    if is_prefixed_hex(address, 40) {
        Ok(())
    } else {
        Err(ValidationError::new(parameter, "must be `0x` followed by 40 hex digits").into())
    }
}

/// Checks a builder code against the `X-Builder-Code` header pattern `^0x[a-fA-F0-9]{64}$`
/// (`components/parameters/BuilderCodeHeader` in `docs/specs/bridge-openapi.yaml`).
fn validate_builder_code(code: &str) -> Result<()> {
    if is_prefixed_hex(code, 64) {
        Ok(())
    } else {
        Err(ValidationError::new(
            BUILDER_CODE_HEADER,
            "must be `0x` followed by 64 hex digits (bytes32)",
        )
        .into())
    }
}

/// The optional header attributing deposit/withdrawal requests to an integration.
const BUILDER_CODE_HEADER: &str = "X-Builder-Code";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_patterns() {
        assert!(validate_address("a", "0x56687bf447db6ffa42ffe2204a05edaa20f55839").is_ok());
        assert!(validate_address("a", "56687bf447db6ffa42ffe2204a05edaa20f55839").is_err());
        assert!(validate_address("a", "0x56687bf447db6ffa42ffe2204a05edaa20f5583").is_err());
        assert!(
            validate_builder_code(
                "0x00000000000000000000000000000000000000000000000000000000abcd1234"
            )
            .is_ok()
        );
        assert!(validate_builder_code("0xabcd1234").is_err());
        assert!(
            validate_builder_code(
                "0x00000000000000000000000000000000000000000000000000000000abcd123z"
            )
            .is_err()
        );
    }
}
