//! `/status`.

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::common;

#[tokio::test]
async fn status_returns_plain_text() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/status"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("OK", "text/plain"))
        .expect(1)
        .mount(&server)
        .await;

    let status = common::polymarket(&server).gamma().status().await.unwrap();
    assert_eq!(status, "OK");
}

#[tokio::test]
async fn status_reports_failures() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/status"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = common::polymarket(&server)
        .gamma()
        .status()
        .await
        .unwrap_err();
    assert!(err.is_not_found());
}
