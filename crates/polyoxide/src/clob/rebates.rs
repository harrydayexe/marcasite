//! Maker rebates: `GET /rebates/current`.

use chrono::NaiveDate;
use polyoxide_core::{
    Query, Result,
    types::{Address, ConditionId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{ClobClient, types::require_id};

/// Fees rebated to a maker on one market and date (`components/schemas/RebatedFees`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RebatedFees {
    /// Date of the rebate.
    ///
    /// The spec documents `format: date` (`YYYY-MM-DD`), but live sends an RFC 3339
    /// date-time at midnight UTC (e.g. `"2026-09-25T00:00:00Z"`). Both are accepted; the
    /// time of day is dropped. Serializes as `YYYY-MM-DD`.
    #[serde(with = "date_or_datetime")]
    pub date: NaiveDate,
    /// Condition id of the market.
    pub condition_id: ConditionId,
    /// Asset address (e.g. the USDC contract).
    pub asset_address: Address,
    /// The maker's address.
    pub maker_address: Address,
    /// Rebated fee amount in USDC (a numeric string on the wire).
    pub rebated_fees_usdc: Decimal,
}

/// A date the API sends as `YYYY-MM-DD` (documented) or as an RFC 3339 date-time (live);
/// serializes as `YYYY-MM-DD`.
mod date_or_datetime {
    use chrono::NaiveDate;
    use polyoxide_core::serde_util;
    use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

    pub(super) fn serialize<S: Serializer>(
        value: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&value.format("%Y-%m-%d"))
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error> {
        let s = String::deserialize(deserializer)?;
        serde_util::parse_datetime(&s)
            .map(|time| time.date_naive())
            .ok_or_else(|| D::Error::custom(format!("invalid date: {s:?}")))
    }
}

impl ClobClient {
    /// Gets the fees rebated to a maker on a given date, per market
    /// (`GET /rebates/current`). No authentication is required.
    ///
    /// A maker without rebates on the date gets an empty list. The spec documents `[]`, but
    /// live answers with the JSON body `null` (HTTP 200), which is mapped to an empty list
    /// too (see `SPEC_DEVIATIONS.md`).
    ///
    /// See <https://docs.polymarket.com/api-reference/rebates/get-current-rebated-fees-for-a-maker>.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`](crate::Error::Validation) if `maker_address` is empty. An
    /// invalid date or maker address is an [`Error::Api`](crate::Error::Api) with status
    /// `400`. See [`Error`](crate::Error) for the other cases.
    pub async fn get_current_rebated_fees(
        &self,
        date: NaiveDate,
        maker_address: impl Into<Address>,
    ) -> Result<Vec<RebatedFees>> {
        let maker_address = maker_address.into();
        require_id("maker_address", maker_address.as_str())?;
        let mut query = Query::new();
        query
            .push("date", date.format("%Y-%m-%d"))
            .push("maker_address", maker_address);
        let fees: Option<Vec<RebatedFees>> = self
            .transport
            .get(&["rebates", "current"])
            .query(query)
            .send()
            .await?;
        Ok(fees.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clob::types::test_util::round_trip;

    /// Trimmed live response of `GET /rebates/current?date=2026-09-25&maker_address=0xBF11..`
    /// (captured 2026-10-02): `date` is an RFC 3339 date-time.
    #[test]
    fn deserializes_live_rebated_fees() {
        let json = r#"[{
            "date": "2026-09-25T00:00:00Z",
            "condition_id": "0x12feff92fb1c0bb769aa0785240e64e577901a7920dc9e6c9bc8543a94a7f4f6",
            "asset_address": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
            "maker_address": "0xBF119Cf0f6ff285688857487E669Adad35cF2CE1",
            "rebated_fees_usdc": "0.990878"
        }]"#;
        let fees: Vec<RebatedFees> = serde_json::from_str(json).unwrap();
        assert_eq!(fees[0].date, NaiveDate::from_ymd_opt(2026, 9, 25).unwrap());
        assert_eq!(
            fees[0].rebated_fees_usdc,
            "0.990878".parse::<Decimal>().unwrap()
        );
        // Serializes as the documented plain date.
        let again = serde_json::to_value(&fees[0]).unwrap();
        assert_eq!(again["date"], "2026-09-25");
        assert!(
            serde_json::from_str::<Vec<RebatedFees>>(&json.replace("2026-09-25T00:00:00Z", "soon"))
                .is_err()
        );
    }

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
        let fees: Vec<RebatedFees> = round_trip(json);
        assert_eq!(fees[0].date, NaiveDate::from_ymd_opt(2026, 2, 27).unwrap());
        assert_eq!(
            fees[0].maker_address,
            "0xFeA4cB3dD4ca7CefD3368653B7D6FF9BcDFca604"
        );
        assert_eq!(
            fees[0].rebated_fees_usdc,
            "0.237519".parse::<Decimal>().unwrap()
        );
    }
}
