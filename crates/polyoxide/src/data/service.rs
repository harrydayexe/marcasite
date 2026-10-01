//! Service: `/v2/status`.

use chrono::{DateTime, Utc};
use polyoxide_core::{Query, Result, serde_util};
use serde::{Deserialize, Serialize};

use super::DataClient;

/// How fresh the data behind the Data API is (`components/schemas/ServiceStatus`).
///
/// Served from a snapshot refreshed in the background: [`computed_at`](Self::computed_at)
/// and [`age_seconds`](Self::age_seconds) say how old it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServiceStatus {
    /// When this snapshot was taken.
    #[serde(with = "serde_util::datetime")]
    pub computed_at: DateTime<Utc>,
    /// How old the snapshot is, in seconds. A value that keeps climbing means the
    /// refresher is not completing.
    pub age_seconds: i64,
    /// Freshness of the projections behind the feeds.
    pub serving: ServingFreshness,
    /// Ingestion health: one cursor per `(contract, event)` stream.
    pub ingestion: IngestionFreshness,
}

/// Freshness of the projections behind the feeds (`components/schemas/ServingFreshness`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServingFreshness {
    /// Every mechanism that produced a candidate freshness row, in a fixed order.
    pub mechanisms: Vec<ServingMechanism>,
    /// The worst age across `mechanisms`, in seconds; `None` only when no mechanism
    /// reported.
    pub lag_seconds: Option<i64>,
    /// Which mechanism `lag_seconds` came from.
    pub worst: Option<ServingMechanismName>,
}

polyoxide_core::string_enum! {
    /// What a serving mechanism produces.
    pub enum ServingMechanismName {
        /// Activity-feed enrichment.
        ActivityFeed => "activity_feed",
        /// Custody-balance ingestion.
        CustodyBalances => "custody_balances",
        /// The PnL engine.
        Pnl => "pnl",
    }
}

/// One serving mechanism's freshness (`components/schemas/ServingMechanism`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServingMechanism {
    /// What this mechanism produces.
    pub name: ServingMechanismName,
    /// Seconds since it last advanced, by its own clock.
    pub age_seconds: i64,
    /// How far behind the ingestion tail it has projected, in blocks; `None` for a
    /// mechanism that records a time but no block.
    pub blocks_behind: Option<i64>,
}

/// Ingestion health over the live streams (`components/schemas/IngestionFreshness`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IngestionFreshness {
    /// How many live streams were found; `0` means no ingestion cursors are visible.
    pub cursors: u64,
    /// The chain id this service is configured for.
    pub chain_id: i64,
    /// The furthest-behind live streams, `most_lagged` first.
    pub lagging: Vec<CursorLag>,
    /// The tail: the furthest-along cursor's block (over all cursors).
    pub max_synced_block: Option<i64>,
    /// The furthest-behind live stream's block.
    pub min_synced_block: Option<i64>,
    /// The single furthest-behind live stream.
    pub most_lagged: Option<CursorLag>,
    /// The chain the ingestion cursors were written for, as the datastore names it;
    /// `None` when no stream declares one.
    pub network: Option<String>,
}

/// One ingestion stream's distance from the furthest-along stream
/// (`components/schemas/CursorLag`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CursorLag {
    /// The stream, as `<contract>_<event>`.
    pub source: String,
    /// Last block the stream has ingested through.
    pub block: i64,
    /// How many blocks this stream trails the furthest-along stream; `0` for the leader.
    pub behind_max: i64,
}

impl DataClient {
    /// Gets how fresh the data behind the Data API is (`GET /v2/status`).
    ///
    /// See <https://docs.polymarket.com/api-reference/service/get-data-freshness>.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error); before the first freshness snapshot exists (or on a
    /// dependency outage) the API answers `503`, an [`Error::Api`](crate::Error::Api) for
    /// which [`Error::is_retryable`](crate::Error::is_retryable) is `true`.
    pub async fn get_status(&self) -> Result<ServiceStatus> {
        self.fetch_data(&["v2", "status"], Query::new()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::Envelope;

    /// Field names and types from `components/schemas/Envelope_ServiceStatus`,
    /// `ServingFreshness`, `ServingMechanism`, `IngestionFreshness` and `CursorLag`.
    #[test]
    fn deserializes_status() {
        let json = r#"{"data":{
            "computed_at": "2026-10-01T12:00:00Z",
            "age_seconds": 4,
            "serving": {
                "lag_seconds": 12,
                "worst": "pnl",
                "mechanisms": [
                    {"name": "activity_feed", "age_seconds": 3, "blocks_behind": 1},
                    {"name": "pnl", "age_seconds": 12, "blocks_behind": null}
                ]
            },
            "ingestion": {
                "cursors": 2,
                "chain_id": 137,
                "lagging": [{"source": "ctf_TransferSingle", "block": 100, "behind_max": 5}],
                "max_synced_block": 105,
                "min_synced_block": 100,
                "most_lagged": {"source": "ctf_TransferSingle", "block": 100, "behind_max": 5},
                "network": "polygon"
            }
        }}"#;
        let status = serde_json::from_str::<Envelope<ServiceStatus>>(json)
            .unwrap()
            .data;
        assert_eq!(status.serving.worst, Some(ServingMechanismName::Pnl));
        assert_eq!(status.ingestion.cursors, 2);
        assert_eq!(
            status.ingestion.most_lagged.as_ref().map(|c| c.behind_max),
            Some(5)
        );
        assert_eq!(status.computed_at.timestamp(), 1_790_856_000);
    }
}
