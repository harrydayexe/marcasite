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
