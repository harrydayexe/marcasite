//! Shared transport, configuration and error types for [`polyoxide`].
//!
//! Most users should depend on the `polyoxide` crate rather than this one directly.
//!
//! # Logging
//!
//! This crate emits diagnostics via [`tracing`]. It never installs a subscriber; that is
//! left to the application.
//!
//! [`polyoxide`]: https://docs.rs/polyoxide
