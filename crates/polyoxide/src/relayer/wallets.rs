//! Wallets: `GET /deployed`.

use polyoxide_core::{Query, Result, types::Address, validate};
use serde::{Deserialize, Serialize};

use super::RelayerClient;

polyoxide_core::string_enum! {
    /// The wallet type to check with [`RelayerClient::check_deployed`] (the `type` query
    /// parameter of `GET /deployed`). The server defaults to [`WalletType::Safe`].
    pub enum WalletType {
        /// `SAFE`: the user's Polymarket Gnosis Safe address (signature type `2`).
        Safe => "SAFE",
        /// `WALLET`: the user's Polymarket Deposit Wallet address (signature type `3`).
        Wallet => "WALLET",
    }
}

/// Whether a wallet is deployed onchain (`components/schemas/DeployedResponse`), returned by
/// [`CheckDeployed::send`].
///
/// The field is optional because the spec does not mark it as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeploymentStatus {
    /// Whether the wallet is deployed.
    pub deployed: Option<bool>,
}

impl RelayerClient {
    /// Checks whether the wallet at `address` is deployed onchain (`GET /deployed`).
    ///
    /// Without [`CheckDeployed::wallet_type`] the server checks a Gnosis Safe
    /// ([`WalletType::Safe`]). Errors are reported by [`CheckDeployed::send`].
    ///
    /// See <https://docs.polymarket.com/api-reference/relayer/check-if-a-wallet-is-deployed>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::relayer::{RelayerClient, WalletType};
    ///
    /// let relayer = RelayerClient::new()?;
    /// let status = relayer
    ///     .check_deployed("0x6d8c4e9aDF5748Af82Dabe2C6225207770d6B4fa")
    ///     .wallet_type(WalletType::Wallet)
    ///     .send()
    ///     .await?;
    /// println!("deployed: {:?}", status.deployed);
    /// # Ok(())
    /// # }
    /// ```
    pub fn check_deployed(&self, address: impl Into<Address>) -> CheckDeployed {
        CheckDeployed {
            client: self.clone(),
            address: address.into(),
            wallet_type: None,
        }
    }
}

/// Request builder for [`RelayerClient::check_deployed`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CheckDeployed {
    client: RelayerClient,
    address: Address,
    wallet_type: Option<WalletType>,
}

impl CheckDeployed {
    /// The wallet type to check (the `type` query parameter). The server defaults to
    /// [`WalletType::Safe`] when it is not sent.
    pub fn wallet_type(mut self, wallet_type: WalletType) -> Self {
        self.wallet_type = Some(wallet_type);
        self
    }

    fn query(&self) -> Result<Query> {
        validate::evm_address("address", self.address.as_str())?;
        let mut query = Query::new();
        query
            .push("address", &self.address)
            .push_opt("type", self.wallet_type.as_ref());
        Ok(query)
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if the address is not `0x`
    ///   followed by 40 hex digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an address the server
    ///   rejects.
    /// - Any other [`Error`](crate::Error) for transport, server or decoding failures.
    pub async fn send(self) -> Result<DeploymentStatus> {
        let query = self.query()?;
        self.client
            .transport
            .get(&["deployed"])
            .query(query)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `200` example of `GET /deployed` in `docs/specs/relayer-openapi.yaml`.
    #[test]
    fn deserializes_example() {
        let status: DeploymentStatus = serde_json::from_str(r#"{"deployed":true}"#).unwrap();
        assert_eq!(status.deployed, Some(true));
    }

    #[test]
    fn wallet_type_wire_values() {
        assert_eq!(WalletType::Safe.as_str(), "SAFE");
        assert_eq!(WalletType::Wallet.as_str(), "WALLET");
    }
}
