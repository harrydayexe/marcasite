//! Geoblock live test (`GET https://polymarket.com/api/geoblock`).

use marcasite::geoblock::GeoblockStatus;

use crate::common::{GEOBLOCK, check, get, pm};

#[tokio::test]
#[ignore = "live network"]
async fn get_status() {
    let status = pm().geoblock().get_status().await.unwrap();
    assert_eq!(status.country.len(), 2, "{status:?}");
    let raw = get(GEOBLOCK, "/api/geoblock", &[]).await;
    check::<GeoblockStatus>("GET /api/geoblock", &raw);
}
