//! Quotes: `POST /quote`.

use polyoxide_core::{Result, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{BridgeClient, ChainId, required};

polyoxide_core::string_id! {
    /// A bridge quote id, e.g.
    /// `"0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"`.
    pub struct QuoteId;
}

/// The body of `POST /quote` (`components/schemas/QuoteRequest`), for
/// [`BridgeClient::get_quote`].
///
/// The API requires every field. Each one has a named setter, and there is deliberately no
/// constructor with positional arguments, so that a source and a destination value (or a
/// token and a recipient address) cannot be swapped by accident. [`BridgeClient::get_quote`]
/// reports a field that was not set as an [`Error::Validation`] naming it (by its wire
/// name), before anything is sent.
///
/// ```
/// use polyoxide::bridge::QuoteRequest;
///
/// // The documented example: from Polygon to Polygon.
/// let request = QuoteRequest::new()
///     .from_amount_base_unit("10000000")
///     .from_chain_id("137")
///     .from_token_address("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")
///     .to_chain_id("137")
///     .to_token_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
///     .recipient_address("0x17eC161f126e82A8ba337f4022d574DBEaFef575");
/// # let _ = request;
/// ```
///
/// [`Error::Validation`]: crate::Error::Validation
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[must_use]
pub struct QuoteRequest {
    from_amount_base_unit: Option<String>,
    from_chain_id: Option<ChainId>,
    from_token_address: Option<String>,
    recipient_address: Option<String>,
    to_chain_id: Option<ChainId>,
    to_token_address: Option<String>,
}

/// The wire form of a complete [`QuoteRequest`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRequestBody<'a> {
    from_amount_base_unit: &'a str,
    from_chain_id: &'a ChainId,
    from_token_address: &'a str,
    recipient_address: &'a str,
    to_chain_id: &'a ChainId,
    to_token_address: &'a str,
}

impl QuoteRequest {
    /// An empty request; set every field with the setters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Amount of tokens to send (`fromAmountBaseUnit`), in the source token's base units
    /// (the documented example is `"10000000"`).
    pub fn from_amount_base_unit(mut self, amount: impl Into<String>) -> Self {
        self.from_amount_base_unit = Some(amount.into());
        self
    }

    /// Source chain id (`fromChainId`, e.g. `"137"`).
    pub fn from_chain_id(mut self, chain_id: impl Into<ChainId>) -> Self {
        self.from_chain_id = Some(chain_id.into());
        self
    }

    /// Source token address (`fromTokenAddress`).
    pub fn from_token_address(mut self, address: impl Into<String>) -> Self {
        self.from_token_address = Some(address.into());
        self
    }

    /// Address of the recipient (`recipientAddress`).
    pub fn recipient_address(mut self, address: impl Into<String>) -> Self {
        self.recipient_address = Some(address.into());
        self
    }

    /// Destination chain id (`toChainId`, e.g. `"137"`).
    pub fn to_chain_id(mut self, chain_id: impl Into<ChainId>) -> Self {
        self.to_chain_id = Some(chain_id.into());
        self
    }

    /// Destination token address (`toTokenAddress`).
    pub fn to_token_address(mut self, address: impl Into<String>) -> Self {
        self.to_token_address = Some(address.into());
        self
    }

    /// The wire body, or an [`Error::Validation`](crate::Error::Validation) naming the
    /// first field (in the spec's order) that was not set.
    fn body(&self) -> Result<QuoteRequestBody<'_>> {
        Ok(QuoteRequestBody {
            from_amount_base_unit: required("fromAmountBaseUnit", &self.from_amount_base_unit)?,
            from_chain_id: required("fromChainId", &self.from_chain_id)?,
            from_token_address: required("fromTokenAddress", &self.from_token_address)?,
            recipient_address: required("recipientAddress", &self.recipient_address)?,
            to_chain_id: required("toChainId", &self.to_chain_id)?,
            to_token_address: required("toTokenAddress", &self.to_token_address)?,
        })
    }
}

/// A bridge quote (`components/schemas/QuoteResponse`), returned by
/// [`BridgeClient::get_quote`].
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Quote {
    /// Estimated time to complete the checkout, in milliseconds.
    pub est_checkout_time_ms: Option<u64>,
    /// Breakdown of the estimated fees.
    pub est_fee_breakdown: Option<FeeBreakdown>,
    /// `estInputUsd`: the estimated value of the amount **sent** (the input), in USD.
    ///
    /// The spec's descriptions of `estInputUsd` and `estOutputUsd` are swapped; the live
    /// API (2026-10-02) sends the input value here (about the amount sent) and the lower,
    /// after-fees value in [`est_output_usd`](Self::est_output_usd). See `SPEC_DEVIATIONS.md`.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub est_input_usd: Option<Decimal>,
    /// `estOutputUsd`: the estimated value of the amount **received** (the output), in USD.
    /// See [`est_input_usd`](Self::est_input_usd) for the swapped spec descriptions.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub est_output_usd: Option<Decimal>,
    /// Estimated amount of the destination token received (`estToTokenBaseUnit`; the
    /// documented example is `"14491203"`).
    pub est_to_token_base_unit: Option<String>,
    /// Unique quote id of the request.
    pub quote_id: Option<QuoteId>,
}

/// Breakdown of the estimated fees of a [`Quote`] (`components/schemas/FeeBreakdown`).
///
/// Percentages are as sent by the API and use a scale where `1` means 1% (observed live:
/// `swapImpact` `0.0226` next to `swapImpactUsd` `0.002261` on a roughly 10 USD transfer is
/// 0.0226%); the spec does not say.
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FeeBreakdown {
    /// Label of the app fee, e.g. `"Fun.xyz fee"`.
    pub app_fee_label: Option<String>,
    /// App fees as a percentage of the total amount sent.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub app_fee_percent: Option<Decimal>,
    /// App fees in USD.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub app_fee_usd: Option<Decimal>,
    /// Fill cost as a percentage of the total amount sent.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub fill_cost_percent: Option<Decimal>,
    /// Fill cost in USD.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub fill_cost_usd: Option<Decimal>,
    /// Gas fee in USD.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub gas_usd: Option<Decimal>,
    /// Maximum potential slippage, as a percentage.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub max_slippage: Option<Decimal>,
    /// Amount after factoring in slippage.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub min_received: Option<Decimal>,
    /// Swap impact as a percentage of the total amount sent.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub swap_impact: Option<Decimal>,
    /// Swap impact in USD.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub swap_impact_usd: Option<Decimal>,
    /// Total impact as a percentage of the total amount sent.
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_impact: Option<Decimal>,
    /// Impact cost of the transaction (USD).
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub total_impact_usd: Option<Decimal>,
}

impl BridgeClient {
    /// Gets a quote for a transfer (`POST /quote`).
    ///
    /// The request only reads data, so it is retried like a `GET` under the client's
    /// [`RetryPolicy`](crate::RetryPolicy).
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/get-a-quote>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::bridge::{BridgeClient, QuoteRequest};
    ///
    /// let bridge = BridgeClient::new()?;
    /// let request = QuoteRequest::new()
    ///     .from_amount_base_unit("10000000")
    ///     .from_chain_id("137")
    ///     .from_token_address("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")
    ///     .to_chain_id("137")
    ///     .to_token_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
    ///     .recipient_address("0x17eC161f126e82A8ba337f4022d574DBEaFef575");
    /// let quote = bridge.get_quote(request).await?;
    /// println!("receive {:?} base units", quote.est_to_token_base_unit);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if a field of `request` was not
    ///   set; its [`parameter`](crate::ValidationError::parameter) is the field's wire name,
    ///   e.g. `fromAmountBaseUnit` (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` if the server rejects a field
    ///   (e.g. `"fromAmountBaseUnit is required"`), or `500` if no quote can be made
    ///   (`"cannot get quote"`).
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures.
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<Quote> {
        let body = request.body()?;
        self.transport
            .post(&["quote"])
            .json(&body)
            .idempotent(true)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use polyoxide_core::Error;

    use super::*;

    /// Request example of `POST /quote` in `docs/specs/bridge-openapi.yaml`.
    fn documented_request() -> QuoteRequest {
        QuoteRequest::new()
            .from_amount_base_unit("10000000")
            .from_chain_id("137")
            .from_token_address("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359")
            .recipient_address("0x17eC161f126e82A8ba337f4022d574DBEaFef575")
            .to_chain_id("137")
            .to_token_address("0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB")
    }

    #[test]
    fn serializes_documented_request() {
        let request = documented_request();
        let expected = serde_json::json!({
            "fromAmountBaseUnit": "10000000",
            "fromChainId": "137",
            "fromTokenAddress": "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
            "recipientAddress": "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB"
        });
        assert_eq!(
            serde_json::to_value(request.body().unwrap()).unwrap(),
            expected
        );
    }

    #[test]
    fn missing_fields_are_named_by_their_wire_name() {
        let missing = |request: QuoteRequest| match request.body() {
            Err(Error::Validation(err)) => err.parameter().to_owned(),
            other => panic!("expected a validation error, got {other:?}"),
        };
        assert_eq!(missing(QuoteRequest::new()), "fromAmountBaseUnit");
        let without_destination_chain = QuoteRequest::new()
            .from_amount_base_unit("1")
            .from_chain_id("137")
            .from_token_address("0x1")
            .recipient_address("0x2")
            .to_token_address("0x3");
        assert_eq!(missing(without_destination_chain), "toChainId");
        let without_recipient = QuoteRequest::new()
            .from_amount_base_unit("1")
            .from_chain_id("137")
            .from_token_address("0x1")
            .to_chain_id("1")
            .to_token_address("0x3");
        assert_eq!(missing(without_recipient), "recipientAddress");
        // Setters may be called in any order, and a later call replaces an earlier one.
        let reordered = documented_request()
            .to_token_address("0x4")
            .from_chain_id("1");
        assert!(reordered.body().is_ok());
    }

    /// `200` example of `POST /quote` in `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn deserializes_documented_response() {
        let json = r#"{
            "estCheckoutTimeMs": 25000,
            "estFeeBreakdown": {
                "appFeeLabel": "Fun.xyz fee",
                "appFeePercent": 0,
                "appFeeUsd": 0,
                "fillCostPercent": 0,
                "fillCostUsd": 0,
                "gasUsd": 0.003854,
                "maxSlippage": 0,
                "minReceived": 14.488305,
                "swapImpact": 0,
                "swapImpactUsd": 0,
                "totalImpact": 0,
                "totalImpactUsd": 0
            },
            "estInputUsd": 14.488305,
            "estOutputUsd": 14.488305,
            "estToTokenBaseUnit": "14491203",
            "quoteId": "0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"
        }"#;
        let quote: Quote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.est_checkout_time_ms, Some(25_000));
        assert_eq!(quote.est_input_usd, Some(Decimal::new(14_488_305, 6)));
        assert_eq!(quote.est_output_usd, Some(Decimal::new(14_488_305, 6)));
        assert_eq!(quote.est_to_token_base_unit.as_deref(), Some("14491203"));
        assert_eq!(
            quote.quote_id,
            Some(QuoteId::from(
                "0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"
            ))
        );
        let fees = quote.est_fee_breakdown.as_ref().unwrap();
        assert_eq!(fees.app_fee_label.as_deref(), Some("Fun.xyz fee"));
        assert_eq!(fees.gas_usd, Some(Decimal::new(3_854, 6)));
        assert_eq!(fees.min_received, Some(Decimal::new(14_488_305, 6)));
        assert_eq!(fees.total_impact_usd, Some(Decimal::ZERO));

        let value = serde_json::to_value(&quote).unwrap();
        assert_eq!(value["estFeeBreakdown"]["gasUsd"], 0.003854);
        let again: Quote = serde_json::from_value(value).unwrap();
        assert_eq!(again, quote);
    }

    #[test]
    fn empty_quote() {
        let quote: Quote = serde_json::from_str("{}").unwrap();
        assert_eq!(quote.est_fee_breakdown, None);
        assert_eq!(quote.est_input_usd, None);
    }
}
