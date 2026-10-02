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

<!-- clob entries -->

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
