//! Deposit and withdrawal addresses: `POST /deposit`, `POST /withdraw`.

use polyoxide_core::{Result, types::Address, validate};
use serde::{Deserialize, Serialize};

use super::{BUILDER_CODE_HEADER, BridgeClient, ChainId};

/// The body of `POST /deposit` (`components/schemas/DepositRequest`).
#[derive(Debug, Serialize)]
struct DepositRequest<'a> {
    address: &'a Address,
}

/// The body of `POST /withdraw` (`components/schemas/WithdrawalRequest`). Every field is
/// required.
///
/// ```
/// use polyoxide::bridge::WithdrawalRequest;
///
/// // The documented example: withdraw to USDC on Ethereum.
/// let request = WithdrawalRequest::new(
///     "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
///     "1",
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
///     "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
/// );
/// assert_eq!(request.to_chain_id.as_str(), "1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WithdrawalRequest {
    /// Source Polymarket wallet address on Polygon.
    pub address: Address,
    /// Destination chain id (e.g. `"1"` for Ethereum, `"8453"` for Base,
    /// `"1151111081099710"` for Solana).
    pub to_chain_id: ChainId,
    /// Destination token contract address.
    pub to_token_address: String,
    /// Destination wallet address where funds will be sent (wire name `recipientAddr`).
    pub recipient_addr: String,
}

impl WithdrawalRequest {
    /// Creates a withdrawal request. Arguments follow the field order of the spec.
    pub fn new(
        address: impl Into<Address>,
        to_chain_id: impl Into<ChainId>,
        to_token_address: impl Into<String>,
        recipient_addr: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            to_chain_id: to_chain_id.into(),
            to_token_address: to_token_address.into(),
            recipient_addr: recipient_addr.into(),
        }
    }
}

/// Bridge addresses created by `POST /deposit` or `POST /withdraw`
/// (`components/schemas/DepositResponse`).
///
/// Send funds to one of these addresses to bridge them. Pass an address to
/// [`BridgeClient::get_transaction_status`] to track the transfers it receives. Every field
/// is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BridgeAddresses {
    /// The bridge addresses, one per blockchain network family.
    pub address: Option<ChainAddresses>,
    /// Additional information about the bridge addresses.
    pub note: Option<String>,
}

/// Bridge addresses for different blockchain networks (`DepositResponse.address`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChainAddresses {
    /// EVM-compatible bridge address (Ethereum, Polygon, Arbitrum, Base, etc.).
    pub evm: Option<String>,
    /// Solana Virtual Machine bridge address.
    pub svm: Option<String>,
    /// Bitcoin bridge address.
    pub btc: Option<String>,
    /// Tron bridge address.
    pub tron: Option<String>,
}

impl BridgeClient {
    /// Creates bridge addresses that credit deposits to the Polymarket wallet `address`
    /// as pUSD (`POST /deposit`).
    ///
    /// The request creates server-side state, so it is **not** retried automatically.
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/create-bridge-addresses>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// let bridge = polyoxide::bridge::BridgeClient::new()?;
    /// let created = bridge
    ///     .create_deposit_addresses("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
    ///     .builder_code("0x00000000000000000000000000000000000000000000000000000000abcd1234")
    ///     .send()
    ///     .await?;
    /// println!("send USDC on an EVM chain to {:?}", created.address.and_then(|a| a.evm));
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_deposit_addresses(&self, address: impl Into<Address>) -> CreateDepositAddresses {
        CreateDepositAddresses {
            client: self.clone(),
            address: address.into(),
            builder_code: None,
        }
    }

    /// Creates bridge addresses that withdraw funds from a Polymarket wallet to another
    /// chain and token (`POST /withdraw`).
    ///
    /// The request creates server-side state, so it is **not** retried automatically.
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/create-withdrawal-addresses>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::bridge::{BridgeClient, WithdrawalRequest};
    ///
    /// let bridge = BridgeClient::new()?;
    /// let created = bridge
    ///     .create_withdrawal_addresses(WithdrawalRequest::new(
    ///         "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
    ///         "1",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    ///     ))
    ///     .send()
    ///     .await?;
    /// println!("{:?}", created.note);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_withdrawal_addresses(
        &self,
        request: WithdrawalRequest,
    ) -> CreateWithdrawalAddresses {
        CreateWithdrawalAddresses {
            client: self.clone(),
            request,
            builder_code: None,
        }
    }
}

/// Request builder for [`BridgeClient::create_deposit_addresses`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CreateDepositAddresses {
    client: BridgeClient,
    address: Address,
    builder_code: Option<String>,
}

impl CreateDepositAddresses {
    /// Attributes the request to your integration with your builder code (a bytes32 hex
    /// string, `0x` followed by 64 hex digits), sent as the `X-Builder-Code` header.
    ///
    /// The header is optional; per the docs, omitting it still succeeds but the server
    /// returns a `missing_builder_code` warning.
    pub fn builder_code(mut self, code: impl Into<String>) -> Self {
        self.builder_code = Some(code.into());
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if the address is not `0x`
    ///   followed by 40 hex digits, or the builder code is not `0x` followed by 64 hex
    ///   digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an address, body or
    ///   builder code the server rejects, or `500` on a server error.
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures.
    pub async fn send(self) -> Result<BridgeAddresses> {
        validate::evm_address("address", self.address.as_str())?;
        let body = DepositRequest {
            address: &self.address,
        };
        let mut request = self.client.transport.post(&["deposit"]).json(&body);
        if let Some(code) = self.builder_code {
            validate::bytes32(BUILDER_CODE_HEADER, &code)?;
            request = request.header(BUILDER_CODE_HEADER, code);
        }
        request.send().await
    }
}

/// Request builder for [`BridgeClient::create_withdrawal_addresses`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` is awaited"]
pub struct CreateWithdrawalAddresses {
    client: BridgeClient,
    request: WithdrawalRequest,
    builder_code: Option<String>,
}

impl CreateWithdrawalAddresses {
    /// Attributes the request to your integration with your builder code (a bytes32 hex
    /// string, `0x` followed by 64 hex digits), sent as the `X-Builder-Code` header.
    ///
    /// The header is optional; per the docs, omitting it still succeeds but the server
    /// returns a `missing_builder_code` warning.
    pub fn builder_code(mut self, code: impl Into<String>) -> Self {
        self.builder_code = Some(code.into());
        self
    }

    /// Sends the request.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if
    ///   [`WithdrawalRequest::address`] is not `0x` followed by 40 hex digits, or the
    ///   builder code is not `0x` followed by 64 hex digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for invalid or missing
    ///   parameters or a builder code the server rejects, or `500` on a server error.
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures.
    pub async fn send(self) -> Result<BridgeAddresses> {
        validate::evm_address("address", self.request.address.as_str())?;
        let mut request = self
            .client
            .transport
            .post(&["withdraw"])
            .json(&self.request);
        if let Some(code) = self.builder_code {
            validate::bytes32(BUILDER_CODE_HEADER, &code)?;
            request = request.header(BUILDER_CODE_HEADER, code);
        }
        request.send().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Request example of `POST /deposit` in `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn serializes_deposit_request() {
        let address = Address::from("0x56687bf447db6ffa42ffe2204a05edaa20f55839");
        let body = serde_json::to_value(DepositRequest { address: &address }).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"address": "0x56687bf447db6ffa42ffe2204a05edaa20f55839"})
        );
    }

    /// Request example of `POST /withdraw` in `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn serializes_withdrawal_request() {
        let request = WithdrawalRequest::new(
            "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
            "1",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        );
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            serde_json::json!({
                "address": "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
                "toChainId": "1",
                "toTokenAddress": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "recipientAddr": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
            })
        );
    }

    /// `201` example of `POST /withdraw` in `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn deserializes_documented_response() {
        let json = r#"{
            "address": {
                "evm": "0x23566f8b2E82aDfCf01846E54899d110e97AC053",
                "svm": "CrvTBvzryYxBHbWu2TiQpcqD5M7Le7iBKzVmEj3f36Jb",
                "btc": "bc1q8eau83qffxcj8ht4hsjdza3lha9r3egfqysj3g"
            },
            "note": "Send funds to these addresses to bridge to your destination chain and token."
        }"#;
        let created: BridgeAddresses = serde_json::from_str(json).unwrap();
        let address = created.address.as_ref().unwrap();
        assert_eq!(
            address.evm.as_deref(),
            Some("0x23566f8b2E82aDfCf01846E54899d110e97AC053")
        );
        assert_eq!(
            address.svm.as_deref(),
            Some("CrvTBvzryYxBHbWu2TiQpcqD5M7Le7iBKzVmEj3f36Jb")
        );
        assert_eq!(
            address.btc.as_deref(),
            Some("bc1q8eau83qffxcj8ht4hsjdza3lha9r3egfqysj3g")
        );
        assert_eq!(address.tron, None);
        assert!(created.note.unwrap().starts_with("Send funds"));
    }
}
