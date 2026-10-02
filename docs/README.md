# docs

| Folder | What | Edit? |
|---|---|---|
| [`polymarket/`](polymarket/) | Verbatim copy of the official Polymarket Predictions API reference (pages and OpenAPI/AsyncAPI specs). The starting point for every statement the SDK makes about the API. Start with [`polymarket/AGENTS.md`](polymarket/AGENTS.md). | Never. Re-fetch to update. |
| [`sdk/`](sdk/) | This SDK's own public API as markdown (modules, types, methods, signatures and rustdoc), generated from rustdoc for LLMs and coding agents. Start with [`sdk/README.md`](sdk/README.md). | Never. Regenerate with `just docs-md`; CI fails if it is stale. |
