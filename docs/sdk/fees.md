# Module `marcasite::fees`

> Generated from marcasite 0.2.0 (all features) by `just docs-md`. Do not edit.

Taker fee computation.

Polymarket charges a fee to the taker of a trade, on markets that have fees enabled:

```text
fee = shares × rate × (price × (1 − price)) ^ exponent
```

`shares` is the number of shares traded, `price` the fill price (`0..=1`), and `rate` and
`exponent` are the market's fee parameters (CLOB `fd.r` / `fd.e`, Gamma
`feeSchedule.rate` / `feeSchedule.exponent`). The fee is in USDC, and makers pay no fee.
See <https://docs.polymarket.com/trading/fees> and
<https://docs.polymarket.com/market-data/market-details#trading-fees>; the latter documents
the exponent as "exponent applied to the price component". Observed live
(2026-11-17, 200 top markets): the exponent is always `1`.

**Rates change.** Category rates change on 2026-11-20 (see
<https://docs.polymarket.com/programs/fee-and-rebate-changes-november-2026>). Read
the rate and exponent from the market being traded, not from a hard-coded table.

## Rounding

The documentation says "Fees are rounded to 5 decimal places. The smallest fee charged is
0.00001 USDC. Anything smaller rounds to zero" ([`FEE_DECIMALS`](fees.md#constant.FEE_DECIMALS)) but does not say which
rounding mode is used. [`taker_fee`](fees.md#fn.taker_fee) therefore returns the unrounded (computed in `Decimal`, accurate to about 28 significant digits; `Decimal` rounds beyond that) value; it is
an estimate of the charged fee to within [`FEE_DECIMALS`](fees.md#constant.FEE_DECIMALS) decimal places. See
`OPEN_QUESTIONS.md`.

```rust
use marcasite_core::fees::taker_fee;
use rust_decimal::Decimal;

// 100 shares at 0.50 with rate 0.07 and exponent 1: $1.75 (docs, trading/fees).
let fee = taker_fee(Decimal::from(100), "0.50".parse()?, "0.07".parse()?, 1)?;
assert_eq!(fee, "1.75".parse()?);
```

## Index

- **Constants:** [`FEE_DECIMALS`](#constant.FEE_DECIMALS), [`MAX_EXPONENT`](#constant.MAX_EXPONENT)
- **Functions:** [`exponent_from_decimal`](#fn.exponent_from_decimal), [`taker_fee`](#fn.taker_fee)

## Constants

### <a id="constant.FEE_DECIMALS"></a>`const FEE_DECIMALS`

```rust
pub const FEE_DECIMALS: u32 = 5;
```

Number of decimal places the documentation says fees are rounded to (5, so the smallest
fee charged is 0.00001 USDC). The rounding mode is not documented.

### <a id="constant.MAX_EXPONENT"></a>`const MAX_EXPONENT`

```rust
pub const MAX_EXPONENT: u32 = 8;
```

Largest exponent [`taker_fee`](fees.md#fn.taker_fee) accepts. Live markets use `1`; the cap only bounds the
work and rejects nonsense values.

## Functions

### <a id="fn.exponent_from_decimal"></a>`fn exponent_from_decimal`

```rust
pub fn exponent_from_decimal(exponent: Decimal) -> Result<u32>
```

Converts a wire exponent (a JSON number, read as `Decimal`) into the integer
[`taker_fee`](fees.md#fn.taker_fee) takes.

#### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `exponent` is negative, not an
integer, or above [`MAX_EXPONENT`](fees.md#constant.MAX_EXPONENT).

### <a id="fn.taker_fee"></a>`fn taker_fee`

```rust
pub fn taker_fee(shares: Decimal, price: Decimal, rate: Decimal, exponent: u32) -> Result<Decimal>
```

Computes the unrounded (computed in `Decimal`, accurate to about 28 significant digits; `Decimal` rounds beyond that) taker fee in USDC:
`shares × rate × (price × (1 − price)) ^ exponent`.

See the [module documentation](fees.md) for the sources, rounding and rate changes.

#### Errors

Returns [`Error::Validation`](marcasite.md#enum.Error) if `price` is outside `0..=1`,
`shares` or `rate` is negative, `exponent` exceeds [`MAX_EXPONENT`](fees.md#constant.MAX_EXPONENT), or an intermediate
product overflows `Decimal`.
