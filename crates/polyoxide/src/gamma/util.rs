//! Private helpers shared by the Gamma endpoint modules.

use chrono::{DateTime, SecondsFormat, Utc};
use polyoxide_core::{Query, Result, ValidationError};

/// Generates builder setter methods on a request builder whose optional parameters live in
/// a `params` field.
///
/// Each entry is `/// docs  name: [into|many] Type;`:
///
/// - `name: Type` stores `Some(value)` in `self.params.name: Option<Type>`,
/// - `name: into Type` takes `impl Into<Type>` and stores it in
///   `self.params.name: Option<Type>`,
/// - `name: many Type` takes any iterator of `impl Into<Type>` and replaces
///   `self.params.name: Vec<Type>` (sent as repeated query keys).
macro_rules! setters {
    () => {};
    ($(#[$meta:meta])* $name:ident: into $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        pub fn $name(mut self, $name: impl Into<$ty>) -> Self {
            self.params.$name = Some($name.into());
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
    ($(#[$meta:meta])* $name:ident: many $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        ///
        /// Replaces any previously set values; an empty iterator removes the filter.
        pub fn $name<I>(mut self, $name: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<$ty>,
        {
            self.params.$name = $name.into_iter().map(Into::into).collect();
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
    ($(#[$meta:meta])* $name:ident: $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        pub fn $name(mut self, $name: $ty) -> Self {
            self.params.$name = Some($name);
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
}
pub(crate) use setters;

/// The four standard offset-pagination parameters.
#[derive(Debug, Clone, Default)]
pub(crate) struct PageParams {
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) order: Option<String>,
    pub(crate) ascending: Option<bool>,
}

impl PageParams {
    pub(crate) fn query(&self, offset: Option<u64>) -> Query {
        let mut q = Query::new();
        q.push_opt("limit", self.limit)
            .push_opt("offset", offset)
            .push_opt("order", self.order.as_deref())
            .push_opt("ascending", self.ascending);
        q
    }
}

/// Looks an entity up either by its id or by its slug.
#[derive(Debug, Clone)]
pub(crate) enum Lookup<Id> {
    /// `/{collection}/{id}`.
    Id(Id),
    /// `/{collection}/slug/{slug}`.
    Slug(String),
}

/// Formats a date-time query parameter (`format: date-time`) as RFC 3339 in UTC, e.g.
/// `2024-01-02T03:04:05Z`.
pub(crate) fn rfc3339(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// Parses an identifier that the API types as `integer` (e.g. in a JSON request body).
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `id` is not an integer.
pub(crate) fn integer_id(parameter: &'static str, id: &str) -> Result<i64> {
    id.trim().parse::<i64>().map_err(|_| {
        ValidationError::new(parameter, format!("expected an integer id, got {id:?}")).into()
    })
}

/// Checks the documented `limit` range (`1..=100`) of the keyset endpoints.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is out of range.
pub(crate) fn validate_keyset_limit(limit: Option<u64>) -> Result<()> {
    match limit {
        Some(limit) if !(KEYSET_MIN_LIMIT..=KEYSET_MAX_LIMIT).contains(&limit) => {
            Err(ValidationError::new(
                "limit",
                format!("must be between {KEYSET_MIN_LIMIT} and {KEYSET_MAX_LIMIT}, got {limit}"),
            )
            .into())
        }
        _ => Ok(()),
    }
}

/// Smallest `limit` accepted by the keyset endpoints.
pub(crate) const KEYSET_MIN_LIMIT: u64 = 1;
/// Largest `limit` accepted by the keyset endpoints.
pub(crate) const KEYSET_MAX_LIMIT: u64 = 100;

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    #[test]
    fn formats_rfc3339() {
        let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(rfc3339(&at), "2024-01-02T03:04:05Z");
    }

    #[test]
    fn validates_keyset_limit() {
        assert!(validate_keyset_limit(None).is_ok());
        assert!(validate_keyset_limit(Some(1)).is_ok());
        assert!(validate_keyset_limit(Some(100)).is_ok());
        assert!(validate_keyset_limit(Some(0)).is_err());
        assert!(validate_keyset_limit(Some(101)).is_err());
    }

    #[test]
    fn parses_integer_ids() {
        assert_eq!(integer_id("id", "42").unwrap(), 42);
        assert!(integer_id("id", "abc").is_err());
    }
}
