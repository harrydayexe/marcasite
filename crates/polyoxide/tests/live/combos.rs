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

/// The spec says the default page size is 50 and the maximum 100; the SDK enforces the
/// maximum client-side. Record what the live API returns without a `limit` and above 100.
#[tokio::test]
#[ignore = "live network"]
async fn list_combo_markets_limits() {
    let err = pm()
        .combos()
        .list_combo_markets()
        .limit(101)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");

    let default = pm().combos().list_combo_markets().send().await.unwrap();
    assert!(!default.items().is_empty());
    eprintln!(
        "NOTE combo-markets without `limit`: {} markets (spec default: 50)",
        default.items().len()
    );
    let over = get(COMBOS, "/v1/rfq/combo-markets", &[("limit", "101")]).await;
    eprintln!(
        "NOTE combo-markets limit=101: {} markets (spec maximum: 100)",
        over.json["markets"].as_array().map_or(0, Vec::len)
    );
}
