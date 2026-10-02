//! Private helpers shared by the Gamma endpoint modules.

use chrono::{DateTime, SecondsFormat, Utc};
use polyoxide_core::{Query, Result, ValidationError};

/// Generates builder setter methods on a request builder whose optional parameters live in
/// a `params` field.
///
/// Each entry is `/// docs  name => field: [into|many] Type;` (or `name: ...` when the
/// setter and the field share a name):
///
/// - `name: Type` stores `Some(value)` in `self.params.name: Option<Type>`,
/// - `name: into Type` takes `impl Into<Type>` and stores it in
///   `self.params.name: Option<Type>`,
/// - `name: many Type` takes any iterator of `impl Into<Type>` and replaces
///   `self.params.name: Vec<Type>` (sent as repeated query keys).
macro_rules! setters {
    () => {};
    ($(#[$meta:meta])* $name:ident: $($rest:tt)*) => {
        $crate::gamma::util::setters!($(#[$meta])* $name => $name: $($rest)*);
    };
    ($(#[$meta:meta])* $name:ident => $field:ident: into $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        pub fn $name(mut self, $name: impl Into<$ty>) -> Self {
            self.params.$field = Some($name.into());
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
    ($(#[$meta:meta])* $name:ident => $field:ident: many $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        ///
        /// Replaces any previously set values; an empty iterator removes the filter.
        pub fn $name<I>(mut self, $name: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<$ty>,
        {
            self.params.$field = $name.into_iter().map(Into::into).collect();
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
    ($(#[$meta:meta])* $name:ident => $field:ident: $ty:ty; $($rest:tt)*) => {
        $(#[$meta])*
        pub fn $name(mut self, $name: $ty) -> Self {
            self.params.$field = Some($name);
            self
        }
        $crate::gamma::util::setters!($($rest)*);
    };
}
pub(crate) use setters;

/// The four standard offset-pagination parameters.
#[derive(Debug, Clone, Default)]
pub(crate) struct PageParams {
    pub(crate) limit: Option<u32>,
    pub(crate) offset: Option<u32>,
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

impl<Id: AsRef<str>> Lookup<Id> {
    /// Checks the id (an `integer` path parameter, `id`) or the slug (`slug`) and returns
    /// the path `/{collection}/{id}/{suffix...}` or `/{collection}/slug/{slug}/{suffix...}`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`](crate::Error::Validation) if the id is not an integer
    /// or the slug is empty or a dot segment.
    pub(crate) fn path<'a>(
        &'a self,
        collection: &'a str,
        suffix: &[&'a str],
    ) -> Result<Vec<&'a str>> {
        let mut segments = vec![collection];
        match self {
            Self::Id(id) => {
                check_integer_id("id", id.as_ref())?;
                segments.push(id.as_ref());
            }
            Self::Slug(slug) => {
                check_path_text("slug", slug)?;
                segments.extend(["slug", slug.as_str()]);
            }
        }
        segments.extend_from_slice(suffix);
        Ok(segments)
    }
}

/// Formats a date-time query parameter (`format: date-time`) as RFC 3339 in UTC, e.g.
/// `2024-01-02T03:04:05Z`.
pub(crate) fn rfc3339(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// Checks an identifier that the spec types as `integer`: one or more ASCII digits.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) naming `parameter` otherwise.
pub(crate) fn check_integer_id(parameter: &'static str, id: &str) -> Result<()> {
    if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(());
    }
    Err(ValidationError::new(
        parameter,
        format!("must be an integer id (one or more ASCII digits), got {id:?}"),
    )
    .into())
}

/// Checks every identifier of an `integer`-typed list parameter (see [`check_integer_id`]).
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) for the first invalid id.
pub(crate) fn check_integer_ids<T: AsRef<str>>(parameter: &'static str, ids: &[T]) -> Result<()> {
    ids.iter()
        .try_for_each(|id| check_integer_id(parameter, id.as_ref()))
}

/// Parses an identifier that the API types as `integer` (e.g. in a JSON request body).
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `id` is not one or more ASCII
/// digits, or does not fit in an `i64`.
pub(crate) fn integer_id(parameter: &'static str, id: &str) -> Result<i64> {
    check_integer_id(parameter, id)?;
    id.parse::<i64>().map_err(|_| {
        ValidationError::new(parameter, format!("integer id {id:?} is out of range")).into()
    })
}

/// The canonical form of an id accepted by [`check_integer_id`], for comparisons: without
/// leading zeros (`"007"` and `"7"` are the same integer).
pub(crate) fn canonical_integer(id: &str) -> &str {
    let trimmed = id.trim_start_matches('0');
    if trimmed.is_empty() && !id.is_empty() {
        "0"
    } else {
        trimmed
    }
}

/// Checks a free-text path parameter (a slug or an address): it must be non-empty and not
/// `.` or `..`, which would change the request path.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) naming `parameter` otherwise.
pub(crate) fn check_path_text(parameter: &'static str, value: &str) -> Result<()> {
    let problem = match value {
        "" => "must not be empty",
        "." | ".." => "must not be `.` or `..`",
        _ => return Ok(()),
    };
    Err(ValidationError::new(parameter, format!("{problem}, got {value:?}")).into())
}

/// Checks the documented `limit` range (`1..=100`) of the keyset endpoints.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `limit` is out of range.
pub(crate) fn validate_keyset_limit(limit: Option<u32>) -> Result<()> {
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

/// Largest `offset` the server accepts on `GET /markets`, `GET /events` and
/// `GET /events/pagination` (observed live; not in the spec).
pub(crate) const MAX_OFFSET: u64 = 2000;
/// Largest `offset` the server accepts on `GET /comments` and
/// `GET /comments/user_address/{user_address}` (observed live; not in the spec).
pub(crate) const MAX_COMMENTS_OFFSET: u64 = 200;

/// Checks an `offset` against the largest value the server accepts on a route. `deeper`
/// names the alternative for walking further.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) (parameter `offset`) if `offset`
/// exceeds `max`.
pub(crate) fn validate_offset(offset: Option<u64>, max: u64, deeper: &str) -> Result<()> {
    match offset {
        Some(offset) if offset > max => Err(ValidationError::new(
            "offset",
            format!(
                "must be at most {max} (the server rejects larger offsets), got {offset}; {deeper}"
            ),
        )
        .into()),
        _ => Ok(()),
    }
}

/// Smallest `limit` accepted by the keyset endpoints.
pub(crate) const KEYSET_MIN_LIMIT: u32 = 1;
/// Largest `limit` accepted by the keyset endpoints.
pub(crate) const KEYSET_MAX_LIMIT: u32 = 100;

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    /// The parameter named by a validation error, or `""` for any other outcome.
    fn parameter_of<T>(result: Result<T>) -> String {
        match result {
            Err(crate::Error::Validation(v)) => v.parameter().to_owned(),
            _ => String::new(),
        }
    }

    #[test]
    fn formats_rfc3339() {
        let at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(rfc3339(&at), "2024-01-02T03:04:05Z");
    }

    #[test]
    fn validates_offset_cap() {
        assert!(validate_offset(None, MAX_OFFSET, "x").is_ok());
        assert!(validate_offset(Some(2000), MAX_OFFSET, "x").is_ok());
        let err = validate_offset(Some(2001), MAX_OFFSET, "use the keyset listing").unwrap_err();
        let crate::Error::Validation(v) = &err else {
            panic!("expected a validation error, got {err:?}")
        };
        assert_eq!(v.parameter(), "offset");
        assert!(err.to_string().contains("2001"), "{err}");
        assert!(err.to_string().contains("keyset"), "{err}");
        assert!(validate_offset(Some(200), MAX_COMMENTS_OFFSET, "x").is_ok());
        assert!(validate_offset(Some(201), MAX_COMMENTS_OFFSET, "x").is_err());
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
    fn integer_ids_are_ascii_digits() {
        for ok in ["0", "42", "007", "18446744073709551616"] {
            assert!(check_integer_id("id", ok).is_ok(), "{ok}");
        }
        for bad in [
            "", " 42", "42 ", "-1", "+1", "4.2", "1e3", "abc", "keyset", ".", "..", "٣",
        ] {
            assert_eq!(parameter_of(check_integer_id("id", bad)), "id", "{bad:?}");
        }
        assert!(check_integer_ids("tag_id", &["1", "2"]).is_ok());
        assert_eq!(
            parameter_of(check_integer_ids("tag_id", &["1", "x"])),
            "tag_id"
        );
    }

    #[test]
    fn parses_integer_ids() {
        assert_eq!(integer_id("id", "42").unwrap(), 42);
        assert_eq!(integer_id("id", "007").unwrap(), 7);
        assert_eq!(parameter_of(integer_id("id", "abc")), "id");
        assert_eq!(parameter_of(integer_id("id", " 42")), "id");
        assert_eq!(parameter_of(integer_id("tagId", "-5")), "tagId");
        assert_eq!(parameter_of(integer_id("id", "99999999999999999999")), "id");
    }

    #[test]
    fn canonical_integers_drop_leading_zeros() {
        assert_eq!(canonical_integer("007"), "7");
        assert_eq!(canonical_integer("7"), "7");
        assert_eq!(canonical_integer("000"), "0");
        assert_eq!(canonical_integer("0"), "0");
        assert_eq!(canonical_integer("100"), "100");
    }

    #[test]
    fn path_text_rejects_empty_and_dot_segments() {
        assert!(check_path_text("slug", "us-election").is_ok());
        assert!(check_path_text("slug", "a/b").is_ok());
        assert!(check_path_text("slug", "...").is_ok());
        for bad in ["", ".", ".."] {
            assert_eq!(parameter_of(check_path_text("slug", bad)), "slug");
        }
    }

    #[test]
    fn lookup_paths_are_validated() {
        let by_id: Lookup<String> = Lookup::Id("12".to_owned());
        assert_eq!(
            by_id.path("markets", &["tags"]).unwrap(),
            vec!["markets", "12", "tags"]
        );
        let by_slug: Lookup<String> = Lookup::Slug("will-it-rain".to_owned());
        assert_eq!(
            by_slug.path("markets", &[]).unwrap(),
            vec!["markets", "slug", "will-it-rain"]
        );
        let bad_id: Lookup<String> = Lookup::Id("keyset".to_owned());
        assert_eq!(parameter_of(bad_id.path("events", &[])), "id");
        let bad_slug: Lookup<String> = Lookup::Slug("..".to_owned());
        assert_eq!(parameter_of(bad_slug.path("events", &[])), "slug");
    }
}
