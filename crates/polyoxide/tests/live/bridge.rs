//! Bridge API live tests (`GET /supported-assets`, `POST /quote`, `GET /status/{address}`).
//!
//! `POST /deposit` and `POST /withdraw` are deliberately **not** called: they create bridge
//! addresses, so they are not read-only.

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{
    Error,
    bridge::{Quote, QuoteRequest, SupportedAssets, TransactionStatusPage},
};
use serde_json::json;

use crate::common::{BRIDGE, check, get, pm, post, sample};

/// An address that appears as a bridge deposit address in
/// `docs/api-reference/bridge/create-bridge-addresses.md`; it has a history of transfers.
const DOCUMENTED_BRIDGE_ADDRESS: &str = "0x23566f8b2E82aDfCf01846E54899d110e97AC053";

/// Polygon USDC and pUSD, as in the `POST /quote` example of `docs/specs/bridge-openapi.yaml`.
const POLYGON_USDC: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
const POLYGON_PUSD: &str = "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB";

#[tokio::test]
#[ignore = "live network"]
async fn get_supported_assets() {
    let assets = pm().bridge().get_supported_assets().await.unwrap();
    assert!(!assets.assets().is_empty());
    for asset in assets.assets() {
        assert!(
            asset.chain_id.is_some() && asset.token.is_some(),
            "{asset:?}"
        );
    }
    check::<SupportedAssets>(
        "GET /supported-assets",
        &get(BRIDGE, "/supported-assets", &[]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn get_quote() {
    let s = sample().await;
    let request = QuoteRequest::new()
        .from_amount_base_unit("10000000")
        .from_chain_id("137")
        .from_token_address(POLYGON_USDC)
        .recipient_address(s.user.as_str())
        .to_chain_id("137")
        .to_token_address(POLYGON_PUSD);
    let quote = pm().bridge().get_quote(request).await.unwrap();
    assert!(quote.quote_id.is_some() && quote.est_fee_breakdown.is_some());
    assert!(
        quote
            .est_to_token_base_unit
            .as_deref()
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())),
        "{quote:?}"
    );
    let raw = post(
        BRIDGE,
        "/quote",
        &json!({
            "fromAmountBaseUnit": "10000000",
            "fromChainId": "137",
            "fromTokenAddress": POLYGON_USDC,
            "recipientAddress": s.user,
            "toChainId": "137",
            "toTokenAddress": POLYGON_PUSD,
        }),
    )
    .await;
    // Open question 23: for a 10 USDC input, which of the two USD fields is the amount sent?
    eprintln!(
        "NOTE quote: estInputUsd={} estOutputUsd={} swapImpact={} swapImpactUsd={}",
        raw.json["estInputUsd"],
        raw.json["estOutputUsd"],
        raw.json["estFeeBreakdown"]["swapImpact"],
        raw.json["estFeeBreakdown"]["swapImpactUsd"],
    );
    check::<Quote>("POST /quote", &raw);
}

/// A missing field is rejected client-side, naming the wire field.
#[tokio::test]
#[ignore = "live network"]
async fn get_quote_incomplete_request_is_rejected() {
    let err = pm()
        .bridge()
        .get_quote(QuoteRequest::new().from_chain_id("137"))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
}

/// The spec documents `/status/{address}` for a *bridge* address (from `/deposit` or
/// `/withdraw`). A plain wallet address, such as the sample user's, is what a caller might
/// try first: record what the live API does with it. It must end in a typed outcome.
#[tokio::test]
#[ignore = "live network"]
async fn list_transactions_for_user_wallet() {
    let s = sample().await;
    match pm()
        .bridge()
        .list_transactions(s.user.as_str())
        .send()
        .await
    {
        Ok(page) => eprintln!(
            "NOTE GET /status/<user wallet>: {} transactions",
            page.items().len()
        ),
        Err(Error::Api(api)) => eprintln!(
            "NOTE GET /status/<user wallet>: HTTP {} {:?} (spec 500 example: `cannot get transaction status`)",
            api.status(),
            api.message()
        ),
        Err(other) => panic!("GET /status/<user wallet>: unexpected error {other:?}"),
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn list_transactions_for_bridge_address() {
    let page = pm()
        .bridge()
        .list_transactions(DOCUMENTED_BRIDGE_ADDRESS)
        .limit(3)
        .send()
        .await
        .unwrap();
    assert!(!page.items().is_empty() && page.items().len() <= 3);
    let path = format!("/status/{DOCUMENTED_BRIDGE_ADDRESS}");
    check::<TransactionStatusPage>(
        "GET /status/{address}?limit=3",
        &get(BRIDGE, &path, &[("limit", "3")]).await,
    );
    check::<TransactionStatusPage>(
        "GET /status/{address}?limit=100",
        &get(BRIDGE, &path, &[("limit", "100")]).await,
    );
    // The legacy `paginate=true` parameter is still accepted.
    check::<TransactionStatusPage>(
        "GET /status/{address}?limit=3&paginate=true",
        &get(BRIDGE, &path, &[("limit", "3"), ("paginate", "true")]).await,
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn list_transactions_stream_pages() {
    let transactions: Vec<_> = pm()
        .bridge()
        .list_transactions(DOCUMENTED_BRIDGE_ADDRESS)
        .limit(3)
        .into_stream()
        .take(5)
        .try_collect()
        .await
        .unwrap();
    assert!(!transactions.is_empty() && transactions.len() <= 5);
}
