//! Market channel (`/ws/market`) against a mock server.
//!
//! Message bodies are the examples in `docs/specs/asyncapi.json`.

use std::time::Duration;

use futures_core::stream::FusedStream as _;
use polyoxide::{
    Decimal, Error,
    types::Side,
    ws::{
        MarketChannel, MarketEvent, MarketSubscription, MarketSubscriptionUpdate, SubscriptionLevel,
    },
};
use serde_json::json;

use super::mock::{close, drain, next, recv_json, recv_text, send, serve};

const ASSET_A: &str =
    "65818619657568813474341868652308942079804919287380422192892211131408793125422";
const ASSET_B: &str =
    "71321045679252212594626385532706912750332728571942532289631379312455583992563";

const BOOK: &str = r#"{"event_type":"book","asset_id":"65818619657568813474341868652308942079804919287380422192892211131408793125422","market":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af","bids":[{"price":"0.48","size":"30"},{"price":"0.49","size":"20"},{"price":"0.50","size":"15"}],"asks":[{"price":"0.52","size":"25"},{"price":"0.53","size":"60"},{"price":"0.54","size":"10"}],"timestamp":"1757908892351","hash":"0xabc123..."}"#;
const PRICE_CHANGE: &str = r#"{"event_type":"price_change","market":"0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1","price_changes":[{"asset_id":"71321045679252212594626385532706912750332728571942532289631379312455583992563","price":"0.5","size":"200","side":"BUY","hash":"56621a121a47ed9333273e21c83b660cff37ae50","best_bid":"0.5","best_ask":"1"}],"timestamp":"1757908892351"}"#;
const LAST_TRADE_PRICE: &str = r#"{"event_type":"last_trade_price","asset_id":"114122071509644379678018727908709560226618148003371446110114509806601493071694","market":"0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347","price":"0.456","size":"219.217767","fee_rate_bps":"0","side":"BUY","timestamp":"1750428146322","transaction_hash":"0xeeefffggghhh"}"#;
const TICK_SIZE_CHANGE: &str = r#"{"event_type":"tick_size_change","asset_id":"65818619657568813474341868652308942079804919287380422192892211131408793125422","market":"0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af","old_tick_size":"0.01","new_tick_size":"0.001","timestamp":"1757908892351"}"#;
const MARKET_RESOLVED: &str = r#"{"event_type":"market_resolved","id":"1031769","market":"0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1","assets_ids":["76043073756653678226373981964075571318267289248134717369284518995922789326425","31690934263385727664202099278545688007799199447969475608906331829650099442770"],"winning_asset_id":"76043073756653678226373981964075571318267289248134717369284518995922789326425","winning_outcome":"Yes","timestamp":"1766790415550","tags":["stocks"]}"#;

async fn connect(url: &str, subscription: MarketSubscription) -> MarketChannel {
    MarketChannel::builder()
        .url(url)
        .connect_timeout(Duration::from_secs(5))
        .connect(subscription)
        .await
        .unwrap()
}

#[tokio::test]
async fn subscribes_on_connect_and_decodes_events() {
    let (url, server) = serve("/ws/market", |mut socket| async move {
        // The documented "Subscription with custom features" example, plus a level.
        assert_eq!(
            recv_json(&mut socket).await,
            json!({
                "assets_ids": [ASSET_A],
                "type": "market",
                "custom_feature_enabled": true,
                "initial_dump": true,
                "level": 2
            })
        );
        send(&mut socket, BOOK).await;
        // Several events in one frame are flattened.
        send(&mut socket, &format!("[{PRICE_CHANGE},{LAST_TRADE_PRICE}]")).await;
        send(&mut socket, MARKET_RESOLVED).await;
        // Unknown message types are surfaced, not dropped or failed.
        send(&mut socket, r#"{"event_type":"brand_new","x":1}"#).await;
        send(&mut socket, "SOMETHING ELSE").await;
        close(&mut socket, 1000, "bye").await;
    })
    .await;

    let mut channel = connect(
        &url,
        MarketSubscription::new([ASSET_A])
            .custom_feature_enabled(true)
            .initial_dump(true)
            .level(SubscriptionLevel::Level2),
    )
    .await;

    let Some(Ok(MarketEvent::Book(book))) = next(&mut channel).await else {
        panic!("expected a book");
    };
    assert_eq!(book.asset_id, ASSET_A);
    assert_eq!(book.bids.len(), 3);
    assert_eq!(book.asks[0].price, Decimal::new(52, 2));

    let Some(Ok(MarketEvent::PriceChange(change))) = next(&mut channel).await else {
        panic!("expected a price change");
    };
    assert_eq!(change.price_changes[0].asset_id, ASSET_B);
    assert_eq!(change.price_changes[0].side, Side::Buy);

    let Some(Ok(MarketEvent::LastTradePrice(trade))) = next(&mut channel).await else {
        panic!("expected a last trade price");
    };
    assert_eq!(trade.size, Decimal::new(219_217_767, 6));

    let Some(Ok(MarketEvent::MarketResolved(resolved))) = next(&mut channel).await else {
        panic!("expected a market resolution");
    };
    assert_eq!(resolved.winning_outcome, "Yes");

    let Some(Ok(MarketEvent::Unknown(unknown))) = next(&mut channel).await else {
        panic!("expected an unknown event");
    };
    assert_eq!(unknown, json!({"event_type": "brand_new", "x": 1}));

    let Some(Ok(MarketEvent::Unknown(text))) = next(&mut channel).await else {
        panic!("expected an unknown text frame");
    };
    assert_eq!(text, json!("SOMETHING ELSE"));

    // A normal close ends the stream without an error.
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    server.await.unwrap();
}

#[tokio::test]
async fn sends_ping_heartbeat_and_hides_pong() {
    let (url, server) = serve("/ws/market", |mut socket| async move {
        recv_json(&mut socket).await;
        assert_eq!(recv_text(&mut socket).await, "PING");
        send(&mut socket, "PONG").await;
        assert_eq!(recv_text(&mut socket).await, "PING");
        send(&mut socket, "PONG").await;
        send(&mut socket, TICK_SIZE_CHANGE).await;
        drain(&mut socket).await;
    })
    .await;

    let mut channel = MarketChannel::builder()
        .url(&url)
        .heartbeat_interval(Duration::from_millis(50))
        .connect(MarketSubscription::new([ASSET_A]))
        .await
        .unwrap();

    // The PONG replies are not surfaced: the first event is the tick size change.
    let Some(Ok(MarketEvent::TickSizeChange(tick))) = next(&mut channel).await else {
        panic!("expected a tick size change");
    };
    assert_eq!(tick.new_tick_size, Decimal::new(1, 3));
    channel.close();
    assert!(next(&mut channel).await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn updates_subscriptions_without_reconnecting() {
    let (url, server) = serve("/ws/market", |mut socket| async move {
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"assets_ids": [ASSET_A], "type": "market"})
        );
        // The documented "Subscribe to more assets" / "Unsubscribe from assets" examples.
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"operation": "subscribe", "assets_ids": [ASSET_B]})
        );
        assert_eq!(
            recv_json(&mut socket).await,
            json!({"operation": "unsubscribe", "assets_ids": [ASSET_A]})
        );
        assert_eq!(
            recv_json(&mut socket).await,
            json!({
                "operation": "subscribe",
                "assets_ids": [ASSET_A],
                "level": 3,
                "custom_feature_enabled": true
            })
        );
        drain(&mut socket).await;
    })
    .await;

    let channel = connect(&url, MarketSubscription::new([ASSET_A])).await;
    channel.subscribe([ASSET_B]).unwrap();
    channel.unsubscribe([ASSET_A]).unwrap();
    channel
        .update_subscription(
            MarketSubscriptionUpdate::subscribe([ASSET_A])
                .level(SubscriptionLevel::Level3)
                .custom_feature_enabled(true),
        )
        .unwrap();

    // Empty updates are rejected locally and never sent.
    let err = channel.subscribe(Vec::<String>::new()).unwrap_err();
    assert!(matches!(err, Error::Validation(ref v) if v.parameter() == "assets_ids"));
    let err = channel.unsubscribe(Vec::<String>::new()).unwrap_err();
    assert!(matches!(err, Error::Validation(_)));

    drop(channel);
    server.await.unwrap();
}

#[tokio::test]
async fn decode_errors_do_not_end_the_stream() {
    let (url, server) = serve("/ws/market", |mut socket| async move {
        recv_json(&mut socket).await;
        // A `book` without its required fields.
        send(&mut socket, r#"{"event_type":"book","market":"0x1"}"#).await;
        send(&mut socket, BOOK).await;
        drain(&mut socket).await;
    })
    .await;

    let mut channel = connect(&url, MarketSubscription::new([ASSET_A])).await;
    let Some(Err(Error::WebSocket(err))) = next(&mut channel).await else {
        panic!("expected a decode error");
    };
    assert_eq!(err.service(), polyoxide::Service::MarketChannel);
    assert!(err.message().contains(r#""event_type":"book""#), "{err}");
    assert!(err.to_string().contains("invalid `book` event"), "{err}");
    assert_eq!(err.close_code(), None);
    assert!(!channel.is_terminated());

    assert!(matches!(
        next(&mut channel).await,
        Some(Ok(MarketEvent::Book(_)))
    ));
    drop(channel);
    server.await.unwrap();
}

#[tokio::test]
async fn abnormal_close_yields_a_final_error() {
    let (url, server) = serve("/ws/market", |mut socket| async move {
        recv_json(&mut socket).await;
        send(&mut socket, TICK_SIZE_CHANGE).await;
        close(&mut socket, 1011, "internal error").await;
    })
    .await;

    let mut channel = connect(&url, MarketSubscription::new([ASSET_A])).await;
    assert!(matches!(
        next(&mut channel).await,
        Some(Ok(MarketEvent::TickSizeChange(_)))
    ));
    let Some(Err(Error::WebSocket(err))) = next(&mut channel).await else {
        panic!("expected a close error");
    };
    assert_eq!(err.close_code(), Some(1011));
    assert_eq!(err.close_reason(), Some("internal error"));
    assert!(next(&mut channel).await.is_none());
    assert!(channel.is_terminated());
    // Sending on a terminated connection fails.
    assert!(matches!(
        channel.subscribe([ASSET_B]),
        Err(Error::WebSocket(_))
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn invalid_settings_are_config_errors() {
    let err = MarketChannel::builder()
        .url("ws://127.0.0.1:9")
        .heartbeat_interval(Duration::ZERO)
        .connect(MarketSubscription::new([ASSET_A]))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");

    let err = MarketChannel::builder()
        .url("https://example.com/ws/market")
        .connect(MarketSubscription::new([ASSET_A]))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
}

#[tokio::test]
async fn connection_failure_is_a_websocket_error() {
    // Bind then drop a listener to get a port nothing listens on.
    let port = {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        listener.local_addr().unwrap().port()
    };
    let err = MarketChannel::builder()
        .url(format!("ws://127.0.0.1:{port}/ws/market"))
        .connect(MarketSubscription::new([ASSET_A]))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::WebSocket(_)), "{err:?}");
}
