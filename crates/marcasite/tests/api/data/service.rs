//! Service endpoint: `/v2/status`.

use marcasite::{Error, data::ServingMechanismName};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use super::fixtures;
use crate::common;

#[tokio::test]
async fn get_status_decodes_freshness() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/status"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixtures::envelope(json!({
                "computed_at": "2026-10-01T12:00:00Z",
                "age_seconds": 4,
                "serving": {
                    "lag_seconds": 30,
                    "worst": "custody_balances",
                    "mechanisms": [
                        {"name": "custody_balances", "age_seconds": 30, "blocks_behind": 12},
                        {"name": "pnl", "age_seconds": 5}
                    ]
                },
                "ingestion": {
                    "cursors": 0,
                    "chain_id": 137,
                    "lagging": [],
                    "max_synced_block": null,
                    "min_synced_block": null,
                    "most_lagged": null,
                    "network": null
                }
            }))),
        )
        .expect(1)
        .mount(&server)
        .await;

    let status = common::polymarket(&server)
        .data()
        .get_status()
        .await
        .unwrap();
    assert_eq!(status.age_seconds, 4);
    assert_eq!(status.serving.lag_seconds, Some(30));
    assert_eq!(
        status.serving.worst,
        Some(ServingMechanismName::CustodyBalances)
    );
    assert_eq!(status.serving.mechanisms[1].blocks_behind, None);
    assert_eq!(status.ingestion.most_lagged, None);
    assert_eq!(status.ingestion.chain_id, 137);
}

#[tokio::test]
async fn get_status_503_before_first_snapshot_is_retryable() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v2/status"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": "freshness not measured yet",
            "code": "dependency_unavailable",
            "retryable": true,
            "trace_id": "t-1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .data()
        .get_status()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Api(_)), "{err:?}");
    assert!(err.is_retryable());
    assert_eq!(
        marcasite::data::ErrorCode::from_error(&err),
        Some(marcasite::data::ErrorCode::DependencyUnavailable)
    );
}
