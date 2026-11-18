//! The reconnecting market channel against a scripted mock server.
//!
//! Real sockets cannot run on paused time (the clock would jump while I/O is pending), so
//! these tests use real time with millisecond backoffs and assert on the values the channel
//! reports (`retry_in`, `attempt`) and on lower bounds of the measured gaps.

use std::{
    future::Future,
    num::NonZeroU32,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_core::stream::FusedStream as _;
use marcasite::ws::SubscriptionLevel;
use marcasite::{
    Error, WebSocketErrorKind,
    ws::{
        MarketChannel, MarketEvent, MarketSubscription, MarketSubscriptionUpdate, ReconnectEvent,
        ReconnectPolicy, ReconnectingMarketChannel,
    },
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::oneshot;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::Instant,
};
use tokio_tungstenite::accept_async;

use super::mock::{ServerSocket, close, drain, next, recv_json, send};

const ASSET_A: &str =
    "65818619657568813474341868652308942079804919287380422192892211131408793125422";
const ASSET_B: &str =
    "71321045679252212594626385532706912750332728571942532289631379312455583992563";

const BOOK: &str = r#"{"event_type":"book","asset_id":"65818619657568813474341868652308942079804919287380422192892211131408793125422","market":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af","bids":[{"price":"0.48","size":"30"}],"asks":[{"price":"0.52","size":"25"}],"timestamp":"1757908892351","hash":"0xabc123..."}"#;

type Handler = Box<dyn FnOnce(TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// A connection that completes the WebSocket handshake and runs `f`.
fn ws<F, Fut>(f: F) -> Handler
where
    F: FnOnce(ServerSocket) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Box::new(move |stream| {
        Box::pin(async move {
            let socket = accept_async(stream).await.unwrap();
            f(socket).await;
        })
    })
}

/// A connection that is dropped before the handshake: the client's connect fails.
fn drop_tcp() -> Handler {
    Box::new(|stream| {
        Box::pin(async move {
            drop(stream);
        })
    })
}

/// A handshake refused with `status` and a `Retry-After` header.
fn refuse(status: &'static str, retry_after: u64) -> Handler {
    Box::new(move |mut stream| {
        Box::pin(async move {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let response = format!(
                "HTTP/1.1 {status}\r\nRetry-After: {retry_after}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            let _ = stream.shutdown().await;
        })
    })
}

/// Serves the scripted connections in order, then stops listening. Returns the URL, the
/// times the connections were accepted at, and the server task.
async fn serve_script(script: Vec<Handler>) -> (String, Arc<Mutex<Vec<Instant>>>, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let accepted = Arc::new(Mutex::new(Vec::new()));
    let times = Arc::clone(&accepted);
    let task = tokio::spawn(async move {
        for handler in script {
            let (stream, _) = listener.accept().await.unwrap();
            times.lock().unwrap().push(Instant::now());
            handler(stream).await;
        }
    });
    (format!("ws://{addr}/ws/market"), accepted, task)
}

fn policy(initial_ms: u64, max_ms: u64) -> ReconnectPolicy {
    ReconnectPolicy::new(
        Duration::from_millis(initial_ms),
        Duration::from_millis(max_ms),
    )
    .unwrap()
}

fn channel(
    url: &str,
    policy: ReconnectPolicy,
    sub: MarketSubscription,
) -> ReconnectingMarketChannel {
    MarketChannel::builder()
        .url(url)
        .connect_timeout(Duration::from_secs(5))
        .reconnecting()
        .policy(policy)
        .build(sub)
        .unwrap()
}

fn ids(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn id_strings(asset_ids: &[marcasite::types::TokenId]) -> Vec<String> {
    asset_ids.iter().map(|t| t.as_str().to_owned()).collect()
}

async fn expect_connected(ch: &mut ReconnectingMarketChannel, want: &[&str], attempt: u32) {
    match next(ch).await {
        Some(Ok(ReconnectEvent::Connected {
            asset_ids,
            attempt: got,
        })) => {
            assert_eq!(id_strings(&asset_ids), ids(want));
            assert_eq!(got, attempt);
        }
        other => panic!("expected Connected, got {other:?}"),
    }
}

async fn expect_disconnected(ch: &mut ReconnectingMarketChannel, want: &[&str]) -> Option<Error> {
    match next(ch).await {
        Some(Ok(ReconnectEvent::Disconnected { asset_ids, error })) => {
            assert_eq!(id_strings(&asset_ids), ids(want));
            error
        }
        other => panic!("expected Disconnected, got {other:?}"),
    }
}

async fn expect_book(ch: &mut ReconnectingMarketChannel) {
    assert!(
        matches!(
            next(ch).await,
            Some(Ok(ReconnectEvent::Market(MarketEvent::Book(_))))
        ),
        "expected a book"
    );
}

fn subscription_json(assets: &[&str], custom: bool) -> Value {
    json!({"assets_ids": assets, "type": "market", "custom_feature_enabled": custom})
}

#[tokio::test]
async fn connects_and_forwards_events_then_close_ends_for_good() {
    let (url, accepted, server) = serve_script(vec![
        ws(|mut socket| async move {
            assert_eq!(
                recv_json(&mut socket).await,
                subscription_json(&[ASSET_A], true)
            );
            send(&mut socket, BOOK).await;
            drain(&mut socket).await;
        }),
        // Must never be reached.
        ws(|_| async move { panic!("reconnected after close") }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(10, 20),
        MarketSubscription::new([ASSET_A]).custom_feature_enabled(true),
    );
    let handle = ch.handle();
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    expect_book(&mut ch).await;
    handle.close();
    assert!(next(&mut ch).await.is_none());
    assert!(ch.is_terminated());
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(accepted.lock().unwrap().len(), 1);
    server.abort();
    // The handle of a closed channel fails.
    let err = handle.subscribe([ASSET_B]).unwrap_err();
    assert!(matches!(&err, Error::WebSocket(e) if e.kind() == WebSocketErrorKind::Closed));
}

#[tokio::test]
async fn reconnects_after_a_dropped_connection_and_resubscribes_with_options() {
    let (url, _, server) = serve_script(vec![
        ws(|mut socket| async move {
            let first = recv_json(&mut socket).await;
            assert_eq!(first, subscription_json(&[ASSET_A], true));
            send(&mut socket, BOOK).await;
            // Abruptly gone: no close frame.
            drop(socket);
        }),
        ws(|mut socket| async move {
            // The same subscription, `custom_feature_enabled` preserved.
            assert_eq!(
                recv_json(&mut socket).await,
                subscription_json(&[ASSET_A], true)
            );
            send(&mut socket, BOOK).await;
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(10, 20),
        MarketSubscription::new([ASSET_A]).custom_feature_enabled(true),
    );
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    expect_book(&mut ch).await;
    let error = expect_disconnected(&mut ch, &[ASSET_A]).await;
    assert!(error.is_some(), "a dropped connection is an error");
    // The connection was not stable, so the attempt count continues.
    expect_connected(&mut ch, &[ASSET_A], 2).await;
    expect_book(&mut ch).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn a_stable_connection_resets_the_attempt_count() {
    let (url, _, server) = serve_script(vec![
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            close(&mut socket, 1000, "bye").await;
        }),
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            drain(&mut socket).await;
        }),
    ])
    .await;
    // Zero: every connection counts as stable.
    let mut ch = channel(
        &url,
        policy(10, 20).stable_after(Duration::ZERO),
        MarketSubscription::new([ASSET_A]),
    );
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    // A normal close is a disconnect without an error, and is reconnected.
    assert!(expect_disconnected(&mut ch, &[ASSET_A]).await.is_none());
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn changes_made_while_disconnected_apply_on_reconnect() {
    let (url, _, server) = serve_script(vec![
        ws(|mut socket| async move {
            assert_eq!(
                recv_json(&mut socket).await,
                subscription_json(&[ASSET_A], true)
            );
            // A change on the live connection is forwarded.
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"operation": "subscribe", "assets_ids": [ASSET_B]})
            );
            drop(socket);
        }),
        ws(|mut socket| async move {
            // A is unsubscribed, B (added live) stays; the flag is preserved.
            assert_eq!(
                recv_json(&mut socket).await,
                subscription_json(&[ASSET_B], true)
            );
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(300, 300),
        MarketSubscription::new([ASSET_A]).custom_feature_enabled(true),
    );
    let handle = ch.handle();
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    handle.subscribe([ASSET_B]).unwrap();
    // The tracked set already includes the change.
    assert_eq!(id_strings(&handle.asset_ids()), ids(&[ASSET_A, ASSET_B]));
    expect_disconnected(&mut ch, &[ASSET_A, ASSET_B]).await;
    // Disconnected: changes are only recorded (the backoff is 300 ms).
    handle.unsubscribe([ASSET_A]).unwrap();
    assert!(matches!(
        handle.update_subscription(MarketSubscriptionUpdate::subscribe(Vec::<&str>::new())),
        Err(Error::Validation(_))
    ));
    expect_connected(&mut ch, &[ASSET_B], 2).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn backoff_doubles_up_to_the_cap() {
    let (url, accepted, server) = serve_script(vec![
        drop_tcp(),
        drop_tcp(),
        drop_tcp(),
        drop_tcp(),
        drop_tcp(),
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(&url, policy(20, 80), MarketSubscription::new([ASSET_A]));
    let mut delays = Vec::new();
    loop {
        match next(&mut ch).await {
            Some(Ok(ReconnectEvent::ConnectFailed {
                attempt,
                error,
                retry_in,
            })) => {
                assert_eq!(attempt as usize, delays.len() + 1);
                assert!(
                    matches!(&error, Error::WebSocket(e) if e.kind() == WebSocketErrorKind::Connect)
                );
                delays.push(retry_in);
            }
            Some(Ok(ReconnectEvent::Connected { attempt, .. })) => {
                assert_eq!(attempt, 6);
                break;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    let ms: Vec<u128> = delays.iter().map(Duration::as_millis).collect();
    assert_eq!(ms, [20, 40, 80, 80, 80]);
    // The waits really happened.
    let times = accepted.lock().unwrap().clone();
    for (gap, want) in times.windows(2).zip(&delays) {
        assert!(gap[1] - gap[0] >= *want, "{:?} < {want:?}", gap[1] - gap[0]);
    }
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn honours_retry_after_of_a_refused_handshake() {
    let (url, accepted, server) = serve_script(vec![
        refuse("429 Too Many Requests", 1),
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    match next(&mut ch).await {
        Some(Ok(ReconnectEvent::ConnectFailed {
            attempt: 1,
            error,
            retry_in,
        })) => {
            assert_eq!(error.status().map(|s| s.as_u16()), Some(429));
            assert_eq!(error.retry_after(), Some(Duration::from_secs(1)));
            // The longer of the backoff (10 ms) and the server's delay.
            assert_eq!(retry_in, Duration::from_secs(1));
        }
        other => panic!("unexpected {other:?}"),
    }
    expect_connected(&mut ch, &[ASSET_A], 2).await;
    let times = accepted.lock().unwrap().clone();
    assert!(times[1] - times[0] >= Duration::from_secs(1));
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn caps_an_excessive_retry_after() {
    let (url, _, server) = serve_script(vec![refuse("503 Service Unavailable", 86_400)]).await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    match next(&mut ch).await {
        Some(Ok(ReconnectEvent::ConnectFailed {
            retry_in, error, ..
        })) => {
            assert_eq!(error.retry_after(), Some(Duration::from_secs(86_400)));
            assert_eq!(retry_in, ReconnectPolicy::MAX_RETRY_AFTER);
        }
        other => panic!("unexpected {other:?}"),
    }
    // Closing ends the stream even though it is waiting for an hour.
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn gives_up_after_max_attempts_with_the_last_error() {
    let (url, _, server) = serve_script(vec![drop_tcp(), drop_tcp(), drop_tcp()]).await;
    let mut ch = channel(
        &url,
        policy(5, 10).max_attempts(NonZeroU32::new(3).unwrap()),
        MarketSubscription::new([ASSET_A]),
    );
    for attempt in 1..=2 {
        assert!(matches!(
            next(&mut ch).await,
            Some(Ok(ReconnectEvent::ConnectFailed { attempt: a, .. })) if a == attempt
        ));
    }
    match next(&mut ch).await {
        Some(Err(Error::WebSocket(e))) => assert_eq!(e.kind(), WebSocketErrorKind::Connect),
        other => panic!("expected the last error, got {other:?}"),
    }
    assert!(next(&mut ch).await.is_none());
    assert!(ch.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn gives_up_when_unstable_connections_use_the_attempts() {
    let (url, _, server) = serve_script(vec![ws(|mut socket| async move {
        let _ = recv_json(&mut socket).await;
        drop(socket);
    })])
    .await;
    let mut ch = channel(
        &url,
        policy(5, 10).max_attempts(NonZeroU32::MIN),
        MarketSubscription::new([ASSET_A]),
    );
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    expect_disconnected(&mut ch, &[ASSET_A]).await;
    match next(&mut ch).await {
        Some(Err(Error::WebSocket(e))) => {
            assert_eq!(e.kind(), WebSocketErrorKind::Closed);
            assert!(e.message().contains("gave up"), "{}", e.message());
        }
        other => panic!("expected an error, got {other:?}"),
    }
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn close_while_waiting_to_reconnect_ends_the_stream() {
    let (url, accepted, server) = serve_script(vec![
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            drop(socket);
        }),
        ws(|_| async move { panic!("reconnected after close") }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(60_000, 60_000),
        MarketSubscription::new([ASSET_A]),
    );
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    expect_disconnected(&mut ch, &[ASSET_A]).await;
    let handle = ch.handle();
    let closer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        handle.close();
    });
    // Woken by the close, long before the 60 s backoff.
    assert!(next(&mut ch).await.is_none());
    closer.await.unwrap();
    server.abort();
    assert_eq!(accepted.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn decode_errors_pass_through_without_reconnecting() {
    let (url, accepted, server) = serve_script(vec![
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            // A `book` with a wrong type: recognised but invalid.
            send(&mut socket, r#"{"event_type":"book","asset_id":1}"#).await;
            send(&mut socket, BOOK).await;
            drain(&mut socket).await;
        }),
        ws(|_| async move { panic!("reconnected after a decode error") }),
    ])
    .await;
    let mut ch = channel(&url, policy(5, 10), MarketSubscription::new([ASSET_A]));
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    match next(&mut ch).await {
        Some(Err(Error::WebSocket(e))) => assert_eq!(e.kind(), WebSocketErrorKind::Decode),
        other => panic!("expected a decode error, got {other:?}"),
    }
    expect_book(&mut ch).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.abort();
    assert_eq!(accepted.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn dropping_the_stream_closes_the_connection_and_stops_everything() {
    let (gone_tx, gone_rx) = tokio::sync::oneshot::channel();
    let (url, accepted, server) = serve_script(vec![
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            // Returns when the client goes away.
            drain(&mut socket).await;
            gone_tx.send(()).unwrap();
        }),
        ws(|_| async move { panic!("reconnected after the stream was dropped") }),
    ])
    .await;
    let mut ch = channel(&url, policy(5, 10), MarketSubscription::new([ASSET_A]));
    let handle = ch.handle();
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    drop(ch);
    tokio::time::timeout(Duration::from_secs(5), gone_rx)
        .await
        .expect("the connection was not closed")
        .unwrap();
    assert!(handle.subscribe([ASSET_B]).is_err());
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(accepted.lock().unwrap().len(), 1);
    server.abort();
}

#[tokio::test]
async fn an_empty_subscription_is_allowed() {
    let (url, _, server) = serve_script(vec![ws(|mut socket| async move {
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"assets_ids": [], "type": "market"})
        );
        drain(&mut socket).await;
    })])
    .await;
    let mut ch = channel(
        &url,
        policy(5, 10),
        MarketSubscription::new(Vec::<&str>::new()),
    );
    expect_connected(&mut ch, &[], 1).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn an_invalid_configuration_fails_at_build() {
    let err = MarketChannel::builder()
        .idle_timeout(Duration::ZERO)
        .reconnecting()
        .build(MarketSubscription::new([ASSET_A]))
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)));
    let err = MarketChannel::builder()
        .url("http://example.com")
        .reconnecting()
        .build(MarketSubscription::new([ASSET_A]))
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)));
}

/// A connection whose handshake is only answered once `gate` fires; sets `accepted` if the
/// handshake completes.
fn gated<F, Fut>(gate: oneshot::Receiver<()>, accepted: Arc<AtomicBool>, f: F) -> Handler
where
    F: FnOnce(ServerSocket) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Box::new(move |stream| {
        Box::pin(async move {
            let _ = gate.await;
            if let Ok(socket) = accept_async(stream).await {
                accepted.store(true, Ordering::SeqCst);
                f(socket).await;
            }
        })
    })
}

/// Polls the stream once, so that it starts connecting.
async fn poll_once(ch: &mut ReconnectingMarketChannel) {
    use futures_core::Stream as _;
    std::future::poll_fn(|cx| {
        assert!(std::pin::Pin::new(&mut *ch).poll_next(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn changes_made_while_connecting_reach_the_new_connection() {
    let (release, gate) = oneshot::channel();
    let accepted = Arc::new(AtomicBool::new(false));
    let (url, _, server) = serve_script(vec![gated(
        gate,
        Arc::clone(&accepted),
        |mut socket| async move {
            // The initial frame is the subscription at the time of the attempt.
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"assets_ids": [ASSET_A], "type": "market"})
            );
            // Catch-up: the new settings apply to every tracked asset, then the removal.
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"operation": "subscribe", "assets_ids": [ASSET_B], "custom_feature_enabled": true})
            );
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"operation": "unsubscribe", "assets_ids": [ASSET_A]})
            );
            drain(&mut socket).await;
        },
    )])
    .await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    poll_once(&mut ch).await;
    let handle = ch.handle();
    handle.subscribe([ASSET_B]).unwrap();
    handle.unsubscribe([ASSET_A]).unwrap();
    handle
        .update_subscription(
            MarketSubscriptionUpdate::subscribe([ASSET_B]).custom_feature_enabled(true),
        )
        .unwrap();
    release.send(()).unwrap();
    expect_connected(&mut ch, &[ASSET_B], 1).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.abort();
}

#[tokio::test]
async fn close_during_the_handshake_ends_without_a_connection() {
    let (release, gate) = oneshot::channel();
    let accepted = Arc::new(AtomicBool::new(false));
    let (url, _, server) =
        serve_script(vec![gated(gate, Arc::clone(&accepted), |_| async move {})]).await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    poll_once(&mut ch).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    drop(ch);
    let _ = release.send(());
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        !accepted.load(Ordering::SeqCst),
        "a connection was completed"
    );
    server.abort();
}

#[tokio::test]
async fn close_delivers_buffered_events_first_and_no_disconnected() {
    let (url, _, server) = serve_script(vec![ws(|mut socket| async move {
        let _ = recv_json(&mut socket).await;
        send(&mut socket, BOOK).await;
        send(&mut socket, BOOK).await;
        drain(&mut socket).await;
    })])
    .await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    // Let both frames arrive in the buffer, then close before they are read.
    tokio::time::sleep(Duration::from_millis(200)).await;
    ch.close();
    expect_book(&mut ch).await;
    expect_book(&mut ch).await;
    assert!(next(&mut ch).await.is_none());
    assert!(ch.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn level_and_initial_dump_survive_a_reconnect() {
    let frame =
        json!({"assets_ids": [ASSET_A], "type": "market", "initial_dump": false, "level": 3});
    let (f1, f2) = (frame.clone(), frame);
    let (url, _, server) = serve_script(vec![
        ws(|mut socket| async move {
            assert_eq!(recv_json(&mut socket).await, f1);
            drop(socket);
        }),
        ws(|mut socket| async move {
            assert_eq!(recv_json(&mut socket).await, f2);
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(10, 20),
        MarketSubscription::new([ASSET_A])
            .initial_dump(false)
            .level(SubscriptionLevel::Level3),
    );
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    expect_disconnected(&mut ch, &[ASSET_A]).await;
    expect_connected(&mut ch, &[ASSET_A], 2).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn a_permanent_client_error_is_fatal_unless_opted_in() {
    // Default: a 403 ends the stream with the error; the handles then fail.
    let (url, accepted, server) = serve_script(vec![refuse("403 Forbidden", 0)]).await;
    let mut ch = channel(&url, policy(5, 10), MarketSubscription::new([ASSET_A]));
    let handle = ch.handle();
    match next(&mut ch).await {
        Some(Err(Error::WebSocket(e))) => {
            assert_eq!(e.kind(), WebSocketErrorKind::Connect);
            assert_eq!(e.http_status().map(|s| s.as_u16()), Some(403));
        }
        other => panic!("expected a fatal 403, got {other:?}"),
    }
    assert!(next(&mut ch).await.is_none());
    assert!(matches!(
        handle.subscribe([ASSET_B]),
        Err(Error::WebSocket(e)) if e.kind() == WebSocketErrorKind::Closed
    ));
    assert_eq!(accepted.lock().unwrap().len(), 1);
    server.await.unwrap();

    // Opted in: it is retried like any other failure.
    let (url, _, server) = serve_script(vec![
        refuse("403 Forbidden", 0),
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(5, 10).retry_client_errors(true),
        MarketSubscription::new([ASSET_A]),
    );
    assert!(matches!(
        next(&mut ch).await,
        Some(Ok(ReconnectEvent::ConnectFailed { attempt: 1, .. }))
    ));
    expect_connected(&mut ch, &[ASSET_A], 2).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.abort();
}

#[tokio::test]
async fn handles_fail_after_giving_up() {
    let (url, _, server) = serve_script(vec![drop_tcp()]).await;
    let mut ch = channel(
        &url,
        policy(5, 10).max_attempts(NonZeroU32::MIN),
        MarketSubscription::new([ASSET_A]),
    );
    let handle = ch.handle();
    assert!(matches!(next(&mut ch).await, Some(Err(_))));
    assert!(matches!(
        handle.subscribe([ASSET_B]),
        Err(Error::WebSocket(e)) if e.kind() == WebSocketErrorKind::Closed
    ));
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[test]
fn polling_outside_a_runtime_is_an_error_not_a_panic() {
    use std::task::{Context, Poll, Waker};

    use futures_core::Stream as _;

    let mut ch = ReconnectingMarketChannel::new(MarketSubscription::new([ASSET_A])).unwrap();
    let handle = ch.handle();
    let mut cx = Context::from_waker(Waker::noop());
    match std::pin::Pin::new(&mut ch).poll_next(&mut cx) {
        Poll::Ready(Some(Err(Error::Config(_)))) => {}
        other => panic!("expected a config error, got {other:?}"),
    }
    assert!(matches!(
        std::pin::Pin::new(&mut ch).poll_next(&mut cx),
        Poll::Ready(None)
    ));
    assert!(handle.subscribe([ASSET_B]).is_err());
}

#[tokio::test]
async fn a_connection_that_goes_silent_is_not_stable() {
    // The server accepts, reads the subscription, then sends nothing (not even PONG): every
    // connection ends with the idle timeout. That must not count as a stable connection, or
    // `max_attempts` would never take effect.
    let (url, _, server) = serve_script(vec![ws(|mut socket| async move {
        let _ = recv_json(&mut socket).await;
        drain(&mut socket).await;
    })])
    .await;
    let mut ch = MarketChannel::builder()
        .url(&url)
        .idle_timeout(Duration::from_millis(300))
        .reconnecting()
        .policy(
            policy(5, 10)
                .stable_after(Duration::from_millis(100))
                .max_attempts(NonZeroU32::MIN),
        )
        .build(MarketSubscription::new([ASSET_A]))
        .unwrap();
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    let error = expect_disconnected(&mut ch, &[ASSET_A]).await;
    assert!(
        matches!(&error, Some(Error::WebSocket(e)) if e.kind() == WebSocketErrorKind::Timeout),
        "{error:?}"
    );
    match next(&mut ch).await {
        Some(Err(Error::WebSocket(e))) => assert!(e.message().contains("gave up"), "{e}"),
        other => panic!("expected to give up, got {other:?}"),
    }
    assert!(next(&mut ch).await.is_none());
    server.abort();
}

#[tokio::test]
async fn changes_made_before_the_first_poll_go_in_the_initial_message() {
    let (url, _, server) = serve_script(vec![ws(|mut socket| async move {
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"assets_ids": [ASSET_A, ASSET_B], "type": "market"})
        );
        drain(&mut socket).await;
    })])
    .await;
    let mut ch = channel(&url, policy(10, 20), MarketSubscription::new([ASSET_A]));
    ch.handle().subscribe([ASSET_B]).unwrap();
    expect_connected(&mut ch, &[ASSET_A, ASSET_B], 1).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn an_unsubscribe_does_not_change_the_tracked_level() {
    let (url, _, server) = serve_script(vec![
        ws(|mut socket| async move {
            let _ = recv_json(&mut socket).await;
            // The update is forwarded as given.
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"operation": "unsubscribe", "assets_ids": [ASSET_B], "level": 1})
            );
            drop(socket);
        }),
        ws(|mut socket| async move {
            assert_eq!(
                recv_json(&mut socket).await,
                json!({"assets_ids": [ASSET_A], "type": "market", "level": 3})
            );
            drain(&mut socket).await;
        }),
    ])
    .await;
    let mut ch = channel(
        &url,
        policy(10, 20),
        MarketSubscription::new([ASSET_A]).level(SubscriptionLevel::Level3),
    );
    let handle = ch.handle();
    expect_connected(&mut ch, &[ASSET_A], 1).await;
    handle
        .update_subscription(
            MarketSubscriptionUpdate::unsubscribe([ASSET_B]).level(SubscriptionLevel::Level1),
        )
        .unwrap();
    expect_disconnected(&mut ch, &[ASSET_A]).await;
    expect_connected(&mut ch, &[ASSET_A], 2).await;
    ch.close();
    assert!(next(&mut ch).await.is_none());
    server.await.unwrap();
}
