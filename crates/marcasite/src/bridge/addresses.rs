//! Deposit and withdrawal addresses: `POST /deposit`, `POST /withdraw`.

use marcasite_core::{Result, types::Address, validate};
use serde::{Deserialize, Serialize};

use super::{BUILDER_CODE_HEADER, BridgeClient, ChainId, required};

/// The body of `POST /deposit` (`components/schemas/DepositRequest`).
#[derive(Debug, Serialize)]
struct DepositRequest<'a> {
    address: &'a Address,
}

/// The body of `POST /withdraw` (`components/schemas/WithdrawalRequest`), for
/// [`BridgeClient::create_withdrawal_addresses`].
///
/// The API requires every field. Each one has a named setter, and there is deliberately no
/// constructor with positional arguments, so that the source wallet, the destination token
/// and the recipient (all addresses) cannot be swapped by accident.
/// [`CreateWithdrawalAddresses::send`] reports a field that was not set as an
/// [`Error::Validation`](crate::Error::Validation) naming it (by its wire name), before
/// anything is sent.
///
/// ```
/// use marcasite::bridge::WithdrawalRequest;
///
/// // The documented example: withdraw to USDC on Ethereum.
/// let request = WithdrawalRequest::new()
///     .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
///     .to_chain_id("1")
///     .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
///     .recipient_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
/// # let _ = request;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[must_use]
pub struct WithdrawalRequest {
    address: Option<Address>,
    to_chain_id: Option<ChainId>,
    to_token_address: Option<String>,
    recipient_address: Option<String>,
}

/// The wire form of a complete [`WithdrawalRequest`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WithdrawalRequestBody<'a> {
    address: &'a Address,
    to_chain_id: &'a ChainId,
    to_token_address: &'a str,
    #[serde(rename = "recipientAddr")]
    recipient_address: &'a str,
}

impl WithdrawalRequest {
    /// An empty request; set every field with the setters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Source Polymarket wallet address on Polygon (`address`): `0x` followed by 40 hex
    /// digits.
    pub fn address(mut self, address: impl Into<Address>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Destination chain id (`toChainId`, e.g. `"1"` for Ethereum, `"8453"` for Base,
    /// `"1151111081099710"` for Solana).
    pub fn to_chain_id(mut self, chain_id: impl Into<ChainId>) -> Self {
        self.to_chain_id = Some(chain_id.into());
        self
    }

    /// Destination token contract address (`toTokenAddress`).
    pub fn to_token_address(mut self, address: impl Into<String>) -> Self {
        self.to_token_address = Some(address.into());
        self
    }

    /// Destination wallet address where funds will be sent (wire name `recipientAddr`).
    pub fn recipient_address(mut self, address: impl Into<String>) -> Self {
        self.recipient_address = Some(address.into());
        self
    }

    /// The wire body, or an [`Error::Validation`](crate::Error::Validation) naming the
    /// first field (in the spec's order) that was not set, or an `address` that is not `0x`
    /// followed by 40 hex digits.
    fn body(&self) -> Result<WithdrawalRequestBody<'_>> {
        let address = required("address", &self.address)?;
        validate::evm_address("address", address.as_str())?;
        Ok(WithdrawalRequestBody {
            address,
            to_chain_id: required("toChainId", &self.to_chain_id)?,
            to_token_address: required("toTokenAddress", &self.to_token_address)?,
            recipient_address: required("recipientAddr", &self.recipient_address)?,
        })
    }
}

/// Bridge addresses created by `POST /deposit` or `POST /withdraw`
/// (`components/schemas/DepositResponse`).
///
/// Send funds to one of these addresses to bridge them. Pass an address to
/// [`BridgeClient::list_transactions`] to track the transfers it receives. Every field is
/// optional because the spec marks none as required.
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
    /// The request creates server-side state, so it is **never** retried automatically.
    /// Errors are reported by [`CreateDepositAddresses::send`]; read its "Retrying" section
    /// before repeating a failed request yourself.
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/create-bridge-addresses>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// let bridge = marcasite::bridge::BridgeClient::new()?;
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
    /// The request creates server-side state, so it is **never** retried automatically.
    /// Errors (including fields of `request` that were not set) are reported by
    /// [`CreateWithdrawalAddresses::send`]; read its "Retrying" section before repeating a
    /// failed request yourself.
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/create-withdrawal-addresses>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use marcasite::bridge::{BridgeClient, WithdrawalRequest};
    ///
    /// let bridge = BridgeClient::new()?;
    /// let request = WithdrawalRequest::new()
    ///     .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
    ///     .to_chain_id("1")
    ///     .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    ///     .recipient_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    /// let created = bridge.create_withdrawal_addresses(request).send().await?;
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
    /// # Retrying
    ///
    /// This request is sent exactly once, whatever the client's
    /// [`RetryPolicy`](crate::RetryPolicy). If it fails with a timeout, a connection
    /// error or a `5xx` status, the server may still have created the addresses: such a
    /// failure does not say whether the request was processed, and the docs do not say
    /// whether repeating it returns the same addresses. [`Error::is_retryable`] being `true`
    /// only means that the failure is transient, not that repeating this request is safe;
    /// decide that yourself before sending it again.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if the address is not `0x`
    ///   followed by 40 hex digits, or the builder code is not `0x` followed by 64 hex
    ///   digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an address, body or
    ///   builder code the server rejects, or `500` on a server error.
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures (see "Retrying" above).
    ///
    /// [`Error::is_retryable`]: crate::Error::is_retryable
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
    /// # Retrying
    ///
    /// This request is sent exactly once, whatever the client's
    /// [`RetryPolicy`](crate::RetryPolicy). If it fails with a timeout, a connection
    /// error or a `5xx` status, the server may still have created the addresses: such a
    /// failure does not say whether the request was processed, and the docs do not say
    /// whether repeating it returns the same addresses. [`Error::is_retryable`] being `true`
    /// only means that the failure is transient, not that repeating this request is safe;
    /// decide that yourself before sending it again.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if a field of the
    ///   [`WithdrawalRequest`] was not set (the error's
    ///   [`parameter`](crate::ValidationError::parameter) is its wire name, e.g.
    ///   `recipientAddr`), if its `address` is not `0x` followed by 40 hex digits, or if the
    ///   builder code is not `0x` followed by 64 hex digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for invalid or missing
    ///   parameters or a builder code the server rejects, or `500` on a server error.
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures (see "Retrying" above).
    ///
    /// [`Error::is_retryable`]: crate::Error::is_retryable
    pub async fn send(self) -> Result<BridgeAddresses> {
        let body = self.request.body()?;
        let mut request = self.client.transport.post(&["withdraw"]).json(&body);
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
        let request = WithdrawalRequest::new()
            .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
            .to_chain_id("1")
            .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .recipient_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        assert_eq!(
            serde_json::to_value(request.body().unwrap()).unwrap(),
            serde_json::json!({
                "address": "0x9156dd10bea4c8d7e2d591b633d1694b1d764756",
                "toChainId": "1",
                "toTokenAddress": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "recipientAddr": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
            })
        );
    }

    #[test]
    fn incomplete_withdrawal_requests_are_rejected() {
        let missing = |request: WithdrawalRequest| match request.body() {
            Err(marcasite_core::Error::Validation(err)) => err.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        };
        assert_eq!(missing(WithdrawalRequest::new()), "address");
        let complete = WithdrawalRequest::new()
            .address("0x9156dd10bea4c8d7e2d591b633d1694b1d764756")
            .to_chain_id("1")
            .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(missing(complete.clone()), "recipientAddr");
        assert_eq!(
            missing(complete.recipient_address("0xd8").address("0x123")),
            "address"
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
