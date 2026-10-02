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
- **Live:** without `limit` the API returns `1000` markets; `limit=1000`, `limit=1001`, `limit=5000` and `limit=10000` are all accepted with `200` and return that many markets (no maximum was found below `10000`; `20000` and `100000` get `400`). `limit=0`, a negative or a non-numeric value get `400 {"error":"invalid limit"}`, as documented (`minimum: 1`).
- **SDK:** `ListComboMarkets::limit` accepts `1..=1000` (`ValidationError` otherwise); the old client-side maximum of `100` was dropped. Without `limit` the live default of `1000` applies. The upper bound of `1000` is conservative: the live maximum was only probed, not documented.
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
