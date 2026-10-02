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

Spec lines refer to `docs/specs/gamma-openapi.yaml`. Pinning tests are in
`crates/polyoxide/tests/live/gamma.rs`; responses captured from the live API for offline tests are
in `crates/polyoxide/tests/api/gamma/fixtures/live/` (decoded by `tests/api/gamma/live.rs`).

### Gamma: JSON lists inside strings (Mismatch)

- **Docs say:** `Market.outcomes`, `outcomePrices`, `clobTokenIds` and `umaResolutionStatuses` are
  `type: string` with no encoding (schema `Market`, lines 1901, 1904, 2038, 2210).
- **Live does:** every route sends JSON-encoded lists inside strings (`"[\"Yes\", \"No\"]"`,
  prices as decimal strings), except `GET /public-search?optimized=true`, which sends real JSON
  arrays for `outcomes` and `outcomePrices` (and no `clobTokenIds` / `umaResolutionStatuses`).
  Every value in the tens of thousands of market records sampled parsed (prices are always
  numeric strings).
- **SDK does:** `Market::outcomes` and `uma_resolution_statuses` are `Option<Vec<String>>`,
  `outcome_prices` is `Option<Vec<Decimal>>`, `clob_token_ids` is `Option<Vec<TokenId>>`, decoded
  with `serde_util::json_string_option` (accepts both encodings; `null` and `""` are `None`) and
  serialized back to the string form. Text that is not a JSON list is a decode error.
- **Pinned by:** `market_list_fields_are_json_encoded_strings`, `search_optimized`.
  Offline: `gamma::live::*` and `list_in_string_fields_accept_both_encodings`.

### Gamma: `public-search?optimized=true` changes the response shape (Mismatch)

- **Docs say:** `optimized` is a bare boolean (line 1672) and the response is schema `Search`
  with `pagination` (line 3196).
- **Live does:** with `optimized=true` the response has a top-level `hasMore` and no
  `pagination`; events carry only `active`, `archived`, `closed`, `closedTime`, `endDate`,
  `ended`, `id`, `image`, `markets`, `negRisk`, `slug`, `startDate` and `title`, and their
  markets only `active`, `archived`, `bestAsk`, `bestBid`, `closed`, `closedTime`,
  `groupItemTitle`, `lastTradePrice`, `outcomePrices`, `outcomes`, `question`, `slug` and
  `spread`; `outcomes` and `outcomePrices` are real arrays (see above).
- **SDK does:** `SearchResults::has_more` (`hasMore`) next to `pagination`, both `Option`; the
  slim events and markets decode into the ordinary `Event` and `Market` (absent fields are
  `None`). The `Search::optimized` rustdoc says so.
- **Pinned by:** `search_optimized`.

### Gamma: comment parent entity types (Mismatch)

- **Docs say:** `GET /comments` `parent_entity_type` is one of `Event`, `Series`, `market`
  (lines 1436 to 1443).
- **Live does:** accepts `Event`, `Series` and `PerpsAsset`; `market` is a `422`
  (`expected value to be one of "Event, Series, PerpsAsset"`). Comments on perps assets come back
  with `parentEntityType: "PerpsAsset"`.
- **SDK does:** `CommentParentEntityType` has `Event`, `Series`, `PerpsAsset` (+ `Unknown`); there
  is no `Market` variant. `Comment::parent_entity_type` is now this enum instead of a `String`.
- **Pinned by:** `list_comments_market_parent_type` (the 422 for `market`),
  `list_comments_perps_asset_parent_type`.

### Gamma: comments require a parent type and id (Mismatch)

- **Docs say:** no parameter of `GET /comments` is required (lines 1425 to 1458).
- **Live does:** `parent_entity_type` and `parent_entity_id` are both required; omitting either is
  a `422` (`required query parameter is missing`).
- **SDK does:** `GammaClient::list_comments(parent_entity_type, parent_entity_id)` takes both as
  required arguments (breaking change: the optional setters are gone).
- **Pinned by:** `comments_require_parent_type_and_id`.

### Gamma: `order` takes camelCase field names (Mismatch)

- **Docs say:** `order` is a "comma-separated list of JSON field names"; the keyset example is
  `volume_num,liquidity_num` (lines 882 to 884); the shared parameter says only "fields to order
  by" (lines 1697 to 1702).
- **Live does:** snake_case names are a `422` (`order fields are not valid`) on `/markets`,
  `/markets/keyset`, `/events`, `/events/keyset`, `/events/pagination`, `/series`, `/tags`,
  `/teams` and `/comments`; the camelCase JSON names work (`volumeNum`, `liquidityNum`,
  `volume24hr`, `startDate`, `endDate`, `createdAt`, `id`).
- **SDK does:** `order` stays a free-form string passed through unchanged; the setters' rustdoc
  lists the camelCase names and warns about snake_case. (Request filters such as
  `volume_num_min` stay snake_case; they work.)
- **Pinned by:** `order_fields_are_camel_case`.

### Gamma: `decimalized` on the markets keyset filters by tick size (Mismatch)

- **Docs say:** `decimalized` is a bare boolean (lines 920 to 923).
- **Live does:** it filters on `orderPriceMinTickSize`: `true` returns only markets with `0.001`,
  `false` only `0.01`; omitting it returns both.
- **SDK does:** `ListMarketsKeyset::decimalized` is sent as given; the rustdoc describes the
  filter semantics.
- **Pinned by:** `decimalized_filters_by_tick_size`.

### Gamma: offset caps (Undocumented)

- **Docs say:** `offset` is an integer with `minimum: 0` and no maximum (components/parameters
  `offset`, lines 1691 to 1696).
- **Live does:** `offset` above 2000 on `GET /markets`, `/events` and `/events/pagination` is a
  `422` (`offset too large, use /markets/keyset for deeper pagination`); above 200 on
  `GET /comments` and `/comments/user_address/{user_address}` likewise (`/comments` points at a
  `/comments/keyset` route that the docs and specs do not contain). `/tags`, `/series`,
  `/teams`, `/events/results` and `/events/creators` accept any offset.
- **SDK does:** refuses an offset above the cap before sending with `Error::Validation`
  (parameter `offset`, message pointing at `list_markets_keyset` / `list_events_keyset`); the
  offset streams yield every page up to the last accepted offset, then that validation error, and
  end. `GET /comments/keyset` is not implemented.
- **Pinned by:** `offset_past_2000_is_typed_error`, `comment_offset_cap_is_200`.

### Gamma: timestamps are not uniformly RFC 3339 (Mismatch)

- **Docs say:** `format: date-time` for most timestamps; plain strings for `Market.closedTime`,
  `gameStartTime`, `umaEndDate` and `Tag.publishedAt` (lines 1942, 2032, 1978, 1798).
- **Live does:** fractional seconds of any length (`2025-09-22T21:37:46.914643Z`, `...46.9Z`);
  space-separated `2026-05-28 05:29:05+00` and `... 05:29:05.547+00` for `closedTime`,
  `gameStartTime`, `Tag.publishedAt`, `Event.published_at` and `Series.publishedAt`; some
  `umaEndDate` values end in a `+00:00:00` offset.
- **SDK does:** `DateTime<Utc>` fields use the lenient `serde_util::datetime_option` (every format
  above except the odd offset, which only occurs in `String` fields); the fields the spec types
  as plain strings stay `String`s, kept exactly as sent.
- **Pinned by:** `timestamp_formats_beyond_rfc3339`.

### Gamma: series ordered by volume24hr or liquidity descending returns 500 (Server bug)

- **Docs say:** `GET /series` takes `order` and `ascending` like every list route (line 1295).
- **Live does:** `order=volume24hr&ascending=false` answers `500` (`internal error`) once `limit`
  is large enough (20 and up on 2026-10-02; it failed from 11 earlier that day, so the threshold
  follows the data); `order=liquidity&ascending=false` does the same from `limit` 50. A small
  `limit`, ascending order or `order=volume` work. `GET /events?order=competitive` is also a
  `500`.
- **SDK does:** nothing (no retry or fallback); the `order` rustdoc warns. The 500 surfaces as
  `Error::Api`.
- **Pinned by:** `list_series_order_volume24hr_descending` (asserts the 500, so a fix fails the
  test).

### Gamma: undocumented fields of `Event` (Undocumented)

- **Docs say:** schema `Event` (line 2291) lists none of these keys.
- **Live does:** sends `countryName`, `cumulativeMarkets`, `electionType`, `eventMetadata`
  (free-form object), `gameId` (integer), `negRiskAugmented`, `parentEventId` (integer), `sport`
  (an object shaped like a `/sports` entry), `turnProviderId`, `usId`, `version`, `liquidityAmm`
  and `teams` (array shaped like `/teams` entries, with an extra `ordering`).
- **SDK does:** models each as `Option` (`event_metadata` as `serde_json::Value`, `sport` as
  `SportsMetadata`, `teams` as `Vec<Team>`).
- **Pinned by:** `undocumented_event_keys` (all but `turnProviderId`, `usId`, `parentEventId` and
  `liquidityAmm`, which are rare and only covered by the captured fixtures).

### Gamma: undocumented fields of `Market` (Undocumented)

- **Docs say:** schema `Market` (line 1835) lists none of these keys.
- **Live does:** sends `negRisk`, `negRiskMarketID`, `negRiskRequestID`, `approved`,
  `comboStatus` (`enabled`, `disabled`, `pending`), `cyom`, `feeType`, `holdingRewardsEnabled`,
  `pagerDutyNotificationEnabled`, `positionIds` (real array), `submitted_by`, `version`,
  `marketMetadata` (free-form), `marketMakerAddress`, `liquidityAmm`, `volumeAmm`,
  `volume1moAmm`, `volume1wkAmm`, `volume1yrAmm`, `volume24hrAmm`, `fpmmLive`,
  `categoryMailchimpTag`, `sentDiscord`, `twitterCardLocation`, `twitterCardLastRefreshed`
  (milliseconds as a string), `twitterCardLastValidated` (seconds as a string), `subcategory` and
  `clobRewards` (array of `{id, conditionId, assetAddress, rewardsAmount, rewardsDailyRate,
  startDate, endDate}`).
- **SDK does:** models each as `Option` (amounts as `Decimal`, `submitted_by` and
  `market_maker_address` as `Address`, `clob_rewards` as `Vec<ClobReward>`, the metadata object as
  `serde_json::Value`, the Twitter-card strings and `comboStatus` kept as sent).
- **Pinned by:** `undocumented_market_keys` (the legacy AMM and Twitter-card keys only appear on
  old markets and are covered by the captured fixture `MarketOld.json`).

### Gamma: undocumented fields of `Comment` (Undocumented)

- **Docs say:** schema `Comment` (line 2887) has no `media`.
- **Live does:** comments with a GIF carry `media: [{id, commentID, provider, providerMediaId,
  url, mediaType, altText, createdAt}]`.
- **SDK does:** `Comment::media: Option<Vec<CommentMedia>>`.
- **Pinned by:** `undocumented_comment_media` (looks for a comment with media in the busiest
  events; prints a notice instead of failing if none is found).

### Gamma: undocumented fields of `Profile`, `PublicProfile` and `PublicProfileUser` (Undocumented)

- **Docs say:** schemas `Profile` (line 2953), `PublicProfileResponse` (3026) and
  `PublicProfileUser` (3073) have none of these.
- **Live does:** `Profile` and `PublicProfileResponse` carry `takerTier` (integer),
  `takerTierName` (e.g. `Tier 0`, `Silver`, `Obsidian`) and `weightedVolume` (number);
  `PublicProfileUser` carries `communityMod`.
- **SDK does:** `taker_tier: Option<i64>`, `taker_tier_name: Option<String>` and
  `weighted_volume: Option<Decimal>` on both profile types, `community_mod: Option<bool>` on the
  user.
- **Pinned by:** `undocumented_profile_keys`.

### Gamma: undocumented fields of `SeriesSummary`, `SportsMetadata`, `Team` and `Tag` (Undocumented)

- **Docs say:** schemas `SeriesSummary` (line 2729), `SportsMetadata` (3216), `Team` (1753) and
  `Tag` (1784) have none of these.
- **Live does:** `SeriesSummary` carries `volume` and `volume24hr` (numbers); each `/sports` entry
  carries `id`, `name`, `createdAt` and `primaryTagId`; teams carry `color` and `providerId`
  (and, inside an event's `teams`, `ordering`); tags returned by `.../related-tags/tags` carry
  `activeEventsCount`.
- **SDK does:** `SeriesSummary::volume` and `volume_24hr` (`Decimal`); `SportsMetadata::id`,
  `name`, `created_at` and `primary_tag_id`; `Team::color`, `provider_id` and `ordering`;
  `Tag::active_events_count`; all `Option`. `SportsMetadata` is also the type of `Event::sport`.
- **Pinned by:** `undocumented_series_sports_team_and_tag_keys`.

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
  end on every listing; streams never send it.
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

### Gamma id equivalences

- **Kind:** Undocumented
- **Docs say:** `LiveActivityMarket.id` is the "Internal market ID"; `Market.question_id` has
  no description.
- **Live does:** `LiveActivityMarket.id` equals the Gamma market `id` (a JSON number here, a
  string in Gamma) and `Market.question_id` equals the Gamma market `questionID`.
- **SDK does:** types unchanged (`i64` and `String`); rustdoc states the equivalence.
- **Pinning test:** `pin_gamma_id_equivalences`.

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
  validate `0x` plus exactly 62 hex digits, as live does. Regular condition filters
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

### Relayer: malformed `address` is accepted with `200`

- **Kind:** Mismatch (docs: `specs/relayer-openapi.yaml`, `GET /nonce`, `/relay-payload` and `/deployed`, `400` for an invalid address).
- **Live:** `GET /nonce?address=not-an-address&type=PROXY`, `/relay-payload?address=not-an-address&type=SAFE` and `/deployed?address=not-an-address` all answer `200` (with a zero nonce / `deployed: false`). An invalid `type` and a missing `address` do answer `400` (`{"error":"invalid type"}`, `{"error":"invalid address"}`).
- **SDK:** validates the address client-side (`0x` plus 40 hex digits) and returns `Error::Validation` without sending, so a typo cannot silently yield a zero nonce. Documented on `RelayerClient::get_nonce`, `get_relay_payload` and `check_deployed`.
- **Pinning test:** `relayer::malformed_addresses_are_accepted_by_live`.

(`GET /transaction` for an unknown id answers `404 {"error":"transaction not found"}` as documented; no deviation.)

## Bridge API

### Bridge: `GET /supported-assets` has a top-level `note`

- **Kind:** Undocumented (docs: `specs/bridge-openapi.yaml`, `SupportedAssetsResponse` has only `supportedAssets`).
- **Live:** the body also has `"note": "These are the currently supported chains and assets for deposits and withdrawals."`.
- **SDK:** `SupportedAssets::note: Option<String>`.
- **Pinning test:** `bridge::get_supported_assets`.

### Bridge: `estInputUsd` / `estOutputUsd` descriptions are swapped

- **Kind:** Mismatch (docs: `specs/bridge-openapi.yaml`, `QuoteResponse`: `estInputUsd` "Estimated token amount received in USD", `estOutputUsd` "Estimated token amount sent in USD").
- **Live:** for a 10 USDC (Polygon) to pUSD quote, `estInputUsd` is `9.9995` (about the amount sent) and `estOutputUsd` is `9.997239` (the amount received, after fees).
- **SDK:** field names follow the wire; the rustdoc of `Quote::est_input_usd` / `est_output_usd` states the live meaning.
- **Pinning test:** `bridge::get_quote` (asserts input is about 10 and output does not exceed input).

### Bridge: percent fields use a 1 = 1% scale

- **Kind:** Undocumented (the spec says "percentage" without a scale; `FeeBreakdown`).
- **Live:** `swapImpact` `0.0226` next to `swapImpactUsd` `0.002261` on a transfer of about 10 USD, i.e. `swapImpact` is in percent (1 = 1%, not 0.01).
- **SDK:** values are kept as sent (`Decimal`); documented on `FeeBreakdown`.
- **Pinning test:** `bridge::get_quote` (asserts `swapImpact * estInputUsd / 100` is about `swapImpactUsd`).

### Bridge: `GET /status/{address}` answers `500` for an address that is not a bridge address

- **Kind:** Server bug (docs: `specs/bridge-openapi.yaml` only describes bridge addresses from `/deposit` and `/withdraw`; `500` appears as a generic example).
- **Live:** a plain wallet address (e.g. a trader's proxy wallet) gets HTTP `500` `{"error":"cannot get transaction status"}`, not a `400` or an empty list.
- **SDK:** surfaces `Error::Api` with status `500` and the message; documented on `BridgeClient::list_transactions`. Callers cannot distinguish this from a transient server error by status alone.
- **Pinning test:** `bridge::list_transactions_for_user_wallet_is_a_500`.

(`createdTimeMs` is always an integer live: `bridge::list_transactions_for_bridge_address`. The SDK decodes it as integer milliseconds.)

## Combos / RFQ REST

### Combos: combo markets have a `pending` field

- **Kind:** Undocumented (docs: `specs/combos-rfq-openapi.yaml`, `ComboMarket`).
- **Live:** every market has `"pending": false` (a boolean).
- **SDK:** `ComboMarket::pending: Option<bool>` (optional so the documented example, which has none, still decodes).
- **Pinning test:** `combos::list_combo_markets` (every market has `pending`).

### Combos: `GET /v1/rfq/combo-markets` page size

- **Kind:** Mismatch (docs: `limit` default `50`, maximum `100`).
- **Live:** without `limit` the API returns `1000` markets; any `limit` from `1` to `10000` is accepted with `200` and returns up to that many markets. The maximum is exactly `10000`: `limit=10001` gets `400 {"error":"invalid rfq: invalid limit"}`, as do `limit=0`, a negative or a non-numeric value (the spec's `minimum: 1` matches).
- **SDK:** `ListComboMarkets::limit` accepts `1..=10000`, the live range (`ValidationError` otherwise). Without `limit` the live default of `1000` applies.
- **Pinning test:** `combos::list_combo_markets_limits`.

(`volume` is an integer for some markets and a float for others; `serde_util::decimal_number` handles both. Ids are full length live, the docs' example abbreviates them.)

## WebSocket channels

### Sports channel: the update shape is not the spec's

- **Kind:** Mismatch (docs: `specs/asyncapi-sports.json`, `SportResult`: required `slug`; snake_case `last_update`, `finished_timestamp`; `period` values such as `1H`, `Q1`, `Top 1st`).
- **Live (400 frames, 2026-10-02: tennis, cricket, esports, MLBB):** camelCase keys and a different set. Running games: `gameId` (an **integer**), `leagueAbbreviation`, `homeTeam`, `awayTeam`, `status`, `score`, `period`, `live`, `ended`. Ended games: `metadataGameId` (a **string**, e.g. `"id2704888975110644"`) instead of `gameId`, no teams or status, sometimes `finishedTimestamp` (RFC 3339 with nanoseconds, e.g. `2026-10-02T09:46:11.137533661Z`). Never seen in this capture: `slug`, `last_update`, snake_case `finished_timestamp`, `elapsed`, `turn`; `elapsed`, `turn`, `turnProviderId` and `sportradarGameId` were seen on frames of an earlier run (types of the last two not captured: modelled as strings that also accept integers). `status` is `running` (esports) or `inprogress` (an earlier run also saw `InProgress`); `period` is e.g. `S1`-`S3`, `1/3`-`3/3`, `2/5`, `FT`, `Live`/`LIVE`; `score` is sport-specific (`6-7(3-7), 6-3, 2-1`, `123-125`, `000-000|0-1|Bo5`). Leagues are lower case (`atp`, `wta`, `wta challenger`, `challenger`, `cricket`, `mlbb`, `lol`, `cs2`, `r6siege`, `val`, `ow`, `dota2`).
- **SDK (breaking):** `SportResult` models the live shape (`game_id: Option<u64>`, `metadata_game_id: Option<String>`, `league_abbreviation`, `home_team`, `away_team`, `status: Option<String>` (raw, with `status_is` for case-insensitive comparison, since `string_enum!` has no case-insensitive matching), `score`, `period`, `elapsed`, `live`, `ended`, `turn`, `turn_provider_id`, `sportradar_game_id`, `finished_timestamp`). `slug` and `last_update` were dropped. `game_id` and `metadata_game_id` are two fields because they differ in type and no relation is known. A frame is `SportsEvent::Update` if it has `gameId` or `metadataGameId`; the spec's `slug` shape is now `SportsEvent::Unknown`. `SportsEvent::Update` is boxed.
- **Pinning tests:** `ws::sports_channel_raw_drift` (no `slug`/`last_update`/`finished_timestamp`; `gameId` or `metadataGameId` on every frame; no frame decodes as `Unknown`), `ws::sports_channel_sdk` (every event is the typed update).

### Sports channel: heartbeat is a protocol-level ping every 15 s, not a text `ping` every 5 s

- **Kind:** Mismatch (docs: `specs/asyncapi-sports.json` and `docs/api-reference/wss/sports.md`: the server sends a text `ping` every 5 s and expects `pong` within 10 s).
- **Live:** no text `ping` in 400 frames or in 90 s of idle observation; the server sends an empty **WebSocket protocol ping frame every 15 s** (observed at 15.0, 30.1, 45.0, 60.0, 75.0 and 90.0 s) and answers a client protocol ping with a pong. A client text `ping` gets no reply.
- **SDK:** protocol pings are answered automatically by the transport and count as activity. The text `ping` to `pong` auto-reply is kept in case the documented behaviour appears. **Bug fixed:** `SportsChannel::DEFAULT_IDLE_TIMEOUT` was 15 s (three times the documented 5 s), which equals the live ping interval, so a quiet channel could time out right as a ping was due. It is now 45 s (three times the live interval).
- **Pinning tests:** `ws::sports_channel_sends_protocol_pings` (a protocol ping arrives within 25 s, no text `ping`); offline `protocol_pings_keep_a_quiet_channel_alive`.

### Market channel: `book` has `tick_size` and `last_trade_price`

- **Kind:** Undocumented (docs: `specs/asyncapi.json`, `book` message).
- **Live:** the initial `book` snapshots also carry `tick_size` (a string, e.g. `"0.01"`) and `last_trade_price` (a string; `""` on an empty book).
- **SDK:** `BookEvent::tick_size: Option<Decimal>` and `last_trade_price: Option<Decimal>` (`""` or absent becomes `None`).
- **Pinning test:** `ws::market_channel_raw_drift` (both present on every initial book), `ws::market_channel_sdk`.

### Market channel: the initial snapshots arrive as a JSON array; an unknown token gets `[]`; levels are ordered worst-first

- **Kind:** Mismatch (docs: `specs/asyncapi.json` shows single-object frames and sorted example levels).
- **Live:** on subscribe the server sends one JSON **array** with a `book` per token (a bare `[]` frame for an unknown or closed token; the connection stays healthy). An empty book side is `[]`, never `""`. Bids arrive ascending (worst first) and asks descending. Every timestamp on the channel is Unix milliseconds (strings).
- **SDK:** array frames decode to one event per element (`[]` to none); ordering is not changed (documented on `BookEvent::bids`); empty sides are empty vectors. `best_bid_ask` still accepts `""` as `None` for its price fields (unchanged; not observed this run).
- **Pinning tests:** `ws::market_channel_raw_drift` (first frame is an array of one `book` per token, bids ascending, ms timestamps), `ws::market_channel_unknown_token_stays_healthy`.

### Market channel: `new_market` has `taker_base_fee`, `fees_enabled` and `fee_schedule`

- **Kind:** Undocumented (docs: `specs/asyncapi.json`, `new_market` message).
- **Live:** with `custom_feature_enabled`, `new_market` carries `taker_base_fee` (string, e.g. `"1000"`), `fees_enabled` (bool) and `fee_schedule` (`{"exponent":"1","rate":"0.07","taker_only":true,"rebate_rate":"0.2"}`: decimal strings and a bool).
- **SDK:** `NewMarketEvent::{taker_base_fee: Option<Decimal>, fees_enabled: Option<bool>, fee_schedule: Option<FeeSchedule>}`; `FeeSchedule` has optional fields (only the types were observed, not whether each is always present). The unit of `taker_base_fee` is undocumented.
- **Pinning test:** `ws::market_channel_new_market_fee_fields` (reports instead of failing if no `new_market` arrives within 60 s).

### PolyBolt `price.polymarket` is public

- **Kind:** Mismatch between docs pages only (the overview page says credentials are required; `specs/polybolt-asyncapi.json` says public).
- **Live:** works with no credentials; the spec is right. No SDK deviation.
- **Pinning test:** `ws::polybolt_price_polymarket_sdk`.
