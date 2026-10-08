//! Parses the endpoint tables of `ENDPOINTS.md` for the MCP index.
//!
//! The tables have one of three shapes, chosen by their header row:
//!
//! - `| Status | Method | Path | Docs | Rust |` (REST endpoints),
//! - `| Status | Channel | URL | Docs | Rust |` (WebSocket channels),
//! - `| Service | Method | Path | Docs |` (out of scope, not implemented).
//!
//! A `## <service>` heading starts each section; the `module `marcasite::x`` mention in its
//! intro gives the module the Rust methods live in.

use anyhow::bail;
use serde::{Deserialize, Serialize};

/// Implementation status of an endpoint row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Status {
    /// `[x]`: implemented and tested.
    Implemented,
    /// `[ ]`: in scope, not implemented yet.
    NotImplemented,
    /// Listed under "Out of scope" (needs authentication).
    OutOfScope,
}

/// One endpoint row, before its Rust methods are resolved to index items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) service: String,
    pub(crate) module: Option<String>,
    pub(crate) status: Status,
    /// HTTP method; `None` for WebSocket channels.
    pub(crate) method: Option<String>,
    /// HTTP path, or the URL of a WebSocket channel.
    pub(crate) path: String,
    /// WebSocket channel name.
    pub(crate) channel: Option<String>,
    /// Online documentation links.
    pub(crate) doc_urls: Vec<String>,
    /// Links into the local copy (`docs/polymarket/...`).
    pub(crate) local_docs: Vec<String>,
    /// The spec `operationId` of endpoints documented only in the spec.
    pub(crate) operation_id: Option<String>,
    /// The `Rust` cell as written (empty for out-of-scope rows).
    pub(crate) rust: String,
    /// `Type::method` mentions in the `Rust` cell, arguments stripped.
    pub(crate) rust_paths: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Rest,
    Channel,
    OutOfScope,
}

/// Parses every endpoint table in `md`.
pub(crate) fn parse(md: &str) -> anyhow::Result<Vec<Row>> {
    let mut rows = Vec::new();
    let mut service = String::new();
    let mut module = None;
    let mut shape = None;
    for (n, line) in md.lines().enumerate() {
        let line = line.trim();
        if let Some(heading) = line.strip_prefix("## ") {
            service = heading.trim().to_owned();
            module = None;
            shape = None;
            continue;
        }
        if module.is_none()
            && let Some(m) = module_mention(line)
        {
            module = Some(m);
        }
        if !line.starts_with('|') {
            shape = None;
            continue;
        }
        let cells = cells(line);
        let header: Vec<&str> = cells.iter().map(String::as_str).collect();
        match header.as_slice() {
            ["Status", "Method", "Path", "Docs", "Rust"] => {
                shape = Some(Shape::Rest);
                continue;
            }
            ["Status", "Channel", "URL", "Docs", "Rust"] => {
                shape = Some(Shape::Channel);
                continue;
            }
            ["Service", "Method", "Path", "Docs"] => {
                shape = Some(Shape::OutOfScope);
                continue;
            }
            _ => {}
        }
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        let Some(shape) = shape else {
            bail!("ENDPOINTS.md:{}: table row without a known header", n + 1);
        };
        let row = match (shape, cells.as_slice()) {
            (Shape::Rest, [status, method, path, docs, rust]) => Row {
                service: service.clone(),
                module: module.clone(),
                status: parse_status(status, n)?,
                method: Some(unquote(method)),
                path: unquote(path),
                channel: None,
                doc_urls: links(docs, |d| d.starts_with("http")),
                local_docs: links(docs, |d| !d.starts_with("http")),
                operation_id: operation_id(docs),
                rust: rust.clone(),
                rust_paths: rust_paths(rust),
            },
            (Shape::Channel, [status, channel, url, docs, rust]) => Row {
                service: service.clone(),
                module: module.clone(),
                status: parse_status(status, n)?,
                method: None,
                path: unquote(url),
                channel: Some(channel.clone()),
                doc_urls: links(docs, |d| d.starts_with("http")),
                local_docs: links(docs, |d| !d.starts_with("http")),
                operation_id: None,
                rust: rust.clone(),
                rust_paths: rust_paths(rust),
            },
            (Shape::OutOfScope, [svc, method, path, docs]) => Row {
                service: svc.clone(),
                module: None,
                status: Status::OutOfScope,
                method: Some(unquote(method)),
                path: unquote(path),
                channel: None,
                doc_urls: links(docs, |d| d.starts_with("http")),
                local_docs: links(docs, |d| !d.starts_with("http")),
                operation_id: operation_id(docs),
                rust: String::new(),
                rust_paths: Vec::new(),
            },
            _ => bail!(
                "ENDPOINTS.md:{}: expected {} cells, found {}",
                n + 1,
                if shape == Shape::OutOfScope { 4 } else { 5 },
                cells.len()
            ),
        };
        rows.push(row);
    }
    Ok(rows)
}

fn parse_status(cell: &str, line: usize) -> anyhow::Result<Status> {
    match cell {
        "[x]" => Ok(Status::Implemented),
        "[ ]" => Ok(Status::NotImplemented),
        other => bail!(
            "ENDPOINTS.md:{}: status must be `[x]` or `[ ]`, found {other:?}",
            line + 1
        ),
    }
}

/// `marcasite::gamma` from "... module `marcasite::gamma`.".
fn module_mention(line: &str) -> Option<String> {
    let start = line.find("module `marcasite")? + "module `".len();
    let rest = line.get(start..)?;
    let end = rest.find('`')?;
    rest.get(..end).map(str::to_owned)
}

/// Splits a table row into trimmed cells, ignoring `|` inside code spans.
fn cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut in_code = false;
    for c in inner.chars() {
        match c {
            '`' => {
                in_code = !in_code;
                cell.push(c);
            }
            '|' if !in_code => cells.push(std::mem::take(&mut cell).trim().to_owned()),
            _ => cell.push(c),
        }
    }
    cells.push(cell.trim().to_owned());
    cells
}

/// Strips the backticks around a cell like `` `GET` ``.
fn unquote(cell: &str) -> String {
    cell.trim().trim_matches('`').to_owned()
}

/// The destinations of the `[text](dest)` links in `cell` accepted by `keep`.
fn links(cell: &str, keep: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = cell;
    while let Some(i) = rest.find("](") {
        let Some(after) = rest.get(i + 2..) else {
            break;
        };
        let Some(end) = after.find(')') else {
            break;
        };
        let dest = after.get(..end).unwrap_or_default();
        if keep(dest) {
            out.push(dest.to_owned());
        }
        rest = after.get(end..).unwrap_or_default();
    }
    out
}

/// The `operationId` of a "spec only (`getTeam`)" docs cell.
fn operation_id(cell: &str) -> Option<String> {
    let rest = cell.get(cell.find("spec only")?..)?;
    code_spans(rest).into_iter().next()
}

/// The contents of the code spans in `s`.
fn code_spans(s: &str) -> Vec<String> {
    s.split('`').skip(1).step_by(2).map(str::to_owned).collect()
}

/// `Type::method` mentions (code spans starting with an upper-case type name), without the
/// argument list: `` `MarketChannel::connect(MarketSubscription)` `` gives
/// `MarketChannel::connect`.
fn rust_paths(cell: &str) -> Vec<String> {
    code_spans(cell)
        .into_iter()
        .filter_map(|span| {
            let path = span.split('(').next().unwrap_or_default().trim();
            let (ty, method) = path.split_once("::")?;
            let is_ident =
                |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            (ty.starts_with(|c: char| c.is_ascii_uppercase())
                && is_ident(ty)
                && is_ident(method)
                && method.starts_with(|c: char| c.is_ascii_lowercase()))
            .then(|| path.to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "\
# Endpoint checklist

## Gamma API

Base URL `https://gamma-api.polymarket.com`, feature `gamma`, module `marcasite::gamma`.

| Status | Method | Path | Docs | Rust |
|---|---|---|---|---|
| [x] | `GET` | `/teams/{id}` | spec only (`getTeam`) | `GammaClient::get_team` |
| [x] | `GET` | `/tags/{id}` | [local](docs/polymarket/api-reference/tags/get-tag-by-id.md) / [online](https://docs.polymarket.com/api-reference/tags/get-tag-by-id) | `GammaClient::get_tag` |
| [ ] | `GET` | `/books` | spec only (`getBooksGet`) | none: not implemented; use the `POST` form |

## WebSocket channels (public)

Feature `ws`, module `marcasite::ws`.

| Status | Channel | URL | Docs | Rust |
|---|---|---|---|---|
| [x] | Market channel | `wss://x/ws/market` | [online](https://docs.polymarket.com/api-reference/wss/market) | `MarketChannel::connect(MarketSubscription)` → `Stream<Item = Result<MarketEvent>>`; `subscribe` |

## Out of scope (requires authentication)

| Service | Method | Path | Docs |
|---|---|---|---|
| CLOB API | `POST` | `/order` | [online](https://docs.polymarket.com/api-reference/trade/post-a-new-order) |
";

    #[test]
    fn parses_every_table_shape() {
        let rows = parse(MD).unwrap();
        assert_eq!(rows.len(), 5);

        let team = &rows[0];
        assert_eq!(team.service, "Gamma API");
        assert_eq!(team.module.as_deref(), Some("marcasite::gamma"));
        assert_eq!(team.status, Status::Implemented);
        assert_eq!(team.method.as_deref(), Some("GET"));
        assert_eq!(team.path, "/teams/{id}");
        assert_eq!(team.operation_id.as_deref(), Some("getTeam"));
        assert_eq!(team.rust_paths, ["GammaClient::get_team"]);

        let tag = &rows[1];
        assert_eq!(
            tag.doc_urls,
            ["https://docs.polymarket.com/api-reference/tags/get-tag-by-id"]
        );
        assert_eq!(
            tag.local_docs,
            ["docs/polymarket/api-reference/tags/get-tag-by-id.md"]
        );
        assert_eq!(tag.operation_id, None);

        let books = &rows[2];
        assert_eq!(books.status, Status::NotImplemented);
        assert!(books.rust_paths.is_empty());

        let market = &rows[3];
        assert_eq!(market.module.as_deref(), Some("marcasite::ws"));
        assert_eq!(market.method, None);
        assert_eq!(market.channel.as_deref(), Some("Market channel"));
        assert_eq!(market.path, "wss://x/ws/market");
        assert_eq!(market.rust_paths, ["MarketChannel::connect"]);

        let order = &rows[4];
        assert_eq!(order.status, Status::OutOfScope);
        assert_eq!(order.service, "CLOB API");
        assert_eq!(order.module, None);
    }

    #[test]
    fn rejects_unknown_status() {
        let md = "| Status | Method | Path | Docs | Rust |\n|---|---|---|---|---|\n| x | a | b | c | d |";
        assert!(parse(md).is_err());
    }
}
