//! `GET /v1/rfq/combo-markets`.

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{
    Decimal, Error,
    combos::{ComboMarketId, PositionId},
    types::ConditionId,
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use crate::common;

/// `200` example of `GET /v1/rfq/combo-markets` in `docs/specs/combos-rfq-openapi.yaml`.
const EXAMPLE: &str = r#"{
    "markets": [
        {
            "id": "1897034",
            "condition_id": "0x4cd7...110ff",
            "position_ids": ["1012585...362880", "1012585...362881"],
            "slug": "fifwc-mex-rsa-2026-06-11-mex",
            "title": "Will Mexico win on 2026-06-11?",
            "outcomes": ["Yes", "No"],
            "outcome_prices": ["0.685", "0.315"],
            "image": "https://...",
            "volume": 330327.7128580074,
            "tags": ["sports", "soccer", "games", "world-cup"]
        }
    ],
    "next_cursor": "Mg"
}"#;

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
        .and(query_param("exclude", "0x4cd7...110ff,0x0391ab0e..."))
        .and(query_param_is_missing("cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(EXAMPLE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let page = common::polymarket(&server)
        .combos()
        .list_combo_markets()
        .limit(10)
        .exclude(["0x4cd7...110ff", "0x0391ab0e..."])
        .send()
        .await
        .unwrap();
    assert_eq!(page.next_cursor.as_deref(), Some("Mg"));
    let market = &page.markets[0];
    assert_eq!(market.id, ComboMarketId::from("1897034"));
    assert_eq!(market.condition_id, ConditionId::from("0x4cd7...110ff"));
    assert_eq!(
        market.yes_position_id(),
        Some(&PositionId::from("1012585...362880"))
    );
    assert_eq!(market.no_price(), Some(Decimal::new(315, 3)));
    assert_eq!(
        market.volume,
        "330327.7128580074".parse::<Decimal>().unwrap()
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
    // The shared `BadRequest` response of `docs/specs/combos-rfq-openapi.yaml`.
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
    for limit in [0, 101] {
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
        .limit(500)
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
