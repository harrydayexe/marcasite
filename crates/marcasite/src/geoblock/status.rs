//! The geoblock status: `GET /api/geoblock`.

use std::net::IpAddr;

use marcasite_core::Result;
use serde::{Deserialize, Serialize};

use super::GeoblockClient;

/// The geoblock status of the requesting IP address, returned by
/// [`GeoblockClient::get_status`].
///
/// This is a check only; see the [module documentation](crate::geoblock). Every field is
/// required, as the docs list them without any optionality. Whether `region` can be absent
/// for countries without sub-national regions has not been observed (see `OPEN_QUESTIONS.md`).
///
/// See <https://docs.polymarket.com/api-reference/geoblock>.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GeoblockStatus {
    /// Whether the address is blocked from placing orders (the docs' wording).
    ///
    /// The docs also describe *close-only* jurisdictions, where existing positions can be
    /// closed but no new ones opened. Whether those report `true` here is not documented
    /// (see `OPEN_QUESTIONS.md`), so treat `true` as "do not open new positions" and do not
    /// read `false` as proof that every action is allowed.
    pub blocked: bool,
    /// The IP address Polymarket detected for the request.
    pub ip: IpAddr,
    /// ISO 3166-1 alpha-2 country code, e.g. `"US"`.
    pub country: String,
    /// Region or state code, e.g. `"OH"`.
    pub region: String,
}

impl GeoblockClient {
    /// Checks the geographic eligibility of the IP address this request is sent from
    /// (`GET /api/geoblock`).
    ///
    /// See <https://docs.polymarket.com/api-reference/geoblock>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// let status = marcasite::geoblock::GeoblockClient::new()?.get_status().await?;
    /// if status.blocked {
    ///     println!("trading not available in {}", status.country);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Api`](crate::Error::Api) on a non-2xx status,
    /// [`Error::RateLimited`](crate::Error::RateLimited) on `429`, or any other
    /// [`Error`](crate::Error) for transport or decoding failures.
    pub async fn get_status(&self) -> Result<GeoblockStatus> {
        self.transport.get(&["api", "geoblock"]).send().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The response example in `docs/polymarket/api-reference/geoblock.md`.
    #[test]
    fn deserializes_documented_example() {
        let json = r#"{"blocked":true,"ip":"203.0.113.42","country":"US","region":"NY"}"#;
        let status: GeoblockStatus = serde_json::from_str(json).unwrap();
        assert!(status.blocked);
        assert_eq!(status.ip, "203.0.113.42".parse::<IpAddr>().unwrap());
        assert_eq!(status.country, "US");
        assert_eq!(status.region, "NY");
        assert_eq!(serde_json::to_string(&status).unwrap(), json);
    }

    /// Captured from `GET https://polymarket.com/api/geoblock` on 2026-11-17.
    #[test]
    fn deserializes_live_capture() {
        let json = r#"{"blocked":true,"ip":"203.0.113.7","country":"US","region":"OH"}"#;
        let status: GeoblockStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.country, "US");
        assert_eq!(status.region, "OH");
    }

    #[test]
    fn accepts_ipv6() {
        let json = r#"{"blocked":false,"ip":"2001:db8::1","country":"CH","region":"ZH"}"#;
        let status: GeoblockStatus = serde_json::from_str(json).unwrap();
        assert!(!status.blocked);
        assert!(status.ip.is_ipv6());
    }

    #[test]
    fn missing_field_is_an_error() {
        let json = r#"{"blocked":false,"ip":"203.0.113.42","country":"CH"}"#;
        assert!(serde_json::from_str::<GeoblockStatus>(json).is_err());
    }
}
