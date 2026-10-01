//! `/comments` endpoints.

use futures_util::TryStreamExt as _;
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
        .list_comments()
        .limit(5)
        .offset(10)
        .order("createdAt")
        .ascending(false)
        .parent_entity_type(CommentParentEntityType::Market)
        .parent_entity_id(239_826)
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
            ("parent_entity_type", "market"),
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
