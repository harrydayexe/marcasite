//! Shared transport, configuration and error types for [`polyoxide`].
//!
//! **This crate is an implementation detail of `polyoxide`. Its API has no stability
//! guarantees of its own: any release may change or remove any item, whatever its version
//! number says.** Depend on the `polyoxide` crate instead, which re-exports everything
//! users need from here (errors, the HTTP client and its settings, identifier types,
//! [`Paginated`](pagination::Paginated), ...); only those re-exports follow semantic
//! versioning, as part of `polyoxide`'s API. `polyoxide` requires the exact version of this
//! crate it was released with.
//!
//! The building blocks in this crate are public so that the service clients in `polyoxide`
//! can use them, and are documented for maintainers.
//!
//! # Contents
//!
//! - [`Error`] / [`Result`]: the single error type returned by every fallible operation.
//! - [`HttpClient`] / [`HttpClientBuilder`]: the shared, connection-pooled HTTP client and its
//!   settings (timeouts, user agent, [`RetryPolicy`]).
//! - [`transport`]: per-service request execution, error-body parsing and response decoding.
//! - [`Query`]: an ordered query-string builder supporting repeated and comma-separated keys.
//! - [`pagination`]: helpers that turn cursor/offset pagination into a [`Stream`].
//! - [`types`]: identifier newtypes shared by several services.
//! - [`serde_util`]: (de)serialization helpers for the wire formats the APIs use.
//! - [`validate`]: client-side checks of documented parameter patterns (hex ids, addresses).
//! - `ws` (feature `ws`): a WebSocket connection driver used by the streaming channels.
//!
//! # Logging
//!
//! This crate emits diagnostics via [`tracing`]. It never installs a subscriber; that is
//! left to the application. Requests, retries and failures are logged at `DEBUG` (method,
//! path, status, latency), and response bodies (truncated) and WebSocket frames at `TRACE`
//! (outgoing frames by length only). Failures that are returned to the caller (failed
//! requests, decode failures, terminated WebSocket connections) are never logged above
//! `DEBUG`, so they are not reported twice.
//!
//! Spans: `polyoxide.request` (one per HTTP request including retries; records `status`,
//! `attempts`, `elapsed_ms`, `trace_id` on completion) and `polyoxide.ws` (one per WebSocket
//! connection, also wrapping the spawned driver task).
//!
//! [`polyoxide`]: https://docs.rs/polyoxide
//! [`Stream`]: futures_core::Stream
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod config;
pub mod error;
pub mod pagination;
pub mod query;
pub mod serde_util;
pub mod transport;
pub mod types;
pub mod validate;
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub mod ws;

#[cfg(test)]
mod test_tracing;

pub use config::{HttpClient, HttpClientBuilder, RetryPolicy};
pub use error::{
    ApiError, ConfigError, DecodeError, Error, Result, Service, TransportError, ValidationError,
};
#[cfg(feature = "ws")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub use error::{WebSocketError, WebSocketErrorKind};
pub use query::Query;
pub use transport::Transport;

/// Re-export of [`http::Method`], used in error context.
pub use http::Method;
/// Re-export of [`http::StatusCode`], used in [`ApiError`].
pub use http::StatusCode;
/// Re-export of [`url::Url`], used for base URLs.
pub use url::Url;

#[doc(hidden)]
pub mod __private {
    //! Implementation details of the exported macros. Not part of the public API.

    pub use serde;

    /// Deserializes an identifier from a JSON string or integer.
    ///
    /// # Errors
    ///
    /// Fails for any other JSON type.
    pub fn deserialize_string_id<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<String, D::Error> {
        struct IdVisitor;

        impl serde::de::Visitor<'_> for IdVisitor {
            type Value = String;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an identifier string or integer")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<String, E> {
                Ok(v.to_owned())
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<String, E> {
                Ok(v)
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<String, E> {
                Ok(v.to_string())
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<String, E> {
                Ok(v.to_string())
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}
