//! Private helpers shared by the Gamma endpoint modules.

use chrono::{DateTime, SecondsFormat, Utc};
use polyoxide_core::{Query, Result, ValidationError, types::Address};

/// Serde helper for `type: number` fields modelled as `Option<Decimal>`.
///
/// Deserializes from a JSON number (or a numeric string, which `Decimal` also accepts) and
/// serializes back to a JSON number, like the wire format: integral values as integers,
/// others as floats.
///
/// Use with `#[serde(default, with = "crate::gamma::util::number_option")]`.
pub(crate) mod number_option {
    use rust_decimal::{Decimal, prelude::ToPrimitive as _};
    use serde::{Deserialize, Deserializer, Serializer, ser::Error as _};

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<Decimal>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let Some(value) = value else {
            return serializer.serialize_none();
        };
        if value.fract().is_zero()
            && let Some(integer) = value.to_i64()
        {
            return serializer.serialize_some(&integer);
        }
        match value.to_f64() {
            Some(float) => serializer.serialize_some(&float),
            None => Err(S::Error::custom(format!(
                "decimal {value} cannot be represented as a JSON number"
            ))),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Decimal>, D::Error> {
        Option::<Decimal>::deserialize(deserializer)
    }
}

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

/// Serde helper for optional identifier newtypes whose wire type is `integer`.
///
/// Deserializes through the newtype (which accepts a JSON integer or string) and serializes
/// back to a JSON integer when the id is numeric, so a round trip preserves the wire format.
///
/// Use with `#[serde(default, with = "crate::gamma::util::integer_id_option")]`.
pub(crate) mod integer_id_option {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<T: AsRef<str>, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value.as_ref().map(AsRef::as_ref) {
            None => serializer.serialize_none(),
            Some(id) => match id.parse::<i64>() {
                Ok(number) => serializer.serialize_some(&number),
                Err(_) => serializer.serialize_some(id),
            },
        }
    }

    pub(crate) fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        Option::<T>::deserialize(deserializer)
    }
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

/// Checks the documented wallet-address pattern `^0x[a-fA-F0-9]{40}$`.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `address` does not match.
pub(crate) fn validate_address(parameter: &'static str, address: &Address) -> Result<()> {
    let s = address.as_str();
    let valid =
        s.len() == 42 && s.starts_with("0x") && s.bytes().skip(2).all(|b| b.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            parameter,
            format!("must match ^0x[a-fA-F0-9]{{40}}$, got {s:?}"),
        )
        .into())
    }
}

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
    fn validates_addresses() {
        let ok = Address::from("0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b");
        assert!(validate_address("address", &ok).is_ok());
        let upper = Address::from("0x7C3DB723F1D4D8CB9C550095203B686CB11E5C6B");
        assert!(validate_address("address", &upper).is_ok());
        for bad in [
            "",
            "0x",
            "7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6bb",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6g",
            "0X7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
        ] {
            let err = validate_address("address", &Address::from(bad)).unwrap_err();
            assert!(matches!(err, crate::Error::Validation(_)), "{bad}");
        }
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Numbers {
        #[serde(default, with = "number_option")]
        value: Option<rust_decimal::Decimal>,
    }

    #[test]
    fn numbers_roundtrip_as_json_numbers() {
        for json in [
            r#"{"value":3}"#,
            r#"{"value":-2}"#,
            r#"{"value":12.5}"#,
            r#"{"value":0.001}"#,
            r#"{"value":null}"#,
        ] {
            let parsed: Numbers = serde_json::from_str(json).unwrap();
            assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
        }
        let missing: Numbers = serde_json::from_str("{}").unwrap();
        assert_eq!(missing.value, None);
        let from_string: Numbers = serde_json::from_str(r#"{"value":"0.1"}"#).unwrap();
        assert_eq!(from_string.value, Some(rust_decimal::Decimal::new(1, 1)));
        assert!(serde_json::from_str::<Numbers>(r#"{"value":"abc"}"#).is_err());
    }

    #[test]
    fn parses_integer_ids() {
        assert_eq!(integer_id("id", "42").unwrap(), 42);
        assert!(integer_id("id", "abc").is_err());
    }
}
