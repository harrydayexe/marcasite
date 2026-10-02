//! Loading rustdoc JSON and resolving ids across the two workspace crates.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, bail};
use rustdoc_types::{Crate, Id, Item, ItemEnum};

/// Which workspace crate an id belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Krate {
    /// `marcasite`, the documented facade.
    Main,
    /// `marcasite-core`, whose public items `marcasite` re-exports.
    Core,
}

/// An item in one of the two crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ItemRef {
    pub(crate) krate: Krate,
    pub(crate) id: Id,
}

/// What an id seen from one crate refers to.
pub(crate) enum Resolved {
    /// An item documented in one of the workspace crates.
    Local(ItemRef),
    /// An item of another crate (`chrono`, `http`, ...).
    External { crate_name: String },
}

/// The rustdoc JSON of `marcasite` and `marcasite-core`.
pub(crate) struct Crates {
    main: Crate,
    core: Crate,
    /// `marcasite_core`'s crate number inside `marcasite`'s JSON.
    core_in_main: Option<u32>,
    /// `marcasite_core`'s own items by definition path.
    core_by_path: HashMap<Vec<String>, Id>,
}

impl Crates {
    /// Loads `<doc_dir>/marcasite.json` and `<doc_dir>/marcasite_core.json`.
    pub(crate) fn load(doc_dir: &Path) -> anyhow::Result<Self> {
        let main = load_one(&doc_dir.join("marcasite.json"))?;
        let core = load_one(&doc_dir.join("marcasite_core.json"))?;
        let core_in_main = main
            .external_crates
            .iter()
            .find(|(_, c)| c.name == "marcasite_core")
            .map(|(n, _)| *n);
        let core_by_path = core
            .paths
            .iter()
            .filter(|(_, s)| s.crate_id == 0)
            .map(|(id, s)| (s.path.clone(), *id))
            .collect();
        Ok(Self {
            main,
            core,
            core_in_main,
            core_by_path,
        })
    }

    pub(crate) fn krate(&self, k: Krate) -> &Crate {
        match k {
            Krate::Main => &self.main,
            Krate::Core => &self.core,
        }
    }

    /// The item behind `r`.
    pub(crate) fn item(&self, r: ItemRef) -> anyhow::Result<&Item> {
        self.krate(r.krate).index.get(&r.id).with_context(|| {
            format!(
                "{:?} item {:?} missing from the rustdoc JSON",
                r.krate, r.id
            )
        })
    }

    /// The `marcasite` crate root.
    pub(crate) fn root(&self) -> ItemRef {
        ItemRef {
            krate: Krate::Main,
            id: self.main.root,
        }
    }

    /// Resolves `id` as referenced from crate `from` (a `use` target or an intra-doc link).
    /// Returns `None` if the id is unknown.
    pub(crate) fn resolve(&self, from: Krate, id: Id) -> Option<Resolved> {
        let krate = self.krate(from);
        if krate.index.contains_key(&id) {
            return Some(Resolved::Local(ItemRef { krate: from, id }));
        }
        let summary = krate.paths.get(&id)?;
        if from == Krate::Main && Some(summary.crate_id) == self.core_in_main {
            return self.core_path(&summary.path).map(|id| {
                Resolved::Local(ItemRef {
                    krate: Krate::Core,
                    id,
                })
            });
        }
        let crate_name = krate.external_crates.get(&summary.crate_id)?.name.clone();
        Some(Resolved::External { crate_name })
    }

    /// Finds a `marcasite_core` item by definition path. Items without a path entry (methods)
    /// are looked up among the inherent impls of their parent type.
    fn core_path(&self, path: &[String]) -> Option<Id> {
        if let Some(id) = self.core_by_path.get(path) {
            return Some(*id);
        }
        let (name, parent) = path.split_last()?;
        let parent = self.core.index.get(self.core_by_path.get(parent)?)?;
        let impls = match &parent.inner {
            ItemEnum::Struct(s) => &s.impls,
            ItemEnum::Enum(e) => &e.impls,
            _ => return None,
        };
        impls
            .iter()
            .filter_map(|i| match &self.core.index.get(i)?.inner {
                ItemEnum::Impl(imp) if imp.trait_.is_none() => Some(&imp.items),
                _ => None,
            })
            .flatten()
            .find(|i| self.core.index.get(i).and_then(|it| it.name.as_ref()) == Some(name))
            .copied()
    }
}

fn load_one(path: &Path) -> anyhow::Result<Crate> {
    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "reading {} (run `just docs-md`, which builds it first)",
            path.display()
        )
    })?;
    // A different format version usually fails to parse, so check the version first.
    let format_version = serde_json::from_slice::<serde_json::Value>(&bytes)
        .with_context(|| format!("parsing {}", path.display()))?
        .get("format_version")
        .and_then(serde_json::Value::as_u64);
    if format_version != Some(u64::from(rustdoc_types::FORMAT_VERSION)) {
        bail!(
            "{} has rustdoc JSON format {}, but xtask's rustdoc-types expects {}. Bump the \
             `docs-nightly` toolchain in the justfile and the `rustdoc-types` version in \
             xtask/Cargo.toml together.",
            path.display(),
            format_version.map_or_else(|| "unknown".to_owned(), |v| v.to_string()),
            rustdoc_types::FORMAT_VERSION,
        );
    }
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}
