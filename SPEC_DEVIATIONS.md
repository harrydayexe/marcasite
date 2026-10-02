# Spec deviations

Places where the live Polymarket API disagrees with its documentation (`docs/`, fetched
2026-10-01), and what polyoxide does about it. **The SDK follows the live API** (see `AGENTS.md`);
this file records each departure so it can be revisited if Polymarket changes the API or fixes the
docs.

Each entry names a **pinning live test** in `crates/polyoxide/tests/live/`. The test asserts the
live behaviour described here, so if Polymarket changes it the test fails and points back to this
entry. Run them with `just test-live`.

Kinds:

- **Mismatch**: the docs and live disagree (shape, type, format, parameters, limits). The SDK
  follows live.
- **Undocumented**: live sends fields, values or behaviour the docs do not mention. The SDK models
  them.
- **Server bug**: live misbehaves in a way no client change can fix. Documented only.

Observed: 2026-10-02 unless stated.

## Gamma API

<!-- gamma entries -->

## CLOB API

All CLOB pinning tests are in `crates/polyoxide/tests/live/clob.rs`. Spec paths below are in
`docs/specs/clob-openapi.yaml`.

### `GET /midpoint` response field is `mid`, not `mid_price`

- **Kind:** Mismatch
- **Docs say:** `getMidpoint`, 200 response requires `mid_price` (a numeric string), example
  `{"mid_price": "0.45"}`.
- **Live does:** `{"mid":"0.0005"}`; `mid_price` is never sent.
- **SDK does:** `Midpoint.mid` (renamed from `mid_price`; breaking), wire name `mid`, with
  `#[serde(alias = "mid_price")]` so the documented spelling still decodes. Serializes as `mid`.
- **Pinning test:** `pin_midpoint_field_is_mid`.

### Plural `GET` query forms always fail

- **Kind:** Server bug
- **Docs say:** `getMidpointsGet` (`GET /midpoints`), `getLastTradesPricesGet`
  (`GET /last-trades-prices`), `getPricesGet` (`GET /prices`) and `getBooksGet` (`GET /books`)
  take a comma-separated `token_ids` query parameter (`/prices` also `sides`), documented as
  working alternatives to the `POST` forms.
- **Live does:** `400 {"error":"Invalid payload"}` for every encoding tried (comma-separated,
  repeated `token_ids` keys, a single id, no parameter). The `POST` forms work.
- **SDK does:** the four `GET` methods are removed (breaking). The `POST` methods took over
  their names: `get_midpoints`, `get_last_trade_prices`, `get_prices`, `get_order_books` (were
  `*_by_body`). The 500-id limit of last-trade-prices and the other client-side validation
  are kept. `ENDPOINTS.md` lists the `GET` rows as not implemented.
- **Pinning test:** `pin_plural_get_forms_are_rejected` (fails when a `GET` form starts
  working, as a prompt to implement it).

### `POST /prices` accepts an item without `side` and silently prices `SELL` only

- **Kind:** Undocumented
- **Docs say:** `getPricesPost`: each request must include both `token_id` and `side`.
- **Live does:** `[{"token_id": T}]` answers `200 {"T":{"SELL":"0.001"}}` (a `BUY` entry
  is not returned).
- **SDK does:** keeps the documented requirement client-side: a request without a side is an
  `Error::Validation` (parameter `side`), so the silent default cannot be hit by accident.
- **Pinning test:** `pin_post_prices_without_side_is_sell_only`.

### Price values are numeric strings

- **Kind:** Mismatch
- **Docs say:** `getPrice` returns `{"price": 0.45}` (a JSON number); `getPricesPost` returns
  `{"<token>": {"BUY": 0.45}}` (numbers).
- **Live does:** strings: `{"price":"0.999"}`, `{"<token>":{"BUY":"0.999"}}`. (`/midpoints`,
  `/spread`, `/last-trade-price` are strings in the docs too; `/tick-size`
  `minimum_tick_size` is a number in both.)
- **SDK does:** decodes both (a `Decimal` accepts either); `Price.price` now serializes back
  as a string like live (it serialized as a number before).
- **Pinning test:** `pin_price_is_a_string`.

### `GET /book` `timestamp` is Unix milliseconds; `hash` and `market` formats

- **Kind:** Mismatch
- **Docs say:** `OrderBookSummary.timestamp` is a string with no stated unit (example
  `"1234567890"`, 10 digits), `hash` the placeholder `"a1b2c3d4e5f6..."`, `market` a
  placeholder address.
- **Live does:** `timestamp` is a 13-digit string of Unix **milliseconds** (e.g.
  `"1790934258308"`); `hash` is 40 hex characters without `0x`; `market` is the 66-character
  `0x` condition id.
- **SDK does:** `OrderBookSummary.timestamp` is a `DateTime<Utc>` read as milliseconds (was a
  raw `String`; breaking) and serialized back as a string; `hash` stays a `String`.
- **Pinning test:** `pin_book_formats` (and the `timestamp` range check in `get_order_book`).

### `GET /rebates/current` `date` is an RFC 3339 date-time

- **Kind:** Mismatch
- **Docs say:** `RebatedFees.date` is a string, "Date of the rebate (YYYY-MM-DD)", example
  `"2026-02-27"`.
- **Live does:** `"2026-09-25T00:00:00Z"` (midnight UTC; no non-midnight value seen).
  Sending the date-time as the `date` query parameter is rejected (`400 Invalid date`); the
  request still takes `YYYY-MM-DD`.
- **SDK does:** `RebatedFees.date` is a `NaiveDate` read from either spelling (the time of
  day is dropped) and serialized as `YYYY-MM-DD`.
- **Pinning test:** `pin_rebates_shape`.

### `GET /rebates/current` answers `null` for a maker without rebates

- **Kind:** Mismatch
- **Docs say:** `getCurrentRebatedFees` 200 is an array of `RebatedFees`.
- **Live does:** a maker with no rebates on the date gets HTTP 200 with the body `null`.
- **SDK does:** `get_current_rebated_fees` maps `null` to an empty `Vec`.
- **Pinning test:** `pin_rebates_shape`.

### `GET /clob-markets/{condition_id}` has undocumented keys

- **Kind:** Undocumented
- **Docs say:** `getClobMarketInfo` / `ClobMarketDetails` list `gst`, `r`, `t`, `mos`, `mts`,
  `mbf`, `tbf`, `rfqe`, `itode`, `ibce`, `fd`, `oas`.
- **Live does:** also sends `c` (condition id), `sd` (integer, `1` or `3` on sports markets,
  omitted when 0, equal to the `seconds_delay` of the market listings), `ao` (bool, accepting
  orders), `aot` (RFC 3339, accepting-order timestamp), `nr` (bool, only on neg-risk markets),
  `cbos` (bool, on every market; meaning unknown) and `v` (`"v1"` on every market). `rfqe`,
  `itode` and `oas` were not seen on the markets checked; `mbf`/`tbf`/`fd` only on markets
  with fees. The rewards object `r` has abbreviated keys (`mi`, `ma`, `moas`, `e`) and stays
  raw JSON.
- **SDK does:** models all seven as `Option`s on `ClobMarketDetails`: `condition_id`,
  `seconds_delay` (`u64`), `accepting_orders`, `accepting_order_timestamp`, `neg_risk`,
  `cbos`, `version`. The meaning of `cbos` is not guessed (field keeps the wire name).
- **Pinning test:** `pin_clob_market_details_undocumented_keys`.

### `GET /builder/trades`: `builder` is empty; `builderCode` and `builderFee` are undocumented

- **Kind:** Undocumented
- **Docs say:** `BuilderTrade.builder` is the builder code the trade is attributed to
  (example `0x00...01`); there is no `builderCode` or `builderFee`.
- **Live does:** `builder` is always `""`; the code is in `builderCode`, and `builderFee` (a
  decimal string) is sent as well.
- **SDK does:** keeps `builder` (required, may be empty) and adds `builder_code:
  Option<BuilderCode>` and `builder_fee: Option<Decimal>`. Rustdoc says to use
  `builder_code`.
- **Pinning test:** `pin_builder_trade_fields`.

### `GET /builder/trades` amounts are decimal units

- **Kind:** Mismatch
- **Docs say:** the example shows micro-units (`size` `"100000000"`, `sizeUsdc`
  `"50000000"`, `fee` `"300000"`); the spec gives no unit.
- **Live does:** decimal units: 5 shares at `0.01` is `size` `"5"`, `sizeUsdc` `"0.05"`;
  `fee`, `feeUsdc` and `builderFee` are decimal too. `outcome` is the market's label
  (`"Up"`, `"Down"`), not `"YES"`; `tradeType` is `"MAKER"` on the trades seen.
- **SDK does:** types are unchanged (`Decimal`); rustdoc states the live units.
- **Pinning test:** `pin_builder_trade_fields` (checks `sizeUsdc = size * price`).

### `GET /markets-by-token/{token_id}`: "primary" is not the Yes token

- **Kind:** Mismatch
- **Docs say:** `MarketByTokenResponse.primary_token_id` is "the primary (Yes) token ID",
  `secondary_token_id` "the secondary (No) token ID".
- **Live does:** the two ids are the market's two tokens, but which is "primary" does not
  follow outcome order: on 20 sampled Yes/No markets, about half have `primary_token_id` equal
  to the first ("Yes") token and half to the second. Either token of the market gives the same
  answer.
- **SDK does:** types unchanged; rustdoc warns not to assume "primary" is "Yes" and points to
  `get_clob_market_info` for outcomes.
- **Pinning test:** `pin_market_by_token_primary_is_not_always_yes`.

### `GET /prices-history` `interval` and `fidelity` rules

- **Kind:** Undocumented
- **Docs say:** `interval` is one of `max`, `all`, `1m`, `1w`, `1d`, `6h`, `1h` ("Time interval
  for data aggregation"); `fidelity` is "accuracy in minutes"; nothing on how they combine.
  The same applies to `POST /batch-prices-history`.
- **Live does:** `1m` is one **month** (about 30 days) and answers `400 invalid filters:
  minimum 'fidelity' for '1m' range is 10` below a fidelity of 10, including when `fidelity`
  is omitted (the default is 1); `1w` needs at least 5; `1h`, `6h`, `1d` accept 1. `max` and
  `all` return the same series (the whole history, capped at roughly 4,300 points, so a small
  fidelity only covers the recent past). With `startTs`/`endTs` the window of the timestamps
  is used, but the interval's fidelity minimum still applies; `interval` and the timestamps
  are not mutually exclusive. `startTs`/`endTs` are Unix seconds (milliseconds give `400`
  "interval is too long"); `endTs` alone is rejected.
- **SDK does:** documents all of it on `PriceHistoryInterval`; the request builders reject
  `1m` without `fidelity >= 10` and `1w` without `fidelity >= 5` with an
  `Error::Validation` (parameter `fidelity`) before sending (`MIN_FIDELITY_ONE_MONTH`,
  `MIN_FIDELITY_ONE_WEEK`).
- **Pinning test:** `pin_prices_history_interval_rules`.

### `GET /prices-history` appends a point at the current time

- **Kind:** Undocumented
- **Docs say:** `startTs`/`endTs` bound the returned points.
- **Live does:** the series covers the window, plus one extra final point stamped with the
  current time even when `endTs` lies in the past (e.g. a 2-day window ending 3 days ago ends
  with a point from now). `POST /batch-prices-history` does the same.
- **SDK does:** returns the points as sent and documents the quirk; it does not filter.
- **Pinning test:** `pin_prices_history_appends_a_now_point`.

### Cursor listings end with `"LTE="`; `"LTE="` is not a valid request cursor

- **Kind:** Undocumented
- **Docs say:** `"LTE="` as the last-page `next_cursor` is documented for the rewards
  listings and `/builder/trades`; the market listings (`/simplified-markets`,
  `/sampling-markets`, `/sampling-simplified-markets`) document no end marker.
- **Live does:** every cursor-paged route checked (the three market listings,
  `/rewards/markets/current`, `/rewards/markets/multi`, `/builder/trades`) ends with
  `next_cursor: "LTE="`, and a past-the-end cursor gives an empty page ending with it.
  Sending `LTE=` as `next_cursor` is rejected with `400 next item should be greater than or
  equal to 0`. Cursors are opaque base64 (`"aWQ6MjQ5Mzk2"` is `id:249396` on the market
  listings, an offset such as `"NTAw"` on the others).
- **SDK does:** `END_CURSOR` / `next_cursor()` treat `"LTE="` (and an empty cursor) as the
  end on every listing (resolves open question 15); streams never send it.
- **Pinning test:** `pin_end_cursor_is_lte`.

### Market listing pages: 1000 items, `rewards.rates: null`

- **Kind:** Undocumented
- **Docs say:** `PaginatedSimplifiedMarkets` / `PaginatedMarkets` have `limit`, `count`,
  `next_cursor`, `data`, with no sizes; `Rewards.rates` is an array.
- **Live does:** pages hold 1000 markets, and markets that are closed or have no rewards
  program carry `rewards: {"rates": null, "min_size": 0, "max_spread": 0}`. Closed markets
  are listed too (`active: true, closed: true, accepting_orders: false`); some tokens have an
  empty `outcome`.
- **SDK does:** `Rewards.rates` is `Option<Vec<_>>` (already); nothing else needed.
- **Pinning test:** `pin_market_listing_shape`.

### `GET /rewards/markets/multi` `end_date` format

- **Kind:** Mismatch
- **Docs say:** `MultiMarketInfo.end_date` is a string with no format (example
  `"2024-08-10 00:00:00"`).
- **Live does:** `"2025-05-01 12:00:00+00"` (UTC, space separator, short `+00` offset), or
  `""` for a few markets.
- **SDK does:** `MultiMarketInfo.end_date` is `Option<DateTime<Utc>>` (was `Option<String>`;
  breaking), reading both spellings as UTC and `""` as `None`; serializes as RFC 3339.
- **Pinning test:** `list_markets_with_rewards` (decodes a live page; the typed field fails
  the decode if the format changes).

### Gamma id equivalences (open question 17)

- **Kind:** Undocumented
- **Docs say:** `LiveActivityMarket.id` is the "Internal market ID"; `Market.question_id` has
  no description.
- **Live does:** `LiveActivityMarket.id` equals the Gamma market `id` (a JSON number here, a
  string in Gamma) and `Market.question_id` equals the Gamma market `questionID`.
- **SDK does:** types unchanged (`i64` and `String`); rustdoc states the equivalence.
- **Pinning test:** `pin_gamma_id_equivalences`.

## Data API v2

<!-- data entries -->

## Relayer API

<!-- relayer entries -->

## Bridge API

<!-- bridge entries -->

## Combos / RFQ REST

<!-- combos entries -->

## WebSocket channels

<!-- ws entries -->
