> ## Documentation Index
> Fetch the complete documentation index at: https://docs.polymarket.com/llms.txt
> Use this file to discover all available pages before exploring further.

# PolyBolt WebSocket

> Stream crypto, equity and TWAP reference prices over one WebSocket connection.

PolyBolt streams crypto, equity and TWAP reference prices over a single
connection. Subscriptions require CLOB API credentials.

For existing RTDS integrations, see [Migrate from RTDS to PolyBolt](/migrate/rtds-to-polybolt).

## Connect

Open a WebSocket connection to:

```text theme={null}
wss://ws-live-v2.polymarket.com/ws
```

Choose a channel, authenticate, then send a subscription request.

## Channels

| Channel | Data | Filter | Access |
| - | - | - | - |
| `price.crypto` | Crypto reference prices | `{"symbol": "btcusd"}` | CLOB API credentials |
| `price.equity` | Stock, ETF, forex, metal and commodity reference prices | `{"symbol": "aapl"}` | CLOB API credentials |
| `price.crypto.twap` | 60-second time-weighted average prices (TWAP) from Chainlink | `{"symbol": "btcusd", "window_seconds": 60}` | CLOB API credentials |
| `price.equity.twap` | 60-second forex TWAP prices from Chainlink (`eurusd` and `usdjpy`) | `{"symbol": "eurusd", "window_seconds": 60}` | CLOB API credentials |

Use lowercase symbols.

## Authenticate for Gated Channels

Send your
[CLOB API credentials](/getting-started/api#authentication) on the open connection
before subscribing:

```json theme={null}
{
  "op": "auth",
  "rid": "a1",
  "auth": {
    "apiKey": "<api key>",
    "secret": "<api secret>",
    "passphrase": "<api passphrase>"
  }
}
```

Wait for the server to confirm authentication:

```json theme={null}
{
  "op": "authed",
  "rid": "a1"
}
```

The server reports authentication problems with an error acknowledgement, for example:

```json theme={null}
{
  "op": "error",
  "code": "auth_required",
  "channel": "price.crypto",
  "rid": "s1"
}
```

These errors leave the connection open:

| Code | Meaning | Action |
| - | - | - |
| `auth_required` | The subscription was sent before authentication. | Authenticate, then resend the subscription. |
| `auth_invalid` | The credentials were rejected. | Correct the credentials before retrying. |
| `auth_unavailable` | Authentication is temporarily unavailable. | Wait, then retry authentication. |

## Subscribe

Send the channels and filters you want to follow. This request subscribes to the
BTC reference price on the authenticated connection:

```json theme={null}
{
  "op": "subscribe",
  "rid": "s1",
  "subscriptions": [
    { "channel": "price.crypto", "filter": { "symbol": "btcusd" } }
  ]
}
```

A batch counts as one request against the rate limit.

### Pin a Price Provider

Equity prices can come from more than one vendor. Every payload names its
producer in `source`, and on `price.equity` and `price.equity.twap` you can
optionally pin one by adding `provider` to the filter:

```json theme={null}
{
  "op": "subscribe",
  "rid": "s2",
  "subscriptions": [
    { "channel": "price.equity", "filter": { "symbol": "xauusd", "provider": "pyth" } }
  ]
}
```

`provider` accepts `chainlink` or `pyth`. The `subscribed` acknowledgement
names the provider actually served. A pin the server cannot serve for that
symbol falls back to the symbol's default provider, and the acknowledgement
names the fallback, so always read the acknowledgement rather than assuming
the pin held. Omit `provider` to follow the default, which is Chainlink
wherever Chainlink serves the symbol. The field is ignored on channels with a
single provider.

Values other than `chainlink` or `pyth` are rejected with a `bad_filter`
error.

## Envelope

Data envelopes carry snapshots and live updates. Acknowledgements report the
result of a request.

<CodeGroup>
  ```json Live Update theme={null}
  {
    "v": 1,
    "channel": "price.crypto",
    "seq": 2,
    "ts": 1788973001000,
    "payload": {
      "symbol": "btcusd",
      "value": 64126.0,
      "full_accuracy_value": "64126.00000000",
      "timestamp": 1788973001000,
      "source": "pyth"
    }
  }
  ```

  ```json Snapshot theme={null}
  {
    "v": 1,
    "channel": "price.crypto",
    "seq": 1,
    "ts": 1788973000000,
    "snapshot": true,
    "payload": {
      "symbol": "btcusd",
      "source": "pyth",
      "data": [
        {
          "timestamp": 1788972880000,
          "value": 64123.5,
          "full_accuracy_value": "64123.50000000"
        },
        {
          "timestamp": 1788972881000,
          "value": 64125.1,
          "full_accuracy_value": "64125.10000000"
        }
      ]
    }
  }
  ```

  ```json Acknowledgement theme={null}
  {
    "op": "subscribed",
    "channel": "price.crypto",
    "rid": "s1"
  }
  ```
</CodeGroup>

Acknowledgements echo `rid` when the request includes it.

| `op` | Meaning |
| - | - |
| `authed` | Authentication succeeded. You can subscribe to gated channels. |
| `subscribed` | The subscription was accepted. `channel` identifies the channel. |
| `unsubscribed` | The unsubscribe request was accepted. `channel` identifies the channel. |
| `pong` | The server received your application-level `ping` request. |
| `error` | The request failed. `code` identifies the reason. Policy violations can also close the connection. |

Each new subscription receives an acknowledgement and one snapshot. Seed local state from the snapshot, then apply the latest price. Snapshots contain the preceding two minutes of prices, with an empty `data` array when no recent data is available. Repeating an existing channel and filter returns another acknowledgement without a new snapshot.

For data envelopes:

* `seq` is consecutive per connection and per channel, including snapshots. It resets on reconnect.
* `ts` is the producer's event time in Unix milliseconds.
* `snapshot: true` identifies the initial snapshot.
* `dropped`, when present, counts frames dropped on that channel since the last delivered frame because the client fell behind.
* `source` names the vendor that produced the price (`pyth`, `massive` or `chainlink`): on every point of a live update and once per snapshot batch. On single-provider channels it is constant; on the equity channels a symbol's vendor can change between connections, so re-read it after every reconnect.

Compare sequence numbers only within the same connection and channel. See the
[Live Data Channel](/api-reference/wss/polybolt) for every message schema and
`https://ws-live-v2.polymarket.com/asyncapi.json` for the machine-readable contract.

## Unsubscribe

Send the same channel and filter with `op: "unsubscribe"` to stop a subscription:

```json theme={null}
{
  "op": "unsubscribe",
  "subscriptions": [
    { "channel": "price.crypto", "filter": { "symbol": "btcusd" } }
  ]
}
```

The server confirms with `unsubscribed`. Both `subscribe` and `unsubscribe`
accept a single `channel` and `filter` or a `subscriptions` array.

## Supported Symbols

### Crypto Prices

| Channel | Symbols |
| - | - |
| `price.crypto` | `btcusd`, `ethusd`, `solusd`, `dogeusd`, `xrpusd`, `bnbusd` |
| `price.crypto.twap` | `btcusd`, `ethusd`, `solusd`, `xrpusd`, `dogeusd`, `hypeusd`, `bnbusd`, `zecusd` |
| `price.equity.twap` | `eurusd`, `usdjpy` |

For the TWAP channels, use `window_seconds: 60`. Other windows do not provide data.

### Equity Prices

`price.equity` supports these lowercase symbols:

| Asset class | Symbols |
| - | - |
| Stocks | `aapl`, `tsla`, `msft`, `googl`, `amzn`, `meta`, `nvda`, `nflx`, `pltr`, `open`, `rklb`, `abnb`, `coin`, `hood`, `spcx`, `intc`, `mstr`, `crcl`, `mu` |
| ETFs | `qqq`, `spy`, `ewy`, `vxx` |
| Forex | `eurusd`, `gbpusd`, `usdcad`, `usdjpy`, `usdkrw` |
| Precious metals | `xauusd`, `xagusd` |

Commodities serve under `cll` (WTI crude oil front-month futures), `hn`
(Henry Hub natural gas front-month futures) and `cc3m` (the copper 3-month
contract price in USD). `cll` and `hn` carry the front contract's price
under the same symbol as the contract rolls; `cc3m` is a single continuous
contract.

## Limits and Liveness

| Limit | Value | On breach |
| - | - | - |
| Active subscriptions per connection | 64 `(channel, filter)` pairs | `sub_limit` error, then close `4008` |
| Subscribe or unsubscribe frames | 20 per second (a batch counts once) | `rate_limited` error, then close `4008` |
| Frame size | 64 KB | close `4008` |
| Auth frames per connection | 8 | `auth_attempts` error, then close `4008` |

The server sends a WebSocket ping every 25 seconds. Standard clients answer
automatically. Two missed pongs close the connection with `4002`.

You can also send `{ "op": "ping" }` to check the connection at the application
level. The server responds with `pong`.

## Close Codes and Reconnecting

Choose the retry behavior based on the close code:

| Code | Meaning | Client behavior |
| - | - | - |
| `4001` | Authentication failed | Fix authentication before reconnecting. |
| `4002` | Slow consumer or pong timeout | Retry with exponential backoff and jitter. Reduce subscription load if this repeats. |
| `4003` | Server draining | Reconnect after a uniformly random delay of 0 to 10 seconds. |
| `4008` | Policy violation | Fix the request or limit violation before reconnecting. |
| `1006` | Abnormal disconnect | Retry with exponential backoff and full jitter, starting at 1 second and capped at 30 seconds. |

If the connection is refused with HTTP `429` or `503`, use `Retry-After` as the minimum
wait and add jitter. Browser clients see handshake failures as `1006` and should
use exponential backoff with full jitter.

After reconnecting, authenticate again for gated channels and resend your
subscriptions, including any `provider` pins. Initialize from the new
snapshots and re-read `source`: an equity symbol can legitimately arrive from
a different vendor than before the reconnect. Sequence numbers start over.
