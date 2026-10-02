//! Client-wide settings (user agent, timeouts, base URLs), exercised through Gamma.

use std::time::Duration;

use marcasite::{Error, Polymarket};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{header, method, path},
};

use crate::common;

#[tokio::test]
async fn sends_the_user_agent_and_honours_the_timeout() {
    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .and(header("user-agent", "my-app/1.0"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw("[]", "application/json")
                .set_delay(Duration::from_secs(5)),
        )
        .mount(&server)
        .await;

    let pm = Polymarket::builder()
        .gamma_base_url(server.uri())
        .user_agent("my-app/1.0")
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();
    assert!(pm.gamma().list_tags().send().await.unwrap().is_empty());
    let err = pm.gamma().list_events().send().await.unwrap_err();
    assert!(matches!(err, Error::Timeout(_)), "{err:?}");
}

#[tokio::test]
async fn unreachable_server_is_a_connect_error() {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let pm = Polymarket::builder()
        .gamma_base_url(format!("http://127.0.0.1:{port}"))
        .build()
        .unwrap();
    let err = pm.gamma().list_tags().send().await.unwrap_err();
    let Error::Transport(inner) = &err else {
        panic!("expected a transport error, got {err:?}");
    };
    assert!(inner.is_connect());
}

#[test]
fn invalid_base_url_is_a_config_error() {
    let err = Polymarket::builder()
        .gamma_base_url("ftp://example.com")
        .build()
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
}
