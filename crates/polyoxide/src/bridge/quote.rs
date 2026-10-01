//! Quotes: `POST /quote`.

use polyoxide_core::{Result, serde_util};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{BridgeClient, ChainId};

polyoxide_core::string_id! {
    /// A bridge quote id, e.g.
    /// `"0x00c34ba467184b0146406d62b0e60aaa24ed52460bd456222b6155a0d9de0ad5"`.
    pub struct QuoteId;
}

/// The body of `POST /quote` (`components/schemas/QuoteRequest`). Every field is required.
///
/// ```
/// use polyoxide::bridge::QuoteRequest;
///
/// // 10 tokens with 6 decimals, from Polygon to Polygon (the documented example).
/// let request = QuoteRequest::new(
///     "10000000",
///     "137",
///     "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
///     "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
///     "137",
///     "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
/// );
/// assert_eq!(request.from_amount_base_unit, "10000000");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteRequest {
    /// Amount of tokens to send, in the source token's base units (an integer string with no
    /// decimal point, e.g. `"10000000"`).
    pub from_amount_base_unit: String,
    /// Source chain id.
    pub from_chain_id: ChainId,
    /// Source token address.
    pub from_token_address: String,
    /// Address of the recipient.
    pub recipient_address: String,
    /// Destination chain id.
    pub to_chain_id: ChainId,
    /// Destination token address.
    pub to_token_address: String,
}

impl QuoteRequest {
    /// Creates a quote request. Arguments follow the field order of the spec; see the
    /// field docs for their meaning.
    pub fn new(
        from_amount_base_unit: impl Into<String>,
        from_chain_id: impl Into<ChainId>,
        from_token_address: impl Into<String>,
        recipient_address: impl Into<String>,
        to_chain_id: impl Into<ChainId>,
        to_token_address: impl Into<String>,
    ) -> Self {
        Self {
            from_amount_base_unit: from_amount_base_unit.into(),
            from_chain_id: from_chain_id.into(),
            from_token_address: from_token_address.into(),
            recipient_address: recipient_address.into(),
            to_chain_id: to_chain_id.into(),
            to_token_address: to_token_address.into(),
        }
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
    /// `estInputUsd`. The spec describes it as "Estimated token amount received in USD".
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub est_input_usd: Option<Decimal>,
    /// `estOutputUsd`. The spec describes it as "Estimated token amount sent in USD".
    #[serde(default, with = "serde_util::decimal_number_option")]
    pub est_output_usd: Option<Decimal>,
    /// Estimated amount of the destination token received, in its base units (an integer
    /// string).
    pub est_to_token_base_unit: Option<String>,
    /// Unique quote id of the request.
    pub quote_id: Option<QuoteId>,
}

/// Breakdown of the estimated fees of a [`Quote`] (`components/schemas/FeeBreakdown`).
///
/// Percentages are as sent by the API; the spec does not say whether `1` means 1% or 100%.
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
    /// let quote = bridge
    ///     .get_quote(&QuoteRequest::new(
    ///         "10000000",
    ///         "137",
    ///         "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
    ///         "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
    ///         "137",
    ///         "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
    ///     ))
    ///     .await?;
    /// println!("receive {:?} base units", quote.est_to_token_base_unit);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Api`](crate::Error::Api) with status `400` if the server rejects a field
    ///   (e.g. `"fromAmountBaseUnit is required"`), or `500` if no quote can be made
    ///   (`"cannot get quote"`).
    /// - Any other [`Error`](crate::Error) for transport, rate limiting or decoding
    ///   failures.
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote> {
        self.transport
            .post(&["quote"])
            .json(request)
            .idempotent(true)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Request example of `POST /quote` in `docs/specs/bridge-openapi.yaml`.
    #[test]
    fn serializes_documented_request() {
        let request = QuoteRequest::new(
            "10000000",
            "137",
            "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
            "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
            "137",
            "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
        );
        let expected = serde_json::json!({
            "fromAmountBaseUnit": "10000000",
            "fromChainId": "137",
            "fromTokenAddress": "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
            "recipientAddress": "0x17eC161f126e82A8ba337f4022d574DBEaFef575",
            "toChainId": "137",
            "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB"
        });
        assert_eq!(serde_json::to_value(&request).unwrap(), expected);
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
