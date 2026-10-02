# Open questions

Points where `docs/` is ambiguous, contradictory or silent. Each has been modelled conservatively
(noted below) and needs a decision from the maintainer, or clarification from Polymarket, before
v1. Remove an entry once it is resolved.

## Project-wide

1. **Decimal precision from JSON numbers.** Amounts sent as JSON numbers are read via `f64`; values
   with more than ~15 significant digits may be rounded. Enabling `serde_json/arbitrary_precision`
   would make them exact but changes `serde_json` for every crate in the user's build. *Current:*
   documented bound.
2. **Sentinel values** (`outcome_index: 999`, `last_event_at: 0`, `end_date: 1970-01-01`,
   `BiggestWinner.event_id: 0`, `""`): map to `None` or keep raw? *Current:* raw, documented.
3. **Schemas with no `required` list** (all of Gamma; most Relayer/Bridge responses): every field is
   `Option`. Should obviously-present fields become required?
4. **Multiple HTTP forms of one endpoint** (CLOB GET-query / POST-body / path variants): keep separate
   `*_by_body` / `*_by_path` methods? *Current:* kept, cross-referenced.
5. **MSRV** and the signing/auth design are still undecided (see `AGENTS.md`).
6. Request futures from plain `async fn` endpoint methods borrow the client (not `'static`); builders
   and streams are `'static`. Unify?

## Gamma

7. Are string-typed amounts (`Market.liquidity`, `volume`, `fee`, `umaBond`, `umaReward`,
   `CommentPosition.positionSize`) always numeric? *Current:* `Decimal`, `""` → `None`.
8. `outcomes`, `outcomePrices`, `clobTokenIds`, … are typed `string` with no documented encoding. Add
   typed accessors that parse them as JSON lists? *Current:* raw `String`.
9. Keyset endpoints mention BestLines / `external_partners` / Teams / `clob_rewards` and spell
   `fee_schedule` in snake_case, none of which is in the schemas. *Current:* not modelled.
10. `order` field names: snake_case (`volume_num`) or camelCase (`volumeNum`)?
11. Semantics of undocumented filters (`closed`, `active`, `omit_empty`, `tag_match`,
    `events_status`, `recurrence`, …), maximum `limit`, meaning of `limit=0`, search page numbering.
12. Spec type inconsistencies to raise with Polymarket (`templateVariables`, `competitive`,
    `createdBy`, `team*ID`, `closedTime` differ between schemas); `Comment.parentEntityType` values.
13. Should date-times without an offset be rejected rather than read as UTC?

## CLOB

14. Units: REST `/book` `timestamp` (raw `String`), `MultiMarketInfo.end_date` format,
    `prices-history` `startTs`/`endTs` on the GET form, `BuilderTrade` size/fee units.
15. `"LTE="` as end sentinel for simplified/sampling market listings (assumed).
16. `interval` values: is `1m` a month? `max` vs `all`? Is `interval` exclusive with start/end?
17. Are `LiveActivityMarket.id` / `Market.question_id` the shared Gamma `MarketId` / `QuestionId`?

## Data API v2

18. `ComboPosition.first_entry_at` on NULL-tail rows (modelled `Option`, `""` → `None`).
19. Unranked leaderboard rank: `0` (endpoint text) or `null` (schema)? *Current:* raw.
20. `prices-history`: does a cursor page need the window restated; is the 15-day cap a 400 or clamp?
21. Validate `user` as an EVM address on every route (only two document the format)?
22. `source_fidelity` vocabulary; activity `type` values for deposits/withdrawals; `/v2/trades`
    `condition` + `event_id` together; `title` 200-char limit in chars or bytes; builders board
    `builder` vs `builder_name`.

## Relayer / Bridge

23. Bridge `estInputUsd` / `estOutputUsd` descriptions look swapped; `appFeePercent` scale (1 = 1%?).
24. Where the `missing_builder_code` warning appears; are `POST /deposit` / `/withdraw` idempotent?
25. Can `createdTimeMs` be fractional?

## WebSockets

26. Units of market-channel `timestamp` on `tick_size_change`, `best_bid_ask`, `new_market`,
    `market_resolved` (examples look like ms; kept raw). Meaning of subscription `level` 1–3.
27. How is an empty book side sent (`""` assumed → `None`)? Can the market channel send JSON arrays?
28. PolyBolt `price.polymarket`: public (spec) or credentials required (overview page)? Is "64 KB"
    64,000 or 65,536 bytes (64,000 used)?
29. Server heartbeat tolerance and reconnection policy (idle-timeout defaults are 3× the documented
    heartbeat).
