//! `/markets` endpoints.

use futures_util::TryStreamExt as _;
use marcasite::{
    Decimal, Error,
    chrono::{TimeZone as _, Utc},
    gamma::{MarketId, QuestionId, TagId},
    types::{ConditionId, TokenId},
};
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{body_json, method, path, query_param},
};

use super::{
    fixture, internal_error, json, json_value, no_retry_gamma, pairs, query_of, requests,
    service_unavailable, validation_error,
};
use crate::common;

#[tokio::test]
async fn list_markets_sends_every_filter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(json_value(&json!([fixture("Market")])))
        .expect(1)
        .mount(&server)
        .await;

    let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let markets = common::polymarket(&server)
        .gamma()
        .list_markets()
        .limit(2)
        .offset(4)
        .order("volume_num")
        .ascending(false)
        .ids(["1", "2"])
        .slugs(["a-b"])
        .clob_token_ids([TokenId::from("71321")])
        .condition_ids([ConditionId::from("0xabc")])
        .liquidity_num_min(10)
        .liquidity_num_max(Decimal::new(205, 1))
        .volume_num_min(1)
        .volume_num_max(2)
        .start_date_min(at)
        .start_date_max(at)
        .end_date_min(at)
        .end_date_max(at)
        .tag_id("100381")
        .related_tags(true)
        .cyom(false)
        .uma_resolution_status("resolved")
        .game_id("g1")
        .sports_market_types(["t1", "t2"])
        .rewards_min_size(Decimal::new(5, 1))
        .question_ids([QuestionId::from("0xq")])
        .include_tag(true)
        .closed(true)
        .send()
        .await
        .unwrap();

    let ts = "2024-01-02T03:04:05Z";
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "2"),
            ("offset", "4"),
            ("order", "volume_num"),
            ("ascending", "false"),
            ("id", "1"),
            ("id", "2"),
            ("slug", "a-b"),
            ("clob_token_ids", "71321"),
            ("condition_ids", "0xabc"),
            ("liquidity_num_min", "10"),
            ("liquidity_num_max", "20.5"),
            ("volume_num_min", "1"),
            ("volume_num_max", "2"),
            ("start_date_min", ts),
            ("start_date_max", ts),
            ("end_date_min", ts),
            ("end_date_max", ts),
            ("tag_id", "100381"),
            ("related_tags", "true"),
            ("cyom", "false"),
            ("uma_resolution_status", "resolved"),
            ("game_id", "g1"),
            ("sports_market_types", "t1"),
            ("sports_market_types", "t2"),
            ("rewards_min_size", "0.5"),
            ("question_ids", "0xq"),
            ("include_tag", "true"),
            ("closed", "true"),
        ])
    );
    assert_eq!(markets.len(), 1);
    let market = &markets[0];
    assert_eq!(market.id, Some(MarketId::from("id-value")));
    assert_eq!(market.liquidity, Some(Decimal::new(15, 1)));
    assert_eq!(market.volume_num, Some(Decimal::new(15, 1)));
    assert_eq!(
        market.end_date,
        Some(Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap())
    );
}

#[tokio::test]
async fn list_markets_sends_no_query_by_default() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let markets = common::polymarket(&server)
        .gamma()
        .list_markets()
        .send()
        .await
        .unwrap();
    assert!(markets.is_empty());
    assert_eq!(query_of(&server, 0).await, pairs(&[]));
}

#[tokio::test]
async fn list_markets_stream_walks_offsets() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("offset", "0"))
        .respond_with(json(r#"[{"id":"1"},{"id":"2"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("offset", "2"))
        .respond_with(json(r#"[{"id":"3"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    // A short page does not end the stream (the server may cap `limit`); an empty one does.
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("offset", "3"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_markets()
        .limit(2)
        .into_stream()
        .map_ok(|m| m.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        ids,
        vec![
            MarketId::from("1"),
            MarketId::from("2"),
            MarketId::from("3")
        ]
    );
}

#[tokio::test]
async fn get_market_by_id_and_slug() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/239826"))
        .and(query_param("include_tag", "true"))
        .respond_with(json_value(&fixture("Market")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets/slug/will-it%20rain"))
        .respond_with(json(r#"{"id":"5","slug":"will-it rain"}"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let market = gamma
        .get_market("239826")
        .include_tag(true)
        .send()
        .await
        .unwrap();
    assert_eq!(market.question.as_deref(), Some("question-value"));
    assert_eq!(market.tags.unwrap().len(), 1);

    let by_slug = gamma
        .get_market_by_slug("will-it rain")
        .send()
        .await
        .unwrap();
    assert_eq!(by_slug.id, Some(MarketId::from("5")));
    assert_eq!(query_of(&server, 1).await, pairs(&[]));
}

#[tokio::test]
async fn get_market_reports_404() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/1"))
        .respond_with(ResponseTemplate::new(404).set_body_raw("Not found", "text/plain"))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .get_market("1")
        .send()
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "{err}");
}

#[tokio::test]
async fn get_market_tags_and_description() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/12/tags"))
        .respond_with(json_value(&json!([fixture("Tag")])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets/12/description"))
        .respond_with(json_value(&fixture("MarketDescription")))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let tags = gamma.get_market_tags("12").await.unwrap();
    assert_eq!(tags[0].label.as_deref(), Some("label-value"));
    let description = gamma.get_market_description("12").await.unwrap();
    assert_eq!(
        description.description.as_deref(),
        Some("description-value")
    );
}

#[tokio::test]
async fn list_markets_keyset_sends_filters_and_decodes_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(json_value(&json!({
            "markets": [fixture("Market")],
            "next_cursor": "next-1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let page = common::polymarket(&server)
        .gamma()
        .list_markets_keyset()
        .limit(100)
        .order("volume_num,liquidity_num")
        .ascending(true)
        .cursor("c0")
        .ids(["1"])
        .slugs(["s"])
        .closed(false)
        .decimalized(true)
        .clob_token_ids(["7"])
        .condition_ids(["0xc"])
        .question_ids(["0xq"])
        .liquidity_num_min(1)
        .liquidity_num_max(2)
        .volume_num_min(3)
        .volume_num_max(4)
        .start_date_min(at)
        .start_date_max(at)
        .end_date_min(at)
        .end_date_max(at)
        .tag_ids(["1", "2"])
        .related_tags(false)
        .tag_match("any")
        .cyom(true)
        .rfq_enabled(true)
        .uma_resolution_status("proposed")
        .game_id("g")
        .sports_market_types(["t"])
        .include_tag(true)
        .locale("en")
        .send()
        .await
        .unwrap();

    let ts = "2024-01-02T03:04:05Z";
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "100"),
            ("order", "volume_num,liquidity_num"),
            ("ascending", "true"),
            ("after_cursor", "c0"),
            ("id", "1"),
            ("slug", "s"),
            ("closed", "false"),
            ("decimalized", "true"),
            ("clob_token_ids", "7"),
            ("condition_ids", "0xc"),
            ("question_ids", "0xq"),
            ("liquidity_num_min", "1"),
            ("liquidity_num_max", "2"),
            ("volume_num_min", "3"),
            ("volume_num_max", "4"),
            ("start_date_min", ts),
            ("start_date_max", ts),
            ("end_date_min", ts),
            ("end_date_max", ts),
            ("tag_id", "1"),
            ("tag_id", "2"),
            ("related_tags", "false"),
            ("tag_match", "any"),
            ("cyom", "true"),
            ("rfq_enabled", "true"),
            ("uma_resolution_status", "proposed"),
            ("game_id", "g"),
            ("sports_market_types", "t"),
            ("include_tag", "true"),
            ("locale", "en"),
        ])
    );
    assert_eq!(page.items().len(), 1);
    assert_eq!(page.next_cursor(), Some("next-1"));
    assert_eq!(page.markets.unwrap().len(), 1);
}

#[tokio::test]
async fn list_markets_keyset_stream_follows_cursors() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("after_cursor", "c1"))
        .respond_with(json(r#"{"markets":[{"id":"3"}]}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(json(
            r#"{"markets":[{"id":"1"},{"id":"2"}],"next_cursor":"c1"}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let ids: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_markets_keyset()
        .limit(2)
        .into_stream()
        .map_ok(|m| m.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        ids,
        vec![
            MarketId::from("1"),
            MarketId::from("2"),
            MarketId::from("3")
        ]
    );
    assert_eq!(query_of(&server, 0).await, pairs(&[("limit", "2")]));
    assert_eq!(
        query_of(&server, 1).await,
        pairs(&[("limit", "2"), ("after_cursor", "c1")])
    );
}

#[tokio::test]
async fn list_markets_keyset_validates_limit_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();
    for limit in [0, 101] {
        let err = gamma
            .list_markets_keyset()
            .limit(limit)
            .send()
            .await
            .unwrap_err();
        let Error::Validation(v) = &err else {
            panic!("expected Error::Validation, got {err:?}")
        };
        assert_eq!(v.parameter(), "limit");
        let streamed: Result<Vec<_>, _> = gamma
            .list_markets_keyset()
            .limit(limit)
            .into_stream()
            .try_collect()
            .await;
        assert!(matches!(streamed, Err(Error::Validation(_))));
    }
    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn list_markets_keyset_surfaces_gamma_validation_errors() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(validation_error())
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .list_markets_keyset()
        .cursor("bogus")
        .send()
        .await
        .unwrap_err();
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.status().as_u16(), 422);
    assert_eq!(api.error_type(), Some("validation error"));
    assert_eq!(
        api.message(),
        Some("offset is not allowed on keyset endpoints")
    );
}

#[tokio::test]
async fn markets_information_posts_json_body() {
    let server = common::server().await;
    let expected_body = json!({
        "id": [1, 2],
        "slug": ["a"],
        "closed": false,
        "clobTokenIds": ["71321"],
        "conditionIds": ["0xc"],
        "liquidityNumMin": 10,
        "liquidityNumMax": 20.5,
        "volumeNumMin": 1,
        "volumeNumMax": 2,
        "startDateMin": "2024-01-02T03:04:05Z",
        "startDateMax": "2024-01-02T03:04:05Z",
        "endDateMin": "2024-01-02T03:04:05Z",
        "endDateMax": "2024-01-02T03:04:05Z",
        "relatedTags": true,
        "tagId": 100381,
        "cyom": false,
        "umaResolutionStatus": "resolved",
        "gameId": "g",
        "sportsMarketTypes": ["t"],
        "rewardsMinSize": 0.5,
        "questionIds": ["0xq"],
        "includeTags": true
    });
    Mock::given(method("POST"))
        .and(path("/markets/information"))
        .and(body_json(&expected_body))
        .respond_with(json_value(&json!([fixture("Market")])))
        .expect(1)
        .mount(&server)
        .await;

    let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
    let markets = common::polymarket(&server)
        .gamma()
        .get_markets_information()
        .ids(["1", "2"])
        .slugs(["a"])
        .closed(false)
        .clob_token_ids(["71321"])
        .condition_ids(["0xc"])
        .liquidity_num_min(10)
        .liquidity_num_max(Decimal::new(205, 1))
        .volume_num_min(1)
        .volume_num_max(2)
        .start_date_min(at)
        .start_date_max(at)
        .end_date_min(at)
        .end_date_max(at)
        .related_tags(true)
        .tag_id(TagId::from("100381"))
        .cyom(false)
        .uma_resolution_status("resolved")
        .game_id("g")
        .sports_market_types(["t"])
        .rewards_min_size(Decimal::new(5, 1))
        .question_ids(["0xq"])
        .include_tags(true)
        .send()
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    let requests = requests(&server).await;
    assert_eq!(
        requests[0]
            .headers
            .get("content-type")
            .map(|v| v.to_str().unwrap()),
        Some("application/json")
    );
}

#[tokio::test]
async fn abridged_markets_posts_empty_body_and_reports_422() {
    let server = common::server().await;
    Mock::given(method("POST"))
        .and(path("/markets/abridged"))
        .and(body_json(json!({})))
        .respond_with(json(r#"[{"id":"1"}]"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let markets = gamma.get_abridged_markets().send().await.unwrap();
    assert_eq!(markets[0].id, Some(MarketId::from("1")));

    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/markets/abridged"))
        .respond_with(ResponseTemplate::new(422).set_body_raw(
            r#"{"type":"validation error","error":"bad filter"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let err = gamma
        .get_abridged_markets()
        .closed(true)
        .send()
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(422));
    assert_eq!(err.api_error().unwrap().message(), Some("bad filter"));
}

#[tokio::test]
async fn markets_information_rejects_non_integer_ids_before_sending() {
    let server = common::server().await;
    let err = common::polymarket(&server)
        .gamma()
        .get_markets_information()
        .ids(["not-a-number"])
        .send()
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "id");
    assert!(requests(&server).await.is_empty());
}

#[tokio::test]
async fn malformed_market_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/1"))
        .respond_with(json(r#"{"id":"1","volumeNum":"lots"}"#))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .get_market("1")
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "volumeNum");
}

#[tokio::test]
async fn list_markets_keyset_reports_documented_500_and_503() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("limit", "5"))
        .respond_with(internal_error())
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(service_unavailable())
        .mount(&server)
        .await;

    let gamma = no_retry_gamma(&server);
    let err = gamma
        .list_markets_keyset()
        .limit(5)
        .send()
        .await
        .unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.status().as_u16(), 500);
    assert_eq!(api.error_type(), Some("internal error"));

    let err = gamma.list_markets_keyset().send().await.unwrap_err();
    let api = err.api_error().unwrap();
    assert_eq!(api.status().as_u16(), 503);
    assert_eq!(api.error_type(), Some("service unavailable"));
    assert_eq!(api.message(), Some("keyset pagination is not configured"));
}

#[tokio::test]
async fn markets_information_rejects_non_integer_tag_id_before_sending() {
    let server = common::server().await;
    let err = common::polymarket(&server)
        .gamma()
        .get_markets_information()
        .tag_id("abc")
        .send()
        .await
        .unwrap_err();
    let Error::Validation(v) = &err else {
        panic!("expected Error::Validation, got {err:?}")
    };
    assert_eq!(v.parameter(), "tagId");
    assert!(requests(&server).await.is_empty());
}

/// One empty string-typed amount no longer fails the whole page: `""` decodes as `None`.
#[tokio::test]
async fn empty_string_amounts_decode_as_none() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(json(
            r#"[{"id":"1","liquidity":"","volume":"12.5","fee":null,"umaBond":"","umaReward":""}]"#,
        ))
        .mount(&server)
        .await;

    let markets = common::polymarket(&server)
        .gamma()
        .list_markets()
        .send()
        .await
        .unwrap();
    assert_eq!(markets[0].liquidity, None);
    assert_eq!(markets[0].volume, Some(Decimal::new(125, 1)));
    assert_eq!(markets[0].uma_bond, None);
}

#[tokio::test]
async fn non_numeric_string_amount_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(json(r#"[{"id":"1","liquidity":"lots"}]"#))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .list_markets()
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "[0].liquidity");
}
