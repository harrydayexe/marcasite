//! Shared plumbing for the channel types: decoding text frames into typed events and
//! exposing them as a [`Stream`].
//!
//! [`Stream`]: futures_core::Stream

use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;

use polyoxide_core::{
    ConfigError, Error, Result, Service, WebSocketError, WebSocketErrorKind,
    ws::{WsConfig, WsConnection, parse_ws_url},
};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Maximum number of bytes of a frame quoted in a decode error message.
const MAX_SNIPPET_BYTES: usize = 256;

/// The idle timeout chosen on a channel builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdleTimeout {
    /// The channel's default, derived from its documented heartbeat cadence.
    Default,
    /// No idle timeout.
    Disabled,
    /// A custom timeout.
    After(Duration),
}

/// Connection settings shared by every channel builder.
#[derive(Debug, Clone)]
pub(crate) struct ConnectOptions {
    pub(crate) url: Option<String>,
    pub(crate) buffer: Option<usize>,
    pub(crate) connect_timeout: Option<Duration>,
    pub(crate) idle_timeout: IdleTimeout,
}

impl ConnectOptions {
    pub(crate) const fn new() -> Self {
        Self {
            url: None,
            buffer: None,
            connect_timeout: None,
            idle_timeout: IdleTimeout::Default,
        }
    }

    /// Builds the driver configuration for `service`, using `default_url` unless a URL
    /// was set and `default_idle_timeout` unless an idle timeout was set or disabled.
    ///
    /// Fails with [`Error::Config`] for an invalid URL or a zero idle timeout.
    pub(crate) fn config(
        &self,
        service: Service,
        default_url: &str,
        default_idle_timeout: Duration,
    ) -> Result<WsConfig> {
        let url = parse_ws_url(self.url.as_deref().unwrap_or(default_url))?;
        let mut config = WsConfig::new(service, url);
        if let Some(buffer) = self.buffer {
            config = config.buffer(buffer);
        }
        if let Some(timeout) = self.connect_timeout {
            config = config.connect_timeout(timeout);
        }
        match self.idle_timeout {
            IdleTimeout::Default => config = config.idle_timeout(default_idle_timeout),
            IdleTimeout::After(timeout) => {
                if timeout.is_zero() {
                    return Err(
                        ConfigError::new("the idle timeout must be greater than zero").into(),
                    );
                }
                config = config.idle_timeout(timeout);
            }
            IdleTimeout::Disabled => {}
        }
        Ok(config)
    }
}

/// Rejects a zero heartbeat interval, which the driver cannot schedule.
pub(crate) fn check_interval(interval: Duration) -> Result<()> {
    if interval.is_zero() {
        return Err(ConfigError::new("the heartbeat interval must be greater than zero").into());
    }
    Ok(())
}

/// A connection plus a queue of decoded events not yet handed out.
///
/// A text frame may decode to several events (a JSON array), so decoded events are queued
/// and drained before the next frame is read.
#[derive(Debug)]
pub(crate) struct EventStream<E> {
    conn: WsConnection,
    pending: VecDeque<Result<E>>,
    terminated: bool,
}

impl<E: DeserializeOwned> EventStream<E> {
    pub(crate) fn new(conn: WsConnection) -> Self {
        Self {
            conn,
            pending: VecDeque::new(),
            terminated: false,
        }
    }

    pub(crate) fn connection(&self) -> &WsConnection {
        &self.conn
    }

    pub(crate) fn is_terminated(&self) -> bool {
        self.terminated
    }

    pub(crate) fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<E>>> {
        loop {
            if let Some(item) = self.pending.pop_front() {
                return Poll::Ready(Some(item));
            }
            if self.terminated {
                return Poll::Ready(None);
            }
            match Pin::new(&mut self.conn).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    self.terminated = true;
                    return Poll::Ready(None);
                }
                // A connection error is always the driver's last item.
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(Some(Ok(text))) => {
                    decode_frame(self.conn.service(), &text, &mut self.pending);
                }
            }
        }
    }
}

/// Decodes one text frame into zero or more events, appending them to `out`.
///
/// - A frame that is not JSON is passed to `E` as a JSON string, so that an undocumented
///   plain-text message becomes the event type's catch-all variant instead of an error.
/// - A JSON array is flattened: each element is decoded on its own.
/// - An element that fails to decode yields a non-fatal `Err(Error::WebSocket)` of kind
///   [`WebSocketErrorKind::Decode`] whose source is the `serde_json` error; the other
///   elements are unaffected.
pub(crate) fn decode_frame<E: DeserializeOwned>(
    service: Service,
    text: &str,
    out: &mut VecDeque<Result<E>>,
) {
    let value = serde_json::from_str::<Value>(text).unwrap_or_else(|_| {
        tracing::debug!(service = %service, "received a non-JSON text frame");
        Value::String(text.to_owned())
    });
    match value {
        Value::Array(items) => {
            for item in items {
                out.push_back(decode_value(service, item));
            }
        }
        other => out.push_back(decode_value(service, other)),
    }
}

fn decode_value<E: DeserializeOwned>(service: Service, value: Value) -> Result<E> {
    E::deserialize(&value).map_err(|source| {
        let snippet = snippet(&value);
        tracing::debug!(service = %service, error = %source, message = %snippet, "failed to decode message");
        Error::WebSocket(Box::new(
            WebSocketError::new(
                service,
                WebSocketErrorKind::Decode,
                format!("failed to decode message `{snippet}`"),
            )
            .with_source(source),
        ))
    })
}

/// The compact JSON of `value`, truncated on a char boundary.
fn snippet(value: &Value) -> String {
    let mut text = value.to_string();
    if text.len() > MAX_SNIPPET_BYTES {
        let mut end = MAX_SNIPPET_BYTES;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        text.push('…');
    }
    text
}

/// The string value of `key` in `value`, if `value` is an object holding a string there.
pub(crate) fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Probe {
        Number { n: u32 },
        Other(Value),
    }

    #[test]
    fn flattens_arrays_and_wraps_plain_text() {
        let mut out = VecDeque::new();
        decode_frame::<Probe>(Service::MarketChannel, r#"[{"n":1},{"n":2}]"#, &mut out);
        decode_frame::<Probe>(Service::MarketChannel, "hello", &mut out);
        decode_frame::<Probe>(Service::MarketChannel, "[]", &mut out);
        let decoded: Vec<_> = out.into_iter().map(Result::unwrap).collect();
        assert_eq!(
            decoded,
            vec![
                Probe::Number { n: 1 },
                Probe::Number { n: 2 },
                Probe::Other(Value::String("hello".to_owned())),
            ]
        );
    }

    #[test]
    fn decode_errors_carry_a_snippet_and_source() {
        let mut out = VecDeque::new();
        decode_frame::<u32>(
            Service::SportsChannel,
            &format!("\"{}\"", "x".repeat(400)),
            &mut out,
        );
        let err = out.pop_front().unwrap().unwrap_err();
        let Error::WebSocket(ws) = &err else {
            panic!("unexpected error {err:?}");
        };
        assert_eq!(ws.kind(), WebSocketErrorKind::Decode);
        assert!(ws.message().starts_with("failed to decode message `\"xxx"));
        assert!(ws.message().ends_with("…`"));
        assert!(std::error::Error::source(ws.as_ref()).is_some());
    }

    #[test]
    fn idle_timeout_choices() {
        let default = Duration::from_secs(15);
        let mut options = ConnectOptions::new();
        assert!(
            options
                .config(Service::SportsChannel, "ws://127.0.0.1:1", default)
                .is_ok()
        );
        options.idle_timeout = IdleTimeout::Disabled;
        assert!(
            options
                .config(Service::SportsChannel, "ws://127.0.0.1:1", default)
                .is_ok()
        );
        options.idle_timeout = IdleTimeout::After(Duration::ZERO);
        assert!(matches!(
            options.config(Service::SportsChannel, "ws://127.0.0.1:1", default),
            Err(Error::Config(_))
        ));
    }

    #[test]
    fn zero_interval_is_rejected() {
        assert!(matches!(
            check_interval(Duration::ZERO),
            Err(Error::Config(_))
        ));
        assert!(check_interval(Duration::from_millis(1)).is_ok());
    }
}
