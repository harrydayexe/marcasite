//! (De)serialization helpers for the wire formats used by the Polymarket APIs.
//!
//! Use these with `#[serde(with = "...")]` / `#[serde(deserialize_with = "...")]` on wire
//! types. Every helper accepts exactly the documented representation, plus a small set of
//! lossless alternatives (e.g. a number sent as a numeric string) so a minor server-side
//! encoding change does not break deserialization of a whole response.
//!
//! Decimal values ([`rust_decimal::Decimal`]) need no helper to deserialize: they accept
//! both JSON strings and JSON numbers. Their default serialization is a JSON string; for
//! fields the API sends as JSON **numbers**, use [`decimal_number`] /
//! [`decimal_number_option`], which serialize back to a JSON number (integers as integers).
//!
//! # Precision of decimals sent as JSON numbers
//!
//! A decimal sent as a JSON **string** is parsed exactly. A decimal sent as a JSON
//! **number** is exact only within these bounds, because `serde_json` (used without its
//! `arbitrary_precision` feature) hands non-integer numbers to [`Decimal`] as an `f64`:
//!
//! - integers that fit in `i64` or `u64` are exact;
//! - other numbers are exact if they have at most 15 significant digits; longer ones are
//!   rounded to the nearest `f64`, which keeps about 15 to 17 significant digits (for
//!   example `12345678901.123456` decodes as `12345678901.123455`).
//!
//! A number the server itself produced from an `f64` decodes to the decimal that `f64`
//! prints as (its shortest round-trip representation), so nothing is lost in that case.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive as _};
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

/// Parses a date-time ending in a short `+HH` / `-HH` offset.
///
/// Network input: never slice at a byte offset that may fall inside a multi-byte
/// character (`str::get` returns `None` there instead of panicking).
fn short_offset(s: &str) -> Option<DateTime<Utc>> {
    let split = s.len().checked_sub(3)?;
    let (head, tail) = (s.get(..split)?, s.get(split..)?);
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

/// Unix timestamps in **microseconds** (JSON number or numeric string) as `DateTime<Utc>`.
///
/// Serializes as a JSON integer.
pub mod timestamp_micros {
    use super::*;

    /// Serializes as integer microseconds.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(value.timestamp_micros())
    }

    /// Deserializes from integer microseconds or a numeric string.
    ///
    /// # Errors
    ///
    /// Fails on non-numeric input or an out-of-range timestamp.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let micros = integer_from_value::<D::Error>(&value, "unix timestamp (microseconds)")?;
        DateTime::from_timestamp_micros(micros)
            .ok_or_else(|| D::Error::custom(format!("unix timestamp out of range: {micros}")))
    }
}

/// Optional variant of [`timestamp_micros`]; `null`, a missing field and `""` become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod timestamp_micros_option {
    use super::*;

    /// Serializes `Some` as integer microseconds and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_some(&v.timestamp_micros()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from integer microseconds, a numeric string, `""` or `null`.
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
                let micros =
                    integer_from_value::<D::Error>(&value, "unix timestamp (microseconds)")?;
                DateTime::from_timestamp_micros(micros)
                    .map(Some)
                    .ok_or_else(|| {
                        D::Error::custom(format!("unix timestamp out of range: {micros}"))
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

/// Serializes a decimal as a JSON number: an integral value as an integer, any other value
/// as a float.
struct DecimalAsNumber<'a>(&'a Decimal);

impl serde::Serialize for DecimalAsNumber<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = self.0;
        if value.fract().is_zero() {
            if let Some(integer) = value.to_i64() {
                return serializer.serialize_i64(integer);
            }
            if let Some(integer) = value.to_u64() {
                return serializer.serialize_u64(integer);
            }
        }
        match value.to_f64() {
            Some(float) => serializer.serialize_f64(float),
            None => Err(serde::ser::Error::custom(format!(
                "decimal {value} cannot be represented as a JSON number"
            ))),
        }
    }
}

/// A [`Decimal`] that the API sends as a JSON **number** (`type: number`).
///
/// Deserializes from a JSON number or a numeric string (like `Decimal` itself) and
/// serializes back to a JSON number, as on the wire: integral values as integers (`3`, not
/// `3.0`), others as floats. Unlike `rust_decimal::serde::float`, which always writes a
/// float, a round trip therefore preserves the wire representation of integers.
pub mod decimal_number {
    use super::*;

    /// Serializes as a JSON number (integers as integers).
    ///
    /// # Errors
    ///
    /// Fails if the value cannot be represented as a JSON number; propagates serializer
    /// errors.
    pub fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&DecimalAsNumber(value), serializer)
    }

    /// Deserializes from a JSON number or a numeric string.
    ///
    /// # Errors
    ///
    /// Fails if the value is not a valid decimal.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        <Decimal as Deserialize>::deserialize(deserializer)
    }
}

/// Optional variant of [`decimal_number`]; `null` and a missing field become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod decimal_number_option {
    use super::*;

    /// Serializes `Some` as a JSON number (integers as integers) and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Fails if the value cannot be represented as a JSON number; propagates serializer
    /// errors.
    pub fn serialize<S: Serializer>(
        value: &Option<Decimal>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => serializer.serialize_some(&DecimalAsNumber(v)),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes from a JSON number, a numeric string or `null`.
    ///
    /// # Errors
    ///
    /// Fails if a non-null value is not a valid decimal.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Decimal>, D::Error> {
        Option::<Decimal>::deserialize(deserializer)
    }
}

/// Writes a string identifier as a JSON integer when it is one, otherwise as a string.
fn serialize_integer_id<S: Serializer>(id: &str, serializer: S) -> Result<S::Ok, S::Error> {
    match id.parse::<i64>() {
        Ok(number) => serializer.serialize_i64(number),
        Err(_) => serializer.serialize_str(id),
    }
}

/// Identifier newtypes (see [`string_id!`](crate::string_id)) whose wire type is
/// `integer`.
///
/// Deserializes through the newtype, which accepts a JSON integer or string, and serializes
/// back to a JSON integer when the id is numeric (otherwise as a string), so a round trip
/// preserves the wire format.
pub mod integer_id {
    use super::*;

    /// Serializes as a JSON integer if the id parses as `i64`, otherwise as a string.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<T: AsRef<str>, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serialize_integer_id(value.as_ref(), serializer)
    }

    /// Deserializes with `T`'s own `Deserialize`.
    ///
    /// # Errors
    ///
    /// Fails if the value is not a valid `T`.
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        T::deserialize(deserializer)
    }
}

/// Optional variant of [`integer_id`]; `null` and a missing field become `None`.
///
/// Use with `#[serde(default, with = "...")]`.
pub mod integer_id_option {
    use super::*;

    /// Serializes `Some` like [`integer_id`] and `None` as `null`.
    ///
    /// # Errors
    ///
    /// Propagates serializer errors.
    pub fn serialize<T: AsRef<str>, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        struct IntegerId<'a>(&'a str);

        impl serde::Serialize for IntegerId<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serialize_integer_id(self.0, serializer)
            }
        }

        match value {
            Some(id) => serializer.serialize_some(&IntegerId(id.as_ref())),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes with `Option<T>`'s own `Deserialize`.
    ///
    /// # Errors
    ///
    /// Fails if a non-null value is not a valid `T`.
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        Option::<T>::deserialize(deserializer)
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
    fn decimal_numbers_are_exact_up_to_15_significant_digits() {
        for exact in [
            "0.1",
            "45159.4653",
            "123456789.012345",
            "-0.000000000000001",
        ] {
            let d: Decimal = serde_json::from_str(exact).unwrap();
            assert_eq!(d.to_string(), exact);
        }
        // Integers within `i64` / `u64` are exact whatever their length.
        let d: Decimal = serde_json::from_str("18446744073709551615").unwrap();
        assert_eq!(d.to_string(), "18446744073709551615");
    }

    /// Documents a known limitation (see the module docs): JSON numbers with more than 15
    /// significant digits go through `f64` and may be rounded. JSON strings never are.
    #[test]
    fn decimal_numbers_beyond_15_significant_digits_go_through_f64() {
        let from_number: Decimal = serde_json::from_str("12345678901.123456").unwrap();
        assert_eq!(from_number.to_string(), "12345678901.123455");
        let from_string: Decimal = serde_json::from_str("\"12345678901.123456\"").unwrap();
        assert_eq!(from_string.to_string(), "12345678901.123456");
        // Same through the `decimal_number` helper.
        #[derive(Deserialize)]
        struct Wire {
            #[serde(with = "decimal_number")]
            v: Decimal,
        }
        let wire: Wire = serde_json::from_str(r#"{"v":12345678901.123456}"#).unwrap();
        assert_eq!(wire.v, from_number);
    }

    #[test]
    fn micros_roundtrip() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Micros {
            #[serde(with = "timestamp_micros")]
            at: DateTime<Utc>,
            #[serde(default, with = "timestamp_micros_option")]
            maybe: Option<DateTime<Utc>>,
        }
        let wire: Micros = serde_json::from_str(r#"{"at":1700000000123456,"maybe":null}"#).unwrap();
        assert_eq!(wire.at.timestamp_micros(), 1_700_000_000_123_456);
        assert_eq!(wire.maybe, None);
        let encoded = serde_json::to_string(&wire).unwrap();
        assert_eq!(encoded, r#"{"at":1700000000123456,"maybe":null}"#);
        assert_eq!(serde_json::from_str::<Micros>(&encoded).unwrap(), wire);

        let lenient: Micros =
            serde_json::from_str(r#"{"at":"1700000000123456","maybe":"7"}"#).unwrap();
        assert_eq!(lenient.at, wire.at);
        assert_eq!(lenient.maybe.map(|d| d.timestamp_micros()), Some(7));
        let missing: Micros = serde_json::from_str(r#"{"at":0,"maybe":""}"#).unwrap();
        assert_eq!(missing.maybe, None);
        assert!(serde_json::from_str::<Micros>(r#"{"at":"soon"}"#).is_err());
        assert!(serde_json::from_str::<Micros>(r#"{"at":true}"#).is_err());
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Numbers {
        #[serde(with = "decimal_number")]
        required: Decimal,
        #[serde(default, with = "decimal_number_option")]
        value: Option<Decimal>,
    }

    #[test]
    fn decimal_numbers_roundtrip_as_json_numbers() {
        for json in [
            r#"{"required":3,"value":-2}"#,
            r#"{"required":12.5,"value":0.001}"#,
            r#"{"required":0,"value":null}"#,
            r#"{"required":330327.7128580074,"value":45}"#,
            r#"{"required":18446744073709551615,"value":-9223372036854775808}"#,
        ] {
            let parsed: Numbers = serde_json::from_str(json).unwrap();
            assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
        }
        // Integral values written with a scale stay integers.
        let scaled = Numbers {
            required: Decimal::new(300, 2),
            value: Some(Decimal::new(-1500, 3)),
        };
        assert_eq!(
            serde_json::to_string(&scaled).unwrap(),
            r#"{"required":3,"value":-1.5}"#
        );
        let missing: Numbers = serde_json::from_str(r#"{"required":"1"}"#).unwrap();
        assert_eq!(missing.required, Decimal::ONE);
        assert_eq!(missing.value, None);
        let from_string: Numbers =
            serde_json::from_str(r#"{"required":"0.1","value":"0.1"}"#).unwrap();
        assert_eq!(from_string.value, Some(Decimal::new(1, 1)));
        assert!(serde_json::from_str::<Numbers>(r#"{"required":"abc"}"#).is_err());
        assert!(serde_json::from_str::<Numbers>(r#"{"required":1,"value":"abc"}"#).is_err());
        assert!(serde_json::from_str::<Numbers>(r#"{"required":1,"value":[]}"#).is_err());
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Ids {
        #[serde(with = "integer_id")]
        id: crate::types::TokenId,
        #[serde(default, with = "integer_id_option")]
        other: Option<crate::types::TokenId>,
    }

    #[test]
    fn integer_ids_roundtrip_as_integers() {
        let ids: Ids = serde_json::from_str(r#"{"id":42,"other":"7"}"#).unwrap();
        assert_eq!(ids.id, "42");
        assert_eq!(
            serde_json::to_string(&ids).unwrap(),
            r#"{"id":42,"other":7}"#
        );
        let non_numeric = Ids {
            id: "abc".into(),
            other: None,
        };
        assert_eq!(
            serde_json::to_string(&non_numeric).unwrap(),
            r#"{"id":"abc","other":null}"#
        );
        let missing: Ids = serde_json::from_str(r#"{"id":"1"}"#).unwrap();
        assert_eq!(missing.other, None);
        assert!(serde_json::from_str::<Ids>(r#"{"id":[1]}"#).is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(serde_json::from_str::<Wire>(r#"{"secs":"soon"}"#).is_err());
    }

    #[test]
    fn multi_byte_input_does_not_panic() {
        for s in [
            "€ab",
            "aaaaaaaaaaaaaaaa€a",
            "x€y",
            "2024-01-02T03:04:05€",
            "2024-01-02 03:04:05+0€",
            "2024-01-02T03:04:05+€0",
            "€",
            "😀",
            "é+0",
        ] {
            assert_eq!(parse_datetime(s), None, "{s}");
        }
        // Still parsed when the multi-byte character is not at the end.
        assert!(parse_datetime(" 2024-01-02 03:04:05+00 ").is_some());
    }

    /// A small deterministic xorshift generator, so the fuzz test needs no dependency and
    /// is reproducible.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, n: usize) -> usize {
            usize::try_from(self.next() % n as u64).unwrap()
        }
    }

    /// Characters chosen to hit the parsers' edge cases: digits, separators, signs,
    /// whitespace, and 2-, 3- and 4-byte UTF-8 characters.
    const ALPHABET: [&str; 16] = [
        "0", "9", "-", "+", ":", "T", " ", ".", "Z", "e", "é", "€", "😀", "\u{301}", "\"", "\n",
    ];

    /// Prefixes that get the parsers past their first checks.
    const PREFIXES: [&str; 8] = [
        "",
        "2024-01-02",
        "2024-01-02T03:04:05",
        "2024-01-02 03:04:05+0",
        "2024-01-02T03:04:05.123",
        "1700000000",
        "-1",
        "[\"a\",",
    ];

    /// Every string of up to three alphabet characters, plus random strings that start
    /// with a plausible prefix.
    fn fuzz_strings() -> Vec<String> {
        let mut out = vec![String::new()];
        let mut frontier = vec![String::new()];
        for _ in 0..3 {
            let mut next = Vec::new();
            for prefix in &frontier {
                for c in ALPHABET {
                    next.push(format!("{prefix}{c}"));
                }
            }
            out.extend(next.iter().cloned());
            frontier = next;
        }
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for _ in 0..4000 {
            let mut s = PREFIXES[rng.below(PREFIXES.len())].to_owned();
            for _ in 0..rng.below(8) {
                s.push_str(ALPHABET[rng.below(ALPHABET.len())]);
            }
            out.push(s);
        }
        out
    }

    /// JSON values of every type, including numbers at and beyond the edges of the
    /// integer and float ranges.
    fn fuzz_values() -> Vec<Value> {
        let mut values: Vec<Value> = fuzz_strings().into_iter().map(Value::String).collect();
        for raw in [
            "null",
            "true",
            "false",
            "0",
            "-0",
            "-0.0",
            "1.5",
            "1e300",
            "-1e300",
            "1e-300",
            "9223372036854775807",
            "-9223372036854775808",
            "18446744073709551615",
            "18446744073709551616",
            "1e19",
            "1700000000.0",
            "1700000000.5",
            "79228162514264337593543950336",
            "123456789012345678901234567890.123",
            "[]",
            "[1, \"2\"]",
            "{}",
            "{\"a\": 1}",
        ] {
            values.push(serde_json::from_str(raw).unwrap());
        }
        values
    }

    /// Runs `T`'s deserializer on `value`; on success, serializes the result again (which
    /// must not panic either).
    fn exercise<T: DeserializeOwned + Serialize>(value: &Value) {
        if let Ok(decoded) = T::deserialize(value) {
            let _ = serde_json::to_string(&decoded);
        }
    }

    macro_rules! fuzz_fields {
        ($value:expr; $($name:ident: $ty:ty => $attr:meta;)*) => {
            $(
                #[derive(Deserialize, Serialize)]
                struct $name {
                    #[$attr]
                    #[allow(dead_code)]
                    v: $ty,
                }
                exercise::<$name>(&serde_json::json!({ "v": $value }));
            )*
        };
    }

    #[test]
    fn helpers_never_panic_on_arbitrary_input() {
        for value in fuzz_values() {
            if let Value::String(s) = &value {
                let _ = parse_datetime(s);
            }
            fuzz_fields! { value.clone();
                Secs: DateTime<Utc> => serde(with = "timestamp_seconds");
                SecsOpt: Option<DateTime<Utc>> => serde(default, with = "timestamp_seconds_option");
                Millis: DateTime<Utc> => serde(with = "timestamp_millis");
                MillisOpt: Option<DateTime<Utc>> => serde(default, with = "timestamp_millis_option");
                Micros: DateTime<Utc> => serde(with = "timestamp_micros");
                MicrosOpt: Option<DateTime<Utc>> => serde(default, with = "timestamp_micros_option");
                Date: DateTime<Utc> => serde(with = "datetime");
                DateOpt: Option<DateTime<Utc>> => serde(default, with = "datetime_option");
                JsonList: Vec<String> => serde(with = "json_string");
                JsonAny: Value => serde(with = "json_string");
                JsonOpt: Option<Vec<i64>> => serde(default, with = "json_string_option");
                NumU64: u64 => serde(with = "string_or_number");
                NumI64: i64 => serde(with = "string_or_number");
                NumDec: Decimal => serde(with = "string_or_number");
                NumOpt: Option<Decimal> => serde(default, with = "string_or_number_option");
                NumOptI32: Option<i32> => serde(default, with = "string_or_number_option");
                DecNum: Decimal => serde(with = "decimal_number");
                DecNumOpt: Option<Decimal> => serde(default, with = "decimal_number_option");
                Id: crate::types::TokenId => serde(with = "integer_id");
                IdOpt: Option<crate::types::TokenId> => serde(default, with = "integer_id_option");
                Empty: Option<String> => serde(default, deserialize_with = "empty_string_as_none");
                EmptyDec: Option<Decimal> => serde(default, deserialize_with = "empty_string_as_none");
                PlainDec: Decimal => serde(default);
            }
        }
    }
}
