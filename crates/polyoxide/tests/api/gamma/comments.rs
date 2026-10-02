//! `/comments` endpoints.

use futures_util::{StreamExt as _, TryStreamExt as _};
use polyoxide::{
    Decimal,
    gamma::{CommentId, CommentParentEntityType},
    types::{Address, TokenId},
};
use serde_json::json;
use wiremock::{
    Mock,
    matchers::{method, path, query_param},
};

use super::{fixture, json, json_value, pairs, query_of};
use crate::common;

const ADDRESS: &str = "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b";

#[tokio::test]
async fn list_comments_sends_every_filter_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/comments"))
        .respond_with(json_value(&json!([fixture("Comment")])))
        .expect(1)
        .mount(&server)
        .await;

    let comments = common::polymarket(&server)
        .gamma()
        .list_comments(CommentParentEntityType::Event, 239_826)
        .limit(5)
        .offset(10)
        .order("createdAt")
        .ascending(false)
        .get_positions(true)
        .holders_only(false)
        .send()
        .await
        .unwrap();

    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "5"),
            ("offset", "10"),
            ("order", "createdAt"),
            ("ascending", "false"),
            ("parent_entity_type", "Event"),
            ("parent_entity_id", "239826"),
            ("get_positions", "true"),
            ("holders_only", "false"),
        ])
    );
    let comment = &comments[0];
    assert_eq!(comment.user_address, Some(Address::from(ADDRESS)));
    let position = &comment
        .profile
        .as_ref()
        .unwrap()
        .positions
        .as_ref()
        .unwrap()[0];
    assert_eq!(position.token_id, Some(TokenId::from("tokenId-value")));
    assert_eq!(position.position_size, Some(Decimal::new(15, 1)));
    assert_eq!(
        comment.reactions.as_ref().unwrap()[0].comment_id,
        Some(CommentId::from("7"))
    );
}

#[tokio::test]
async fn get_comments_by_id() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/comments/100"))
        .and(query_param("get_positions", "true"))
        .respond_with(json(
            r#"[{"id":"100"},{"id":"101","parentCommentID":"100"}]"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let comments = common::polymarket(&server)
        .gamma()
        .get_comments_by_id("100")
        .get_positions(true)
        .send()
        .await
        .unwrap();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[1].parent_comment_id, Some(CommentId::from("100")));
}

#[tokio::test]
async fn list_comments_by_user_pages_and_streams() {
    let server = common::server().await;
    let user_path = format!("/comments/user_address/{ADDRESS}");
    Mock::given(method("GET"))
        .and(path(user_path.as_str()))
        .and(query_param("offset", "0"))
        .respond_with(json(r#"[{"id":"1"},{"id":"2"}]"#))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(user_path.as_str()))
        .and(query_param("offset", "2"))
        .respond_with(json(r#"[{"id":"3"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    // A short page does not end the stream (the server may cap `limit`); an empty one does.
    Mock::given(method("GET"))
        .and(path(user_path.as_str()))
        .and(query_param("offset", "3"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let page = gamma
        .list_comments_by_user(ADDRESS)
        .limit(2)
        .offset(0)
        .order("createdAt")
        .ascending(true)
        .send()
        .await
        .unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "2"),
            ("offset", "0"),
            ("order", "createdAt"),
            ("ascending", "true"),
        ])
    );

    let all: Vec<_> = gamma
        .list_comments_by_user(Address::from(ADDRESS))
        .limit(2)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn list_comments_sends_the_required_parent_even_without_options() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/comments"))
        .respond_with(json("[]"))
        .expect(2)
        .mount(&server)
        .await;
    let gamma = common::polymarket(&server).gamma().clone();

    gamma
        .list_comments(CommentParentEntityType::Series, 10_345)
        .send()
        .await
        .unwrap();
    // `PerpsAsset` is accepted live; the spec's `market` is not a variant.
    gamma
        .list_comments(CommentParentEntityType::PerpsAsset, 1)
        .send()
        .await
        .unwrap();
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("parent_entity_type", "Series"),
            ("parent_entity_id", "10345")
        ])
    );
    assert_eq!(
        query_of(&server, 1).await,
        pairs(&[
            ("parent_entity_type", "PerpsAsset"),
            ("parent_entity_id", "1")
        ])
    );
}

#[tokio::test]
async fn list_comments_stream_keeps_the_parent_on_every_page() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/comments"))
        .and(query_param("parent_entity_type", "Event"))
        .and(query_param("parent_entity_id", "16167"))
        .and(query_param("offset", "0"))
        .respond_with(json(r#"[{"id":"1"},{"id":"2"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/comments"))
        .and(query_param("parent_entity_type", "Event"))
        .and(query_param("parent_entity_id", "16167"))
        .and(query_param("offset", "2"))
        .respond_with(json("[]"))
        .expect(1)
        .mount(&server)
        .await;

    let all: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_comments(CommentParentEntityType::Event, 16_167)
        .limit(2)
        .into_stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn comment_listings_reject_offsets_above_200_before_sending() {
    let server = common::server().await;
    let gamma = common::polymarket(&server).gamma().clone();

    let err = gamma
        .list_comments(CommentParentEntityType::Event, 1)
        .offset(201)
        .send()
        .await
        .unwrap_err();
    let polyoxide::Error::Validation(v) = &err else {
        panic!("expected a validation error, got {err:?}")
    };
    assert_eq!(v.parameter(), "offset");
    assert!(err.to_string().contains("201"), "{err}");

    let err = gamma
        .list_comments_by_user(ADDRESS)
        .offset(201)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(&err, polyoxide::Error::Validation(v) if v.parameter() == "offset"));
    assert!(super::requests(&server).await.is_empty());
}

#[tokio::test]
async fn comment_stream_ends_with_a_validation_error_past_offset_200() {
    let server = common::server().await;
    // Offsets 0, 100 and 200 are served; 200 is the last accepted offset, so the request
    // after its full page (offset 300) is refused client-side.
    for offset in ["0", "100", "200"] {
        let body: Vec<String> = (0..100)
            .map(|i| format!(r#"{{"id":"{offset}-{i}"}}"#))
            .collect();
        Mock::given(method("GET"))
            .and(path("/comments"))
            .and(query_param("offset", offset))
            .respond_with(json(format!("[{}]", body.join(","))))
            .expect(1)
            .mount(&server)
            .await;
    }

    let results: Vec<_> = common::polymarket(&server)
        .gamma()
        .list_comments(CommentParentEntityType::Event, 1)
        .limit(100)
        .into_stream()
        .collect()
        .await;
    assert_eq!(results.len(), 301);
    assert!(results[..300].iter().all(Result::is_ok));
    assert!(
        matches!(&results[300], Err(polyoxide::Error::Validation(v)) if v.parameter() == "offset")
    );
    assert_eq!(super::requests(&server).await.len(), 3);
}

#[tokio::test]
async fn comment_decodes_media_array() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/comments"))
        .respond_with(json_value(&json!([super::live_fixture("Comment")])))
        .expect(1)
        .mount(&server)
        .await;
    let comments = common::polymarket(&server)
        .gamma()
        .list_comments(CommentParentEntityType::Event, 45_915)
        .send()
        .await
        .unwrap();
    let media = comments[0].media.as_ref().unwrap();
    assert_eq!(media[0].provider.as_deref(), Some("giphy"));
}
