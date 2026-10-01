//! Supported assets: `GET /supported-assets`.

use polyoxide_core::{Result, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::BridgeClient;

polyoxide_core::string_id! {
    /// A chain id as the Bridge API sends it: a decimal string, e.g. `"1"` (Ethereum),
    /// `"8453"` (Base) or `"1151111081099710"` (Solana).
    pub struct ChainId;
}

/// The assets the bridge supports (`components/schemas/SupportedAssetsResponse`), returned
/// by [`BridgeClient::get_supported_assets`].
///
/// The field is optional because the spec does not mark it as required; use
/// [`SupportedAssets::assets`] for a slice that is empty when it is absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SupportedAssets {
    /// Supported assets with the minimum amounts for deposits and withdrawals.
    pub supported_assets: Option<Vec<SupportedAsset>>,
}

impl SupportedAssets {
    /// The supported assets, or an empty slice if the field was absent.
    #[must_use]
    pub fn assets(&self) -> &[SupportedAsset] {
        self.supported_assets.as_deref().unwrap_or_default()
    }
}

/// A token the bridge supports on one chain (`components/schemas/SupportedAsset`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SupportedAsset {
    /// Chain id.
    pub chain_id: Option<ChainId>,
    /// Human-readable chain name, e.g. `"Ethereum"`.
    pub chain_name: Option<String>,
    /// The token.
    pub token: Option<Token>,
    /// Minimum amount in USD for deposits and withdrawals (a JSON number on the wire).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub min_checkout_usd: Option<Decimal>,
}

/// A token on a supported chain (`components/schemas/Token`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Token {
    /// Full token name, e.g. `"USD Coin"`.
    pub name: Option<String>,
    /// Token symbol, e.g. `"USDC"`.
    pub symbol: Option<String>,
    /// Token contract address, in the chain's own address format (not necessarily an EVM
    /// address).
    pub address: Option<String>,
    /// Token decimals: the number of decimal places between base units and whole tokens.
    pub decimals: Option<u32>,
}

impl BridgeClient {
    /// Lists the chains and tokens the bridge supports, with minimum amounts
    /// (`GET /supported-assets`).
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/get-supported-assets>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let bridge = polyoxide::bridge::BridgeClient::new()?;
    /// for asset in bridge.get_supported_assets().await?.assets() {
    ///     println!("{:?} on {:?}", asset.token, asset.chain_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Api`](crate::Error::Api) with status `500` on a server error, or any other
    /// [`Error`](crate::Error) for transport or decoding failures.
    pub async fn get_supported_assets(&self) -> Result<SupportedAssets> {
        self.transport.get(&["supported-assets"]).send().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec has no response example for `GET /supported-assets`; this body is assembled
    /// from the per-field `example` values of `SupportedAsset` and `Token` in
    /// `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn deserializes_schema_examples() {
        let json = r#"{"supportedAssets":[{
            "chainId": "1",
            "chainName": "Ethereum",
            "token": {
                "name": "USD Coin",
                "symbol": "USDC",
                "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "decimals": 6
            },
            "minCheckoutUsd": 45
        }]}"#;
        let assets: SupportedAssets = serde_json::from_str(json).unwrap();
        let asset = &assets.assets()[0];
        assert_eq!(asset.chain_id, Some(ChainId::from("1")));
        assert_eq!(asset.chain_name.as_deref(), Some("Ethereum"));
        assert_eq!(asset.min_checkout_usd, Some(Decimal::from(45)));
        let token = asset.token.as_ref().unwrap();
        assert_eq!(token.symbol.as_deref(), Some("USDC"));
        assert_eq!(token.decimals, Some(6));

        // Numbers stay numbers when re-serialized, and integers stay integers.
        let value = serde_json::to_value(&assets).unwrap();
        assert_eq!(
            value["supportedAssets"][0]["minCheckoutUsd"],
            serde_json::json!(45)
        );
    }

    #[test]
    fn missing_list_is_an_empty_slice() {
        let assets: SupportedAssets = serde_json::from_str("{}").unwrap();
        assert_eq!(assets.supported_assets, None);
        assert!(assets.assets().is_empty());
    }

    #[test]
    fn fractional_minimum_is_exact() {
        let asset: SupportedAsset = serde_json::from_str(r#"{"minCheckoutUsd":2.5}"#).unwrap();
        assert_eq!(asset.min_checkout_usd, Some(Decimal::new(25, 1)));
    }
}
