//! WebSocket live tests: the CLOB market channel, the sports channel and the PolyBolt
//! `price.polymarket` channel.
//!
//! Each channel has two kinds of test:
//!
//! * `*_sdk`: the SDK's channel type, driven for a bounded time with [`tokio::time::timeout`];
//! * `*_raw_drift`: a raw `tokio-tungstenite` connection with the same subscription, so the
//!   exact frames can be fed to the SDK's event type and compared (as for REST bodies) with
//!   [`check_value`]. Frames the SDK keeps as `Unknown(..)` are reported as drift too.
//!
//! The user channel, RFQ gateway and gated PolyBolt channels need credentials and are not
//! covered. Everything here only subscribes; nothing is written.

use std::{collections::BTreeSet, fmt::Debug, io::Write as _, time::Duration};

use chrono::{DateTime, Utc};
use futures_core::Stream;
use futures_util::{SinkExt as _, StreamExt as _};
use polyoxide::{
    Error, WebSocketErrorKind,
    ws::{
        MarketChannel, MarketEvent, MarketSubscription, PolyBoltChannel, PolyBoltErrorCode,
        PolyBoltEvent, PolyBoltSubscription, SportsChannel, SportsEvent,
    },
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::time::{Instant, timeout};
use tokio_tungstenite::tungstenite::Message;

use crate::common::{check_value, sample};

const MARKET_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
const SPORTS_URL: &str = "wss://sports-api.polymarket.com/ws";
const POLYBOLT_URL: &str = "wss://ws-live-v2.polymarket.com/ws";

/// How long a test listens for events.
const WINDOW: Duration = Duration::from_secs(20);
const CONNECT: Duration = Duration::from_secs(15);

// ---------------------------------------------------------------------------------------
// Drift reporting
// ---------------------------------------------------------------------------------------

/// Reports a drift line like `common::check_value` does (stderr, and the report file).
fn drift(label: &str, what: &str) {
    let line = format!("DRIFT {label}: {what}");
    eprintln!("{line}");
    if let Ok(path) = std::env::var("POLYOXIDE_LIVE_REPORT") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        writeln!(file, "{line}").expect("report writes");
    }
    assert!(
        std::env::var("POLYOXIDE_LIVE_STRICT").as_deref() != Ok("1"),
        "{label}: drift from the live API (POLYOXIDE_LIVE_STRICT=1)"
    );
}

/// The shape of `value`: its key names (recursively, first array element only), so
/// that each distinct frame shape is checked once rather than once per frame.
fn skeleton(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut parts: Vec<String> = map
                .iter()
                .map(|(key, child)| format!("{key}{}", skeleton(child)))
                .collect();
            parts.sort();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(items) => items
            .first()
            .map_or_else(|| "[]".to_owned(), |first| format!("[{}]", skeleton(first))),
        _ => String::new(),
    }
}

fn snippet(text: &str) -> String {
    text.chars().take(300).collect()
}

/// A short description of a JSON message for drift lines.
fn describe(value: &Value) -> String {
    let tag = ["event_type", "op", "channel", "type"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .unwrap_or("-");
    let keys: Vec<&str> = value
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default();
    format!("tag `{tag}`, keys {keys:?}")
}

/// Decodes every raw `frames` entry (flattening array frames) as `E`, collecting decode
/// failures, and drift-checks one frame per distinct shape. `unknown` says whether the
/// SDK kept the decoded event as its `Unknown(..)` catch-all. Returns the decoded events.
fn check_frames<E>(label: &str, frames: &[String], unknown: impl Fn(&E) -> bool) -> Vec<E>
where
    E: DeserializeOwned + Serialize + Debug,
{
    let mut events = Vec::new();
    let mut failures = BTreeSet::new();
    let mut shapes = BTreeSet::new();
    let mut unknown_shapes = BTreeSet::new();
    for frame in frames {
        let Ok(json) = serde_json::from_str::<Value>(frame) else {
            drift(label, &format!("frame is not JSON: {}", snippet(frame)));
            continue;
        };
        let items = match json {
            Value::Array(items) => items,
            other => vec![other],
        };
        for item in items {
            match serde_json::from_value::<E>(item.clone()) {
                Ok(event) => {
                    if unknown(&event) {
                        if unknown_shapes.insert(skeleton(&item)) {
                            drift(
                                label,
                                &format!(
                                    "frame kept as Unknown(..) by the SDK: {}",
                                    describe(&item)
                                ),
                            );
                        }
                    } else if shapes.insert(skeleton(&item)) {
                        check_value(label, &item, &event);
                    }
                    events.push(event);
                }
                Err(e) => {
                    failures.insert(format!("{e}; frame: {}", snippet(&item.to_string())));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{label}: frames the SDK failed to decode: {failures:#?}"
    );
    events
}

// ---------------------------------------------------------------------------------------
// Raw connection helpers
// ---------------------------------------------------------------------------------------

/// Frames captured from a raw connection.
#[derive(Debug, Default)]
struct Capture {
    /// Non-heartbeat text frames, in arrival order.
    frames: Vec<String>,
    /// Heartbeat text frames (`PONG`, `ping`, ...).
    heartbeats: Vec<String>,
    /// Why the connection ended early, if it did.
    ended: Option<String>,
}

/// Connects to `url`, sends `hello` (if any), and collects text frames until `want` data
/// frames arrived or `window` elapsed. `heartbeat` is a text frame sent every 10 s;
/// a text frame `ping` is answered with `pong` (sports channel).
async fn capture(
    url: &str,
    hello: Option<String>,
    heartbeat: Option<&str>,
    window: Duration,
    want: usize,
) -> Capture {
    let (mut ws, _) = timeout(CONNECT, tokio_tungstenite::connect_async(url))
        .await
        .unwrap_or_else(|_| panic!("connecting to {url} timed out"))
        .unwrap_or_else(|e| panic!("connecting to {url}: {e}"));
    if let Some(hello) = hello {
        ws.send(Message::text(hello)).await.expect("hello sends");
    }
    let mut out = Capture::default();
    let deadline = tokio::time::sleep(window);
    tokio::pin!(deadline);
    let mut beat = tokio::time::interval(Duration::from_secs(10));
    beat.tick().await;
    while out.frames.len() < want {
        tokio::select! {
            () = &mut deadline => break,
            _ = beat.tick(), if heartbeat.is_some() => {
                ws.send(Message::text(heartbeat.unwrap_or_default())).await.expect("heartbeat sends");
            }
            message = ws.next() => match message {
                None => {
                    out.ended = Some("stream ended without a close frame".to_owned());
                    break;
                }
                Some(Err(e)) => {
                    out.ended = Some(format!("error: {e}"));
                    break;
                }
                Some(Ok(Message::Close(close))) => {
                    out.ended = Some(format!("closed by the server: {close:?}"));
                    break;
                }
                Some(Ok(Message::Text(text))) => match text.as_str() {
                    "ping" => {
                        out.heartbeats.push("ping".to_owned());
                        ws.send(Message::text("pong")).await.expect("pong sends");
                    }
                    "PONG" | "pong" => out.heartbeats.push(text.as_str().to_owned()),
                    _ => out.frames.push(text.as_str().to_owned()),
                },
                Some(Ok(_)) => {}
            }
        }
    }
    let _ = timeout(Duration::from_secs(3), ws.close(None)).await;
    out
}

/// What an SDK channel yielded in a bounded time.
#[derive(Debug)]
struct Collected<E> {
    events: Vec<E>,
    /// Non-fatal decode errors (the stream continues after each).
    decode_errors: Vec<String>,
    /// A terminal error, if the stream failed.
    terminal: Option<Error>,
    /// Whether the stream ended (without error) before `done` or the deadline.
    ended: bool,
}

/// Polls `stream` until `done(&events)` or `window` elapses. Never hangs: every poll is
/// wrapped in a timeout.
async fn collect<S, E>(
    stream: &mut S,
    window: Duration,
    done: impl Fn(&[E]) -> bool,
) -> Collected<E>
where
    S: Stream<Item = polyoxide::Result<E>> + Unpin,
{
    let deadline = Instant::now() + window;
    let mut out = Collected {
        events: Vec::new(),
        decode_errors: Vec::new(),
        terminal: None,
        ended: false,
    };
    while !done(&out.events) {
        let Ok(item) = tokio::time::timeout_at(deadline, stream.next()).await else {
            break;
        };
        match item {
            None => {
                out.ended = true;
                break;
            }
            Some(Ok(event)) => out.events.push(event),
            Some(Err(Error::WebSocket(e))) if e.kind() == WebSocketErrorKind::Decode => {
                out.decode_errors.push(e.to_string());
            }
            Some(Err(e)) => {
                out.terminal = Some(e);
                break;
            }
        }
    }
    out
}

fn within_a_day(time: DateTime<Utc>) -> bool {
    (Utc::now() - time).num_seconds().abs() < 86_400
}

// ---------------------------------------------------------------------------------------
// Market channel
// ---------------------------------------------------------------------------------------

fn market_hello(tokens: &[String], custom_features: bool) -> String {
    serde_json::json!({
        "assets_ids": tokens,
        "type": "market",
        "custom_feature_enabled": custom_features,
    })
    .to_string()
}

/// Subscribes and receives the initial `book` snapshots, then a few more events.
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_sdk() {
    let s = sample().await;
    let mut channel = timeout(
        CONNECT,
        MarketChannel::connect(MarketSubscription::new(s.token_ids.clone())),
    )
    .await
    .expect("connect timed out")
    .unwrap();
    let got = collect(&mut channel, WINDOW, |events| {
        events.len() >= 10 && events.iter().any(|e| matches!(e, MarketEvent::Book(_)))
    })
    .await;
    channel.close();
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(got.decode_errors.is_empty(), "{:#?}", got.decode_errors);
    let books: Vec<_> = got
        .events
        .iter()
        .filter_map(|e| match e {
            MarketEvent::Book(book) => Some(book),
            _ => None,
        })
        .collect();
    assert!(!books.is_empty(), "no book event within {WINDOW:?}");
    for book in &books {
        assert!(s.token_ids.iter().any(|t| t == book.asset_id.as_str()));
        assert_eq!(book.market.as_str(), s.condition_id);
        // The `timestamp` is Unix milliseconds.
        assert!(within_a_day(book.timestamp), "{:?}", book.timestamp);
    }
    // SPEC_DEVIATIONS.md, WebSocket market: the initial snapshots carry `tick_size` (and
    // `last_trade_price`, empty on an empty book).
    assert!(
        books.iter().any(|book| book.tick_size.is_some()),
        "no book carried `tick_size`: {books:#?}"
    );
    for event in &got.events {
        if let MarketEvent::Unknown(value) = event {
            panic!("unexpected Unknown market event: {value}");
        }
    }
}

/// With `custom_feature_enabled` the channel also sends `best_bid_ask` (and `new_market`,
/// `market_resolved`) events.
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_custom_features_sdk() {
    let s = sample().await;
    let mut channel = timeout(
        CONNECT,
        MarketChannel::connect(
            MarketSubscription::new(s.token_ids.clone()).custom_feature_enabled(true),
        ),
    )
    .await
    .expect("connect timed out")
    .unwrap();
    let got = collect(&mut channel, WINDOW, |events| {
        events
            .iter()
            .any(|e| matches!(e, MarketEvent::BestBidAsk(_)))
            && events.iter().any(|e| matches!(e, MarketEvent::Book(_)))
    })
    .await;
    channel.close();
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(got.decode_errors.is_empty(), "{:#?}", got.decode_errors);
    assert!(
        got.events.iter().any(|e| matches!(e, MarketEvent::Book(_))),
        "no book within {WINDOW:?}"
    );
    // `best_bid_ask` only arrives when the top of book moves, which a quiet market may not
    // do within the window; check it when it does.
    for best in got.events.iter().filter_map(|e| match e {
        MarketEvent::BestBidAsk(best) => Some(best),
        _ => None,
    }) {
        // `best_bid_ask.timestamp` is Unix milliseconds too.
        assert!(within_a_day(best.timestamp_millis().unwrap()), "{best:?}");
    }
}

/// SPEC_DEVIATIONS.md, WebSocket market: `new_market` carries the undocumented
/// `taker_base_fee`, `fees_enabled` and `fee_schedule`. New (short-lived) markets are
/// announced to every `custom_feature_enabled` subscriber; if none is announced within the
/// window the test only reports it, because nothing can be pinned then.
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_new_market_fee_fields() {
    let s = sample().await;
    let raw = capture(
        MARKET_URL,
        Some(market_hello(&s.token_ids, true)),
        Some("PING"),
        Duration::from_secs(60),
        usize::MAX,
    )
    .await;
    let new_markets: Vec<Value> = raw
        .frames
        .iter()
        .filter_map(|f| serde_json::from_str::<Value>(f).ok())
        .flat_map(|v| match v {
            Value::Array(items) => items,
            other => vec![other],
        })
        .filter(|v| v["event_type"] == "new_market")
        .collect();
    if new_markets.is_empty() {
        eprintln!("NOTE no new_market event within 60 s: fee fields not pinned this run");
        return;
    }
    for frame in &new_markets {
        assert!(frame["taker_base_fee"].is_string(), "{frame}");
        assert!(frame["fees_enabled"].is_boolean(), "{frame}");
        let schedule = &frame["fee_schedule"];
        for key in ["exponent", "rate", "rebate_rate"] {
            assert!(schedule[key].is_string(), "{key} in {frame}");
        }
        assert!(schedule["taker_only"].is_boolean(), "{frame}");
        let event: MarketEvent = serde_json::from_value(frame.clone()).unwrap();
        let MarketEvent::NewMarket(market) = event else {
            panic!("not a NewMarket: {event:?}");
        };
        assert!(market.taker_base_fee.is_some() && market.fees_enabled.is_some());
        assert!(market.fee_schedule.is_some(), "{market:?}");
        // Every timestamp on the channel is Unix milliseconds.
        assert!(
            within_a_day(market.timestamp_millis().unwrap()),
            "{market:?}"
        );
        check_value(
            "WS market new_market",
            frame,
            &MarketEvent::NewMarket(market),
        );
    }
}

/// Adds a token to a running subscription with `subscribe`, then removes it.
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_update_subscription() {
    let s = sample().await;
    let first = s.token_ids[0].clone();
    let second = s.token_ids[1].clone();
    let mut channel = timeout(
        CONNECT,
        MarketChannel::connect(MarketSubscription::new([first.clone()])),
    )
    .await
    .expect("connect timed out")
    .unwrap();
    let seen = |events: &[MarketEvent], token: &str| {
        events.iter().any(|e| match e {
            MarketEvent::Book(book) => book.asset_id.as_str() == token,
            MarketEvent::PriceChange(change) => change
                .price_changes
                .iter()
                .any(|c| c.asset_id.as_str() == token),
            _ => false,
        })
    };
    let got = collect(&mut channel, WINDOW, |events| seen(events, &first)).await;
    assert!(seen(&got.events, &first), "no event for the first token");
    channel.subscribe([second.clone()]).unwrap();
    let got = collect(&mut channel, WINDOW, |events| seen(events, &second)).await;
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(seen(&got.events, &second), "no event for the second token");
    channel.unsubscribe([second]).unwrap();
    channel.close();
}

/// An unknown or closed token: the server answers with an empty JSON array and the
/// connection stays healthy (the channel does send JSON arrays).
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_unknown_token_stays_healthy() {
    // A token of a long-closed market (Gamma `closed=true` listing).
    let token = "53135072462907880191400140706440867753044989936304433583131786753949599718775";
    let raw = capture(
        MARKET_URL,
        Some(market_hello(&[token.to_owned()], false)),
        Some("PING"),
        Duration::from_secs(12),
        usize::MAX,
    )
    .await;
    eprintln!("NOTE market unknown token: frames={:?}", raw.frames);
    assert!(raw.ended.is_none(), "{:?}", raw.ended);
    assert!(
        raw.frames.iter().all(|f| f.trim() == "[]"),
        "{:?}",
        raw.frames
    );
    assert!(!raw.heartbeats.is_empty(), "no PONG for our PING");

    let mut channel = timeout(
        CONNECT,
        MarketChannel::connect(MarketSubscription::new([token])),
    )
    .await
    .expect("connect timed out")
    .unwrap();
    let got = collect(&mut channel, Duration::from_secs(6), |_| false).await;
    assert!(got.terminal.is_none() && !got.ended, "{got:?}");
    assert!(got.events.is_empty(), "{:?}", got.events);
}

/// Raw frames of the market channel, decoded with `MarketEvent` and drift-checked.
#[tokio::test]
#[ignore = "live network"]
async fn market_channel_raw_drift() {
    let s = sample().await;
    for (label, custom) in [
        ("WS market (default subscription)", false),
        ("WS market (custom_feature_enabled)", true),
    ] {
        let raw = capture(
            MARKET_URL,
            Some(market_hello(&s.token_ids, custom)),
            Some("PING"),
            Duration::from_secs(15),
            400,
        )
        .await;
        assert!(!raw.frames.is_empty(), "{label}: no frames");
        // SPEC_DEVIATIONS.md, WebSocket market: the initial snapshots come as one JSON array
        // with a `book` per token (the spec shows single objects).
        let first: Value = serde_json::from_str(&raw.frames[0]).unwrap();
        let books = first
            .as_array()
            .unwrap_or_else(|| panic!("{label}: the first frame is no longer an array: {first}"));
        assert_eq!(books.len(), s.token_ids.len(), "{label}");
        for book in books {
            assert_eq!(book["event_type"], "book");
            // Undocumented `tick_size` and `last_trade_price` (`""` on an empty book).
            assert!(book["tick_size"].is_string(), "{label}: {book}");
            assert!(book["last_trade_price"].is_string(), "{label}: {book}");
            // An empty side is `[]`, never `""`; timestamps are Unix milliseconds.
            assert!(
                book["bids"].is_array() && book["asks"].is_array(),
                "{label}: {book}"
            );
            let millis: i64 = book["timestamp"].as_str().unwrap().parse().unwrap();
            assert!(
                (Utc::now().timestamp_millis() - millis).abs() < 86_400_000,
                "{label}: {book}"
            );
            // Bids arrive ascending (worst first).
            let bids: Vec<f64> = book["bids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| l["price"].as_str().unwrap().parse().unwrap())
                .collect();
            assert!(
                bids.windows(2).all(|w| w[0] <= w[1]),
                "{label}: bids are no longer ascending: {bids:?}"
            );
        }
        eprintln!("NOTE {label}: {} frames", raw.frames.len());
        let events = check_frames::<MarketEvent>(label, &raw.frames, |e| {
            matches!(e, MarketEvent::Unknown(_))
        });
        assert!(events.iter().any(|e| matches!(e, MarketEvent::Book(_))));
    }
}

// ---------------------------------------------------------------------------------------
// Sports channel
// ---------------------------------------------------------------------------------------

/// Connects and listens. Events may or may not arrive (it depends on live games); either
/// way the connection must stay healthy.
#[tokio::test]
#[ignore = "live network"]
async fn sports_channel_sdk() {
    let mut channel = timeout(CONNECT, SportsChannel::connect())
        .await
        .expect("connect timed out")
        .unwrap();
    let got = collect(&mut channel, WINDOW, |events| events.len() >= 10).await;
    channel.close();
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(!got.ended, "the stream ended without an error");
    let updates: Vec<_> = got
        .events
        .iter()
        .filter_map(|e| match e {
            SportsEvent::Update(update) => Some(update),
            SportsEvent::Unknown(_) => None,
            _ => None,
        })
        .collect();
    eprintln!(
        "NOTE sports: {} events in {WINDOW:?}, {} typed; decode errors: {:?}",
        got.events.len(),
        updates.len(),
        got.decode_errors,
    );
    // SPEC_DEVIATIONS.md, WebSocket sports: every live frame decodes as the typed update.
    let unknown: Vec<_> = got
        .events
        .iter()
        .filter_map(|e| match e {
            SportsEvent::Unknown(value) => Some(describe(value)),
            _ => None,
        })
        .collect();
    assert!(unknown.is_empty(), "events kept as Unknown: {unknown:?}");
    assert!(got.decode_errors.is_empty(), "{:#?}", got.decode_errors);
    for update in updates {
        assert!(
            update.game_id.is_some() || update.metadata_game_id.is_some(),
            "{update:?}"
        );
        assert!(!update.league_abbreviation.is_empty(), "{update:?}");
        if update.ended {
            assert!(!update.live, "{update:?}");
        }
    }
}

/// Raw sports frames, decoded with `SportsEvent` and drift-checked.
#[tokio::test]
#[ignore = "live network"]
async fn sports_channel_raw_drift() {
    let raw = capture(SPORTS_URL, None, None, Duration::from_secs(30), 25).await;
    assert!(raw.ended.is_none(), "{:?}", raw.ended);
    // SPEC_DEVIATIONS.md, WebSocket sports: no text `ping` (the spec says every 5 s).
    assert!(
        raw.heartbeats.is_empty(),
        "the sports channel now sends text heartbeats: {:?}",
        raw.heartbeats
    );
    // Raw frames use the live shape: `gameId` or `metadataGameId` and camelCase keys, never the
    // spec's `slug` / `last_update` / `finished_timestamp`.
    assert!(!raw.frames.is_empty(), "no sports frames in 30 s");
    for frame in &raw.frames {
        let value: Value = serde_json::from_str(frame).unwrap();
        let object = value.as_object().expect("a sports frame is an object");
        for spec_only in ["slug", "last_update", "finished_timestamp"] {
            assert!(
                !object.contains_key(spec_only),
                "sports frames now have `{spec_only}`: update SPEC_DEVIATIONS.md: {frame}"
            );
        }
        assert!(
            object.contains_key("gameId") || object.contains_key("metadataGameId"),
            "{frame}"
        );
        assert!(object.contains_key("leagueAbbreviation"), "{frame}");
    }
    let events = check_frames::<SportsEvent>("WS sports", &raw.frames, |e| {
        matches!(e, SportsEvent::Unknown(_))
    });
    assert!(
        events.iter().all(|e| matches!(e, SportsEvent::Update(_))),
        "frames kept as Unknown"
    );
}

/// SPEC_DEVIATIONS.md, WebSocket sports: the server keeps the connection alive with
/// protocol-level ping frames (every 15 s), not the documented text `ping` every 5 s, so the
/// SDK's idle timeout defaults to three times that interval.
#[tokio::test]
#[ignore = "live network"]
async fn sports_channel_sends_protocol_pings() {
    let (mut ws, _) = timeout(CONNECT, tokio_tungstenite::connect_async(SPORTS_URL))
        .await
        .expect("connect timed out")
        .unwrap();
    let start = Instant::now();
    let deadline = start + Duration::from_secs(25);
    let mut ping_at = None;
    while let Ok(Some(message)) = tokio::time::timeout_at(deadline, ws.next()).await {
        match message.unwrap() {
            Message::Ping(_) => {
                ping_at = Some(start.elapsed());
                break;
            }
            Message::Text(text) => assert_ne!(text.as_str(), "ping", "a text `ping` arrived"),
            _ => {}
        }
    }
    let ping_at = ping_at.expect("no protocol ping within 25 s (it used to arrive every 15 s)");
    assert!(
        ping_at < SportsChannel::DEFAULT_IDLE_TIMEOUT,
        "{ping_at:?} vs the idle timeout"
    );
}

// ---------------------------------------------------------------------------------------
// PolyBolt `price.polymarket`
// ---------------------------------------------------------------------------------------

fn polybolt_hello(tokens: &[String], rid: &str) -> String {
    let subscriptions: Vec<Value> = tokens
        .iter()
        .map(|t| serde_json::json!({"channel": "price.polymarket", "filter": {"asset_id": t}}))
        .collect();
    serde_json::json!({"op": "subscribe", "rid": rid, "subscriptions": subscriptions}).to_string()
}

/// Subscribes (no credentials), receives the acks and a snapshot per token, a few live
/// frames, then pings and unsubscribes.
#[tokio::test]
#[ignore = "live network"]
async fn polybolt_price_polymarket_sdk() {
    let s = sample().await;
    let mut channel = timeout(CONNECT, PolyBoltChannel::connect())
        .await
        .expect("connect timed out")
        .unwrap();
    channel
        .subscribe(PolyBoltSubscription::price_polymarket(s.token_ids.clone()).rid("live-1"))
        .unwrap();
    assert_eq!(channel.active_subscriptions(), 2);
    let got = collect(&mut channel, WINDOW, |events| {
        let snapshots = events
            .iter()
            .filter(|e| matches!(e, PolyBoltEvent::PricePolymarket(p) if p.is_snapshot()))
            .count();
        let live = events
            .iter()
            .filter(|e| matches!(e, PolyBoltEvent::PricePolymarket(p) if !p.is_snapshot()))
            .count();
        snapshots >= 2 && live >= 3
    })
    .await;
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(got.decode_errors.is_empty(), "{:#?}", got.decode_errors);
    assert!(
        got.events.iter().any(
            |e| matches!(e, PolyBoltEvent::Subscribed(ack) if ack.rid.as_deref() == Some("live-1"))
        ),
        "no `subscribed` ack: {:#?}",
        got.events
    );
    let envelopes: Vec<_> = got
        .events
        .iter()
        .filter_map(|e| match e {
            PolyBoltEvent::PricePolymarket(p) => Some(p),
            _ => None,
        })
        .collect();
    assert!(envelopes.iter().any(|p| p.is_snapshot()), "no snapshot");
    for pair in envelopes.windows(2) {
        assert!(pair[1].seq > pair[0].seq, "seq not increasing: {pair:?}");
    }
    let gaps = envelopes
        .windows(2)
        .filter(|pair| pair[1].seq != pair[0].seq + 1)
        .count();
    eprintln!(
        "NOTE polybolt: {} envelopes, {gaps} seq gaps",
        envelopes.len()
    );
    for envelope in &envelopes {
        // The envelope `ts` and the payload `timestamp` are Unix milliseconds.
        assert!(within_a_day(envelope.ts), "{envelope:?}");
        if let Some(quote) = &envelope.payload {
            assert!(s.token_ids.iter().any(|t| t == quote.asset_id.as_str()));
            assert!(within_a_day(quote.timestamp), "{quote:?}");
        }
    }
    for event in &got.events {
        if let PolyBoltEvent::Unknown(value) = event {
            panic!("unexpected Unknown PolyBolt message: {value}");
        }
    }

    // Application-level ping, then unsubscribe from one token.
    channel.ping_with_rid("live-ping").unwrap();
    channel
        .unsubscribe(PolyBoltSubscription::price_polymarket([s.token_ids[0].clone()]).rid("live-2"))
        .unwrap();
    let got = collect(&mut channel, WINDOW, |events| {
        let pong = events
            .iter()
            .any(|e| matches!(e, PolyBoltEvent::Pong(p) if p.rid.as_deref() == Some("live-ping")));
        let unsub = events.iter().any(
            |e| matches!(e, PolyBoltEvent::Unsubscribed(a) if a.rid.as_deref() == Some("live-2")),
        );
        pong && unsub
    })
    .await;
    channel.close();
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(
        got.events
            .iter()
            .any(|e| matches!(e, PolyBoltEvent::Pong(p) if p.rid.as_deref() == Some("live-ping"))),
        "no pong"
    );
    assert!(
        got.events.iter().any(
            |e| matches!(e, PolyBoltEvent::Unsubscribed(a) if a.rid.as_deref() == Some("live-2"))
        ),
        "no unsubscribed ack"
    );
}

/// A token with no recent trade gets an empty-array snapshot payload (a "cold" snapshot).
#[tokio::test]
#[ignore = "live network"]
async fn polybolt_cold_token_snapshot() {
    let mut channel = timeout(CONNECT, PolyBoltChannel::connect())
        .await
        .expect("connect timed out")
        .unwrap();
    channel
        .subscribe(PolyBoltSubscription::price_polymarket(["1"]).rid("cold"))
        .unwrap();
    let got = collect(&mut channel, WINDOW, |events| {
        events
            .iter()
            .any(|e| matches!(e, PolyBoltEvent::PricePolymarket(_)))
    })
    .await;
    channel.close();
    assert!(got.terminal.is_none(), "{:?}", got.terminal);
    assert!(got.decode_errors.is_empty(), "{:#?}", got.decode_errors);
    let snapshot = got.events.iter().find_map(|e| match e {
        PolyBoltEvent::PricePolymarket(p) => Some(p),
        _ => None,
    });
    let snapshot = snapshot.unwrap_or_else(|| panic!("no snapshot: {:#?}", got.events));
    assert!(snapshot.is_snapshot());
    assert!(snapshot.payload.is_none(), "{snapshot:?}");
}

/// Raw PolyBolt frames (acks, snapshots, live frames and an error ack), decoded with
/// `PolyBoltEvent` and drift-checked.
#[tokio::test]
#[ignore = "live network"]
async fn polybolt_raw_drift() {
    let s = sample().await;
    let live = capture(
        POLYBOLT_URL,
        Some(polybolt_hello(&s.token_ids, "raw-1")),
        None,
        Duration::from_secs(15),
        40,
    )
    .await;
    assert!(live.ended.is_none(), "{:?}", live.ended);
    let events = check_frames::<PolyBoltEvent>("WS polybolt price.polymarket", &live.frames, |e| {
        matches!(e, PolyBoltEvent::Unknown(_))
    });
    assert!(
        events
            .iter()
            .any(|e| matches!(e, PolyBoltEvent::PricePolymarket(_)))
    );

    // Cold snapshot, pong and error acks.
    let hello = serde_json::json!({
        "op": "subscribe",
        "rid": "cold",
        "subscriptions": [{"channel": "price.polymarket", "filter": {"asset_id": "1"}}],
    })
    .to_string();
    let cold = capture(POLYBOLT_URL, Some(hello), None, Duration::from_secs(5), 2).await;
    check_frames::<PolyBoltEvent>("WS polybolt cold snapshot", &cold.frames, |e| {
        matches!(e, PolyBoltEvent::Unknown(_))
    });
    let ping = capture(
        POLYBOLT_URL,
        Some(r#"{"op":"ping","rid":"p1"}"#.to_owned()),
        None,
        Duration::from_secs(5),
        1,
    )
    .await;
    check_frames::<PolyBoltEvent>("WS polybolt pong", &ping.frames, |e| {
        matches!(e, PolyBoltEvent::Unknown(_))
    });
    let bad = capture(
        POLYBOLT_URL,
        Some(
            r#"{"op":"subscribe","rid":"bad","subscriptions":[{"channel":"price.polymarket","filter":{"asset_id":"abc"}}]}"#
                .to_owned(),
        ),
        None,
        Duration::from_secs(5),
        1,
    )
    .await;
    let errors = check_frames::<PolyBoltEvent>("WS polybolt error ack", &bad.frames, |e| {
        matches!(e, PolyBoltEvent::Unknown(_))
    });
    assert!(
        errors.iter().any(
            |e| matches!(e, PolyBoltEvent::Error(ack) if ack.code == PolyBoltErrorCode::BadFilter)
        ),
        "{errors:#?}"
    );
}
