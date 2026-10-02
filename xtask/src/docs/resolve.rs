//! Builds the public module tree of `marcasite`, following re-exports into private modules and
//! into `marcasite-core`, and decides which page documents each item.

use std::collections::HashMap;

use anyhow::{Context, bail};
use rustdoc_types::{ItemEnum, StructKind, VariantKind, Visibility};

use super::json::{Crates, ItemRef, Krate, Resolved};

/// One generated markdown file: a public module.
pub(crate) struct Page {
    /// Module path, starting with `marcasite`.
    pub(crate) path: Vec<String>,
    pub(crate) module: ItemRef,
    pub(crate) file: String,
    /// Sorted by name.
    pub(crate) entries: Vec<Entry>,
}

/// Something a module exposes.
pub(crate) enum Entry {
    /// A public submodule (its own page).
    Module { name: String, item: ItemRef },
    /// An item documented in full on this page.
    Item { name: String, item: ItemRef },
    /// An item documented on another page.
    Reexport { name: String, item: ItemRef },
    /// A re-export of another crate or of one of its items.
    External {
        name: String,
        source: String,
        crate_name: String,
        use_item: ItemRef,
    },
}

impl Entry {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Module { name, .. }
            | Self::Item { name, .. }
            | Self::Reexport { name, .. }
            | Self::External { name, .. } => name,
        }
    }
}

/// Where an item is documented.
#[derive(Clone)]
pub(crate) struct Loc {
    pub(crate) file: String,
    pub(crate) anchor: Option<String>,
    /// The public path, e.g. `marcasite::types::MarketId`.
    pub(crate) path: String,
}

impl Loc {
    pub(crate) fn url(&self) -> String {
        match &self.anchor {
            Some(a) => format!("{}#{a}", self.file),
            None => self.file.clone(),
        }
    }
}

/// The resolved documentation model.
pub(crate) struct Model {
    /// Root first, then by module path.
    pub(crate) pages: Vec<Page>,
    pub(crate) locs: HashMap<ItemRef, Loc>,
}

enum Target {
    Local(ItemRef),
    External {
        source: String,
        crate_name: String,
        use_item: ItemRef,
    },
}

/// A module's public names and what each refers to.
type Exposures = Vec<(String, Target)>;

pub(crate) fn build(crates: &Crates) -> anyhow::Result<Model> {
    // 1. Collect every public module with what it exposes.
    let mut modules: Vec<(Vec<String>, ItemRef, Exposures)> = Vec::new();
    let mut stack = vec![(vec!["marcasite".to_owned()], crates.root())];
    while let Some((path, module)) = stack.pop() {
        let exposures = exposures(crates, module)?;
        for (name, target) in &exposures {
            if let Target::Local(r) = target
                && matches!(crates.item(*r)?.inner, ItemEnum::Module(_))
            {
                let mut child = path.clone();
                child.push(name.clone());
                stack.push((child, *r));
            }
        }
        modules.push((path, module, exposures));
    }
    // Root first, then `types` (shared ids, re-exported by the service modules), then the rest.
    modules.sort_by_key(|(path, ..)| {
        (
            path.len() > 1,
            path.get(1).map(String::as_str) != Some("types"),
            path.clone(),
        )
    });

    // 2. Give each item one canonical page.
    let mut locs: HashMap<ItemRef, Loc> = HashMap::new();
    let mut pages = Vec::new();
    for (path, module, mut exposures) in modules {
        exposures.sort_by(|a, b| a.0.cmp(&b.0));
        let file = file_name(&path);
        locs.insert(
            module,
            Loc {
                file: file.clone(),
                anchor: None,
                path: path.join("::"),
            },
        );
        let mut entries = Vec::new();
        for (name, target) in exposures {
            entries.push(match target {
                Target::External {
                    source,
                    crate_name,
                    use_item,
                } => Entry::External {
                    name,
                    source,
                    crate_name,
                    use_item,
                },
                Target::Local(item) => {
                    if matches!(crates.item(item)?.inner, ItemEnum::Module(_)) {
                        Entry::Module { name, item }
                    } else if locs.contains_key(&item) {
                        Entry::Reexport { name, item }
                    } else {
                        let item_path = format!("{}::{name}", path.join("::"));
                        register(crates, &mut locs, item, &name, &file, &item_path)?;
                        Entry::Item { name, item }
                    }
                }
            });
        }
        pages.push(Page {
            path,
            module,
            file,
            entries,
        });
    }
    Ok(Model { pages, locs })
}

/// The public items of a module, with re-exports resolved to their targets.
fn exposures(crates: &Crates, module: ItemRef) -> anyhow::Result<Exposures> {
    let ItemEnum::Module(m) = &crates.item(module)?.inner else {
        bail!("{module:?} is not a module");
    };
    let mut out = Vec::new();
    for id in &m.items {
        let r = ItemRef {
            krate: module.krate,
            id: *id,
        };
        let item = crates.item(r)?;
        if !matches!(item.visibility, Visibility::Public) {
            continue;
        }
        match &item.inner {
            ItemEnum::Use(u) => {
                let resolved = u.id.and_then(|id| crates.resolve(module.krate, id));
                let target = match resolved {
                    Some(Resolved::Local(t)) => t,
                    Some(Resolved::External { crate_name }) => {
                        out.push((
                            u.name.clone(),
                            Target::External {
                                source: u.source.clone(),
                                crate_name,
                                use_item: r,
                            },
                        ));
                        continue;
                    }
                    None => bail!("cannot resolve `pub use {};` in {module:?}", u.source),
                };
                if u.is_glob {
                    out.extend(exposures(crates, target)?);
                } else {
                    out.push((u.name.clone(), Target::Local(target)));
                }
            }
            ItemEnum::Impl(_) => {}
            _ => {
                let name = item
                    .name
                    .clone()
                    .with_context(|| format!("unnamed item {r:?}"))?;
                out.push((name, Target::Local(r)));
            }
        }
    }
    Ok(out)
}

/// Records where `item` and its members (variants, fields, inherent methods) are documented.
fn register(
    crates: &Crates,
    locs: &mut HashMap<ItemRef, Loc>,
    item: ItemRef,
    name: &str,
    file: &str,
    path: &str,
) -> anyhow::Result<()> {
    let it = crates.item(item)?;
    let anchor = format!("{}.{name}", kind(&it.inner));
    let loc = |anchor: String, path: String| Loc {
        file: file.to_owned(),
        anchor: Some(anchor),
        path,
    };
    locs.insert(item, loc(anchor.clone(), path.to_owned()));
    let child = |id| ItemRef {
        krate: item.krate,
        id,
    };

    // Variants and fields are shown inside the item's definition, so they link to the item.
    let mut members = Vec::new();
    let mut impls: &[rustdoc_types::Id] = &[];
    match &it.inner {
        ItemEnum::Struct(s) => {
            impls = &s.impls;
            match &s.kind {
                StructKind::Plain { fields, .. } => members.extend(fields.iter().copied()),
                StructKind::Tuple(fields) => members.extend(fields.iter().flatten().copied()),
                StructKind::Unit => {}
            }
        }
        ItemEnum::Enum(e) => {
            impls = &e.impls;
            for v in &e.variants {
                members.push(*v);
                if let ItemEnum::Variant(var) = &crates.item(child(*v))?.inner {
                    match &var.kind {
                        VariantKind::Struct { fields, .. } => {
                            members.extend(fields.iter().copied())
                        }
                        VariantKind::Tuple(fields) => {
                            members.extend(fields.iter().flatten().copied())
                        }
                        VariantKind::Plain => {}
                    }
                }
            }
        }
        ItemEnum::Trait(t) => {
            for id in &t.items {
                let m = crates.item(child(*id))?;
                if let Some(n) = &m.name {
                    locs.insert(
                        child(*id),
                        loc(
                            format!("{name}.{}.{n}", kind(&m.inner)),
                            format!("{path}::{n}"),
                        ),
                    );
                }
            }
        }
        _ => {}
    }
    for m in members {
        let n = crates.item(child(m))?.name.clone().unwrap_or_default();
        locs.insert(child(m), loc(anchor.clone(), format!("{path}::{n}")));
    }
    for imp in inherent_impls(crates, item.krate, impls)? {
        for id in &imp.items {
            let m = crates.item(child(*id))?;
            if let (Visibility::Public, Some(n)) = (&m.visibility, &m.name) {
                locs.insert(
                    child(*id),
                    loc(
                        format!("{name}.{}.{n}", kind(&m.inner)),
                        format!("{path}::{n}"),
                    ),
                );
            }
        }
    }
    Ok(())
}

/// The inherent (non-trait, non-blanket) impls among `impls`.
pub(crate) fn inherent_impls<'a>(
    crates: &'a Crates,
    krate: Krate,
    impls: &[rustdoc_types::Id],
) -> anyhow::Result<Vec<&'a rustdoc_types::Impl>> {
    let mut out = Vec::new();
    for id in impls {
        if let ItemEnum::Impl(imp) = &crates.item(ItemRef { krate, id: *id })?.inner
            && imp.trait_.is_none()
            && imp.blanket_impl.is_none()
            && !imp.is_synthetic
        {
            out.push(imp);
        }
    }
    Ok(out)
}

/// The anchor prefix for an item kind (follows rustdoc's HTML naming).
pub(crate) fn kind(inner: &ItemEnum) -> &'static str {
    match inner {
        ItemEnum::Module(_) => "mod",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::Union(_) => "union",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::StructField(_) => "field",
        ItemEnum::Function(_) => "fn",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TypeAlias(_) => "type",
        ItemEnum::Constant { .. } | ItemEnum::AssocConst { .. } => "constant",
        ItemEnum::Static(_) => "static",
        ItemEnum::AssocType { .. } => "associatedtype",
        ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => "macro",
        _ => "item",
    }
}

fn file_name(path: &[String]) -> String {
    match path {
        [root] => format!("{root}.md"),
        [_, rest @ ..] => format!("{}.md", rest.join("-")),
        [] => "index.md".to_owned(),
    }
}
