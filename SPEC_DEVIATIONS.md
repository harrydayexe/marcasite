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

<!-- clob entries -->

## Data API v2

Pinning tests are in `crates/polyoxide/tests/live/data.rs` (`just test-live data`) unless a path is
given; the offline decode tests of captured rows are in
`crates/polyoxide/tests/api/data/live_fixtures.rs`. Spec = `docs/specs/data-v2-openapi.json`.

### Combo condition ids are `0x` + 62 hex digits

- **Kind**: Mismatch.
- **Docs say**: `ComboPosition.combo_condition_id` and the `condition` filter of
  `/v2/positions/combos` and `/v2/activity/combos` are "combo condition id (structural,
  `0x03`-prefixed)"; the overview describes the `condition` key as `0x` plus 64 hex digits.
- **Live does**: every combo condition id served (428 positions and 526 activity rows sampled) is
  `0x03...` plus **62** hex digits in total (31 bytes, e.g.
  `0x033c72a79df1dfd46683b15b5c0ce78ef50000000000000000000000000000`). The `condition` filter accepts
  exactly that: any `0x` plus 62 hex digits (either case, any leading byte); a 64-digit id, a 61- or
  63-digit one, a bare `0x03` or an id without the prefix is `400 invalid combo condition id`
  (`parameter=condition`).
- **SDK does**: the combo filters (`ListComboPositions::conditions`, `ListComboActivity::conditions`)
  validate `0x` plus 1 to 64 hex digits (deliberately looser than live, so a change of the id length
  does not become a client-side rejection; the server has the last word). Regular condition filters
  keep the bytes32 check. Ids are decoded as `ConditionId` without validation.
- **Pinning test**: `combo_condition_filter_accepts_live_ids`.

### Combo `leg_condition_id` and leg `end_date` differ between the two combo routes

- **Kind**: Mismatch.
- **Docs say**: `ComboLeg.leg_condition_id` is the "on-chain condition id of the leg's market" and
  `ComboLegMarket.end_date` is "RFC3339; empty when Gamma has none", on both routes (same schema).
- **Live does**: `/v2/activity/combos` serves the market's bytes32 condition id and an RFC 3339
  `end_date` (`2026-10-09T03:00:00Z`). `/v2/positions/combos` serves, for the same leg (same
  `leg_position_id`), a 62-digit `0x01`/`0x02`-prefixed value that is not the market's condition id
  (e.g. `0x0104db2bc4f21eef1caf06186806e548800000000000000000000000000000`) and a bare-date
  `end_date` (`2026-10-09`).
- **SDK does**: keeps `leg_condition_id` as served in a `ConditionId` (no validation), documents the
  difference on `ComboLeg::leg_condition_id`, and decodes both `end_date` forms (a bare date is
  midnight UTC).
- **Pinning test**: `combo_leg_condition_ids_differ_between_routes`; offline
  `combo_position_with_62_digit_ids` and `combo_activity_with_bytes32_leg_ids`.

### `Position.first_entry_at`

- **Kind**: Undocumented.
- **Docs say**: the `Position` schema has no such field.
- **Live does**: every `/v2/positions` row carries `first_entry_at`, an integer in epoch seconds
  (`0` on rows with no native state, which also have `last_event_at: 0`). Never `null` or absent in
  ~8 900 rows sampled.
- **SDK does**: `Position::first_entry_at: Option<DateTime<Utc>>`; `0`, `null` and an absent key
  decode as `None`, and `None` serializes back as `0`.
- **Pinning test**: `list_positions_filters` (asserts the raw integer and the model value);
  offline `position_without_native_state_has_no_first_entry`.

### Activity: types missing from the docs, and `""` ids on non-trade rows

- **Kind**: Undocumented (types), Mismatch (ids).
- **Docs say**: `Activity.type` is "TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION, …" (plus the
  opt-in `TIP`); `condition_id` and `token_id` are required strings described as a condition id and
  a CLOB asset id; `exclude_deposits_withdrawals` defaults to `true`.
- **Live does**: also serves `MAKER_REBATE`, `TAKER_REBATE`, `YIELD`, `REFERRAL_REWARD` and, with
  `exclude_deposits_withdrawals=false`, `DEPOSIT` and `WITHDRAWAL`. Rows that touch no market
  (those types and `REWARD`) send `condition_id`, `token_id` and `side` as `""`; `MERGE`, `SPLIT`
  and `CONVERSION` rows send `token_id: ""` with a real `condition_id`. `type=DEPOSIT` or
  `type=WITHDRAWAL` is an empty page under the default flag and returns rows with
  `exclude_deposits_withdrawals=false` (so it is the documented default at work, not a server bug);
  `type=YIELD`, `MAKER_REBATE`, `TAKER_REBATE` and `REFERRAL_REWARD` filter normally. An unknown
  type is `400` (`parameter=type`, `unknown activity type: ...`).
- **SDK does**: `ActivityType` has `MakerRebate`, `TakerRebate`, `Yield`, `ReferralReward`,
  `Deposit`, `Withdrawal`. `Activity::condition_id` and `token_id` are `Option` (`""` decodes as
  `None`, serializes back as `""`, like `side`). The filter is not restricted client-side (it is
  the same enum and a valid request); the rustdoc of `ListActivity::types` and
  `exclude_deposits_withdrawals` says that Deposit/Withdrawal need the flag off.
- **Pinning tests**: `activity_live_only_types_decode`, `activity_type_filters_and_the_deposit_flag`,
  `list_activity_type_filters`; offline `non_trade_activity_rows_decode_with_empty_ids`.

### `condition` together with `event_id` is rejected

- **Kind**: Undocumented.
- **Docs say**: on `/v2/activity` `event_id` is "mutually exclusive with `condition`"; on
  `/v2/positions` it is "user-anchored only" and `condition` "narrows that user's positions"; on
  `/v2/trades` both are plain filters.
- **Live does**: `400` "must provide either eventId or condition, not both" on all three routes,
  with or without `user`. Either filter alone works.
- **SDK does**: `ListPositions`, `ListTrades` and `ListActivity` reject the pair with a
  `ValidationError` on `event_id` before sending.
- **Pinning test**: `condition_with_event_id_is_rejected_live`.

### `user` validation is inconsistent across routes

- **Kind**: Mismatch.
- **Docs say**: only `/v2/approvals` and `/v2/user-stats` document the address format ("Required EVM
  address", "`0x` followed by 40 hexadecimal characters"); `/v2/user-pnl` just says "Proxy wallet".
- **Live does**: `400 invalid user address` (`parameter=user`) on `/v2/approvals`, `/v2/user-pnl`
  and `/v2/user-stats`; every other route (`value`, `user-volume`, `positions`, `positions/combos`,
  `activity`, `activity/combos`, `trades`) accepts any string and answers `200` with an empty page
  or zeros (`/v2/value` echoes the string back in `proxy_wallet`).
- **SDK does**: validates `0x` plus 40 hex digits client-side on approvals, user-stats and
  user-pnl; elsewhere only a blank `user` is rejected and the value is forwarded.
- **Pinning test**: `malformed_user_is_validated_on_three_routes_only`.

### `/v2/user-stats` answers `data: null` for some active users

- **Kind**: Mismatch.
- **Docs say**: `data: null` means the wallet is not a known user; a known user that never traded
  gets zeros.
- **Live does**: an arbitrary unknown wallet gets `data: null`, as documented, but so do some
  active, ranked users (the day leaderboard's first and third place on 2026-10-02: 4 of 111 board
  wallets). The wallet `0x0000000000000000000000000000000000000001` gets a row of zeros.
- **SDK does**: `get_user_stats` returns `Ok(None)` for `data: null`; the rustdoc says `None` means
  "no stats available", not "no such user".
- **Pinning test**: `get_user_stats_miss_is_none` (fails if no board wallet has `null` stats any
  more).

### `/v2/user-pnl` points: the cash-flow amounts are always `null`

- **Kind**: Mismatch.
- **Docs say**: `deposits`, `withdrawals` and `cashflow_net` are nullable amounts; `unrealized_pnl`
  and `position_pnl` are nullable ("`None` when its source was unavailable"). The offline fixtures
  assumed `unrealized_pnl` / `position_pnl` are the `null` ones.
- **Live does**: over ~188 000 points (25 wallets, intervals `1d` and `max`) only `deposits`,
  `withdrawals` and `cashflow_net` are ever `null` (always); every other optional amount is
  present, `unrealized_pnl` and `position_pnl` included. `fidelity` echoes the request (default
  `1h`), `interval` defaults to `1d`, and `source_fidelity` was `1d` every time.
- **SDK does**: all stay `Option<Decimal>` (the schema allows `null`); the rustdoc states the live
  pattern.
- **Pinning test**: `get_user_pnl`; offline `pnl_point_has_the_live_null_pattern`.

### `/v2/prices-history`: window rules

- **Kind**: Mismatch (cap, cursor), Undocumented (alignment).
- **Docs say**: `start` "is capped at 15 days back from now" (read as: clamped); the cursor is
  "opaque" (nothing says the window must be restated); `0` bounds are documented as `400`.
- **Live does**: an explicit window longer than 15 days (`end - start`, or now `- start` without an
  `end`) is `400 'start' to 'end' must span at most 15 days`, not clamped; exactly 15 days is fine.
  A page with a cursor but no window form (`start`, `interval` or `as_of`) is `400` ("provide a time
  component"); restating any one form works, and the page's `pagination.offset` is the running row
  count. `start`, `end` and `as_of` of `0` are `400` naming the parameter; `end` before `start` is
  `400`. Points are bucket-aligned: the first can precede `start` by less than one bucket, and
  `resolution_seconds` echoes the requested bucket; the final point of the window is an exact tick
  (`resolution_seconds: 0`, unaligned timestamp).
- **SDK does**: `ListPricesHistory` rejects, before sending, a cursor without a window form, a
  `start`..`end` span over 15 days, `end` before `start` and `0` bounds, all with a
  `ValidationError`. A `start` alone over 15 days back cannot be checked without the clock and is
  left to the server.
- **Pinning tests**: `prices_history_window_rules`, `prices_history_points_are_bucket_aligned`,
  `prices_history_zero_start_is_400_with_parameter`.

### Leaderboard: unranked is `null`, never `0`

- **Kind**: Mismatch (the docs contradict each other).
- **Docs say**: the `LeaderboardUserEntry` schema says `rank_pnl` / `rank_volume` are `null` when
  unranked; the `/v2/leaderboard` description says a rank of `0` means unranked, not first.
- **Live does**: `null` (a wallet ranked on one board only has `null` on the other); `0` never
  appears. A wallet on neither board is `data: null`.
- **SDK does**: `Option<u32>` as served; the rustdoc says `None` is unranked and treats a
  hypothetical `Some(0)` as unranked too.
- **Pinning test**: `get_leaderboard_standing_unranked` (fails on any `Some(0)`).

### Builders leaderboard: `builder_name`, not `builder`

- **Kind**: Mismatch (docs prose vs schema).
- **Docs say**: the endpoint page's prose calls the display name `builder`; the schema and the
  volume-over-time page use `builder_name` (and `builder_code` as the key).
- **Live does**: `builder_name` and `builder_code`.
- **SDK does**: follows the schema and live.
- **Pinning test**: `list_builders_leaderboard`.

### Decimals in exponent notation

- **Kind**: Undocumented.
- **Docs say**: amounts are `number` / `double`; no example shows exponent notation.
- **Live does**: tiny amounts are JSON numbers in exponent form (`"entry_fees_usdc": 9e-6`,
  `"total_cost_usdc": 1e-6` on CLOSED positions and others).
- **SDK does**: decodes them exactly into `Decimal` (`9e-6` is `0.000009`); amounts serialize back as JSON numbers.
- **Pinning test**: offline `closed_position_with_exponent_decimals_and_first_entry_at`; live
  `list_positions_filters` decodes live CLOSED pages.

Checked and **not** a deviation: the `/v2/positions` `title` limit is 200 *characters*, not bytes
(`positions_title_limit_counts_characters`); the SDK counts characters.

## Relayer API

<!-- relayer entries -->

## Bridge API

<!-- bridge entries -->

## Combos / RFQ REST

<!-- combos entries -->

## WebSocket channels

<!-- ws entries -->
