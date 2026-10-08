//! Lookups over the index and their markdown answers: path resolution, search, module views and
//! endpoint coverage.

use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Write as _};

use crate::index::{Endpoint, Index, Item, Module, Status};

/// Item kind sections in display order: (kind, heading).
const SECTIONS: &[(&str, &str)] = &[
    ("struct", "Structs"),
    ("enum", "Enums"),
    ("union", "Unions"),
    ("trait", "Traits"),
    ("type", "Type aliases"),
    ("constant", "Constants"),
    ("static", "Statics"),
    ("fn", "Functions"),
    ("macro", "Macros"),
];

/// HTTP methods recognised at the start of a `find_endpoint` query.
const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE"];

/// The index with lookup tables.
pub(crate) struct Docs {
    index: Index,
    /// Canonical item path → item.
    by_path: HashMap<String, usize>,
    /// Every public path of an item (canonical and re-exported, members included).
    aliases: Vec<(String, usize)>,
    /// Lower-cased (name, path, summary, docs) per item, for search.
    haystacks: Vec<[String; 4]>,
}

impl fmt::Debug for Docs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Docs")
            .field("marcasite_version", &self.index.marcasite_version)
            .field("modules", &self.index.modules.len())
            .field("items", &self.index.items.len())
            .field("endpoints", &self.index.endpoints.len())
            .finish()
    }
}

/// What a path resolves to.
#[derive(Debug)]
pub(crate) enum Lookup<'a> {
    Item(&'a Item),
    Module(&'a Module),
    /// Several items match a short path.
    Ambiguous(Vec<&'a Item>),
    /// A field or variant, documented inside its parent's definition.
    Member {
        parent: &'a Item,
        name: String,
    },
    NotFound,
}

/// One search result.
#[derive(Debug, PartialEq)]
pub(crate) struct Hit<'a> {
    pub(crate) path: &'a str,
    pub(crate) kind: &'a str,
    pub(crate) summary: &'a str,
    score: u32,
}

/// How `find_endpoint` interpreted its query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryKind {
    Url,
    Path,
    Name,
}

impl Docs {
    pub(crate) fn new(index: Index) -> Self {
        let by_path: HashMap<String, usize> = index
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| (item.path.clone(), i))
            .collect();
        let mut aliases: Vec<(String, usize)> =
            by_path.iter().map(|(p, i)| (p.clone(), *i)).collect();
        for item in &index.items {
            let Some(&i) = by_path.get(&item.path) else {
                continue;
            };
            for alias in &item.reexported_as {
                aliases.push((alias.clone(), i));
                for member in &item.members {
                    if let Some(&m) = by_path.get(member)
                        && let Some(mi) = index.items.get(m)
                    {
                        aliases.push((format!("{alias}::{}", mi.name), m));
                    }
                }
            }
        }
        aliases.sort();
        aliases.dedup();
        let haystacks = index
            .items
            .iter()
            .map(|i| {
                [
                    i.name.to_lowercase(),
                    i.path.to_lowercase(),
                    i.summary.to_lowercase(),
                    i.docs.to_lowercase(),
                ]
            })
            .collect();
        Self {
            index,
            by_path,
            aliases,
            haystacks,
        }
    }

    pub(crate) fn version(&self) -> &str {
        &self.index.marcasite_version
    }

    pub(crate) fn modules(&self) -> &[Module] {
        &self.index.modules
    }

    pub(crate) fn item_count(&self) -> usize {
        self.index.items.len()
    }

    fn item(&self, path: &str) -> Option<&Item> {
        self.index.items.get(*self.by_path.get(path)?)
    }

    /// Finds a module by `gamma`, `marcasite::gamma` or `marcasite`.
    pub(crate) fn module(&self, name: &str) -> Option<&Module> {
        let full = full_path(&normalize(name));
        self.index.modules.iter().find(|m| m.path == full)
    }

    /// The resource name of a module: `marcasite` for the root, else its path below it
    /// (`gamma`).
    pub(crate) fn module_id(module: &Module) -> &str {
        module
            .path
            .strip_prefix("marcasite::")
            .unwrap_or(&module.path)
    }

    /// Resolves a full or short path (`marcasite::gamma::GammaClient::get_tag`,
    /// `gamma::GammaClient`, `GammaClient::get_tag`, `Event`, a module name).
    pub(crate) fn lookup(&self, input: &str) -> Lookup<'_> {
        let q = normalize(input);
        if q.is_empty() {
            return Lookup::NotFound;
        }
        if let Some(m) = self.module(&q) {
            return Lookup::Module(m);
        }
        match self.lookup_item(&q) {
            Lookup::NotFound => {}
            found => return found,
        }
        // A field or variant: resolve the parent and look for the name in its definition.
        if let Some((parent, name)) = q.rsplit_once("::")
            && let Lookup::Item(p) = self.lookup_item(parent)
            && matches!(p.kind.as_str(), "struct" | "enum" | "union")
            && mentions(&p.signature, name)
        {
            return Lookup::Member {
                parent: p,
                name: name.to_owned(),
            };
        }
        Lookup::NotFound
    }

    fn lookup_item(&self, q: &str) -> Lookup<'_> {
        let full = full_path(q);
        if let Some(i) = self.alias(&full) {
            return Lookup::Item(i);
        }
        let suffix = format!("::{q}");
        for fold in [false, true] {
            let mut found: BTreeSet<usize> = BTreeSet::new();
            for (path, i) in &self.aliases {
                let hit = if fold {
                    path.to_lowercase().ends_with(&suffix.to_lowercase())
                } else {
                    path.ends_with(&suffix)
                };
                if hit {
                    found.insert(*i);
                }
            }
            let items: Vec<&Item> = found
                .into_iter()
                .filter_map(|i| self.index.items.get(i))
                .collect();
            match items.as_slice() {
                [] => {}
                [one] => return Lookup::Item(one),
                _ => return Lookup::Ambiguous(items),
            }
        }
        Lookup::NotFound
    }

    fn alias(&self, path: &str) -> Option<&Item> {
        let at = self
            .aliases
            .binary_search_by(|(p, _)| p.as_str().cmp(path))
            .ok()?;
        let (_, i) = self.aliases.get(at)?;
        self.index.items.get(*i)
    }

    /// Ranked search over item and module names, paths, summaries and docs.
    pub(crate) fn search(
        &self,
        query: &str,
        module: Option<&str>,
        kind: Option<&str>,
        limit: usize,
    ) -> Vec<Hit<'_>> {
        let q = query.trim().trim_matches('`').to_lowercase();
        let tokens: Vec<&str> = q.split_whitespace().collect();
        if tokens.is_empty() {
            return Vec::new();
        }
        let module = module
            .map(|m| full_path(&normalize(m)))
            .filter(|m| m != "marcasite");
        let in_module = |path: &str| {
            module
                .as_ref()
                .is_none_or(|m| path == m || path.starts_with(&format!("{m}::")))
        };
        let kind = kind.map(|k| match k.trim().to_lowercase().as_str() {
            "function" => "fn".to_owned(),
            "const" => "constant".to_owned(),
            "mod" => "module".to_owned(),
            other => other.to_owned(),
        });
        let kind_ok = |k: &str| kind.as_deref().is_none_or(|want| want == k);

        let mut hits = Vec::new();
        for (item, hay) in self.index.items.iter().zip(&self.haystacks) {
            if !in_module(&item.module) || !kind_ok(&item.kind) {
                continue;
            }
            let mut score = score(&q, &tokens, hay);
            if score == 0 {
                continue;
            }
            if item.parent.is_none() {
                score += 5;
            }
            hits.push(Hit {
                path: &item.path,
                kind: &item.kind,
                summary: &item.summary,
                score,
            });
        }
        for m in &self.index.modules {
            if !in_module(&m.path) || !kind_ok("module") {
                continue;
            }
            let hay = [
                Self::module_id(m).to_lowercase(),
                m.path.to_lowercase(),
                m.summary.to_lowercase(),
                m.docs.to_lowercase(),
            ];
            let score = score(&q, &tokens, &hay);
            if score > 0 {
                hits.push(Hit {
                    path: &m.path,
                    kind: "module",
                    summary: &m.summary,
                    score: score + 5,
                });
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(a.path.len().cmp(&b.path.len()))
                .then(a.path.cmp(b.path))
        });
        hits.truncate(limit);
        hits
    }

    /// The answer of the `search` tool.
    pub(crate) fn render_search(
        &self,
        query: &str,
        module: Option<&str>,
        kind: Option<&str>,
        limit: usize,
    ) -> Result<String, String> {
        let hits = self.search(query, module, kind, limit);
        if hits.is_empty() {
            return Err(format!(
                "No marcasite item matches `{query}`{}{}. Try a shorter or different word, or \
                 drop the filters.",
                module
                    .map(|m| format!(" in module `{m}`"))
                    .unwrap_or_default(),
                kind.map(|k| format!(" of kind `{k}`")).unwrap_or_default(),
            ));
        }
        let mut out = format!("Results for `{query}` (best first):\n\n");
        for h in hits {
            let _ = writeln!(out, "- `{}` ({}): {}", h.path, h.kind, h.summary);
        }
        out.push_str("\nUse `get_item` with a path for its full documentation.");
        Ok(out)
    }

    /// The answer of the `get_item` tool.
    pub(crate) fn render_lookup(&self, path: &str) -> Result<String, String> {
        match self.lookup(path) {
            Lookup::Item(item) => Ok(self.render_item(item)),
            Lookup::Module(m) => Ok(self.render_module_index(m)),
            Lookup::Ambiguous(items) => {
                let mut out = format!(
                    "`{path}` is ambiguous; it matches {} items. Call `get_item` again with one \
                     of these paths:\n\n",
                    items.len()
                );
                for i in items {
                    let _ = writeln!(out, "- `{}` ({}): {}", i.path, i.kind, i.summary);
                }
                Ok(out)
            }
            Lookup::Member { parent, name } => Ok(format!(
                "`{name}` is a field or variant of `{}`, documented in its definition below.\n\n{}",
                parent.path,
                self.render_item(parent)
            )),
            Lookup::NotFound => {
                let mut out = format!("No marcasite item or module is named `{path}`.");
                let last = normalize(path);
                let last = last.rsplit("::").next().unwrap_or_default();
                let hits = self.search(last, None, None, 5);
                if !hits.is_empty() {
                    out.push_str(" Similar items:\n\n");
                    for h in hits {
                        let _ = writeln!(out, "- `{}` ({}): {}", h.path, h.kind, h.summary);
                    }
                }
                Err(out)
            }
        }
    }

    /// An item's full documentation, with its associated items listed by signature.
    pub(crate) fn render_item(&self, item: &Item) -> String {
        let mut out = format!("# {} `{}`\n\n", kind_label(&item.kind), item.path);
        if let Some(parent) = &item.parent {
            let _ = writeln!(out, "Associated item of `{parent}`.\n");
        }
        self.item_body(&mut out, item, 1);
        let members: Vec<&Item> = item.members.iter().filter_map(|m| self.item(m)).collect();
        if !members.is_empty() {
            let heading = if item.kind == "trait" {
                "Trait items"
            } else if members.iter().all(|m| m.kind == "method") {
                "Methods"
            } else {
                "Associated items"
            };
            let _ = writeln!(
                out,
                "## {heading}\n\nCall `get_item` with `{}::<name>` for one in full.\n",
                item.path
            );
            for m in members {
                let _ = writeln!(out, "- `{}`", one_line(&m.signature));
                if !m.summary.is_empty() {
                    let _ = writeln!(out, "  {}", m.summary);
                }
            }
            out.push('\n');
        }
        self.item_links(&mut out, item, 2);
        trim_end(out)
    }

    /// Signature, deprecation, docs and implemented traits, headings starting at `level + 1`.
    fn item_body(&self, out: &mut String, item: &Item, level: usize) {
        let _ = writeln!(out, "```rust\n{}\n```\n", item.signature);
        if let Some(d) = &item.deprecated {
            let _ = writeln!(out, "**{d}**\n");
        }
        if !item.docs.is_empty() {
            let _ = writeln!(out, "{}\n", shift_headings(&item.docs, level));
        }
        if !item.implements.is_empty() {
            let list: Vec<String> = item.implements.iter().map(|t| format!("`{t}`")).collect();
            let _ = writeln!(out, "**Implements:** {}\n", list.join(", "));
        }
    }

    /// Polymarket pages, related items and re-exports, as headings at `level`.
    fn item_links(&self, out: &mut String, item: &Item, level: usize) {
        let h = "#".repeat(level.min(6));
        if !item.doc_urls.is_empty() {
            let _ = writeln!(out, "{h} Polymarket docs\n");
            for u in &item.doc_urls {
                let _ = writeln!(out, "- <{u}>");
            }
            out.push('\n');
        }
        if !item.related.is_empty() {
            let _ = writeln!(out, "{h} Related\n");
            for r in &item.related {
                match self.item(r) {
                    Some(ri) => {
                        let _ = writeln!(out, "- `{r}` ({}): {}", ri.kind, ri.summary);
                    }
                    None => {
                        let _ = writeln!(out, "- `{r}`");
                    }
                }
            }
            out.push('\n');
        }
        if !item.reexported_as.is_empty() {
            let list: Vec<String> = item
                .reexported_as
                .iter()
                .map(|p| format!("`{p}`"))
                .collect();
            let _ = writeln!(out, "Also available as {}.\n", list.join(", "));
        }
    }

    /// A module's docs and item index, without item bodies (the `list_module` tool).
    pub(crate) fn render_module_index(&self, m: &Module) -> String {
        let mut out = self.module_head(m);
        for (kind, heading) in SECTIONS {
            let items: Vec<&Item> = m
                .items
                .iter()
                .filter_map(|p| self.item(p))
                .filter(|i| i.kind == *kind)
                .collect();
            if items.is_empty() {
                continue;
            }
            let _ = writeln!(out, "## {heading}\n");
            for i in items {
                let _ = writeln!(out, "- `{}`: {}", i.name, i.summary);
            }
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "Call `get_item` with `{}::<Name>` for an item's full documentation.",
            m.path
        );
        trim_end(out)
    }

    /// A module's full documentation, every item in full (the `marcasite://sdk/{module}`
    /// resource).
    pub(crate) fn render_module_full(&self, m: &Module) -> String {
        let mut out = self.module_head(m);
        for (kind, heading) in SECTIONS {
            let items: Vec<&Item> = m
                .items
                .iter()
                .filter_map(|p| self.item(p))
                .filter(|i| i.kind == *kind)
                .collect();
            if items.is_empty() {
                continue;
            }
            let _ = writeln!(out, "## {heading}\n");
            for i in items {
                let _ = writeln!(out, "### {} `{}`\n", kind_label(&i.kind), i.name);
                self.item_body(&mut out, i, 3);
                for member in i.members.iter().filter_map(|p| self.item(p)) {
                    let _ = writeln!(out, "#### `{}::{}`\n", i.name, member.name);
                    self.item_body(&mut out, member, 4);
                }
            }
        }
        trim_end(out)
    }

    /// Title, docs, submodules and re-exports of a module.
    fn module_head(&self, m: &Module) -> String {
        let title = if m.path == "marcasite" {
            "Crate"
        } else {
            "Module"
        };
        let mut out = format!(
            "# {title} `{}`\n\nmarcasite {}\n\n",
            m.path, self.index.marcasite_version
        );
        if !m.docs.is_empty() {
            let _ = writeln!(out, "{}\n", shift_headings(&m.docs, 1));
        }
        if !m.submodules.is_empty() {
            let _ = writeln!(out, "## Modules\n");
            for s in &m.submodules {
                let summary = self
                    .index
                    .modules
                    .iter()
                    .find(|x| &x.path == s)
                    .map(|x| x.summary.as_str())
                    .unwrap_or_default();
                let _ = writeln!(out, "- `{s}`: {summary}");
            }
            out.push('\n');
        }
        if !m.reexports.is_empty() {
            let _ = writeln!(out, "## Re-exports\n");
            for r in &m.reexports {
                match (&r.target, &r.external) {
                    (Some(t), _) => {
                        let summary = self.item(t).map(|i| i.summary.as_str()).unwrap_or_default();
                        let _ = writeln!(out, "- `{}`: re-export of `{t}`. {summary}", r.name);
                    }
                    (None, Some(e)) => {
                        let _ = writeln!(
                            out,
                            "- `{}`: `pub use {};` from the `{}` crate. {}",
                            r.name, e.source, e.crate_name, e.summary
                        );
                    }
                    (None, None) => {
                        let _ = writeln!(out, "- `{}`", r.name);
                    }
                }
            }
            out.push('\n');
        }
        out
    }

    /// The endpoint coverage table (the `marcasite://sdk/endpoints` resource).
    pub(crate) fn render_endpoints(&self) -> String {
        let mut out = format!(
            "# marcasite endpoint coverage\n\nmarcasite {}. Every Polymarket Predictions API \
             endpoint and whether marcasite implements it. Use `find_endpoint` for one endpoint \
             with its method signatures.\n",
            self.index.marcasite_version
        );
        let mut service = "";
        for e in &self.index.endpoints {
            if e.service != service {
                service = &e.service;
                let _ = writeln!(
                    out,
                    "\n## {service}\n\n| Status | Endpoint | Rust | Docs |\n|---|---|---|---|"
                );
            }
            let status = match e.status {
                Status::Implemented => "implemented",
                Status::NotImplemented => "not implemented",
                Status::OutOfScope => "not implemented (needs auth)",
            };
            let rust = if e.items.is_empty() {
                e.rust.clone()
            } else {
                e.items
                    .iter()
                    .map(|p| format!("`{p}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let docs = match (e.doc_urls.first(), &e.operation_id) {
                (Some(u), _) => u.clone(),
                (None, Some(op)) => format!("spec only (`{op}`)"),
                (None, None) => String::new(),
            };
            let _ = writeln!(
                out,
                "| {status} | {} | {} | {docs} |",
                endpoint_label(e),
                rust.replace('|', "\\|")
            );
        }
        trim_end(out)
    }

    /// The answer of the `find_endpoint` tool.
    pub(crate) fn render_find_endpoint(
        &self,
        query: &str,
        method: Option<&str>,
        service: Option<&str>,
    ) -> Result<String, String> {
        let (kind, rows, citing) = self.find_endpoint(query, method, service);
        if rows.is_empty() && citing.is_empty() {
            let hint = match kind {
                QueryKind::Url => "No endpoint row or item cites this page.",
                QueryKind::Path => {
                    "No endpoint has this path (path parameters like `{id}` match any segment)."
                }
                QueryKind::Name => {
                    "No endpoint has this operation id, doc page slug or Rust method name."
                }
            };
            return Err(format!(
                "No Polymarket endpoint in marcasite's checklist matches `{query}`{}{}. {hint} \
                 Try a docs.polymarket.com URL, an HTTP path such as `/markets/{{id}}` or an \
                 operation name such as `getTeam`; the `marcasite://sdk/endpoints` resource \
                 lists every endpoint.",
                method
                    .map(|m| format!(" with method `{m}`"))
                    .unwrap_or_default(),
                service
                    .map(|s| format!(" in service `{s}`"))
                    .unwrap_or_default(),
            ));
        }
        let mut out = format!("# Endpoints matching `{query}`\n");
        for e in rows {
            let _ = writeln!(out, "\n## {} ({})\n", endpoint_label(e), e.service);
            match e.status {
                Status::Implemented => {
                    let module = e.module.as_deref().unwrap_or("marcasite");
                    let _ = writeln!(out, "**Status:** implemented in `{module}`.\n");
                }
                Status::NotImplemented => {
                    let _ = writeln!(out, "**Status:** not implemented in marcasite yet.\n");
                    if !e.rust.is_empty() {
                        let _ = writeln!(out, "Note: {}\n", e.rust);
                    }
                }
                Status::OutOfScope => {
                    let _ = writeln!(
                        out,
                        "**Status:** not implemented in marcasite yet: it needs authentication, \
                         and marcasite covers only the public (unauthenticated) endpoints so far.\n"
                    );
                }
            }
            for u in &e.doc_urls {
                let _ = writeln!(out, "**Polymarket docs:** <{u}>\n");
            }
            if let Some(op) = &e.operation_id {
                let _ = writeln!(
                    out,
                    "**Spec:** documented only in the OpenAPI spec, operation `{op}`.\n"
                );
            }
            if !e.items.is_empty() {
                let _ = writeln!(out, "Implemented by:\n");
                for p in &e.items {
                    if let Some(i) = self.item(p) {
                        self.endpoint_item(&mut out, i);
                    }
                }
            }
        }
        if !citing.is_empty() {
            let _ = writeln!(out, "\n## Other items citing this page\n");
            for i in citing {
                self.endpoint_item(&mut out, i);
            }
        }
        Ok(trim_end(out))
    }

    fn endpoint_item(&self, out: &mut String, i: &Item) {
        let _ = writeln!(out, "- `{}`: {}\n", i.path, i.summary);
        let _ = writeln!(out, "  ```rust\n  {}\n  ```\n", one_line(&i.signature));
        let types: Vec<&Item> = i
            .related
            .iter()
            .filter_map(|p| self.item(p))
            .filter(|r| matches!(r.kind.as_str(), "struct" | "enum"))
            .collect();
        if !types.is_empty() {
            let _ = writeln!(
                out,
                "  Types in its signature and docs (request builders, ids, responses):\n"
            );
            for t in types {
                let _ = writeln!(out, "  - `{}`: {}", t.path, t.summary);
            }
            out.push('\n');
        }
    }

    /// Endpoint rows matching `query`, and for doc URLs, the items citing that page that no
    /// matching row already lists.
    fn find_endpoint(
        &self,
        query: &str,
        method: Option<&str>,
        service: Option<&str>,
    ) -> (QueryKind, Vec<&Endpoint>, Vec<&Item>) {
        let mut q = query.trim().trim_matches('`').trim();
        let mut method = method.map(|m| m.trim().to_uppercase());
        // `GET /tags/{id}`; but `Get tag by ID` is a doc page title, not a method and path.
        if let Some((first, rest)) = q.split_once(char::is_whitespace)
            && HTTP_METHODS.contains(&first.to_uppercase().as_str())
            && rest.trim_start().starts_with('/')
        {
            method.get_or_insert_with(|| first.to_uppercase());
            q = rest.trim();
        }
        let lower = q.to_lowercase();
        let kind = if lower.starts_with("http")
            || lower.contains("polymarket.com/")
            || lower.starts_with("docs/polymarket/")
            || lower.starts_with("api-reference/")
        {
            QueryKind::Url
        } else if lower.starts_with('/') || lower.starts_with("wss://") {
            QueryKind::Path
        } else {
            QueryKind::Name
        };
        let service = service.map(|s| s.trim().to_lowercase());
        let filters = |e: &Endpoint| {
            method.as_ref().is_none_or(|m| {
                e.method
                    .as_deref()
                    .is_some_and(|em| em.eq_ignore_ascii_case(m))
            }) && service.as_ref().is_none_or(|s| {
                e.service.to_lowercase().contains(s.as_str())
                    || e.module
                        .as_deref()
                        .and_then(|m| m.rsplit("::").next())
                        .is_some_and(|m| m == s)
            })
        };
        let matches = |e: &Endpoint| match kind {
            QueryKind::Url => {
                let want = normalize_url(q);
                e.doc_urls
                    .iter()
                    .chain(&e.local_docs)
                    .any(|u| normalize_url(u) == want)
            }
            QueryKind::Path => path_matches(&e.path, q),
            QueryKind::Name => {
                let slug = slugify(q);
                e.operation_id
                    .as_deref()
                    .is_some_and(|op| op.eq_ignore_ascii_case(q))
                    || e.doc_urls
                        .iter()
                        .any(|u| u.trim_end_matches('/').rsplit('/').next() == Some(slug.as_str()))
                    || e.items.iter().any(|p| {
                        p.ends_with(&format!("::{q}"))
                            || p.rsplit("::")
                                .next()
                                .is_some_and(|n| n.eq_ignore_ascii_case(q))
                    })
                    || e.channel
                        .as_deref()
                        .is_some_and(|c| c.to_lowercase().contains(&lower))
            }
        };
        let rows: Vec<&Endpoint> = self
            .index
            .endpoints
            .iter()
            .filter(|e| filters(e) && matches(e))
            .collect();
        let mut citing = Vec::new();
        if kind == QueryKind::Url {
            let want = normalize_url(q);
            let listed: BTreeSet<&str> = rows
                .iter()
                .flat_map(|e| e.items.iter().map(String::as_str))
                .collect();
            citing = self
                .index
                .items
                .iter()
                .filter(|i| !listed.contains(i.path.as_str()))
                .filter(|i| i.doc_urls.iter().any(|u| normalize_url(u) == want))
                .filter(|i| {
                    service
                        .as_ref()
                        .is_none_or(|s| i.module.rsplit("::").next().is_some_and(|m| m == s))
                })
                .collect();
        }
        (kind, rows, citing)
    }

    /// The `use-marcasite` prompt: idioms from the crate docs, plus how to use the tools.
    pub(crate) fn primer(&self) -> String {
        let root = self.index.modules.first();
        let docs = root.map(|m| shift_headings(&m.docs, 1)).unwrap_or_default();
        format!(
            "You are writing Rust code that uses marcasite {version}, an unofficial Rust SDK for \
             the Polymarket Predictions APIs (not affiliated with Polymarket). Follow its idioms:\n\n\
             - Create one `marcasite::Polymarket` (or one service client such as \
             `marcasite::gamma::GammaClient::new()`) and reuse it; clients are cheap to clone.\n\
             - Endpoints with only required parameters are `async fn`s. Endpoints with optional \
             parameters return a request builder: chain setters, then `.send().await`.\n\
             - Paginated endpoints: `.send()` returns one page; `.into_stream()` returns every \
             item as a `Stream` (use `futures_util::StreamExt`/`TryStreamExt`).\n\
             - Every call returns `marcasite::Result<T>`; match on `marcasite::Error` variants \
             for API errors, rate limits, timeouts and decoding failures.\n\
             - String enums are `#[non_exhaustive]` with an `Unknown(String)` variant: always \
             keep a wildcard arm.\n\
             - Prices, sizes and amounts are `rust_decimal::Decimal` (never `f64`); timestamps \
             are `chrono::DateTime<Utc>`; ids are newtypes such as `TokenId` and `ConditionId`.\n\n\
             Look up the exact API instead of guessing it, with the marcasite docs tools:\n\n\
             - `search`: find items by name or topic.\n\
             - `get_item`: full docs and signature of an item, e.g. `GammaClient::get_tag`.\n\
             - `list_module`: what a module (`gamma`, `clob`, `data`, `ws`, ...) contains.\n\
             - `find_endpoint`: which method implements a Polymarket endpoint (doc URL, HTTP \
             path or operation name), or whether it is not implemented yet.\n\n\
             The crate documentation follows.\n\n{docs}",
            version = self.index.marcasite_version
        )
    }
}

/// Scores `hay` = [name, path, summary, docs] (lower-cased) against the whole query, or failing
/// that against every token. 0 means no match.
fn score(q: &str, tokens: &[&str], hay: &[String; 4]) -> u32 {
    let [name, path, summary, docs] = hay;
    let whole = if name == q {
        1000
    } else if path == q || path.ends_with(&format!("::{q}")) {
        900
    } else if name.starts_with(q) {
        600
    } else if name.contains(q) {
        400
    } else if path.contains(q) {
        300
    } else if summary.contains(q) {
        200
    } else if docs.contains(q) {
        100
    } else {
        0
    };
    if whole > 0 || tokens.len() < 2 {
        return whole;
    }
    let mut total = 0;
    for t in tokens {
        let s = if name == t {
            100
        } else if name.starts_with(t) {
            60
        } else if name.contains(t) {
            40
        } else if path.contains(t) {
            30
        } else if summary.contains(t) {
            20
        } else if docs.contains(t) {
            10
        } else {
            return 0;
        };
        total += s;
    }
    total
}

/// Cleans up a user-supplied path: backticks, `()`, generics, `crate::`, `.` separators.
fn normalize(input: &str) -> String {
    let mut s = input.trim().trim_matches('`').trim().to_owned();
    // Generic arguments: `Paginated<T>` -> `Paginated`.
    let mut out = String::with_capacity(s.len());
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 && !c.is_whitespace() => out.push(c),
            _ => {}
        }
    }
    s = out.trim_end_matches("()").replace('.', "::");
    s = s.trim_start_matches("::").to_owned();
    if let Some(rest) = s.strip_prefix("crate::") {
        s = format!("marcasite::{rest}");
    }
    s.trim_end_matches("::").to_owned()
}

/// Prefixes `marcasite::` unless `q` already starts with it.
fn full_path(q: &str) -> String {
    if q == "marcasite" || q.starts_with("marcasite::") {
        q.to_owned()
    } else {
        format!("marcasite::{q}")
    }
}

/// Whether `name` appears as a whole word in `text`.
fn mentions(text: &str, name: &str) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|w| w == name)
}

/// `api-reference/tags/get-tag-by-id` from any form of a docs link (online URL, local
/// `docs/polymarket/...md` path, with or without fragment or trailing slash).
fn normalize_url(u: &str) -> String {
    let mut s = u.trim().trim_matches(['<', '>']).to_lowercase();
    if let Some(i) = s.find(['#', '?']) {
        s.truncate(i);
    }
    for prefix in [
        "https://",
        "http://",
        "www.",
        "docs.polymarket.com",
        "docs/polymarket",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_owned();
        }
    }
    let s = s.trim_matches('/');
    s.strip_suffix(".md").unwrap_or(s).to_owned()
}

/// `get-tag-by-id` from `Get tag by ID` or `get_tag_by_id`.
fn slugify(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Whether HTTP path `query` matches `pattern`, where `{param}` segments on either side match
/// any segment. A Data API v2 path also matches without its `/v2` prefix.
fn path_matches(pattern: &str, query: &str) -> bool {
    let query = query.split(['?', '#']).next().unwrap_or_default();
    let seg_match = |p: &str, q: &str| {
        let p: Vec<&str> = p.trim_matches('/').split('/').collect();
        let q: Vec<&str> = q.trim_matches('/').split('/').collect();
        p.len() == q.len()
            && p.iter()
                .zip(&q)
                .all(|(a, b)| a.eq_ignore_ascii_case(b) || is_param(a) || is_param(b))
    };
    seg_match(pattern, query)
        || pattern
            .strip_prefix("/v2")
            .is_some_and(|rest| !query.starts_with("/v2") && seg_match(rest, query))
}

fn is_param(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

fn endpoint_label(e: &Endpoint) -> String {
    match (&e.method, &e.channel) {
        (Some(m), _) => format!("`{m} {}`", e.path),
        (None, Some(c)) => format!("{c} (`{}`)", e.path),
        (None, None) => format!("`{}`", e.path),
    }
}

fn kind_label(kind: &str) -> &str {
    match kind {
        "fn" => "Function",
        "struct" => "Struct",
        "enum" => "Enum",
        "union" => "Union",
        "trait" => "Trait",
        "type" => "Type alias",
        "constant" => "Constant",
        "static" => "Static",
        "macro" => "Macro",
        "method" => "Method",
        "associated_const" => "Associated constant",
        "associated_type" => "Associated type",
        other => other,
    }
}

/// A signature on one line: comment and attribute lines dropped, whitespace collapsed.
fn one_line(sig: &str) -> String {
    sig.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//") && !l.starts_with("#["))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pushes markdown headings outside code fences down `by` levels (capped at 6).
fn shift_headings(md: &str, by: usize) -> String {
    let mut out = Vec::new();
    let mut fence: Option<&str> = None;
    for line in md.lines() {
        let t = line.trim_start();
        if let Some(f) = fence {
            if t.starts_with(f) {
                fence = None;
            }
            out.push(line.to_owned());
            continue;
        }
        if t.starts_with("```") {
            fence = Some("```");
            out.push(line.to_owned());
            continue;
        }
        if t.starts_with("~~~") {
            fence = Some("~~~");
            out.push(line.to_owned());
            continue;
        }
        let hashes = t.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && t.get(hashes..).is_some_and(|r| r.starts_with(' ')) {
            let level = (hashes + by).min(6);
            out.push(format!(
                "{}{}",
                "#".repeat(level),
                t.get(hashes..).unwrap_or_default()
            ));
        } else {
            out.push(line.to_owned());
        }
    }
    out.join("\n")
}

fn trim_end(mut s: String) -> String {
    s.truncate(s.trim_end().len());
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;

    fn docs() -> &'static Docs {
        static DOCS: OnceLock<Docs> = OnceLock::new();
        DOCS.get_or_init(|| Docs::new(Index::embedded().unwrap()))
    }

    fn item_path(l: Lookup<'_>) -> String {
        match l {
            Lookup::Item(i) => i.path.clone(),
            other => panic!("expected an item, got {other:?}"),
        }
    }

    #[test]
    fn resolves_full_and_short_paths() {
        let d = docs();
        let want = "marcasite::gamma::GammaClient::get_tag";
        for q in [
            want,
            "gamma::GammaClient::get_tag",
            "GammaClient::get_tag",
            "`GammaClient::get_tag()`",
            "crate::gamma::GammaClient::get_tag",
            "GammaClient.get_tag",
            "gammaclient::GET_TAG",
        ] {
            assert_eq!(item_path(d.lookup(q)), want, "{q}");
        }
        assert_eq!(
            item_path(d.lookup("GammaClient")),
            "marcasite::gamma::GammaClient"
        );
        assert_eq!(item_path(d.lookup("Paginated<T>")), "marcasite::Paginated");
    }

    #[test]
    fn resolves_reexports_to_the_canonical_item() {
        let d = docs();
        let id = d
            .index
            .items
            .iter()
            .find(|i| !i.reexported_as.is_empty() && !i.members.is_empty())
            .expect("a re-exported type with methods");
        let alias = &id.reexported_as[0];
        assert_eq!(item_path(d.lookup(alias)), id.path);
        let member = d.item(&id.members[0]).unwrap();
        assert_eq!(
            item_path(d.lookup(&format!("{alias}::{}", member.name))),
            member.path
        );
    }

    #[test]
    fn ambiguous_and_missing_paths() {
        let d = docs();
        match d.lookup("get_status") {
            Lookup::Ambiguous(items) => {
                let paths: Vec<&str> = items.iter().map(|i| i.path.as_str()).collect();
                assert!(paths.contains(&"marcasite::gamma::GammaClient::get_status"));
                assert!(paths.contains(&"marcasite::data::DataClient::get_status"));
            }
            other => panic!("expected ambiguity, got {other:?}"),
        }
        assert!(matches!(d.lookup("NoSuchThing"), Lookup::NotFound));
        assert!(matches!(d.lookup(""), Lookup::NotFound));
        let err = d.render_lookup("GammaClient::get_tagz").unwrap_err();
        assert!(err.contains("GammaClient::get_tag"), "{err}");
    }

    #[test]
    fn resolves_modules_and_members() {
        let d = docs();
        assert!(matches!(d.lookup("gamma"), Lookup::Module(m) if m.path == "marcasite::gamma"));
        assert!(matches!(d.lookup("marcasite"), Lookup::Module(m) if m.path == "marcasite"));
        // A field of a struct resolves to the struct.
        let event = d.item("marcasite::gamma::Event").unwrap();
        assert!(mentions(&event.signature, "title"));
        assert!(matches!(
            d.lookup("gamma::Event::title"),
            Lookup::Member { parent, name } if parent.path == event.path && name == "title"
        ));
    }

    #[test]
    fn search_ranks_exact_names_first() {
        let d = docs();
        let hits = d.search("GammaClient", None, None, 5);
        assert_eq!(hits[0].path, "marcasite::gamma::GammaClient");
        let hits = d.search("get_tag", None, None, 5);
        assert_eq!(hits[0].path, "marcasite::gamma::GammaClient::get_tag");
        // Prefix before substring.
        let hits = d.search("list_mark", None, Some("method"), 3);
        assert!(hits.iter().all(|h| h.kind == "method"));
        assert!(
            hits[0]
                .path
                .rsplit("::")
                .next()
                .unwrap()
                .starts_with("list_mark")
        );
        // Multi-word queries match every token.
        let hits = d.search("order book", Some("clob"), None, 10);
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|h| h.path.starts_with("marcasite::clob")));
        // Modules are searchable.
        let hits = d.search("gamma", None, Some("module"), 1);
        assert_eq!(hits[0].path, "marcasite::gamma");
        assert!(d.search("zzzzqqq", None, None, 5).is_empty());
        assert_eq!(d.search("client", None, None, 3).len(), 3);
    }

    #[test]
    fn find_endpoint_by_path_url_and_name() {
        let d = docs();
        let get_tag = "marcasite::gamma::GammaClient::get_tag";
        for (q, method) in [
            ("/tags/{id}", Some("GET")),
            ("GET /tags/{id}", None),
            ("/tags/123", Some("get")),
            (
                "https://docs.polymarket.com/api-reference/tags/get-tag-by-id",
                None,
            ),
            (
                "docs.polymarket.com/api-reference/tags/get-tag-by-id.md#x",
                None,
            ),
            ("docs/polymarket/api-reference/tags/get-tag-by-id.md", None),
            ("get-tag-by-id", None),
            ("Get tag by ID", None),
            ("GammaClient::get_tag", None),
        ] {
            let (_, rows, _) = d.find_endpoint(q, method, None);
            assert_eq!(rows.len(), 1, "{q}");
            assert_eq!(rows[0].items, [get_tag], "{q}");
        }
        let (_, rows, _) = d.find_endpoint("getTeam", None, None);
        assert_eq!(rows[0].items, ["marcasite::gamma::GammaClient::get_team"]);
        let (_, rows, _) = d.find_endpoint("/activity", None, Some("data"));
        assert_eq!(rows[0].path, "/v2/activity");

        let answer = d
            .render_find_endpoint("GET /tags/{id}", None, None)
            .unwrap();
        assert!(
            answer.contains("implemented in `marcasite::gamma`"),
            "{answer}"
        );
        assert!(answer.contains("pub fn get_tag("), "{answer}");
    }

    #[test]
    fn find_endpoint_reports_missing_endpoints() {
        let d = docs();
        let answer = d.render_find_endpoint("/midpoints", None, None).unwrap();
        assert!(
            answer.contains("not implemented in marcasite yet"),
            "{answer}"
        );
        assert!(answer.contains("ClobClient::get_midpoints"), "{answer}");
        let answer = d.render_find_endpoint("POST /order", None, None).unwrap();
        assert!(answer.contains("needs authentication"), "{answer}");
        assert!(
            d.render_find_endpoint("/no/such/route", None, None)
                .is_err()
        );
        assert!(
            d.render_find_endpoint("GET /tags/{id}", None, Some("clob"))
                .is_err()
        );
    }

    #[test]
    fn renders_items_and_every_module() {
        let d = docs();
        let item = d.render_lookup("GammaClient").unwrap();
        assert!(item.starts_with("# Struct `marcasite::gamma::GammaClient`"));
        assert!(item.contains("## Methods") || item.contains("## Associated items"));
        assert!(item.contains("pub fn get_tag(&self, id: impl Into<TagId>) -> GetTag"));
        let method = d.render_lookup("GammaClient::get_tag").unwrap();
        assert!(method.contains("Associated item of `marcasite::gamma::GammaClient`"));
        assert!(method.contains("https://docs.polymarket.com/api-reference/tags/get-tag-by-id"));
        assert!(method.contains("`marcasite::gamma::GetTag`"));

        for m in d.modules() {
            let index = d.render_module_index(m);
            let full = d.render_module_full(m);
            assert!(index.contains(&format!("`{}`", m.path)));
            assert!(full.len() >= index.len() / 2, "{}", m.path);
            for p in &m.items {
                let name = p.rsplit("::").next().unwrap();
                assert!(
                    index.contains(&format!("`{name}`")),
                    "{p} missing from {}",
                    m.path
                );
                assert!(
                    full.contains(&format!("`{name}`")),
                    "{p} missing from {}",
                    m.path
                );
            }
        }
        assert!(
            d.render_endpoints()
                .contains("| implemented | `GET /tags/{id}` |")
        );
        assert!(d.primer().contains("into_stream"));
    }

    #[test]
    fn helpers() {
        assert_eq!(
            normalize(" `crate::gamma::Paginated<Vec<T>>::next()` "),
            "marcasite::gamma::Paginated::next"
        );
        assert!(path_matches(
            "/tags/{id}/related-tags",
            "/tags/7/related-tags/"
        ));
        assert!(!path_matches("/tags/{id}", "/tags/7/related-tags"));
        assert!(path_matches("/v2/activity", "/activity"));
        assert_eq!(slugify("Get tag by ID"), "get-tag-by-id");
        assert_eq!(
            shift_headings("# A\n```\n# b\n```\n## C", 2),
            "### A\n```\n# b\n```\n#### C"
        );
        assert_eq!(
            one_line("// impl<T> X<T>\n#[must_use]\npub fn a(\n    x: T,\n) -> T"),
            "pub fn a( x: T, ) -> T"
        );
    }
}
