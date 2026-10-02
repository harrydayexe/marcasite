//! Transfer status: `GET /status/{address}`.

use crate::Paginated;
use chrono::{DateTime, Utc};
use marcasite_core::{
    Query, Result, ValidationError,
    pagination::{CursorPage, cursor_stream},
    serde_util,
};
use serde::{Deserialize, Serialize};

use super::{BridgeClient, ChainId};

/// The largest page size `GET /status/{address}` accepts.
const MAX_LIMIT: u32 = 100;

marcasite_core::string_enum! {
    /// The status of a bridge transfer (`Transaction.status`).
    ///
    /// If a transfer fails, remains stuck, or funds are held due to a compliance check, the
    /// docs direct users to the Bridge API provider's support
    /// (<https://intercom.help/funxyz/en/articles/10732578-contact-us>).
    pub enum TransactionStatus {
        /// `DEPOSIT_DETECTED`. Transfers in this status have no
        /// [`created_time`](Transaction::created_time).
        DepositDetected => "DEPOSIT_DETECTED",
        /// `PROCESSING`.
        Processing => "PROCESSING",
        /// `ORIGIN_TX_CONFIRMED`.
        OriginTxConfirmed => "ORIGIN_TX_CONFIRMED",
        /// `SUBMITTED`.
        Submitted => "SUBMITTED",
        /// `COMPLETED`. Only transfers in this status carry a
        /// [`tx_hash`](Transaction::tx_hash).
        Completed => "COMPLETED",
        /// `FAILED`.
        Failed => "FAILED",
    }
}

/// A deposit or withdrawal seen at a bridge address (`components/schemas/Transaction`).
///
/// Every field is optional because the spec marks none as required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Transaction {
    /// Source chain id.
    pub from_chain_id: Option<ChainId>,
    /// Source token contract address, in the source chain's address format.
    pub from_token_address: Option<String>,
    /// Amount sent, in the source token's base units (an integer string with no decimal
    /// point).
    pub from_amount_base_unit: Option<String>,
    /// Destination chain id.
    pub to_chain_id: Option<ChainId>,
    /// Destination token contract address.
    pub to_token_address: Option<String>,
    /// Current status of the transfer.
    pub status: Option<TransactionStatus>,
    /// Transaction hash, only available when the status is
    /// [`Completed`](TransactionStatus::Completed). Its format depends on the chain.
    pub tx_hash: Option<String>,
    /// When the transfer was created (wire name `createdTimeMs`, Unix milliseconds).
    /// Missing while the status is [`DepositDetected`](TransactionStatus::DepositDetected).
    /// Always an integer live (never fractional).
    #[serde(
        rename = "createdTimeMs",
        default,
        with = "serde_util::timestamp_millis_option"
    )]
    pub created_time: Option<DateTime<Utc>>,
}

/// One page of transfers seen at a bridge address
/// (`components/schemas/TransactionStatusResponse`), returned by
/// [`ListTransactions::send`].
///
/// Both fields are required by the spec, so a response without `transactions` or without
/// `nextCursor` fails to decode (with [`Error::Decode`](crate::Error::Decode)) instead of
/// being mistaken for the last page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransactionStatusPage {
    /// One page of transfers, newest first. This is a page, not the full history.
    pub transactions: Vec<Transaction>,
    /// Opaque continuation token for the next page (wire name `nextCursor`), as sent:
    /// `None` when the walk is complete. Prefer [`next_cursor()`](Self::next_cursor),
    /// which also treats an empty string as the end. Pass it back unchanged with
    /// [`ListTransactions::cursor`], for the same address only. Stop on `None`, not on an
    /// empty or short page.
    #[serde(deserialize_with = "Option::deserialize")]
    pub next_cursor: Option<String>,
}

impl TransactionStatusPage {
    /// The transfers on this page, newest first.
    #[must_use]
    pub fn items(&self) -> &[Transaction] {
        &self.transactions
    }

    /// The transfers on this page, newest first, by value.
    #[must_use]
    pub fn into_items(self) -> Vec<Transaction> {
        self.transactions
    }

    /// The cursor for the next page, to pass to [`ListTransactions::cursor`]; `None` on
    /// the last page (a `null` or empty `nextCursor`).
    #[must_use]
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor
            .as_deref()
            .filter(|cursor| !cursor.is_empty())
    }
}

impl BridgeClient {
    /// Lists the deposits and withdrawals seen at a bridge address, newest first
    /// (`GET /status/{address}`, cursor-paginated).
    ///
    /// `address` is a bridge address from [`BridgeClient::create_deposit_addresses`] or
    /// [`BridgeClient::create_withdrawal_addresses`]; EVM, Solana, Tron and Bitcoin formats
    /// are supported. Use [`ListTransactions::send`] for one page (repeat it without a
    /// cursor to track recent activity) or [`ListTransactions::into_stream`] to walk the
    /// full history.
    ///
    /// **A plain wallet address is not a bridge address.** The live API answers `500`
    /// `{"error":"cannot get transaction status"}` (a server bug; the spec only documents
    /// that body as a generic 500 example) for an address it does not know as a bridge
    /// address, which surfaces as [`Error::Api`](crate::Error::Api) with status `500`. See
    /// `SPEC_DEVIATIONS.md`.
    ///
    /// See <https://docs.polymarket.com/api-reference/bridge/get-transaction-status>.
    ///
    /// ```no_run
    /// # async fn run() -> marcasite::Result<()> {
    /// use futures_util::TryStreamExt as _;
    ///
    /// let bridge = marcasite::bridge::BridgeClient::new()?;
    /// let history: Vec<_> = bridge
    ///     .list_transactions("EXoZue2avJae1d45B3fVw2unhkrtToSYQqHtHgfZ2cbE")
    ///     .limit(100)
    ///     .into_stream()
    ///     .try_collect()
    ///     .await?;
    /// # let _ = history;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_transactions(&self, address: impl Into<String>) -> ListTransactions {
        ListTransactions {
            client: self.clone(),
            address: address.into(),
            limit: None,
            cursor: None,
            paginate: false,
        }
    }
}

/// Request builder for [`BridgeClient::list_transactions`].
#[derive(Debug, Clone)]
#[must_use = "requests do nothing until `.send()` or `.into_stream()` is used"]
pub struct ListTransactions {
    client: BridgeClient,
    address: String,
    limit: Option<u32>,
    cursor: Option<String>,
    paginate: bool,
}

impl ListTransactions {
    /// Maximum number of transfers per page, `1..=100` (server default `50`). A page may
    /// hold fewer transfers than requested and still have a following page.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Continue from the `nextCursor` of a previous page (for the same address). Omit it to
    /// request the first page. An invalid, tampered or cross-address cursor fails with
    /// status `400`; restart without a cursor when that happens.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Sends the compatibility parameter `paginate=true`, forwarded upstream for existing
    /// integrations.
    ///
    /// Pagination applies whether or not it is sent; the docs advise new integrations to
    /// omit it and rely on [`cursor`](Self::cursor) and [`limit`](Self::limit) alone.
    pub fn paginate(mut self) -> Self {
        self.paginate = true;
        self
    }

    fn query(&self, cursor: Option<&str>) -> Result<Query> {
        // The address is a path segment: an empty or dot segment would change the path.
        if matches!(self.address.as_str(), "" | "." | "..") {
            return Err(ValidationError::new(
                "address",
                format!("`{}` is not a bridge address", self.address),
            )
            .into());
        }
        if let Some(limit) = self.limit
            && !(1..=MAX_LIMIT).contains(&limit)
        {
            return Err(ValidationError::new(
                "limit",
                format!("must be between 1 and {MAX_LIMIT}, got {limit}"),
            )
            .into());
        }
        let mut query = Query::new();
        query
            .push_opt("limit", self.limit)
            .push_opt("cursor", cursor)
            .push_opt("paginate", self.paginate.then_some("true"));
        Ok(query)
    }

    async fn fetch(&self, cursor: Option<&str>) -> Result<TransactionStatusPage> {
        let query = self.query(cursor)?;
        self.client
            .transport
            .get(&["status", &self.address])
            .query(query)
            .send()
            .await
    }

    /// Fetches one page.
    ///
    /// # Errors
    ///
    /// - [`Error::Validation`](crate::Error::Validation) if the address is empty (or `.` or
    ///   `..`) or the limit is outside `1..=100` (nothing is sent).
    /// - [`Error::Api`](crate::Error::Api) with status `400` for an invalid address, limit
    ///   or cursor (e.g. a stale cursor), or `500` on a server error (also what the live API returns for an address that is not a
    ///   bridge address).
    /// - [`Error::Decode`](crate::Error::Decode) if the response does not match the
    ///   documented schema, including a missing `transactions` or `nextCursor`.
    /// - Any other [`Error`](crate::Error) for transport or rate limiting failures.
    pub async fn send(self) -> Result<TransactionStatusPage> {
        self.fetch(self.cursor.as_deref()).await
    }

    /// Streams every transfer from the configured cursor (or the newest) onwards, following
    /// `nextCursor` until it is `null` (or empty).
    ///
    /// The stream yields the first error (any error of [`send`](Self::send)) and then
    /// ends.
    pub fn into_stream(self) -> Paginated<Transaction> {
        let start = self.cursor.clone();
        cursor_stream(start, move |cursor| {
            let request = self.clone();
            async move {
                let page = request.fetch(cursor.as_deref()).await?;
                let next = page.next_cursor().map(str::to_owned);
                Ok(CursorPage::new(page.into_items(), next))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `200` example of `GET /status/{address}` in `docs/polymarket/specs/bridge-openapi.yaml`.
    #[test]
    fn deserializes_documented_example() {
        let json = r#"{
            "transactions": [
                {
                    "fromChainId": "1151111081099710",
                    "fromTokenAddress": "11111111111111111111111111111111",
                    "fromAmountBaseUnit": "13566635",
                    "toChainId": "137",
                    "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
                    "status": "DEPOSIT_DETECTED"
                },
                {
                    "fromChainId": "1151111081099710",
                    "fromTokenAddress": "11111111111111111111111111111111",
                    "fromAmountBaseUnit": "13400000",
                    "toChainId": "137",
                    "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
                    "createdTimeMs": 1757646914535,
                    "status": "PROCESSING"
                },
                {
                    "fromChainId": "1151111081099710",
                    "fromTokenAddress": "11111111111111111111111111111111",
                    "fromAmountBaseUnit": "13500152",
                    "toChainId": "137",
                    "toTokenAddress": "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB",
                    "txHash": "3atr19NAiNCYt24RHM1WnzZp47RXskpTDzspJoCBBaMFwUB8fk37hFkxz35P5UEnnmWz21rb2t5wJ8pq3EE2XnxU",
                    "createdTimeMs": 1757531217339,
                    "status": "COMPLETED"
                }
            ],
            "nextCursor": "eyJsYXN0SWQiOiI0MiJ9"
        }"#;
        let page: TransactionStatusPage = serde_json::from_str(json).unwrap();
        assert_eq!(page.next_cursor.as_deref(), Some("eyJsYXN0SWQiOiI0MiJ9"));
        assert_eq!(page.transactions.len(), 3);

        let detected = &page.transactions[0];
        assert_eq!(detected.status, Some(TransactionStatus::DepositDetected));
        assert_eq!(detected.created_time, None);
        assert_eq!(
            detected.from_chain_id,
            Some(ChainId::from("1151111081099710"))
        );
        assert_eq!(detected.from_amount_base_unit.as_deref(), Some("13566635"));

        let completed = &page.transactions[2];
        assert_eq!(completed.status, Some(TransactionStatus::Completed));
        assert_eq!(
            completed.created_time.map(|t| t.timestamp_millis()),
            Some(1_757_531_217_339)
        );
        assert!(completed.tx_hash.as_deref().unwrap().starts_with("3atr19"));

        let value = serde_json::to_value(&page).unwrap();
        assert_eq!(
            value["transactions"][2]["createdTimeMs"],
            1_757_531_217_339_i64
        );
        let again: TransactionStatusPage = serde_json::from_value(value).unwrap();
        assert_eq!(again, page);
    }

    #[test]
    fn null_cursor_ends_the_walk() {
        let page: TransactionStatusPage =
            serde_json::from_str(r#"{"transactions":[],"nextCursor":null}"#).unwrap();
        assert_eq!(page.next_cursor, None);
        assert_eq!(page.next_cursor(), None);
        assert!(page.items().is_empty());
        let page: TransactionStatusPage =
            serde_json::from_str(r#"{"transactions":[],"nextCursor":""}"#).unwrap();
        assert_eq!(page.next_cursor.as_deref(), Some(""));
        assert_eq!(page.next_cursor(), None);
    }

    #[test]
    fn required_fields_must_be_present() {
        // `nextCursor` is required (but nullable): a missing key is not the last page.
        let err =
            serde_json::from_str::<TransactionStatusPage>(r#"{"transactions":[]}"#).unwrap_err();
        assert!(err.to_string().contains("nextCursor"), "{err}");
        let err =
            serde_json::from_str::<TransactionStatusPage>(r#"{"nextCursor":null}"#).unwrap_err();
        assert!(err.to_string().contains("transactions"), "{err}");
    }

    #[test]
    fn page_accessors() {
        let page: TransactionStatusPage = serde_json::from_str(
            r#"{"transactions":[{"status":"FAILED"},{"status":"COMPLETED"}],"nextCursor":"abc"}"#,
        )
        .unwrap();
        assert_eq!(page.next_cursor(), Some("abc"));
        assert_eq!(page.items().len(), 2);
        let items = page.into_items();
        assert_eq!(items[0].status, Some(TransactionStatus::Failed));
    }
}
