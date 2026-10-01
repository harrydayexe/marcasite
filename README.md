# polyoxide

An unofficial, idiomatic and fully typed Rust SDK for the Polymarket Predictions APIs.
Not affiliated with or endorsed by Polymarket.

> **Status:** pre-release. The **public (unauthenticated)** endpoints are covered; endpoints that
> need API keys or signing (placing/cancelling orders, user channels, relayer submission, …) are not
> implemented yet. See [`ENDPOINTS.md`](ENDPOINTS.md) for the per-endpoint checklist.

## Services

| Service | Client | Cargo feature |
|---|---|---|
| Gamma API: events, markets, tags, series, comments, sports, search, profiles | `polyoxide::gamma::GammaClient` | `gamma` |
| CLOB API: order books, prices, spreads, markets, price history, rewards, rebates | `polyoxide::clob::ClobClient` | `clob` |
| Data API v2: positions, PnL, trades, activity, leaderboards, holders | `polyoxide::data::DataClient` | `data` |
| Relayer API: transaction status, nonces, wallet deployment | `polyoxide::relayer::RelayerClient` | `relayer` |
| Bridge API: supported assets, quotes, deposit/withdrawal addresses, status | `polyoxide::bridge::BridgeClient` | `bridge` |
| Combos / RFQ: combo-eligible markets | `polyoxide::combos::CombosClient` | `combos` |
| WebSockets: market channel, sports results, PolyBolt public prices | `polyoxide::ws` | `ws` |

All features are on by default. To compile only what you use:

```toml
[dependencies]
polyoxide = { version = "0.1", default-features = false, features = ["gamma", "clob"] }
```

## Quick start

The examples use [`tokio`](https://docs.rs/tokio) (with the `macros` and `rt-multi-thread`
features) and [`futures-util`](https://docs.rs/futures-util) for stream combinators:

```toml
[dependencies]
polyoxide = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

```rust,no_run
use futures_util::TryStreamExt as _;
use polyoxide::Polymarket;

#[tokio::main]
async fn main() -> polyoxide::Result<()> {
    // One handle for every service, sharing a connection pool.
    let pm = Polymarket::new()?;

    // Single page.
    let tags = pm.gamma().list_tags().limit(10).send().await?;
    println!("{} tags", tags.len());

    // Every page, lazily, as a Stream.
    let all_tags: Vec<_> = pm.gamma().list_tags().limit(100).into_stream().try_collect().await?;
    println!("{} tags in total", all_tags.len());
    Ok(())
}
```

## Design

- **Typed everything.** Prices, sizes and amounts are `rust_decimal::Decimal` (never `f64`),
  timestamps are `chrono::DateTime<Utc>`, identifiers are newtypes (`TokenId`, `ConditionId`,
  `Address`, …) and documented string enums are Rust enums with an `Unknown(String)` fallback, so
  new server-side values never break deserialization.
- **Builders for optional parameters.** Required parameters are method arguments; optional ones are
  setters on a request builder finished with `.send().await`.
- **Pagination as streams.** Cursor- and offset-paginated endpoints expose `.send()` for one page
  and `.into_stream()` for every item.
- **Precise errors.** One `polyoxide::Error` enum distinguishes API errors (status, parsed error body,
  trace id), rate limiting (`Retry-After`), timeouts, transport failures, decode failures (with the
  JSON path of the offending field), client-side validation (e.g. batch limits) and configuration
  errors. No panics on network input.
- **Configurable.** Timeouts, user agent, opt-in automatic retries (`RetryPolicy`) and base URLs
  (e.g. for a mock server) via `Polymarket::builder()` or each client's builder.
- **Observable.** Diagnostics via `tracing` (requests at `DEBUG`, bodies at `TRACE`); no subscriber
  is installed by the library.

## Development

The API reference in [`docs/`](docs/) is a verbatim copy of the official documentation and is the
source of truth for every type. Contributor and agent guidance lives in [`AGENTS.md`](AGENTS.md).

```sh
just check   # fmt, clippy, tests, docs, cargo-deny — everything CI runs
```

## License

MIT
