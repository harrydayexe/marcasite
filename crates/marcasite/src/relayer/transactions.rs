//! Transactions: `GET /transaction`.

use chrono::{DateTime, Utc};
use marcasite_core::{Query, Result, ValidationError, serde_util, types::Address};
use serde::{Deserialize, Serialize};

use super::RelayerClient;

marcasite_core::string_id! {
    /// A Relayer transaction id, as returned in `transactionID` by `POST /submit`, e.g.
    /// `"0190b317-a1d3-7bec-9b91-eeb6dcd3a620"`.
    pub struct TransactionId;
}

marcasite_core::string_id! {
    /// An onchain transaction hash (`0x`-prefixed hex), e.g.
    /// `"0x38cbfbeae8fffa4e2b187ee5978d3ee9cafc53af0363ed90a35b7ea9016535d8"`.
    pub struct TransactionHash;
}

marcasite_core::string_enum! {
    /// The state of a Relayer transaction (`RelayerTransaction.state`).
    ///
    /// The spec lists the values but does not describe them individually.
    pub enum TransactionState {
        /// `STATE_NEW`: the state a transaction has right after `POST /submit`.
        New => "STATE_NEW",
        /// `STATE_EXECUTED`.
        Executed => "STATE_EXECUTED",
        /// `STATE_MINED`.
        Mined => "STATE_MINED",
        /// `STATE_CONFIRMED`.
        Confirmed => "STATE_CONFIRMED",
        /// `STATE_INVALID`.
        Invalid => "STATE_INVALID",
        /// `STATE_FAILED`.
        Failed => "STATE_FAILED",
    }
}

marcasite_core::string_enum! {
    /// The type of a Relayer transaction (`RelayerTransaction.type`).
    pub enum TransactionType {
        /// `SAFE`: a Gnosis Safe wallet transaction.
        Safe => "SAFE",
        /// `PROXY`: a Proxy wallet transaction.
        Proxy => "PROXY",
    }
}

/// A transaction submitted to the Relayer (`components/schemas/RelayerTransaction`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RelayerTransaction {
    /// Unique identifier of the transaction (wire name `transactionID`).
    #[serde(rename = "transactionID")]
    pub transaction_id: Option<TransactionId>,
    /// Onchain transaction hash, available once the transaction has been broadcast.
    pub transaction_hash: Option<TransactionHash>,
    /// Signer address.
    pub from: Option<Address>,
    /// Target contract address.
    pub to: Option<Address>,
    /// The user's Polymarket proxy wallet address.
    pub proxy_address: Option<Address>,
    /// Encoded transaction data (`0x`-prefixed hex string).
    pub data: Option<String>,
    /// Transaction nonce, as a decimal string (e.g. `"60"`).
    pub nonce: Option<String>,
    /// Transaction value, as sent by the API (the documented example is `""`).
    pub value: Option<String>,
    /// Transaction signature (`0x`-prefixed hex string).
    pub signature: Option<String>,
    /// Current state of the transaction.
    pub state: Option<TransactionState>,
    /// Transaction type (wire name `type`).
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    /// Owner address.
    pub owner: Option<Address>,
    /// Transaction metadata, as sent by the API (the documented example is `""`).
    pub metadata: Option<String>,
    /// When the transaction was created.
    #[serde(default, with = "serde_util::datetime_option")]
    pub created_at: Option<DateTime<Utc>>,
    /// When the transaction was last updated.
    #[serde(default, with = "serde_util::datetime_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl RelayerClient {
    /// Gets a transaction submitted to the Relayer by its id (`GET /transaction?id=...`).
    ///
    /// Poll this with the `transactionID` returned by `POST /submit` to retrieve the
    /// onchain [`transaction_hash`](RelayerTransaction::transaction_hash) once the
    /// transaction has been broadcast. As documented, the API answers with an **array** of
    /// transactions (the documented example holds exactly one).
    ///
    /// See <https://docs.polymarket.com/api-reference/relayer/get-a-transaction-by-id>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use marcasite::relayer::{RelayerClient, TransactionState};
    ///
    /// let relayer = RelayerClient::new()?;
    /// let txs = relayer
    ///     .get_transaction("0190b317-a1d3-7bec-9b91-eeb6dcd3a620")
    ///     .await?;
    /// for tx in txs {
    ///     if tx.state == Some(TransactionState::Confirmed) {
    ///         println!("mined as {:?}", tx.transaction_hash);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if `id` is empty (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for a missing or invalid id, or
    ///   `404` if no transaction has this id (see
    ///   [`Error::is_not_found`](crate::Error::is_not_found)).
    /// - Any other [`Error`](crate::Error) for transport, server or decoding failures.
    pub async fn get_transaction(
        &self,
        id: impl Into<TransactionId>,
    ) -> Result<Vec<RelayerTransaction>> {
        let id = id.into();
        if id.as_str().is_empty() {
            return Err(ValidationError::new("id", "must not be empty").into());
        }
        let mut query = Query::new();
        query.push("id", &id);
        self.transport
            .get(&["transaction"])
            .query(query)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `200` example of `GET /transaction` in `docs/polymarket/specs/relayer-openapi.yaml`
    /// (also on `docs/polymarket/api-reference/relayer/get-a-transaction-by-id.md`).
    const EXAMPLE: &str = r#"[
        {
            "transactionID": "0190b317-a1d3-7bec-9b91-eeb6dcd3a620",
            "transactionHash": "0x38cbfbeae8fffa4e2b187ee5978d3ee9cafc53af0363ed90a35b7ea9016535d8",
            "from": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "to": "0x2791bca1f2de4661ed88a30c99a7a9449aa84174",
            "proxyAddress": "0x6d8c4e9adf5748af82dabe2c6225207770d6b4fa",
            "data": "0x...",
            "nonce": "60",
            "value": "",
            "signature": "0x01a060c734d7bdf4adde50c4a7e574036b1f8b12890911bdd1c1cfdcd77502381b89fa8a47c36f62a0b9f1cdfee7b260fd8108536db9f6b2089c02637e7de9fc20",
            "state": "STATE_CONFIRMED",
            "type": "SAFE",
            "owner": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "metadata": "",
            "createdAt": "2024-07-14T21:13:08.819782Z",
            "updatedAt": "2024-07-14T21:13:46.576639Z"
        }
    ]"#;

    #[test]
    fn deserializes_documented_example() {
        let txs: Vec<RelayerTransaction> = serde_json::from_str(EXAMPLE).unwrap();
        assert_eq!(txs.len(), 1);
        let tx = &txs[0];
        assert_eq!(
            tx.transaction_id,
            Some(TransactionId::from("0190b317-a1d3-7bec-9b91-eeb6dcd3a620"))
        );
        assert_eq!(
            tx.transaction_hash.as_ref().map(TransactionHash::as_str),
            Some("0x38cbfbeae8fffa4e2b187ee5978d3ee9cafc53af0363ed90a35b7ea9016535d8")
        );
        assert_eq!(
            tx.proxy_address,
            Some(Address::from("0x6d8c4e9adf5748af82dabe2c6225207770d6b4fa"))
        );
        assert_eq!(tx.nonce.as_deref(), Some("60"));
        assert_eq!(tx.value.as_deref(), Some(""));
        assert_eq!(tx.state, Some(TransactionState::Confirmed));
        assert_eq!(tx.transaction_type, Some(TransactionType::Safe));
        assert_eq!(
            tx.created_at.map(|d| d.timestamp_micros()),
            Some(1_720_991_588_819_782)
        );
        assert_eq!(
            tx.updated_at.map(|d| d.timestamp_micros()),
            Some(1_720_991_626_576_639)
        );
    }

    #[test]
    fn roundtrips_wire_names() {
        let txs: Vec<RelayerTransaction> = serde_json::from_str(EXAMPLE).unwrap();
        let value = serde_json::to_value(&txs[0]).unwrap();
        assert_eq!(
            value["transactionID"],
            "0190b317-a1d3-7bec-9b91-eeb6dcd3a620"
        );
        assert_eq!(value["type"], "SAFE");
        assert_eq!(
            value["proxyAddress"],
            "0x6d8c4e9adf5748af82dabe2c6225207770d6b4fa"
        );
        let again: RelayerTransaction = serde_json::from_value(value).unwrap();
        assert_eq!(again, txs[0]);
    }

    #[test]
    fn tolerates_missing_fields_and_unknown_enums() {
        let tx: RelayerTransaction =
            serde_json::from_str(r#"{"state":"STATE_SOMETHING_NEW","type":"EOA"}"#).unwrap();
        assert_eq!(
            tx.state,
            Some(TransactionState::Unknown("STATE_SOMETHING_NEW".to_owned()))
        );
        assert_eq!(
            tx.transaction_type,
            Some(TransactionType::Unknown("EOA".to_owned()))
        );
        assert_eq!(tx.transaction_id, None);
        assert_eq!(tx.created_at, None);
    }

    #[test]
    fn every_documented_state_is_known() {
        for (wire, state) in [
            ("STATE_NEW", TransactionState::New),
            ("STATE_EXECUTED", TransactionState::Executed),
            ("STATE_MINED", TransactionState::Mined),
            ("STATE_CONFIRMED", TransactionState::Confirmed),
            ("STATE_INVALID", TransactionState::Invalid),
            ("STATE_FAILED", TransactionState::Failed),
        ] {
            assert_eq!(TransactionState::from(wire), state);
            assert_eq!(state.as_str(), wire);
        }
    }
}
