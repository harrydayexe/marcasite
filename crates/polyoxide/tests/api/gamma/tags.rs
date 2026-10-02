//! `/tags` endpoints.

use futures_util::TryStreamExt as _;
use polyoxide::{
    Error,
    gamma::{RelatedTagsStatus, TagId},
};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use super::{fixture, json, json_value, pairs, query_of};
use crate::common;

#[tokio::test]
async fn list_tags_sends_query_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[{"id":"1","label":"Politics","slug":"politics"},{"id":"2","label":"Sports"}]"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let pm = common::polymarket(&server);
    let tags = pm
        .gamma()
        .list_tags()
        .limit(2)
        .offset(4)
        .order("label")
        .ascending(true)
        .include_template(true)
        .is_carousel(false)
        .send()
        .await
        .unwrap();
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[
            ("limit", "2"),
            ("offset", "4"),
            ("order", "label"),
            ("ascending", "true"),
            ("include_template", "true"),
            ("is_carousel", "false"),
        ])
    );
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].slug.as_deref(), Some("politics"));
}

#[tokio::test]
async fn list_tags_stream_walks_offsets() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("offset", "0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"[{"id":"1"},{"id":"2"}]"#, "application/json"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("offset", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"[{"id":"3"}]"#, "application/json"),
        )
        .mount(&server)
        .await;
    // A short page does not end the stream (the server may cap `limit`); an empty one does.
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("offset", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let pm = common::polymarket(&server);
    let ids: Vec<_> = pm
        .gamma()
        .list_tags()
        .limit(2)
        .into_stream()
        .map_ok(|t| t.id.unwrap())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(
        ids,
        vec![TagId::from("1"), TagId::from("2"), TagId::from("3")]
    );
}

#[tokio::test]
async fn list_tags_stream_is_a_named_unpin_stream() {
    use futures_util::StreamExt as _;
    use polyoxide::{Paginated, gamma::Tag};

    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("offset", "0"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"[{"id":"1"}]"#, "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("offset", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    // Named (storable in a struct) and `Unpin` (a plain `while let` loop works).
    struct Holder {
        tags: Paginated<Tag>,
    }
    let mut holder = Holder {
        tags: common::polymarket(&server)
            .gamma()
            .list_tags()
            .into_stream(),
    };
    let mut ids = Vec::new();
    while let Some(tag) = holder.tags.next().await {
        ids.push(tag.unwrap().id.unwrap());
    }
    assert_eq!(ids, vec![TagId::from("1")]);
}

#[tokio::test]
async fn get_tag_encodes_path_and_reports_404() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags/42"))
        .and(query_param_is_missing("include_template"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{"id":"42"}"#, "application/json"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tags/404"))
        .respond_with(ResponseTemplate::new(404).set_body_raw("Not found", "text/plain"))
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let tag = gamma.get_tag("42").send().await.unwrap();
    assert_eq!(tag.id, Some(TagId::from("42")));

    let err = gamma.get_tag("404").send().await.unwrap_err();
    assert!(err.is_not_found(), "{err}");
    let Error::Api(api) = &err else {
        panic!("expected Error::Api, got {err:?}")
    };
    assert_eq!(api.message(), Some("Not found"));
}

#[tokio::test]
async fn malformed_body_is_a_decode_error_with_path() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"[{"id":"1","createdBy":"x"}]"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .list_tags()
        .send()
        .await
        .unwrap_err();
    let Error::Decode(decode) = &err else {
        panic!("expected Error::Decode, got {err:?}")
    };
    assert_eq!(decode.path(), "[0].createdBy");
}

#[tokio::test]
async fn rate_limit_is_typed_with_retry_after() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "3")
                .set_body_raw(r#"{"error":"too many requests"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .list_tags()
        .send()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::RateLimited(_)), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(3)));
    assert!(err.is_retryable());
}

#[tokio::test]
async fn get_tag_by_slug_with_template() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags/slug/politics"))
        .and(query_param("include_template", "true"))
        .respond_with(json_value(&fixture("Tag")))
        .expect(1)
        .mount(&server)
        .await;

    let tag = common::polymarket(&server)
        .gamma()
        .get_tag_by_slug("politics")
        .include_template(true)
        .send()
        .await
        .unwrap();
    assert_eq!(tag.slug.as_deref(), Some("slug-value"));
    assert_eq!(tag.created_by, Some(7));
}

#[tokio::test]
async fn related_tag_relationships_by_id_and_slug() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags/100381/related-tags"))
        .respond_with(json(
            r#"[{"id":"1","tagID":100381,"relatedTagID":2,"rank":1}]"#,
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tags/slug/politics/related-tags"))
        .respond_with(json_value(&serde_json::json!([fixture("RelatedTag")])))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let relations = gamma
        .get_related_tag_relationships("100381")
        .omit_empty(true)
        .status(RelatedTagsStatus::Active)
        .send()
        .await
        .unwrap();
    assert_eq!(relations[0].tag_id, Some(TagId::from("100381")));
    assert_eq!(relations[0].related_tag_id, Some(TagId::from("2")));
    assert_eq!(
        query_of(&server, 0).await,
        pairs(&[("omit_empty", "true"), ("status", "active")])
    );

    let by_slug = gamma
        .get_related_tag_relationships_by_slug("politics")
        .send()
        .await
        .unwrap();
    assert_eq!(by_slug[0].rank, Some(7));
    assert_eq!(query_of(&server, 1).await, pairs(&[]));
}

#[tokio::test]
async fn related_tags_by_id_and_slug() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags/1/related-tags/tags"))
        .respond_with(json(r#"[{"id":"2","label":"Elections"}]"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tags/slug/politics/related-tags/tags"))
        .respond_with(json(r#"[{"id":"3"}]"#))
        .expect(1)
        .mount(&server)
        .await;

    let gamma = common::polymarket(&server).gamma().clone();
    let tags = gamma
        .get_related_tags("1")
        .status(RelatedTagsStatus::All)
        .send()
        .await
        .unwrap();
    assert_eq!(tags[0].label.as_deref(), Some("Elections"));
    assert_eq!(query_of(&server, 0).await, pairs(&[("status", "all")]));

    // A status value unknown to this library version is sent verbatim.
    let tags = gamma
        .get_related_tags_by_slug("politics")
        .omit_empty(false)
        .status(RelatedTagsStatus::from("upcoming"))
        .send()
        .await
        .unwrap();
    assert_eq!(tags[0].id, Some(TagId::from("3")));
    assert_eq!(
        query_of(&server, 1).await,
        pairs(&[("omit_empty", "false"), ("status", "upcoming")])
    );
}
