//! Taker fee computation.
//!
//! Polymarket charges a fee to the taker of a trade, on markets that have fees enabled:
//!
//! ```text
//! fee = shares × rate × (price × (1 − price)) ^ exponent
//! ```
//!
//! `shares` is the number of shares traded, `price` the fill price (`0..=1`), and `rate` and
//! `exponent` are the market's fee parameters (CLOB `fd.r` / `fd.e`, Gamma
//! `feeSchedule.rate` / `feeSchedule.exponent`). The fee is in USDC, and makers pay no fee.
//! See <https://docs.polymarket.com/trading/fees> and
//! <https://docs.polymarket.com/market-data/market-details#trading-fees>; the latter documents
//! the exponent as "exponent applied to the price component". Observed live
//! (2026-11-17, 200 top markets; 2026-11-18, about 1,600 markets): the exponent is always `1`.
//!
//! **Exponents other than `1` are unverified.** No documentation gives the formula with the
//! exponent: the docs' formula is `shares × rate × price × (1 − price)`, which this matches
//! for exponent `1`. Raising the whole `price × (1 − price)` term to the exponent is this
//! SDK's reading of "exponent applied to the price component", not a documented formula; treat
//! fees for any other exponent as an estimate. See `OPEN_QUESTIONS.md`.
//!
//! **Rates change.** Category rates change on 2026-11-20 at 23:59 UTC (see
//! <https://docs.polymarket.com/programs/fee-and-rebate-changes-november-2026>). Read
//! the rate and exponent from the market being traded, not from a hard-coded table, and
//! re-fetch them after a change: a rate read before it is stale afterwards.
//!
//! # Rounding
//!
//! The documentation says "Fees are rounded to 5 decimal places. The smallest fee charged is
//! 0.00001 USDC. Anything smaller rounds to zero" ([`FEE_DECIMALS`]) but does not say which
//! rounding mode is used. [`taker_fee`] therefore returns the unrounded (computed in [`Decimal`], accurate to about 28 significant digits; [`Decimal`] rounds beyond that) value; it is
//! an estimate of the charged fee to within [`FEE_DECIMALS`] decimal places. See
//! `OPEN_QUESTIONS.md`.
//!
//! ```
//! use marcasite_core::fees::taker_fee;
//! use rust_decimal::Decimal;
//!
//! // 100 shares at 0.50 with rate 0.07 and exponent 1: $1.75 (docs, trading/fees).
//! let fee = taker_fee(Decimal::from(100), "0.50".parse()?, "0.07".parse()?, 1)?;
//! assert_eq!(fee, "1.75".parse()?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use rust_decimal::Decimal;

use crate::error::{Result, ValidationError};

/// Number of decimal places the documentation says fees are rounded to (5, so the smallest
/// fee charged is 0.00001 USDC). The rounding mode is not documented.
pub const FEE_DECIMALS: u32 = 5;

/// Largest exponent [`taker_fee`] accepts. Live markets use `1`, the only exponent whose
/// formula is documented (see the [module documentation](self)); the cap only bounds the
/// work and rejects nonsense values.
pub const MAX_EXPONENT: u32 = 8;

/// Computes the unrounded (computed in [`Decimal`], accurate to about 28 significant digits; [`Decimal`] rounds beyond that) taker fee in USDC:
/// `shares × rate × (price × (1 − price)) ^ exponent`.
///
/// See the [module documentation](self) for the sources, rounding and rate changes.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `price` is outside `0..=1`,
/// `shares` or `rate` is negative, `exponent` exceeds [`MAX_EXPONENT`], or an intermediate
/// product overflows [`Decimal`].
pub fn taker_fee(shares: Decimal, price: Decimal, rate: Decimal, exponent: u32) -> Result<Decimal> {
    if price < Decimal::ZERO || price > Decimal::ONE {
        return Err(ValidationError::new("price", "must be between 0 and 1").into());
    }
    if shares < Decimal::ZERO {
        return Err(ValidationError::new("shares", "must not be negative").into());
    }
    if rate < Decimal::ZERO {
        return Err(ValidationError::new("rate", "must not be negative").into());
    }
    if exponent > MAX_EXPONENT {
        return Err(
            ValidationError::new("exponent", format!("must be at most {MAX_EXPONENT}")).into(),
        );
    }
    let overflow = || ValidationError::new("fee", "overflows the decimal range");
    // `price` is within 0..=1, so `price * (1 - price)` cannot overflow.
    let base = price * (Decimal::ONE - price);
    let mut curve = Decimal::ONE;
    for _ in 0..exponent {
        curve = curve.checked_mul(base).ok_or_else(overflow)?;
    }
    let fee = shares
        .checked_mul(rate)
        .and_then(|v| v.checked_mul(curve))
        .ok_or_else(overflow)?;
    Ok(fee)
}

/// Converts a wire exponent (a JSON number, read as [`Decimal`]) into the integer
/// [`taker_fee`] takes.
///
/// # Errors
///
/// Returns [`Error::Validation`](crate::Error::Validation) if `exponent` is negative, not an
/// integer, or above [`MAX_EXPONENT`].
pub fn exponent_from_decimal(exponent: Decimal) -> Result<u32> {
    let invalid = || {
        ValidationError::new(
            "exponent",
            format!("must be an integer from 0 to {MAX_EXPONENT}, got {exponent}"),
        )
    };
    if exponent.fract() != Decimal::ZERO {
        return Err(invalid().into());
    }
    match u32::try_from(exponent.trunc()) {
        Ok(e) if e <= MAX_EXPONENT => Ok(e),
        _ => Err(invalid().into()),
    }
}

/// Computes [`taker_fee`] from optional wire parameters, as carried by the fee types.
///
/// Not part of the supported API: it exists for the fee types of the `marcasite` crate and
/// must stay `pub` for that reason.
///
/// # Errors
///
/// As [`taker_fee`] and [`exponent_from_decimal`], plus a validation error if `rate` or
/// `exponent` is `None`.
#[doc(hidden)]
pub fn taker_fee_from_parameters(
    shares: Decimal,
    price: Decimal,
    rate: Option<Decimal>,
    exponent: Option<Decimal>,
) -> Result<Decimal> {
    let rate = rate.ok_or_else(|| ValidationError::new("rate", "the market sent no fee rate"))?;
    let exponent =
        exponent.ok_or_else(|| ValidationError::new("exponent", "the market sent no exponent"))?;
    taker_fee(shares, price, rate, exponent_from_decimal(exponent)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn fee(rate: &str, price: &str) -> Decimal {
        taker_fee(Decimal::from(100), d(price), d(rate), 1).unwrap()
    }

    /// Worked examples of docs/polymarket/trading/fees.md (100 shares, rounded to cents).
    #[test]
    fn docs_worked_examples() {
        let cases = [
            ("0.07", "0.01", "0.0693", "0.07"),
            ("0.07", "0.05", "0.3325", "0.33"),
            ("0.07", "0.50", "1.75", "1.75"),
            ("0.07", "0.99", "0.0693", "0.07"),
            ("0.04", "0.20", "0.64", "0.64"),
            ("0.04", "0.50", "1.00", "1.00"),
            ("0.05", "0.30", "1.05", "1.05"),
            ("0.05", "0.50", "1.25", "1.25"),
        ];
        for (rate, price, exact, cents) in cases {
            let f = fee(rate, price);
            assert_eq!(f, d(exact), "rate {rate} price {price}");
            assert_eq!(
                f.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero),
                d(cents)
            );
        }
    }

    #[test]
    fn symmetric_in_price() {
        assert_eq!(fee("0.07", "0.2"), fee("0.07", "0.8"));
    }

    #[test]
    fn boundaries_are_free() {
        assert_eq!(fee("0.07", "0"), Decimal::ZERO);
        assert_eq!(fee("0.07", "1"), Decimal::ZERO);
        assert_eq!(
            taker_fee(Decimal::ZERO, d("0.5"), d("0.07"), 1).unwrap(),
            Decimal::ZERO
        );
    }

    #[test]
    fn exponents() {
        let (s, p, r) = (Decimal::from(100), d("0.5"), d("0.07"));
        assert_eq!(taker_fee(s, p, r, 0).unwrap(), d("7"));
        assert_eq!(taker_fee(s, p, r, 1).unwrap(), d("1.75"));
        assert_eq!(taker_fee(s, p, r, 2).unwrap(), d("0.4375"));
    }

    fn invalid_param(r: Result<Decimal>) -> String {
        match r.unwrap_err() {
            crate::Error::Validation(e) => e.parameter().to_owned(),
            other => panic!("not a validation error: {other:?}"),
        }
    }

    #[test]
    fn rejects_bad_inputs() {
        let (s, p, r) = (Decimal::from(100), d("0.5"), d("0.07"));
        assert_eq!(invalid_param(taker_fee(s, d("-0.01"), r, 1)), "price");
        assert_eq!(invalid_param(taker_fee(s, d("1.01"), r, 1)), "price");
        assert_eq!(invalid_param(taker_fee(d("-1"), p, r, 1)), "shares");
        assert_eq!(invalid_param(taker_fee(s, p, d("-0.07"), 1)), "rate");
        assert_eq!(
            invalid_param(taker_fee(s, p, r, MAX_EXPONENT + 1)),
            "exponent"
        );
        assert!(taker_fee(s, p, r, MAX_EXPONENT).is_ok());
    }

    #[test]
    fn overflow_is_an_error() {
        assert_eq!(
            invalid_param(taker_fee(Decimal::MAX, d("0.5"), d("2"), 1)),
            "fee"
        );
    }

    #[test]
    fn precision_is_limited_to_decimal() {
        // 1e9 x 0.07 x (0.33 x 0.67)^8 = 399.76795691333717707630193670... in exact arithmetic
        // (Python `decimal`, 50 digits). `0.2211^8` has 32 significant digits, so `Decimal`
        // rounds it: the result agrees to well beyond the 5 decimals of a fee.
        let f = taker_fee(d("1000000000"), d("0.33"), d("0.07"), 8).unwrap();
        let reference = d("399.7679569133371770763019367");
        assert!((f - reference).abs() < d("0.000000000001"), "{f}");
        assert_eq!(f.round_dp(5), d("399.76796"));
    }

    #[test]
    fn exponent_conversion() {
        assert_eq!(exponent_from_decimal(d("1")).unwrap(), 1);
        assert_eq!(exponent_from_decimal(d("1.0")).unwrap(), 1);
        assert!(exponent_from_decimal(d("1.5")).is_err());
        assert!(exponent_from_decimal(d("-1")).is_err());
        assert!(exponent_from_decimal(d("9")).is_err());
        assert!(exponent_from_decimal(Decimal::MAX).is_err());
        assert_eq!(exponent_from_decimal(d("-0")).unwrap(), 0);
        assert_eq!(exponent_from_decimal(d("1.00")).unwrap(), 1);
        assert_eq!(
            exponent_from_decimal(Decimal::from_scientific("1e0").unwrap()).unwrap(),
            1
        );
        assert!(exponent_from_decimal(d("1.0000000000000000000000000001")).is_err());
    }

    #[test]
    fn missing_parameters() {
        let s = Decimal::from(1);
        let p = d("0.5");
        assert_eq!(
            invalid_param(taker_fee_from_parameters(s, p, None, Some(d("1")))),
            "rate"
        );
        assert_eq!(
            invalid_param(taker_fee_from_parameters(s, p, Some(d("0.05")), None)),
            "exponent"
        );
        assert_eq!(
            taker_fee_from_parameters(s, p, Some(d("0.05")), Some(d("1"))).unwrap(),
            d("0.0125")
        );
    }
}
