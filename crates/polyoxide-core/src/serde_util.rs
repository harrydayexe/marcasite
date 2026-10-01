//! (De)serialization helpers for the wire formats used by the Polymarket APIs.
//!
//! Use these with `#[serde(with = "...")]` / `#[serde(deserialize_with = "...")]` on wire
//! types. Every helper accepts exactly the documented representation, plus a small set of
//! lossless alternatives (e.g. a number sent as a numeric string) so a minor server-side
//! encoding change does not break deserialization of a whole response.
//!
//! Decimal values ([`rust_decimal::Decimal`]) need no helper to deserialize: they accept
//! both JSON strings and JSON numbers. To *serialize* a decimal as a JSON number use
//! `rust_decimal::serde::float`; the default serialization is a string.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer, de::DeserializeOwned, de::Error as _};
use serde_json::Value;

/// Parses a timestamp string leniently.
///
/// Accepts RFC 3339 (`2024-01-02T03:04:05Z`, `...+00:00`, fractional seconds), the same with
/// a space instead of `T`, a short `+00` offset, a missing offset (interpreted as UTC), and
/// a bare date (`2024-01-02`, interpreted as midnight UTC).
///
/// Returns `None` if the string matches none of these.
#[must_use]
pub fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    let normalized = s.replacen(' ', "T", 1);
    if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
        return Some(dt.with_timezone(&Utc));
    }
    // Short offsets such as `+00` / `-05`.
    if let Some(dt) = short_offset(&normalized) {
        return Some(dt);
    }
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(&normalized, format) {
            return Some(naive.and_utc());
        }
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|naive| naive.and_utc())
}

fn short_offset(s: &str) -> Option<DateTime<Utc>> {
    let (head, tail) = s.split_at(s.len().checked_sub(3)?);
    let sign = tail.chars().next()?;
    if !(sign == '+' || sign == '-') || !tail.get(1..)?.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    DateTime::parse_from_rfc3339(&format!("{head}{tail}:00"))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn integer_from_value<E: serde::de::Error>(value: &Value, what: &str) -> Result<i64, E> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| {
                // Accept integral floats such as `1700000000.0`.
                n.as_f64()
                    .filter(|f| f.fract() == 0.0 && f.is_finite())
                    .and_then(|f| format!("{f:.0}").parse().ok())
            })
            .ok_or_else(|| E::custom(format!("invalid {what}: {n}"))),
        Value::String(s) => s
            .trim()
            .parse::<i64>()
            .map_err(|_| E::custom(format!("invalid {what}: {s:?}"))),
        other => Err(E::custom(format!(
            "invalid {what}: expected a number or numeric string, found {other}"
        ))),
    }
}

/// Unix timestamps in **seconds** (JSON number or numeric string) as `DateTime<Utc>`.
///
/// Serializes as a JSON integer.
pub mod timestamp_seconds {
    use super::*;

    /// Serializes as integer seconds.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(value.timestamp())
    }

    /// Deserializes from integer seconds or a numeric string.
    ///
    /// # Errors
    ///
    /// Fails on non-numeric input or an out-of-range timestamp.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let secs = integer_from_value::<D::Error>(&value, "unix timestamp (seconds)")?;
        DateTime::from_timestamp(secs, 0)
            .ok_or_else(|| D::Error::custom(format!("unix timestamp out of range: {secs}")))
    }
}

/// Optional variant of [`timestamp_seconds`]; `null`, a missing field and `""` become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod timestamp_seconds_option {
    use super::*;

    /// Serializes `Some` as integer seconds and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_some(&v.timestamp()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from integer seconds, a numeric string, `""` or `null`.
    ///
    /// # Errors
    ///
    /// Fails on non-numeric input or an out-of-range timestamp.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(None),
            Value::String(s) if s.trim().is_empty() => Ok(None),
            value => {
                let secs = integer_from_value::<D::Error>(&value, "unix timestamp (seconds)")?;
                DateTime::from_timestamp(secs, 0)
                    .map(Some)
                    .ok_or_else(|| D::Error::custom(format!("unix timestamp out of range: {secs}")))
            }
        }
    }
}

/// Unix timestamps in **milliseconds** (JSON number or numeric string) as `DateTime<Utc>`.
///
/// Serializes as a JSON integer.
pub mod timestamp_millis {
    use super::*;

    /// Serializes as integer milliseconds.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(value.timestamp_millis())
    }

    /// Deserializes from integer milliseconds or a numeric string.
    ///
    /// # Errors
    ///
    /// Fails on non-numeric input or an out-of-range timestamp.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let millis = integer_from_value::<D::Error>(&value, "unix timestamp (milliseconds)")?;
        DateTime::from_timestamp_millis(millis)
            .ok_or_else(|| D::Error::custom(format!("unix timestamp out of range: {millis}")))
    }
}

/// Optional variant of [`timestamp_millis`]; `null`, a missing field and `""` become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod timestamp_millis_option {
    use super::*;

    /// Serializes `Some` as integer milliseconds and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_some(&v.timestamp_millis()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from integer milliseconds, a numeric string, `""` or `null`.
    ///
    /// # Errors
    ///
    /// Fails on non-numeric input or an out-of-range timestamp.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(None),
            Value::String(s) if s.trim().is_empty() => Ok(None),
            value => {
                let millis =
                    integer_from_value::<D::Error>(&value, "unix timestamp (milliseconds)")?;
                DateTime::from_timestamp_millis(millis)
                    .map(Some)
                    .ok_or_else(|| {
                        D::Error::custom(format!("unix timestamp out of range: {millis}"))
                    })
            }
        }
    }
}

/// Date-time strings (`format: date-time`) as `DateTime<Utc>`, parsed with
/// [`parse_datetime`]. Serializes as RFC 3339.
pub mod datetime {
    use super::*;

    /// Serializes as an RFC 3339 string.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
    }

    /// Deserializes from a date-time string.
    ///
    /// # Errors
    ///
    /// Fails if the string is not a recognised date-time.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_datetime(&s).ok_or_else(|| D::Error::custom(format!("invalid date-time: {s:?}")))
    }
}

/// Optional variant of [`datetime`]; `null`, a missing field and `""` become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod datetime_option {
    use super::*;

    /// Serializes `Some` as an RFC 3339 string and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => {
                serializer.serialize_some(&v.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from a date-time string, `""` or `null`.
    ///
    /// # Errors
    ///
    /// Fails if a non-empty string is not a recognised date-time.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(None),
            Some(s) if s.trim().is_empty() => Ok(None),
            Some(s) => parse_datetime(&s)
                .map(Some)
                .ok_or_else(|| D::Error::custom(format!("invalid date-time: {s:?}"))),
        }
    }
}

/// A value of type `T` that the API sends as a JSON-encoded **string**
/// (e.g. `"[\"Yes\", \"No\"]"`). Serializes back to a JSON string.
pub mod json_string {
    use super::*;

    /// Serializes `value` to JSON and emits it as a string.
    ///
    /// # Errors
    ///
    /// Fails if `value` cannot be serialized to JSON.
    pub fn serialize<T: serde::Serialize, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let encoded = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&encoded)
    }

    /// Deserializes a string and parses its contents as JSON `T`. A value that is already
    /// a JSON array/object (not a string) is accepted too.
    ///
    /// # Errors
    ///
    /// Fails if the contents are not valid JSON for `T`.
    pub fn deserialize<'de, T: DeserializeOwned, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(s) => serde_json::from_str(&s)
                .map_err(|e| D::Error::custom(format!("invalid JSON-encoded string {s:?}: {e}"))),
            other => T::deserialize(other).map_err(D::Error::custom),
        }
    }
}

/// Optional variant of [`json_string`]; `null`, a missing field and `""` become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod json_string_option {
    use super::*;

    /// Serializes `Some` as a JSON-encoded string and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Fails if the value cannot be serialized to JSON.
    pub fn serialize<T: serde::Serialize, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => {
                let encoded = serde_json::to_string(v).map_err(serde::ser::Error::custom)?;
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional JSON-encoded string.
    ///
    /// # Errors
    ///
    /// Fails if a non-empty string is not valid JSON for `T`.
    pub fn deserialize<'de, T: DeserializeOwned, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(None),
            Value::String(s) if s.trim().is_empty() => Ok(None),
            Value::String(s) => serde_json::from_str(&s)
                .map(Some)
                .map_err(|e| D::Error::custom(format!("invalid JSON-encoded string {s:?}: {e}"))),
            other => T::deserialize(other).map(Some).map_err(D::Error::custom),
        }
    }
}

/// A value that the API sends either as a JSON number or as a numeric string, parsed with
/// [`FromStr`]. Serializes with `T`'s own `Serialize`.
pub mod string_or_number {
    use super::*;

    /// Serializes with `T`'s own representation.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<T: serde::Serialize, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }

    /// Deserializes from a JSON number or a numeric string.
    ///
    /// # Errors
    ///
    /// Fails if the value cannot be parsed as `T`.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr + DeserializeOwned,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::String(s) => s
                .trim()
                .parse::<T>()
                .map_err(|e| D::Error::custom(format!("invalid number {s:?}: {e}"))),
            other => T::deserialize(other).map_err(D::Error::custom),
        }
    }
}

/// Optional variant of [`string_or_number`]; `null`, a missing field and `""` become
/// `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod string_or_number_option {
    use serde::Serialize as _;

    use super::*;

    /// Serializes with `T`'s own representation, `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<T: serde::Serialize, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }

    /// Deserializes from a JSON number, a numeric string, `""` or `null`.
    ///
    /// # Errors
    ///
    /// Fails if a non-empty value cannot be parsed as `T`.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr + DeserializeOwned,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(None),
            Value::String(s) if s.trim().is_empty() => Ok(None),
            Value::String(s) => s
                .trim()
                .parse::<T>()
                .map(Some)
                .map_err(|e| D::Error::custom(format!("invalid number {s:?}: {e}"))),
            other => T::deserialize(other).map(Some).map_err(D::Error::custom),
        }
    }
}

/// Deserializes an `Option<T>` where the API sends `""` to mean "absent".
///
/// Use with `#[serde(default, deserialize_with = "...")]`.
///
/// # Errors
///
/// Fails if a non-empty value is not a valid `T`.
pub fn empty_string_as_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        Value::String(s) if s.is_empty() => Ok(None),
        other => T::deserialize(other).map(Some).map_err(D::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn lenient_datetimes() {
        let expected = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        for s in [
            "2024-01-02T03:04:05Z",
            "2024-01-02T03:04:05+00:00",
            "2024-01-02 03:04:05+00",
            "2024-01-02 03:04:05",
            "2024-01-02T03:04:05",
            "2024-01-02T05:04:05+02:00",
        ] {
            assert_eq!(parse_datetime(s), Some(expected), "{s}");
        }
        assert_eq!(
            parse_datetime("2024-01-02"),
            Some(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap())
        );
        assert_eq!(
            parse_datetime("2024-01-02T03:04:05.123456Z").map(|d| d.timestamp_subsec_micros()),
            Some(123_456)
        );
        assert_eq!(parse_datetime("yesterday"), None);
        assert_eq!(parse_datetime(""), None);
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Wire {
        #[serde(with = "timestamp_seconds")]
        secs: DateTime<Utc>,
        #[serde(default, with = "timestamp_millis_option")]
        millis: Option<DateTime<Utc>>,
        #[serde(default, with = "datetime_option")]
        at: Option<DateTime<Utc>>,
        #[serde(with = "json_string")]
        outcomes: Vec<String>,
        #[serde(with = "string_or_number")]
        count: u64,
        #[serde(default, with = "string_or_number_option")]
        maybe: Option<i64>,
        #[serde(default, deserialize_with = "empty_string_as_none")]
        side: Option<String>,
        price: Decimal,
    }

    #[test]
    fn wire_helpers_roundtrip() {
        let json = r#"{"secs":"1700000000","millis":1700000000123,"at":"","outcomes":"[\"Yes\",\"No\"]","count":"42","maybe":null,"side":"","price":"0.55"}"#;
        let wire: Wire = serde_json::from_str(json).unwrap();
        assert_eq!(wire.secs.timestamp(), 1_700_000_000);
        assert_eq!(wire.millis.unwrap().timestamp_millis(), 1_700_000_000_123);
        assert_eq!(wire.at, None);
        assert_eq!(wire.outcomes, vec!["Yes", "No"]);
        assert_eq!(wire.count, 42);
        assert_eq!(wire.maybe, None);
        assert_eq!(wire.side, None);
        assert_eq!(wire.price, Decimal::new(55, 2));

        let encoded = serde_json::to_string(&wire).unwrap();
        let again: Wire = serde_json::from_str(&encoded).unwrap();
        assert_eq!(again, wire);
    }

    #[test]
    fn decimal_accepts_numbers_exactly() {
        let d: Decimal = serde_json::from_str("0.1").unwrap();
        assert_eq!(d.to_string(), "0.1");
        let d: Decimal = serde_json::from_str("45159.4653").unwrap();
        assert_eq!(d.to_string(), "45159.4653");
    }

    #[test]
    fn rejects_garbage() {
        assert!(serde_json::from_str::<Wire>(r#"{"secs":"soon"}"#).is_err());
    }
}
