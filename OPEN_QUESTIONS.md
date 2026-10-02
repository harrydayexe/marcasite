# Open questions

Points the docs (`docs/`) and the live API leave unanswered. Each has been modelled conservatively
(noted below) and needs a decision from the maintainer, or clarification from Polymarket, before
v1. Remove an entry once it is resolved. Where the live API answers a question, the SDK follows it and
the answer is recorded in `SPEC_DEVIATIONS.md` instead.

## Project-wide

1. **Decimal precision from JSON numbers.** Amounts sent as JSON numbers are read via `f64`; values
   with more than ~15 significant digits may be rounded. Enabling `serde_json/arbitrary_precision`
   would make them exact but changes `serde_json` for every crate in the user's build. *Current:*
   documented bound. *Live (2026-10-02):* Data API and Bridge numbers are themselves server-side
   `f64` (e.g. `2021415.2981019993`, `0.024320753943372245`), so only `f64` noise is lost.
2. **Sentinel values** (`outcome_index: 999`, `last_event_at: 0`, `end_date: 1970-01-01`,
   `BiggestWinner.event_id: 0`, `""`): map to `None` or keep raw? *Current:* raw, documented
   (`Position.first_entry_at: 0` and Data activity `""` ids map to `None`).
3. **Schemas with no `required` list** (all of Gamma; most Relayer/Bridge responses): every field is
   `Option`. Should fields that are always present live become required?
4. **Multiple HTTP forms of one endpoint**: CLOB plural `GET` forms were dropped (they answer `400`
   live) and the `POST` forms took the plain names; single-token `*_by_path` forms are kept beside
   the query forms. Keep both single-token forms?
5. **MSRV** and the signing/auth design are still undecided (see `AGENTS.md`).
6. Request futures from plain `async fn` endpoint methods borrow the client (not `'static`); builders
   and streams are `'static`. Unify?

## Gamma

7. Are string-typed amounts (`Market.liquidity`, `volume`, `fee`, `umaBond`, `umaReward`,
   `CommentPosition.positionSize`) always numeric? *Current:* `Decimal`, `""` → `None`. No
   non-numeric value was seen live, but no exhaustive scan was made.
8. Keyset endpoints mention BestLines / `external_partners`, none of which has been seen live or is
   in the schemas. *Current:* not modelled (`clobRewards`, `teams` and `feeSchedule` are).
9. Undocumented filter semantics still unknown: `closed`, `active`, `omit_empty`, `tag_match`,
   `events_status`, `recurrence`, maximum `limit`, meaning of `limit=0`, search page numbering.
   (Known from live: `order` is camelCase, `decimalized` filters by tick size, offset caps.)
10. Spec type inconsistencies (`templateVariables`, `competitive`, `createdBy`, `team*ID`,
    `closedTime` differ between schemas). `/events?order=competitive` answers `500` live.
11. Should date-times without an offset be rejected rather than read as UTC?
12. `/comments/keyset` exists live (same parent parameters, `comments` array) but is in neither the
    docs nor the specs. Implement it?

## CLOB

13. Meaning of the undocumented `ClobMarketDetails` keys `cbos` (bool) and `sd` (equals the
    listing's `seconds_delay`, values 1 and 3; what is delayed?). *Current:* modelled, documented
    as unknown.
14. `LiveActivityMarket.id` is the Gamma market id and `Market.question_id` is Gamma's `questionID`
    (confirmed live). Switch them to the shared `MarketId` / `QuestionId` types? That changes
    `LiveActivityMarket.id` serialization from a number to a string.

## Data API v2

15. Should combo condition ids get their own `ComboConditionId` newtype (breaking)? Note that
    `leg_condition_id` differs between `/positions/combos` (62-digit) and `/activity/combos`
    (bytes32) for the same leg.

## Relayer / Bridge / Combos

16. Where the `missing_builder_code` warning appears; are `POST /deposit` / `/withdraw` idempotent?
    (Not observable read-only.)
17. Combos `limit`: live accepts up to at least `10000` (`20000` is a `400`); the SDK caps at
    `1000`. Raise it?

## WebSockets

18. Meaning of market-channel subscription `level` 1–3 (no observable effect live).
19. Sports channel: `elapsed`, `turn`, `turnProviderId` and `sportradarGameId` were seen on early
    frames but their value types were never captured (modelled as optional strings). Recapture
    during US/EU game hours. `status` casing varies (`inprogress`, `InProgress`, `running`): keep a
    raw string with `status_is`, or a case-insensitive enum?
20. Market-channel `new_market` `fee_schedule`: every field is `Option` because few frames were
    seen. Tighten?
21. PolyBolt: is "64 KB" 64,000 or 65,536 bytes (64,000 used)? Reconnection policy for all channels.
