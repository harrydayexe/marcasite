//! Combos / RFQ REST live tests (`GET /v1/rfq/combo-markets`).

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{Error, combos::ComboMarketsPage};

use crate::common::{COMBOS, check, get, pm};

#[tokio::test]
#[ignore = "live network"]
async fn list_combo_markets() {
    let page = pm()
        .combos()
        .list_combo_markets()
        .limit(5)
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty() && page.items().len() <= 5);
    for market in page.items() {
        // `position_ids`, `outcomes` and `outcome_prices` correspond by index.
        assert_eq!(
            market.position_ids.len(),
            market.outcomes.len(),
            "{market:?}"
        );
        assert_eq!(
            market.outcome_prices.len(),
            market.outcomes.len(),
            "{market:?}"
        );
        assert!(market.yes_position_id().is_some() && market.no_price().is_some());
        // SPEC_DEVIATIONS.md, Combos: the undocumented `pending` is on every market.
        assert!(market.pending.is_some(), "{market:?}");
    }
    assert!(page.next_cursor().is_some());
    check::<ComboMarketsPage>(
        "GET /v1/rfq/combo-markets?limit=5",
        &get(COMBOS, "/v1/rfq/combo-markets", &[("limit", "5")]).await,
    );
    check::<ComboMarketsPage>(
        "GET /v1/rfq/combo-markets?limit=100",
        &get(COMBOS, "/v1/rfq/combo-markets", &[("limit", "100")]).await,
    );
}

/// Several pages of the stream, walked by cursor.
#[tokio::test]
#[ignore = "live network"]
async fn list_combo_markets_stream_pages() {
    let markets: Vec<_> = pm()
        .combos()
        .list_combo_markets()
        .limit(20)
        .into_stream()
        .take(70)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(markets.len(), 70);
}

/// `exclude` omits the listed condition ids.
#[tokio::test]
#[ignore = "live network"]
async fn list_combo_markets_exclude() {
    let first = pm()
        .combos()
        .list_combo_markets()
        .limit(3)
        .send()
        .await
        .unwrap();
    let excluded: Vec<_> = first
        .items()
        .iter()
        .map(|m| m.condition_id.clone())
        .collect();
    let page = pm()
        .combos()
        .list_combo_markets()
        .limit(3)
        .exclude(excluded.clone())
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty());
    for market in page.items() {
        assert!(!excluded.contains(&market.condition_id), "{market:?}");
    }
}

/// SPEC_DEVIATIONS.md, Combos: the spec says the default page size is 50 and the maximum
/// 100; live returns 1000 without a `limit`, accepts `limit` above 100, and rejects 0. The
/// SDK allows `1..=1000`.
#[tokio::test]
#[ignore = "live network"]
async fn list_combo_markets_limits() {
    let err = pm()
        .combos()
        .list_combo_markets()
        .limit(1001)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    let err = pm()
        .combos()
        .list_combo_markets()
        .limit(0)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");

    // No `limit`: far more than the documented default of 50 (and maximum of 100).
    let default = pm().combos().list_combo_markets().send().await.unwrap();
    assert!(
        default.items().len() > 100,
        "without `limit` live returned {} markets (it used to be 1000, spec default 50)",
        default.items().len()
    );
    let raw = get(COMBOS, "/v1/rfq/combo-markets", &[]).await;
    check::<ComboMarketsPage>("GET /v1/rfq/combo-markets (no limit)", &raw);

    // `limit=1000` is accepted by live and by the SDK.
    let big = pm()
        .combos()
        .list_combo_markets()
        .limit(1000)
        .send()
        .await
        .unwrap();
    assert!(big.items().len() > 100, "{}", big.items().len());
    // Above the spec maximum (101) live still answers 200.
    let over = get(COMBOS, "/v1/rfq/combo-markets", &[("limit", "101")]).await;
    assert!(over.json["markets"].as_array().unwrap().len() > 100);

    // `limit=0` is a 400 (the spec says `minimum: 1`, so this one matches).
    let response = reqwest::Client::new()
        .get(format!("{COMBOS}/v1/rfq/combo-markets"))
        .query(&[("limit", "0")])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}
