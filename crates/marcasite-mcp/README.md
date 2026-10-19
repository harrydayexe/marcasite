# marcasite-mcp

An [MCP](https://modelcontextprotocol.io/) server that serves the documentation of
[`marcasite`](https://crates.io/crates/marcasite), an unofficial Rust SDK for the Polymarket
Predictions APIs, to coding agents (Claude Code, Cursor, and any other MCP client). Not affiliated
with or endorsed by Polymarket.

It is a **docs server only**: it speaks MCP over stdio, never calls Polymarket and makes no network
requests. The docs are embedded in the binary, and `marcasite-mcp` is released in lockstep with
`marcasite`, so version `X.Y.Z` serves the docs of `marcasite` `X.Y.Z`.

## Install

```sh
cargo install marcasite-mcp
# or match the marcasite version your project uses:
cargo install marcasite-mcp@<version>
```

Then register it with your MCP client, for example Claude Code:

```sh
claude mcp add marcasite -- marcasite-mcp
```

Other clients take the same command in their MCP configuration, e.g.:

```json
{
  "mcpServers": {
    "marcasite": { "command": "marcasite-mcp" }
  }
}
```

## Tools

| Tool | Input | Returns |
|---|---|---|
| `search` | `query`, optional `module`, `kind`, `limit` | Ranked `path (kind): summary` lines |
| `get_item` | `path`, full or short (`GammaClient::list_markets`, `Event`) | The item's signature or definition, docs, methods, cited Polymarket pages and related items; candidates if the path is ambiguous |
| `list_module` | `module` (`gamma`, `clob`, `marcasite`, ...) | The module's docs and an index of its items |
| `find_endpoint` | `query` (doc URL, HTTP path or operation name), optional `method`, `service` | The SDK method(s) implementing a Polymarket endpoint, or that it is not implemented yet |

## Resources and prompts

- `marcasite://sdk/{module}`: a module's full documentation.
- `marcasite://sdk/endpoints`: the endpoint coverage table.
- Prompt `use-marcasite`: a primer on the SDK's idioms.

## License

MIT OR Apache-2.0
