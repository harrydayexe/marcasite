//! `/tags` endpoints.

use futures_util::TryStreamExt as _;
use polyoxide::{Error, gamma::TagId};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, query_param, query_param_is_missing},
};

use crate::common;

#[tokio::test]
async fn list_tags_sends_query_and_decodes() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(query_param("limit", "2"))
        .and(query_param("ascending", "true"))
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
        .ascending(true)
        .send()
        .await
        .unwrap();
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
