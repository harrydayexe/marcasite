# AGENTS.md

Guidance for AI coding agents working on **polyoxide**, an unofficial Rust SDK for the
Polymarket Predictions APIs.

## Project goals

- Idiomatic, production-grade Rust library following the
  [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Full, strongly-typed coverage of the Polymarket Predictions APIs (REST and WebSocket).
- Precise, typed error handling. No panics on any input coming from the network.
- Unofficial: never imply affiliation with or endorsement by Polymarket.

## Source of truth for the API

**Every statement about the Polymarket API (endpoints, methods, parameters, request bodies,
response types, enums, auth headers, rate limits, error shapes) must be grounded in the local
docs in `docs/` or in `https://docs.polymarket.com/api-reference/predictions/overview.md`
and the pages it links to.** Do not rely on memory, other SDKs, or blog posts.

- Read `docs/AGENTS.md` first; it explains how to navigate the docs cheaply.
- `docs/INDEX.md` lists every page. Endpoint pages live in `docs/api-reference/<group>/`.
- Use `docs/specs/` (OpenAPI/AsyncAPI) for exact schemas, enums, required/optional fields and types.
  These files are large; `grep -n` for the `operationId` or schema name instead of reading them whole.
- `docs/` is a verbatim copy (fetched 2026-10-01). **Never edit it.** Re-fetch to update.
- If the docs are ambiguous, contradictory, or silent, **stop and ask the user**. Do not guess a
  field type, nullability or enum variant. Record unresolved questions in the PR / task summary.
- When adding an endpoint, cite the doc page in the item's rustdoc (e.g.
  `/// See <https://docs.polymarket.com/api-reference/markets/get-market-by-id>`).

### Services (from `docs/api-reference/predictions/overview.md` and the specs)

| Service | Base URL | Spec |
|---|---|---|
| Gamma API (events, markets, tags, series, comments, sports, search, profiles) | `https://gamma-api.polymarket.com` | `specs/gamma-openapi.yaml` |
| CLOB API (order book, pricing, orders, trades, rewards, rebates) | `https://clob.polymarket.com` | `specs/clob-openapi.yaml` |
| Data API v2 (wallet, feeds, boards) | `https://data-api.polymarket.com` (routes under `/v2`) | `specs/data-v2-openapi.json` |
| Relayer API | `https://relayer-v2.polymarket.com` | `specs/relayer-openapi.yaml` |
| Combos / RFQ REST | `https://combos-rfq-api.polymarket.com` | `specs/combos-rfq-openapi.yaml` |
| Bridge API | `https://bridge.polymarket.com` | `specs/bridge-openapi.yaml` |
| WS market / user channels | `ws-subscriptions-clob.polymarket.com` (`/ws/market`, `/ws/user`) | `specs/asyncapi.json`, `specs/asyncapi-user.json` |
| WS sports | `sports-api.polymarket.com` (`/ws`) | `specs/asyncapi-sports.json` |
| WS RFQ Quoter Gateway | `combos-rfq-gateway-quoter.polymarket.com` (`/ws/rfq`) | `specs/asyncapi-rfq.json` |
| WS PolyBolt live data | `ws-live-v2.polymarket.com` | `specs/polybolt-asyncapi.json` |

Base URLs must be configurable (e.g. for tests against a mock server), with these as defaults.

### Facts to respect (verify against the docs before relying on them)

- **Data API v2** (`specs/data-v2-openapi.json`, `info.description`): responses use a `data`
  envelope (paged routes add `pagination`); pagination is cursor-only via
  `pagination.next_cursor` until `null`; `offset` is rejected with `400`; params accept snake_case
  and camelCase; errors are JSON with `error`, `code`, `retryable`, `trace_id` (optionally
  `parameter`); `429`/`503` may carry `Retry-After`; every response has an `x-trace-id` header;
  no auth. A missing/`null` numeric means "unavailable", never zero, so model it as `Option<T>`.
  Data API v1 is out of scope.
- **Gamma** offers both offset listing and keyset (`next_cursor`) listing for events and markets.
- **CLOB auth** uses `POLY_*` headers (`POLY_ADDRESS`, `POLY_SIGNATURE`, `POLY_TIMESTAMP`,
  `POLY_NONCE`, `POLY_API_KEY`, `POLY_PASSPHRASE`, ...). Check `securitySchemes` and each
  operation's `security` in `specs/clob-openapi.yaml` for which headers each endpoint needs.
- **Relayer auth** uses Builder API keys (`POLY_BUILDER_*`) or Relayer API keys
  (`RELAYER_API_KEY`, `RELAYER_API_KEY_ADDRESS`); see `specs/relayer-openapi.yaml`.
- **Rate limits** (`docs/api-reference/rate-limits.md`, `trading-rate-limits.md`): Cloudflare
  IP-based limits that throttle rather than reject, plus per-signer token buckets for CLOB order
  and cancel requests. Batch limits are endpoint-specific (e.g. max 15 orders per
  `post-multiple-orders`, 1000 per `cancel-multiple-orders`, 500 token IDs for last-trade-prices);
  enforce documented limits client-side with a typed error where practical.
- **Geoblock**: see `docs/api-reference/geoblock.md`. Do not add any mechanism to bypass it.

## Architecture

Decisions confirmed with the user (2026-10-01):

- Async runtime **tokio**; HTTP via **reqwest** (rustls); WebSockets via **tokio-tungstenite**
  (rustls, explicit aws-lc-rs provider, native roots).
- Layout: services are **modules of the `polyoxide` crate**, each behind a Cargo feature
  (`gamma`, `clob`, `data`, `relayer`, `bridge`, `combos`, `ws`; all default). Shared transport,
  config, errors, pagination, serde helpers and id newtypes live in `polyoxide-core`.
- String enums: `#[non_exhaustive]` with an `Unknown(String)` catch-all
  (use `polyoxide_core::string_enum!`). Id newtypes: `polyoxide_core::string_id!`.
- Money/price/size: `rust_decimal::Decimal`. Timestamps: `chrono::DateTime<Utc>` (helpers in
  `polyoxide_core::serde_util`).
- Scope so far: **unauthenticated endpoints only**. `ENDPOINTS.md` (repo root) is the checklist of
  every endpoint with doc links and implementation status. **Update it with every endpoint change.**

Still undecided (ask the user before choosing): signing/crypto crates and auth design, MSRV.

Principles once decided:

- One client per service (e.g. `GammaClient`, `ClobClient`, `DataClient`), sharing a common
  transport/config. Gate services behind Cargo features so users compile only what they use.
- Builders for clients and for requests with many optional parameters. Required parameters go in
  the constructor / method signature, not in the builder.
- Keep wire types (serde models) separate from transport logic. Module layout mirrors the doc
  groups (`gamma::markets`, `clob::orders`, `data::wallet`, ...).
- Expose pagination as an ergonomic iterator/`Stream` over cursors, plus raw page access.
- Never log or `Debug`-print secrets (API secrets, passphrases, private keys). Wrap them in a
  redacting type (e.g. `secrecy::SecretString`).

## Types

- Model every documented field. Required fields are non-`Option`; optional/nullable fields are
  `Option<T>`. Do not use `#[serde(default)]` to hide a missing required field.
- Use enums for documented string enums. Mark public enums and non-exhaustive structs
  `#[non_exhaustive]`; consider an `Unknown(String)` / catch-all variant so new server values do
  not break deserialization. Ask the user for the policy if unsure.
- Prices, sizes and amounts often arrive as strings: use a decimal type (e.g. `rust_decimal`),
  never `f64`, for money/price/size. Match the wire format exactly as the spec defines it.
- Newtypes for identifiers (`TokenId`, `ConditionId`, `OrderId`, `MarketId`, addresses) so they
  cannot be mixed up.
- Timestamps: typed (e.g. `chrono`/`time`) with the exact documented format/unit.
- Derive `Debug, Clone, PartialEq` (and `Eq`, `Hash` where valid), `Serialize`/`Deserialize` as
  needed. Use `#[serde(rename_all = ...)]` matching the spec's casing.

## Errors

- One public error enum (via `thiserror`), `#[non_exhaustive]`, with variants that distinguish:
  transport, timeout, HTTP status with parsed API error body, deserialization (include the body
  context), rate limit (with `Retry-After` if present), auth/signing, client-side validation.
- Parse each service's documented error shape (e.g. Data API v2 `error`/`code`/`retryable`/
  `trace_id`) into typed fields. Preserve `x-trace-id` where available.
- Return `Result<T, polyoxide::Error>`. No `unwrap`/`expect`/`panic!` in library code
  (tests and examples excepted). No `anyhow` in the public API.

## Code standards

- `cargo fmt` clean; `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `#![deny(missing_docs)]`, `#![forbid(unsafe_code)]` unless the user approves otherwise.
- Every public item has rustdoc; key entry points have runnable doc examples (use `no_run` for
  network calls).
- Prefer borrowing (`&str`, `impl Into<String>`, `impl AsRef<str>`) in public APIs. Public types
  are `Send + Sync`.
- Keep the public API minimal; `pub(crate)` by default. Avoid leaking dependency types where a
  semver break would follow, unless deliberate (document it).
- Follow SemVer. No `CHANGELOG.md` until the v1 release; start one then.

## Testing

- Unit tests for serialization/deserialization using fixtures taken from documented examples in
  `docs/` (cite the source page). Never invent response bodies that contradict the spec.
- Integration tests against a mock HTTP/WS server (e.g. `wiremock`); no live network in default
  `cargo test`. Live tests, if any, are `#[ignore]` and read-only.
- Test error paths: non-2xx bodies, malformed JSON, rate limiting, unknown enum values.
- Signing/auth code needs deterministic test vectors.

## Implementation patterns (follow the existing code)

- Each service client (`crates/polyoxide/src/<service>/client.rs`) wraps a
  `polyoxide_core::Transport`; endpoints are methods added in `impl <Service>Client` blocks in the
  topic module (e.g. `gamma/tags.rs`). `gamma/tags.rs` is the reference implementation.
- Only required params → `async fn`. Optional params → method returns a `#[must_use]` request
  builder (owns a client clone) with setters and `async fn send(self)`. Paginated endpoints also
  get `into_stream()` via `polyoxide_core::pagination::{cursor_stream, offset_stream}`.
- Paths are built from segments (`transport.get(&["tags", id.as_str()])`), which percent-encodes
  user input. Query strings via `polyoxide_core::Query` (`push_all` = repeated keys, `push_csv` =
  comma-separated).
- Enforce documented limits (batch sizes, ranges) client-side with `ValidationError` before
  sending.
- Unit tests for (de)serialization next to the types; mock-server tests in
  `crates/polyoxide/tests/api/<service>/` (one test binary).

## Commands

| Command | Purpose |
|---|---|
| `just check` | Everything CI runs: fmt check, clippy, test, doc, deny |
| `just fmt` | Format all code |
| `just clippy` | Clippy with `-D warnings` |
| `just test [args]` | `cargo test --all-features`, extra args forwarded |
| `just doc` | Build docs with `-D warnings` |
| `just deny` | `cargo deny check` (advisories, licences, bans, sources) |

Run `just --list --list-submodules` for everything. `just deny` needs `cargo install cargo-deny`.

Layout: Cargo workspace. `crates/polyoxide-core` holds shared transport/config/errors;
`crates/polyoxide` is the user-facing facade. Logging is via `tracing` (never install a
subscriber in library code). Toolchain is pinned in `rust-toolchain.toml`.

## Workflow for agents

1. Find the endpoint in `docs/INDEX.md`, read its page, then confirm types in `docs/specs/`.
2. Ask the user about anything ambiguous or any architectural/dependency choice.
3. Implement types, request, and error handling; add rustdoc with the doc link.
4. Add tests from documented examples; run the commands above.
5. Summarize what changed, which doc pages it is based on, and any open questions.
