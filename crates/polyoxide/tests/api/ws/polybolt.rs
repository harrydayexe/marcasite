//! PolyBolt `price.polymarket` (`/ws`) against a mock server.
//!
//! Message bodies are the examples in `docs/specs/polybolt-asyncapi.json`.

use futures_core::stream::FusedStream as _;
use polyoxide::{
    Decimal, Error,
    ws::{
        PolyBoltChannel, PolyBoltChannelName, PolyBoltCloseCode, PolyBoltErrorCode, PolyBoltEvent,
        PolyBoltSubscription,
    },
};
use serde_json::json;

use super::mock::{close, drain, next, recv_json, send, serve};

const ASSET: &str = "21742633143463906290569050155826241533067272736897614950488156847949938836455";

const SNAPSHOT: &str = r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1788973000123,"snapshot":true,"payload":{"market":"0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c","asset_id":"21742633143463906290569050155826241533067272736897614950488156847949938836455","best_bid":"0.51","best_ask":"0.53","hash":"3f9c1e7a","timestamp":1788972999871}}"#;
const COLD_SNAPSHOT: &str = r#"{"v":1,"channel":"price.polymarket","seq":1,"ts":1788973000123,"snapshot":true,"payload":[]}"#;
const LIVE: &str = r#"{"v":1,"channel":"price.polymarket","seq":2,"ts":1788973004501,"payload":{"market":"0x9deb0baac40648821f96f01339229a422e2f5c877de55dc4dbf981f95a1e709c","asset_id":"21742633143463906290569050155826241533067272736897614950488156847949938836455","best_bid":"0.52","best_ask":"0.53","hash":"a81d02c4","timestamp":1788973004498}}"#;

async fn connect(url: &str) -> PolyBoltChannel {
    PolyBoltChannel::builder().url(url).connect().await.unwrap()
}

#[tokio::test]
async fn subscribes_and_decodes_events() {
    let (url, server) = serve("/ws", |mut socket| async move {
        assert_eq!(
            recv_json(&mut socket).await,
            json!({
                "op": "subscribe",
                "rid": "s1",
                "subscriptions": [
                    {"channel": "price.polymarket", "filter": {"asset_id": ASSET}},
                    {"channel": "price.polymarket", "filter": {"asset_id": "123"}}
                ]
            })
        );
        send(
            &mut socket,
            r#"{"op":"subscribed","channel":"price.polymarket","rid":"s1"}"#,
        )
        .await;
        send(&mut socket, SNAPSHOT).await;
        send(&mut socket, COLD_SNAPSHOT).await;
        send(&mut socket, LIVE).await;
        send(
            &mut socket,
            r#"{"op":"error","code":"bad_filter","channel":"price.polymarket","rid":"s2"}"#,
        )
        .await;
        send(&mut socket, r#"{"op":"authed","rid":"a1"}"#).await;

        assert_eq!(
            recv_json(&mut socket).await,
            json!({
                "op": "unsubscribe",
                "subscriptions": [{"channel": "price.polymarket", "filter": {"asset_id": "123"}}]
            })
        );
        send(
            &mut socket,
            r#"{"op":"unsubscribed","channel":"price.polymarket"}"#,
        )
        .await;

        assert_eq!(recv_json(&mut socket).await, json!({"op": "ping"}));
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"op": "ping", "rid": "p1"})
        );
        send(&mut socket, r#"{"op":"pong","rid":"p1"}"#).await;
        close(&mut socket, 1000, "").await;
    })
    .await;

    let mut channel = connect(&url).await;
    channel
        .subscribe(PolyBoltSubscription::price_polymarket([ASSET, "123"]).rid("s1"))
        .unwrap();
    assert_eq!(channel.active_subscriptions(), 2);

    let Some(Ok(PolyBoltEvent::Subscribed(ack))) = next(&mut channel).await else {
        panic!("expected a subscribed ack");
    };
    assert_eq!(ack.channel, PolyBoltChannelName::PricePolymarket);
    assert_eq!(ack.rid.as_deref(), Some("s1"));

    let Some(Ok(PolyBoltEvent::PricePolymarket(snapshot))) = next(&mut channel).await else {
        panic!("expected a snapshot");
    };
    assert!(snapshot.is_snapshot());
    assert_eq!(snapshot.payload.unwrap().best_bid, Decimal::new(51, 2));

    let Some(Ok(PolyBoltEvent::PricePolymarket(cold))) = next(&mut channel).await else {
        panic!("expected a cold snapshot");
    };
    assert_eq!(cold.payload, None);

    let Some(Ok(PolyBoltEvent::PricePolymarket(live))) = next(&mut channel).await else {
        panic!("expected a live update");
    };
    assert!(!live.is_snapshot());
    assert_eq!(live.seq, 2);
    assert_eq!(live.payload.unwrap().best_bid, Decimal::new(52, 2));

    let Some(Ok(PolyBoltEvent::Error(error))) = next(&mut channel).await else {
        panic!("expected an error ack");
    };
    assert_eq!(error.code, PolyBoltErrorCode::BadFilter);
    assert!(!error.code.closes_connection());

    // Acks for operations this crate never sends are surfaced as unknown.
    let Some(Ok(PolyBoltEvent::Unknown(unknown))) = next(&mut channel).await else {
        panic!("expected an unknown message");
    };
    assert_eq!(unknown, json!({"op": "authed", "rid": "a1"}));

    channel
        .unsubscribe(PolyBoltSubscription::price_polymarket(["123"]))
        .unwrap();
    assert_eq!(channel.active_subscriptions(), 1);
    assert!(matches!(
        next(&mut channel).await,
        Some(Ok(PolyBoltEvent::Unsubscribed(_)))
    ));

    channel.ping().unwrap();
    channel.ping_with_rid("p1").unwrap();
    let Some(Ok(PolyBoltEvent::Pong(pong))) = next(&mut channel).await else {
        panic!("expected a pong");
    };
    assert_eq!(pong.rid.as_deref(), Some("p1"));

    assert!(next(&mut channel).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn enforces_documented_limits_locally() {
    let (url, server) = serve("/ws", |mut socket| async move {
        drain(&mut socket).await;
    })
    .await;
    let mut channel = connect(&url).await;

    // Malformed requests never reach the server.
    let err = channel
        .subscribe(PolyBoltSubscription::price_polymarket(Vec::<String>::new()))
        .unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "subscriptions"));
    let err = channel
        .subscribe(PolyBoltSubscription::price_polymarket(["0xabc"]))
        .unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "asset_id"));
    let err = channel.ping_with_rid("r".repeat(70_000)).unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "frame"));

    // At most 64 active subscriptions; duplicates (after leading-zero normalization) count
    // once.
    let first: Vec<String> = (1..=64).map(|i| i.to_string()).collect();
    channel
        .subscribe(PolyBoltSubscription::price_polymarket(&first))
        .unwrap();
    channel
        .subscribe(PolyBoltSubscription::price_polymarket(["0064", "1"]))
        .unwrap();
    assert_eq!(channel.active_subscriptions(), 64);
    let err = channel
        .subscribe(PolyBoltSubscription::price_polymarket(["65"]))
        .unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "subscriptions"));
    channel
        .unsubscribe(PolyBoltSubscription::price_polymarket(["64"]))
        .unwrap();
    channel
        .subscribe(PolyBoltSubscription::price_polymarket(["65"]))
        .unwrap();
    assert_eq!(channel.active_subscriptions(), 64);

    // At most 20 subscribe/unsubscribe frames per second (4 sent so far).
    for _ in 0..16 {
        channel
            .unsubscribe(PolyBoltSubscription::price_polymarket(["999"]))
            .unwrap();
    }
    let err = channel
        .unsubscribe(PolyBoltSubscription::price_polymarket(["999"]))
        .unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "op"));
    // Pings are not rate limited.
    channel.ping().unwrap();

    drop(channel);
    server.await.unwrap();
}

#[tokio::test]
async fn policy_close_is_reported_with_its_code() {
    let (url, server) = serve("/ws", |mut socket| async move {
        recv_json(&mut socket).await;
        send(
            &mut socket,
            r#"{"op":"error","code":"sub_limit","rid":"s1"}"#,
        )
        .await;
        close(&mut socket, 4008, "policy violation").await;
    })
    .await;

    let mut channel = connect(&url).await;
    channel
        .subscribe(PolyBoltSubscription::price_polymarket([ASSET]).rid("s1"))
        .unwrap();
    let Some(Ok(PolyBoltEvent::Error(error))) = next(&mut channel).await else {
        panic!("expected an error ack");
    };
    assert!(error.code.closes_connection());

    let Some(Err(err)) = next(&mut channel).await else {
        panic!("expected a close error");
    };
    assert_eq!(
        PolyBoltCloseCode::from_error(&err),
        Some(PolyBoltCloseCode::PolicyViolation)
    );
    let Error::WebSocket(ws) = &err else {
        panic!("expected a websocket error, got {err:?}");
    };
    assert_eq!(ws.close_reason(), Some("policy violation"));
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    server.await.unwrap();
}
