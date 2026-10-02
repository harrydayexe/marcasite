//! Bridge API live tests (`GET /supported-assets`, `POST /quote`, `GET /status/{address}`).
//!
//! `POST /deposit` and `POST /withdraw` are deliberately **not** called: they create bridge
//! addresses, so they are not read-only.

use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{
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
    let raw = get(BRIDGE, "/supported-assets", &[]).await;
    // SPEC_DEVIATIONS.md, Bridge: the undocumented top-level `note`.
    assert!(
        raw.json["note"].is_string(),
        "GET /supported-assets no longer has a top-level `note`: update SPEC_DEVIATIONS.md"
    );
    assert!(assets.note.is_some());
    check::<SupportedAssets>("GET /supported-assets", &raw);
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
    // SPEC_DEVIATIONS.md, Bridge: the spec's `estInputUsd` / `estOutputUsd` descriptions are
    // swapped. For a 10 USDC input, `estInputUsd` is about the amount sent (10) and
    // `estOutputUsd` the (slightly lower) amount received.
    let input = raw.json["estInputUsd"].as_f64().expect("estInputUsd");
    let output = raw.json["estOutputUsd"].as_f64().expect("estOutputUsd");
    assert!(
        (input - 10.0).abs() < 0.5 && output <= input + 1e-9,
        "estInputUsd={input} estOutputUsd={output}: the live meaning changed, update SPEC_DEVIATIONS.md"
    );
    assert_eq!(
        quote
            .est_input_usd
            .unwrap()
            .to_string()
            .parse::<f64>()
            .unwrap(),
        input
    );
    // Percent fields use 1 = 1%: `swapImpactUsd` is `swapImpact`% of the amount sent.
    let impact = raw.json["estFeeBreakdown"]["swapImpact"]
        .as_f64()
        .unwrap_or(0.0);
    let impact_usd = raw.json["estFeeBreakdown"]["swapImpactUsd"]
        .as_f64()
        .unwrap_or(0.0);
    assert!(
        (impact * input / 100.0 - impact_usd).abs() < 0.001,
        "swapImpact={impact} swapImpactUsd={impact_usd}: the percent scale changed, update SPEC_DEVIATIONS.md"
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
/// try first. SPEC_DEVIATIONS.md, Bridge (server bug): the live API answers HTTP 500
/// `cannot get transaction status`; the SDK surfaces it as a typed API error.
#[tokio::test]
#[ignore = "live network"]
async fn list_transactions_for_user_wallet_is_a_500() {
    let s = sample().await;
    let err = pm()
        .bridge()
        .list_transactions(s.user.as_str())
        .send()
        .await
        .expect_err("a non-bridge address used to be a 500; update SPEC_DEVIATIONS.md");
    let Error::Api(api) = &err else {
        panic!("expected a typed API error, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 500, "{err:?}");
    assert_eq!(api.message(), Some("cannot get transaction status"));
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
    // `createdTimeMs` is always an integer (the model decodes it as
    // integer milliseconds, so a fractional value would fail the checks below).
    let all = get(BRIDGE, &path, &[("limit", "100")]).await;
    for transaction in all.json["transactions"].as_array().unwrap() {
        if let Some(created) = transaction.get("createdTimeMs") {
            assert!(
                created.is_u64(),
                "createdTimeMs is not an integer: {created}"
            );
        }
    }
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
