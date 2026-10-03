# Polymarket Predictions API Reference (local copy)

Verbatim copy of the official Polymarket Predictions API reference, fetched 2026-10-01 from
`https://docs.polymarket.com/<path>.md`. It is the starting point for every statement about the API;
where the live API behaves differently, the SDK follows live (see `SPEC_DEVIATIONS.md` and the root
`AGENTS.md`). Do not edit the copied pages; re-fetch instead.

## How to navigate (read in this order, stop when you have what you need)

1. `INDEX.md` - one line per page (title, path, description), grouped by section. Start here to find an endpoint.
2. `api-reference/<group>/<slug>.md` - one page per endpoint/channel. Usually the cheapest way to answer
   "what does endpoint X take and return?".
3. `specs/` - machine-readable OpenAPI/AsyncAPI specs, one per service. Use for complete schemas
   (`components/schemas`), enums and exact field types when building Rust types.

Endpoint pages embed the OpenAPI spec for their service, so they can be large (10-30 KB).
Prefer `grep -n` on a spec/page (e.g. `grep -n "operationId: postOrder"`) or read only the
relevant section rather than loading whole files.

## Layout

| Path | Service / content | Spec |
|---|---|---|
| `api-reference/predictions/overview.md` | Overview and base URLs of all services | - |
| `api-reference/rate-limits.md`, `trading-rate-limits.md`, `geoblock.md` | Cross-cutting: rate limits, geographic restrictions | - |
| `api-reference/events/`, `markets/`, `tags/`, `series/`, `comments/`, `sports/`, `search/`, `profiles/` | Gamma API (`gamma-api.polymarket.com`) | `specs/gamma-openapi.yaml` |
| `api-reference/market-data/`, `data/`, `trade/`, `rewards/`, `rebates/` | CLOB API (`clob.polymarket.com`): order book, prices, orders, trades, rewards | `specs/clob-openapi.yaml` |
| `api-reference/data-api/`, `wallet/`, `feeds/`, `boards/`, `service/` | Data API **v2** (`data-api.polymarket.com/v2`) | `specs/data-v2-openapi.json` |
| `api-reference/relayer/`, `relayer-api-keys/` | Relayer API (`relayer-v2.polymarket.com`) | `specs/relayer-openapi.yaml` |
| `api-reference/combo-markets/`, `maker/` | Combos / RFQ REST | `specs/combos-rfq-openapi.yaml` |
| `api-reference/bridge/` | Bridge API (`bridge.polymarket.com`) | `specs/bridge-openapi.yaml` |
| `api-reference/wss/` | WebSocket channels: `market`, `user`, `sports`, `rfq` (Quoter Gateway), `polybolt` | `specs/asyncapi*.json`, `specs/polybolt-asyncapi.json` |
| `api-reference/live-data/`, `websockets/` | PolyBolt (live data) WebSocket pages | `specs/polybolt-asyncapi.json` |

Some endpoints appear in more than one group (e.g. open interest under `markets/`); the Data API v2
pages live in `wallet/`, `feeds/` and `boards/`, not only `data-api/`.

## Specs

- `specs/gamma-openapi.yaml`, `clob-openapi.yaml`, `relayer-openapi.yaml`, `combos-rfq-openapi.yaml`, `bridge-openapi.yaml`
- `specs/data-v2-openapi.json` (from `data-api.polymarket.com/v2/openapi.json`)
- `specs/asyncapi.json` (market channel), `asyncapi-user.json`, `asyncapi-sports.json`, `asyncapi-rfq.json`
- `specs/polybolt-asyncapi.json` (PolyBolt live data, from `ws-live-v2.polymarket.com/asyncapi.json`)

## Scope and known gaps

- Predictions only. Perps docs, Chinese docs and general guides (concepts, trading, programs) are not included.
- Data API **v1 (legacy)** pages are intentionally excluded; v1 is being replaced by v2.
- Pages are not versioned; check the fetch date above before trusting them for recent API changes.
