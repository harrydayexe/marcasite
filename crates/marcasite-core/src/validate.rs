//! Client-side checks of documented parameter patterns.
//!
//! Each function checks a value against one pattern the API documentation specifies for a
//! parameter, so that a malformed value is rejected with a [`ValidationError`] before any
//! request is sent. The error names the parameter, the expected pattern and the rejected
//! value. Every [`ValidationError`] converts into [`Error::Validation`](crate::Error), so
//! the functions can be used with `?` in code returning [`crate::Result`].
//!
//! ```
//! use marcasite_core::validate;
//!
//! assert!(validate::evm_address("user", "0x56687bf447db6ffa42ffe2204a05edaa20f55839").is_ok());
//! let err = validate::evm_address("user", "0x1234").unwrap_err();
//! assert_eq!(err.parameter(), "user");
//! assert!(err.message().contains("^0x[a-fA-F0-9]{40}$"));
//! ```

use crate::error::ValidationError;

/// Number of hex digits of an EVM address.
const ADDRESS_HEX_DIGITS: usize = 40;
/// Number of hex digits of a 32-byte value.
const BYTES32_HEX_DIGITS: usize = 64;

/// Checks that `value` is `0x` followed by exactly `hex_digits` hexadecimal digits
/// (`^0x[a-fA-F0-9]{hex_digits}$`; the prefix is lower-case `0x`, the digits may be of
/// either case).
///
/// # Errors
///
/// Returns a [`ValidationError`] for `parameter` naming the expected pattern if `value`
/// does not match.
pub fn prefixed_hex(
    parameter: &'static str,
    value: &str,
    hex_digits: usize,
) -> Result<(), ValidationError> {
    check(parameter, value, hex_digits, None)
}

/// Checks that `value` is an EVM address: `0x` followed by 40 hexadecimal digits
/// (`^0x[a-fA-F0-9]{40}$`, the `Address` pattern of the API specs). No checksum (EIP-55)
/// validation is applied.
///
/// # Errors
///
/// Returns a [`ValidationError`] for `parameter` naming the expected pattern if `value`
/// does not match.
pub fn evm_address(parameter: &'static str, value: &str) -> Result<(), ValidationError> {
    check(parameter, value, ADDRESS_HEX_DIGITS, Some("an EVM address"))
}

/// Checks that `value` is a 32-byte hex value: `0x` followed by 64 hexadecimal digits
/// (`^0x[a-fA-F0-9]{64}$`), the documented pattern of condition ids and builder codes.
///
/// # Errors
///
/// Returns a [`ValidationError`] for `parameter` naming the expected pattern if `value`
/// does not match.
pub fn bytes32(parameter: &'static str, value: &str) -> Result<(), ValidationError> {
    check(
        parameter,
        value,
        BYTES32_HEX_DIGITS,
        Some("a bytes32 hex value"),
    )
}

fn check(
    parameter: &'static str,
    value: &str,
    hex_digits: usize,
    what: Option<&str>,
) -> Result<(), ValidationError> {
    let valid = value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == hex_digits && hex.bytes().all(|b| b.is_ascii_hexdigit()));
    if valid {
        return Ok(());
    }
    let expected =
        format!("`0x` followed by {hex_digits} hex digits (`^0x[a-fA-F0-9]{{{hex_digits}}}$`)");
    let message = match what {
        Some(what) => format!("must be {what}: {expected}, got {value:?}"),
        None => format!("must be {expected}, got {value:?}"),
    };
    Err(ValidationError::new(parameter, message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn evm_addresses() {
        for ok in [
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            "0x7C3DB723F1D4D8CB9C550095203B686CB11E5C6B",
            "0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b5",
        ] {
            assert!(evm_address("address", ok).is_ok(), "{ok}");
        }
        for bad in [
            "",
            "0x",
            "7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6bb",
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5c6g",
            "0X7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            " 0x7c3db723f1d4d8cb9c550095203b686cb11e5c6b",
            // Multi-byte characters never count as digits.
            "0x7c3db723f1d4d8cb9c550095203b686cb11e5cé",
        ] {
            let err = evm_address("address", bad).unwrap_err();
            assert_eq!(err.parameter(), "address", "{bad}");
        }
    }

    #[test]
    fn bytes32_values() {
        let ok = format!("0x{}", "aB".repeat(32));
        assert!(bytes32("builder_code", &ok).is_ok());
        assert!(bytes32("builder_code", &ok[2..]).is_err());
        assert!(bytes32("builder_code", "0x12").is_err());
        assert!(bytes32("builder_code", &format!("0x{}", "g".repeat(64))).is_err());
        assert!(bytes32("builder_code", &format!("0x{}", "0".repeat(65))).is_err());
    }

    #[test]
    fn arbitrary_lengths() {
        assert!(prefixed_hex("x", "0x", 0).is_ok());
        assert!(prefixed_hex("x", "0xabc", 3).is_ok());
        assert!(prefixed_hex("x", "0xabc", 4).is_err());
    }

    #[test]
    fn messages_name_the_parameter_pattern_and_value() {
        let err = evm_address("user", "0x12").unwrap_err();
        assert_eq!(err.parameter(), "user");
        assert_eq!(
            err.message(),
            "must be an EVM address: `0x` followed by 40 hex digits \
             (`^0x[a-fA-F0-9]{40}$`), got \"0x12\""
        );
        let err = bytes32("market", "nope").unwrap_err();
        assert!(err.message().contains("^0x[a-fA-F0-9]{64}$"), "{err}");
        assert!(err.message().contains("\"nope\""), "{err}");
        let err = prefixed_hex("id", "0x1", 2).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid `id`: must be `0x` followed by 2 hex digits (`^0x[a-fA-F0-9]{2}$`), got \"0x1\""
        );
        assert!(matches!(Error::from(err), Error::Validation(_)));
    }
}
