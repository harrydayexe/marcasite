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

use polyoxide_core::{Result, ValidationError, types::Address};

/// Checks `address` against the spec's `Address` pattern `^0x[a-fA-F0-9]{40}$`
/// (`components/schemas/Address` in `docs/specs/relayer-openapi.yaml`).
fn validate_address(parameter: &'static str, address: &Address) -> Result<()> {
    let valid = address
        .as_str()
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 40 && hex.bytes().all(|b| b.is_ascii_hexdigit()));
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(parameter, "must be `0x` followed by 40 hex digits").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_pattern() {
        assert!(
            validate_address(
                "a",
                &Address::from("0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b5")
            )
            .is_ok()
        );
        for bad in [
            "",
            "0x",
            "6e0c80c90ea6c15917308F820Eac91Ce2724B5b5",
            "0X6e0c80c90ea6c15917308F820Eac91Ce2724B5b5",
            "0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b",
            "0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b5a",
            "0x6e0c80c90ea6c15917308F820Eac91Ce2724B5bg",
        ] {
            assert!(validate_address("a", &Address::from(bad)).is_err(), "{bad}");
        }
    }
}
