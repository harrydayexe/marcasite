//! Builds the MCP server's index (`crates/marcasite-mcp/data/index.json`) from the resolved
//! model and `ENDPOINTS.md`.
//!
//! The index is self-contained: every item's signature and docs are in it, with intra-doc links
//! pointing at item paths (`marcasite::gamma::GammaClient`) instead of markdown files. The
//! server deserializes it with its own copy of these types, so a change here is a change of
//! `FORMAT_VERSION` and of `crates/marcasite-mcp/src/index.rs`.

use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, bail};
use rustdoc_types::{Item, ItemEnum, Visibility};
use serde::{Deserialize, Serialize};

use super::endpoints::{self, Status};
use super::json::{Crates, ItemRef, Krate, Resolved};
use super::render::item::{Ctx, Links, deprecation_note, member_sig};
use super::render::ty;
use super::resolve::{Entry, Model, inherent_impls, kind};

/// Bumped whenever the shape of [`Index`] changes; the server refuses other versions.
pub(crate) const FORMAT_VERSION: u32 = 1;

/// The whole index.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Index {
    pub(crate) format_version: u32,
    /// The `marcasite` version the index was generated from.
    pub(crate) marcasite_version: String,
    /// Root first, then by path.
    pub(crate) modules: Vec<Module>,
    /// Sorted by path.
    pub(crate) items: Vec<IndexItem>,
    /// In `ENDPOINTS.md` order.
    pub(crate) endpoints: Vec<Endpoint>,
}

/// A public module.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Module {
    /// `marcasite` or `marcasite::gamma`.
    pub(crate) path: String,
    pub(crate) summary: String,
    /// The module docs (markdown, headings starting at `#`).
    pub(crate) docs: String,
    pub(crate) submodules: Vec<String>,
    /// Paths of the items documented in this module, by name.
    pub(crate) items: Vec<String>,
    /// Items documented elsewhere and re-exported here, by name.
    pub(crate) reexports: Vec<Reexport>,
}

/// A `pub use` in a module.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Reexport {
    pub(crate) name: String,
    /// The canonical path of a workspace item.
    pub(crate) target: Option<String>,
    /// For a re-export of another crate: the `use` source and the crate's name.
    pub(crate) external: Option<External>,
}

/// A re-export of another crate or one of its items.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct External {
    pub(crate) source: String,
    pub(crate) crate_name: String,
    pub(crate) summary: String,
}

/// A documented item: a top-level item or an associated item (method, constant, type).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct IndexItem {
    /// Canonical public path, e.g. `marcasite::gamma::GammaClient::get_tag`.
    pub(crate) path: String,
    pub(crate) name: String,
    /// `struct`, `enum`, `trait`, `type`, `constant`, `static`, `fn`, `macro`, `union`,
    /// `method`, `associated_const` or `associated_type`.
    pub(crate) kind: String,
    /// The module documenting it.
    pub(crate) module: String,
    /// The owning type or trait, for associated items.
    pub(crate) parent: Option<String>,
    /// The definition or signature as Rust source.
    pub(crate) signature: String,
    /// The first paragraph of the docs, on one line.
    pub(crate) summary: String,
    /// The docs (markdown, headings starting at `#`, intra-doc links pointing at item paths).
    pub(crate) docs: String,
    pub(crate) deprecated: Option<String>,
    /// Implemented traits (`Clone`, `Stream<Item = T>`, ...).
    pub(crate) implements: Vec<String>,
    /// Paths of the associated items, in source order.
    pub(crate) members: Vec<String>,
    /// Other public paths the item is re-exported at.
    pub(crate) reexported_as: Vec<String>,
    /// Polymarket documentation pages cited in the docs.
    pub(crate) doc_urls: Vec<String>,
    /// Workspace items the docs link to or the signature returns.
    pub(crate) related: Vec<String>,
}

/// A row of `ENDPOINTS.md`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Endpoint {
    pub(crate) service: String,
    pub(crate) module: Option<String>,
    pub(crate) status: Status,
    /// HTTP method; `None` for WebSocket channels.
    pub(crate) method: Option<String>,
    /// HTTP path, or the URL of a WebSocket channel.
    pub(crate) path: String,
    pub(crate) channel: Option<String>,
    pub(crate) doc_urls: Vec<String>,
    pub(crate) local_docs: Vec<String>,
    pub(crate) operation_id: Option<String>,
    /// The `Rust` cell of the table, as written.
    pub(crate) rust: String,
    /// Paths of the index items implementing it.
    pub(crate) items: Vec<String>,
}

/// Builds the index.
pub(crate) fn build(crates: &Crates, model: &Model, endpoints_md: &str) -> anyhow::Result<Index> {
    let ctx = Ctx {
        crates,
        model,
        links: Links::Paths,
    };
    let mut modules = Vec::new();
    let mut items: BTreeMap<String, IndexItem> = BTreeMap::new();
    let mut reexported_as: HashMap<String, Vec<String>> = HashMap::new();
    for page in &model.pages {
        let module_path = page.path.join("::");
        let module_item = crates.item(page.module)?;
        let mut module = Module {
            path: module_path.clone(),
            summary: ctx.summary(page.module, module_item),
            docs: ctx.docs(page.module, module_item, 0),
            submodules: Vec::new(),
            items: Vec::new(),
            reexports: Vec::new(),
        };
        for entry in &page.entries {
            match entry {
                Entry::Module { item, .. } => module.submodules.push(path_of(model, *item)?),
                Entry::Item { name, item } => {
                    let path = path_of(model, *item)?;
                    module.items.push(path.clone());
                    add_item(&ctx, &mut items, *item, name, &module_path)?;
                }
                Entry::Reexport { name, item } => {
                    let target = path_of(model, *item)?;
                    reexported_as
                        .entry(target.clone())
                        .or_default()
                        .push(format!("{module_path}::{name}"));
                    module.reexports.push(Reexport {
                        name: name.clone(),
                        target: Some(target),
                        external: None,
                    });
                }
                Entry::External {
                    name,
                    source,
                    crate_name,
                    use_item,
                } => module.reexports.push(Reexport {
                    name: name.clone(),
                    target: None,
                    external: Some(External {
                        source: source.clone(),
                        crate_name: crate_name.clone(),
                        summary: ctx.summary(*use_item, crates.item(*use_item)?),
                    }),
                }),
            }
        }
        modules.push(module);
    }
    for (target, mut paths) in reexported_as {
        if let Some(item) = items.get_mut(&target) {
            paths.sort();
            item.reexported_as = paths;
        }
    }

    let version = crates
        .krate(Krate::Main)
        .crate_version
        .clone()
        .unwrap_or_default();
    let items: Vec<IndexItem> = items.into_values().collect();
    let endpoints = resolve_endpoints(endpoints::parse(endpoints_md)?, &items)?;
    Ok(Index {
        format_version: FORMAT_VERSION,
        marcasite_version: version,
        modules,
        items,
        endpoints,
    })
}

/// Serializes the index as pretty-printed JSON with a trailing newline.
pub(crate) fn to_json(index: &Index) -> anyhow::Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(index)?))
}

fn path_of(model: &Model, item: ItemRef) -> anyhow::Result<String> {
    model
        .locs
        .get(&item)
        .map(|l| l.path.clone())
        .with_context(|| format!("{item:?} has no documented location"))
}

/// Adds a top-level item and its associated items.
fn add_item(
    ctx: &Ctx<'_>,
    items: &mut BTreeMap<String, IndexItem>,
    r: ItemRef,
    name: &str,
    module: &str,
) -> anyhow::Result<()> {
    let crates = ctx.crates;
    let item = crates.item(r)?;
    let path = path_of(ctx.model, r)?;
    let impls = match &item.inner {
        ItemEnum::Struct(s) => s.impls.as_slice(),
        ItemEnum::Enum(e) => e.impls.as_slice(),
        _ => &[],
    };

    // Associated items: inherent impls (public only), or a trait's items.
    let mut members = Vec::new();
    for imp in inherent_impls(crates, r.krate, impls)? {
        // Show the impl header only when it adds information (generics or bounds).
        let header = if imp.generics.params.is_empty() && imp.generics.where_predicates.is_empty() {
            None
        } else {
            Some(format!(
                "impl{} {}{}",
                ty::generic_params(&imp.generics),
                ty::ty(&imp.for_),
                ty::where_clause(&imp.generics)
            ))
        };
        for id in &imp.items {
            members.push((*id, false, header.clone()));
        }
    }
    if let ItemEnum::Trait(t) = &item.inner {
        for id in &t.items {
            members.push((*id, true, None));
        }
    }
    let mut member_paths = Vec::new();
    for (id, in_trait, header) in members {
        let mr = ItemRef { krate: r.krate, id };
        let m = crates.item(mr)?;
        if !in_trait && !matches!(m.visibility, Visibility::Public) {
            continue;
        }
        let mname = m.name.as_deref().context("unnamed associated item")?;
        let Some(sig) = member_sig(m, mname, in_trait) else {
            continue;
        };
        let signature = match header {
            Some(h) => format!("// {h}\n{sig}"),
            None => sig,
        };
        let mpath = path_of(ctx.model, mr)?;
        if items.contains_key(&mpath) {
            // The same name in two impl blocks (different bounds): keep both signatures.
            if let Some(existing) = items.get_mut(&mpath) {
                existing.signature = format!("{}\n\n{signature}", existing.signature);
            }
            continue;
        }
        let kind = match &m.inner {
            ItemEnum::Function(_) => "method",
            ItemEnum::AssocConst { .. } => "associated_const",
            _ => "associated_type",
        };
        member_paths.push(mpath.clone());
        let entry = new_item(
            ctx,
            mr,
            m,
            &mpath,
            mname,
            kind,
            module,
            Some(&path),
            signature,
        )?;
        items.insert(mpath, entry);
    }

    let signature = ctx.definition(r, item, name)?;
    let mut entry = new_item(
        ctx,
        r,
        item,
        &path,
        name,
        kind(&item.inner),
        module,
        None,
        signature,
    )?;
    entry.implements = ctx.trait_impls(r, impls)?;
    entry.members = member_paths;
    items.insert(path, entry);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn new_item(
    ctx: &Ctx<'_>,
    r: ItemRef,
    item: &Item,
    path: &str,
    name: &str,
    kind: &str,
    module: &str,
    parent: Option<&str>,
    signature: String,
) -> anyhow::Result<IndexItem> {
    Ok(IndexItem {
        path: path.to_owned(),
        name: name.to_owned(),
        kind: kind.to_owned(),
        module: module.to_owned(),
        parent: parent.map(str::to_owned),
        signature,
        summary: ctx.summary(r, item),
        docs: ctx.docs(r, item, 0),
        deprecated: deprecation_note(item),
        implements: Vec::new(),
        members: Vec::new(),
        reexported_as: Vec::new(),
        doc_urls: item.docs.as_deref().map(doc_urls).unwrap_or_default(),
        related: related(ctx, r, item, path),
    })
}

/// Items the docs link to, then the items named in a function's return and parameter types.
fn related(ctx: &Ctx<'_>, r: ItemRef, item: &Item, own_path: &str) -> Vec<String> {
    let mut ids: Vec<(Krate, rustdoc_types::Id)> =
        item.links.values().map(|id| (r.krate, *id)).collect();
    if let ItemEnum::Function(f) = &item.inner {
        let mut sig_ids = Vec::new();
        if let Some(out) = &f.sig.output {
            ty::path_ids(out, &mut sig_ids);
        }
        for (_, input) in &f.sig.inputs {
            ty::path_ids(input, &mut sig_ids);
        }
        ids.extend(sig_ids.into_iter().map(|id| (r.krate, id)));
    }
    // Links are a map, so sort them for a stable order; signature types keep theirs.
    let link_count = item.links.len();
    let mut paths: Vec<String> = Vec::new();
    let mut link_paths: Vec<String> = Vec::new();
    for (i, (krate, id)) in ids.into_iter().enumerate() {
        let Some(Resolved::Local(target)) = ctx.crates.resolve(krate, id) else {
            continue;
        };
        let Some(loc) = ctx.model.locs.get(&target) else {
            continue;
        };
        if i < link_count {
            link_paths.push(loc.path.clone());
        } else {
            paths.push(loc.path.clone());
        }
    }
    link_paths.sort();
    let mut out: Vec<String> = Vec::new();
    for p in link_paths.into_iter().chain(paths) {
        if p != own_path && !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The `https://docs.polymarket.com/...` URLs in `docs`, in order, without duplicates.
pub(crate) fn doc_urls(docs: &str) -> Vec<String> {
    const PREFIX: &str = "https://docs.polymarket.com";
    let mut out: Vec<String> = Vec::new();
    let mut rest = docs;
    while let Some(i) = rest.find(PREFIX) {
        let tail = rest.get(i..).unwrap_or_default();
        let end = tail
            .find(|c: char| c.is_whitespace() || matches!(c, '>' | ')' | ']' | '"' | '\'' | '`'))
            .unwrap_or(tail.len());
        let url = tail
            .get(..end)
            .unwrap_or_default()
            .trim_end_matches(['.', ',', ';', ':']);
        if !out.iter().any(|u| u == url) {
            out.push(url.to_owned());
        }
        rest = tail.get(end.max(1)..).unwrap_or_default();
    }
    out
}

/// Resolves each row's `Type::method` mentions to index items. An implemented row must name at
/// least one method, and every name must match an item: this catches drift between
/// `ENDPOINTS.md` and the code.
fn resolve_endpoints(
    rows: Vec<endpoints::Row>,
    items: &[IndexItem],
) -> anyhow::Result<Vec<Endpoint>> {
    let mut out = Vec::new();
    for row in rows {
        let what = format!(
            "ENDPOINTS.md row {} {}",
            row.method
                .as_deref()
                .or(row.channel.as_deref())
                .unwrap_or(""),
            row.path
        );
        if row.status == Status::Implemented && row.rust_paths.is_empty() {
            bail!("{what} is marked implemented but names no `Type::method`");
        }
        let mut resolved = Vec::new();
        for rust in &row.rust_paths {
            let suffix = format!("::{rust}");
            let matches: Vec<&IndexItem> = items
                .iter()
                .filter(|i| i.path.ends_with(&suffix))
                .filter(|i| {
                    row.module
                        .as_ref()
                        .is_none_or(|m| i.path.starts_with(&format!("{m}::")))
                })
                .collect();
            if matches.is_empty() {
                bail!("{what}: `{rust}` does not match any documented item");
            }
            resolved.extend(matches.into_iter().map(|i| i.path.clone()));
        }
        out.push(Endpoint {
            service: row.service,
            module: row.module,
            status: row.status,
            method: row.method,
            path: row.path,
            channel: row.channel,
            doc_urls: row.doc_urls,
            local_docs: row.local_docs,
            operation_id: row.operation_id,
            rust: row.rust,
            items: resolved,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_doc_urls() {
        let docs = "Gets a tag.\n\nSee <https://docs.polymarket.com/api-reference/tags/get-tag-by-id>.\n\
                    Also [x](https://docs.polymarket.com/a/b), https://docs.polymarket.com/a/b.";
        assert_eq!(
            doc_urls(docs),
            [
                "https://docs.polymarket.com/api-reference/tags/get-tag-by-id",
                "https://docs.polymarket.com/a/b"
            ]
        );
    }
}
