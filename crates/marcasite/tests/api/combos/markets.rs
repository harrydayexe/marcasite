//! `GET /v1/rfq/combo-markets`.

use futures_util::{StreamExt as _, TryStreamExt as _};
use marcasite::{
    Decimal, Error,
    combos::{ComboMarketId, PositionId},
    types::ConditionId,
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use crate::common;

/// Captured from `GET https://combos-rfq-api.polymarket.com/v1/rfq/combo-markets` on
/// 2026-10-02 (trimmed to one market and three tags). The docs' example in
/// `docs/polymarket/specs/combos-rfq-openapi.yaml` has abbreviated ids and no `pending`; live has full
/// ids and the undocumented `pending`.
const EXAMPLE: &str = r#"{
    "markets": [
        {
            "id": "665374",
            "condition_id": "0x5db999fad322cea2914535aae5517060c3f80ad6d8c0231cde2124a434d16846",
            "position_ids": ["798559951534518479645224261511384773234863312866932338530531601041078616064", "798559951534518479645224261511384773234863312866932338530531601041078616065"],
            "pending": false,
            "slug": "will-the-us-invade-iran-before-2027",
            "title": "Will the U.S. invade Iran before 2027?",
            "outcomes": ["Yes", "No"],
            "outcome_prices": ["0.145", "0.855"],
            "image": "https://polymarket-upload.s3.us-east-2.amazonaws.com/will-the-us-invade-iran-in-2025-0Eh3J0ku_Fbj.jpg",
            "volume": 70868404.87693602,
            "tags": ["politics", "iran", "trump"]
        }
    ],
    "next_cursor": "MTQwNzI0"
}"#;

const CONDITION_ID: &str = "0x5db999fad322cea2914535aae5517060c3f80ad6d8c0231cde2124a434d16846";
const YES_POSITION: &str =
    "798559951534518479645224261511384773234863312866932338530531601041078616064";

/// A minimal market with every required field, for paging tests.
fn market(id: &str) -> String {
    format!(
        r#"{{"id":"{id}","condition_id":"0x{id}","position_ids":["1","2"],"slug":"s{id}","title":"t","outcomes":["Yes","No"],"outcome_prices":["0.5","0.5"],"image":"","volume":1,"tags":[]}}"#
    )
}

#[tokio::test]
async fn list_sends_query_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .and(query_param("limit", "10"))
        .and(query_param("exclude", format!("{CONDITION_ID},0x0391ab0e")))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .limit(10)
        .exclude([CONDITION_ID, "0x0391ab0e"])
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor.as_deref(), Some("MTQwNzI0"));
    let market = &page.markets[0];
    assert_eq!(market.id, ComboMarketId::from("665374"));
    assert_eq!(market.condition_id, ConditionId::from(CONDITION_ID));
    assert_eq!(market.pending, Some(false));
    assert_eq!(
        market.yes_position_id(),
        Some(&PositionId::from(YES_POSITION))
    );
    assert_eq!(market.no_price(), Some(Decimal::new(855, 3)));
    assert_eq!(
        market.volume,
        "70868404.87693602".parse::<Decimal>().unwrap()
    );
}

#[tokio::test]
async fn empty_exclude_list_is_not_sent() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .and(query_param_is_missing("exclude"))
        .and(query_param_is_missing("limit"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"{"markets":[],"next_cursor":null}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .exclude(Vec::<ConditionId>::new())
        .send()
        .await
        .unwrap();
    assert!(page.markets.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn stream_walks_cursor_until_null() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .and(query_param_is_missing("cursor"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"{{"markets":[{},{}],"next_cursor":"Mg"}}"#,
                market("1"),
                market("2")
            ),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .and(query_param("cursor", "Mg"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(r#"{{"markets":[{}],"next_cursor":null}}"#, market("3")),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .limit(2)
        .into_stream()
        .map_ok(|m| m.id)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        ids,
        [
            ComboMarketId::from("1"),
            ComboMarketId::from("2"),
            ComboMarketId::from("3")
        ]
    );
}

#[tokio::test]
async fn stream_can_resume_from_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .and(query_param("cursor", "Mg"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(r#"{{"markets":[{}],"next_cursor":null}}"#, market("3")),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let markets: Vec<_> = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .cursor("Mg")
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
}

#[tokio::test]
async fn bad_request_is_typed() {
    let server = common::server().await;
    // The shared `BadRequest` response of `docs/polymarket/specs/combos-rfq-openapi.yaml`.
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"error":"invalid quote"}"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .cursor("garbage")
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 400);
    assert_eq!(api.message(), Some("invalid quote"));
}

#[tokio::test]
async fn limit_is_validated_before_sending() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let combos = common::polymarket(&server).combos().clone();
    for limit in [0, 10_001] {
        let err = combos
            .list_combo_markets()
            .limit(limit)
            .send()
            .await
            .unwrap_err();
        assert!(
            matches!(&err, Error::Validation(v) if v.parameter() == "limit"),
            "{err:?}"
        );
    }
    let results: Vec<_> = combos
        .list_combo_markets()
        .limit(10_001)
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(Error::Validation(_))));
}

#[tokio::test]
async fn missing_required_field_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"markets":[{"id":"1","condition_id":"0x1"}],"next_cursor":null}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "markets[0]");
}

#[tokio::test]
async fn missing_next_cursor_is_a_decode_error_not_the_end() {
    let server = common::server().await;
    // `next_cursor` is required (but nullable): a page without it must not end the walk
    // silently.
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(r#"{{"markets":[{}]}}"#, market("1")),
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let results: Vec<_> = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let Err(Error::Decode(decode)) = &results[0] else {
        panic!("expected a decode error, got {results:?}");
    };
    assert!(decode.to_string().contains("next_cursor"), "{decode}");
}

#[tokio::test]
async fn page_accessors_normalize_the_cursor() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor(), Some("MTQwNzI0"));
    assert_eq!(page.items().len(), 1);
    assert_eq!(page.into_items()[0].id, ComboMarketId::from("665374"));
}

#[tokio::test]
async fn an_invalid_price_fails_the_whole_page() {
    let server = common::server().await;
    // `outcome_prices` items are strings in the spec; one that is not a decimal number is
    // reported with its path rather than silently dropped.
    let bad = market("2").replace(r#"["0.5","0.5"]"#, r#"["0.5",""]"#);
    Mock::given(method("GET"))
        .and(path("/v1/rfq/combo-markets"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"{{"markets":[{},{bad}],"next_cursor":null}}"#,
                market("1")
            ),
            "application/json",
        ))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "markets[1].outcome_prices[1]");
}
