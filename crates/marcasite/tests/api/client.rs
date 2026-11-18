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

/// Real time (wiremock uses real sockets): small numbers and generous upper bounds.
#[tokio::test]
async fn rate_limit_spreads_requests_over_time() {
    use std::{num::NonZeroU32, time::Instant};

    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .expect(5)
        .mount(&server)
        .await;
    let pm = Polymarket::builder()
        .gamma_base_url(server.uri())
        .rate_limit(marcasite::RateLimit::per_second(
            NonZeroU32::new(20).unwrap(),
        ))
        .build()
        .unwrap();

    // Five concurrent requests through clones of one client share one budget:
    // the first is immediate, the other four are 50 ms apart, so at least 200 ms in all.
    let start = Instant::now();
    let tasks: Vec<_> = (0..5)
        .map(|_| {
            let pm = pm.clone();
            tokio::spawn(async move { pm.gamma().list_tags().send().await })
        })
        .collect();
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    let took = start.elapsed();
    assert!(took >= Duration::from_millis(195), "{took:?}");
    assert!(took < Duration::from_secs(5), "{took:?}");
}

#[tokio::test]
async fn retries_consume_rate_limit_slots() {
    use std::{num::NonZeroU32, time::Instant};

    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(ResponseTemplate::new(503))
        .expect(3)
        .mount(&server)
        .await;
    let pm = Polymarket::builder()
        .gamma_base_url(server.uri())
        .retry_policy(marcasite::RetryPolicy::new(2).with_initial_backoff(Duration::from_millis(1)))
        .rate_limit(marcasite::RateLimit::per_second(
            NonZeroU32::new(20).unwrap(),
        ))
        .build()
        .unwrap();

    // Three attempts (1 + 2 retries) need three slots: at least 100 ms, although the
    // backoff alone is ~3 ms.
    let start = Instant::now();
    let err = pm.gamma().list_tags().send().await.unwrap_err();
    assert!(err.is_retryable(), "{err:?}");
    let took = start.elapsed();
    assert!(took >= Duration::from_millis(95), "{took:?}");
    assert!(took < Duration::from_secs(5), "{took:?}");
}

#[tokio::test]
async fn rate_limit_wait_does_not_count_against_the_request_timeout() {
    use std::num::NonZeroU32;

    let server = common::server().await;
    Mock::given(method("GET"))
        .and(path("/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&server)
        .await;
    let pm = Polymarket::builder()
        .gamma_base_url(server.uri())
        .timeout(Duration::from_millis(50))
        .rate_limit(marcasite::RateLimit::per_second(
            NonZeroU32::new(20).unwrap(),
        ))
        .build()
        .unwrap();
    // The last request waits ~200 ms for its slot, far more than the 50 ms timeout.
    let tasks: Vec<_> = (0..5)
        .map(|_| {
            let pm = pm.clone();
            tokio::spawn(async move { pm.gamma().list_tags().send().await })
        })
        .collect();
    for task in tasks {
        task.await.unwrap().unwrap();
    }
}
