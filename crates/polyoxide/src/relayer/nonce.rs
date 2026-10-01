//! Nonces: `GET /nonce`, `GET /relay-payload`.

use polyoxide_core::{Query, Result, types::Address};
use serde::{Deserialize, Serialize};

use super::{RelayerClient, validate_address};

polyoxide_core::string_enum! {
    /// The type of nonce to retrieve (the `type` query parameter of `GET /nonce` and
    /// `GET /relay-payload`).
    pub enum NonceType {
        /// `PROXY`: the user's Proxy wallet nonce.
        Proxy => "PROXY",
        /// `SAFE`: the user's Gnosis Safe nonce.
        Safe => "SAFE",
    }
}

/// The current nonce for a user (`components/schemas/NonceResponse`), returned by
/// [`RelayerClient::get_nonce`].
///
/// The field is optional because the spec does not mark it as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Nonce {
    /// Current nonce value, as a decimal string (e.g. `"31"`). Kept as the documented
    /// string so no precision is lost.
    pub nonce: Option<String>,
}

/// The relayer address and nonce for a user (`components/schemas/RelayPayloadResponse`),
/// returned by [`RelayerClient::get_relay_payload`].
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RelayPayload {
    /// Relayer address.
    pub address: Option<Address>,
    /// Current nonce value, as a decimal string (e.g. `"31"`).
    pub nonce: Option<String>,
}

impl RelayerClient {
    /// Gets the current Proxy or Safe nonce for a user (`GET /nonce`).
    ///
    /// `address` is the user's **signer** address.
    ///
    /// See <https://docs.polymarket.com/api-reference/relayer/get-current-nonce-for-a-user>.
    ///
    /// ```no_run
    /// # async fn run() -> polyoxide::Result<()> {
    /// use polyoxide::relayer::{NonceType, RelayerClient};
    ///
    /// let relayer = RelayerClient::new()?;
    /// let nonce = relayer
    ///     .get_nonce("0x77837466dd64fb52ECD00C737F060d0ff5CCB575", NonceType::Proxy)
    ///     .await?;
    /// println!("{:?}", nonce.nonce);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if `address` is not `0x` followed
    ///   by 40 hex digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an address or type the
    ///   server rejects (e.g. an [`NonceType::Unknown`] value).
    /// - Any other [`Error`](crate::Error) for transport, server or decoding failures.
    pub async fn get_nonce(
        &self,
        address: impl Into<Address>,
        nonce_type: NonceType,
    ) -> Result<Nonce> {
        let query = nonce_query(address.into(), &nonce_type)?;
        self.transport.get(&["nonce"]).query(query).send().await
    }

    /// Gets the relayer address and the current nonce for a user (`GET /relay-payload`).
    ///
    /// `address` is the user's **signer** address.
    ///
    /// See <https://docs.polymarket.com/api-reference/relayer/get-relayer-address-and-nonce>.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if `address` is not `0x` followed
    ///   by 40 hex digits (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an address or type the
    ///   server rejects.
    /// - Any other [`Error`](crate::Error) for transport, server or decoding failures.
    pub async fn get_relay_payload(
        &self,
        address: impl Into<Address>,
        nonce_type: NonceType,
    ) -> Result<RelayPayload> {
        let query = nonce_query(address.into(), &nonce_type)?;
        self.transport
            .get(&["relay-payload"])
            .query(query)
            .send()
            .await
    }
}

fn nonce_query(address: Address, nonce_type: &NonceType) -> Result<Query> {
    validate_address("address", &address)?;
    let mut query = Query::new();
    query.push("address", address).push("type", nonce_type);
    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `200` example of `GET /nonce` in `docs/specs/relayer-openapi.yaml`.
    #[test]
    fn deserializes_nonce_example() {
        let nonce: Nonce = serde_json::from_str(r#"{"nonce":"31"}"#).unwrap();
        assert_eq!(nonce.nonce.as_deref(), Some("31"));
        let empty: Nonce = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.nonce, None);
    }

    /// `200` example of `GET /relay-payload` in `docs/specs/relayer-openapi.yaml`.
    #[test]
    fn deserializes_relay_payload_example() {
        let payload: RelayPayload = serde_json::from_str(
            r#"{"address":"0x4da9395388791c22684e03779c3de10934eb9cfb","nonce":"31"}"#,
        )
        .unwrap();
        assert_eq!(
            payload.address,
            Some(Address::from("0x4da9395388791c22684e03779c3de10934eb9cfb"))
        );
        assert_eq!(payload.nonce.as_deref(), Some("31"));
    }

    #[test]
    fn builds_query_in_wire_format() {
        let query = nonce_query(
            Address::from("0x77837466dd64fb52ECD00C737F060d0ff5CCB575"),
            &NonceType::Safe,
        )
        .unwrap();
        assert_eq!(
            query.to_string(),
            "address=0x77837466dd64fb52ECD00C737F060d0ff5CCB575&type=SAFE"
        );
        assert!(nonce_query(Address::from("nope"), &NonceType::Proxy).is_err());
    }
}
