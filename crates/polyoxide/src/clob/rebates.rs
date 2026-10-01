//! Maker rebates: `GET /rebates/current`.

use chrono::NaiveDate;
use polyoxide_core::{
    Query, Result,
    types::{Address, ConditionId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::ClobClient;

/// Fees rebated to a maker on one market and date (`components/schemas/RebatedFees`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RebatedFees {
    /// Date of the rebate (documented format `YYYY-MM-DD`).
    pub date: NaiveDate,
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Asset address (e.g. the USDC contract).
    pub asset_address: Address,
    /// The maker's address.
    pub maker_address: Address,
    /// Rebated fee amount in USDC.
    pub rebated_fees_usdc: Decimal,
}

impl ClobClient {
    /// Gets the fees rebated to a maker on a given date, per market
    /// (`GET /rebates/current`). No authentication is required.
    ///
    /// See <https://docs.polymarket.com/api-reference/rebates/get-current-rebated-fees-for-a-maker>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error). An invalid date or maker address is an
    /// [`Error::Api`](crate::Error::Api) with status `400`.
    pub async fn get_current_rebated_fees(
        &self,
        date: NaiveDate,
        maker_address: impl Into<Address>,
    ) -> Result<Vec<RebatedFees>> {
        let mut query = Query::new();
        query
            .push("date", date.format("%Y-%m-%d"))
            .push("maker_address", maker_address.into());
        self.transport
            .get(&["rebates", "current"])
            .query(query)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Example response of `GET /rebates/current` in docs/specs/clob-openapi.yaml
    /// (docs/api-reference/rebates/get-current-rebated-fees-for-a-maker.md).
    #[test]
    fn deserializes_rebated_fees() {
        let json = r#"[{
            "date": "2026-02-27",
            "condition_id": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "asset_address": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
            "maker_address": "0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604",
            "rebated_fees_usdc": "0.237519"
        }]"#;
        let fees: Vec<RebatedFees> = serde_json::from_str(json).unwrap();
        assert_eq!(fees[0].date, NaiveDate::from_ymd_opt(2026, 2, 27).unwrap());
        assert_eq!(
            fees[0].maker_address,
            "0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604"
        );
        assert_eq!(
            fees[0].rebated_fees_usdc,
            "0.237519".parse::<Decimal>().unwrap()
        );

        let again: Vec<RebatedFees> =
            serde_json::from_str(&serde_json::to_string(&fees).unwrap()).unwrap();
        assert_eq!(again, fees);
    }
}
