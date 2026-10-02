//! Renders the model into markdown files and writes or checks them.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, bail};

use super::json::Crates;
use super::render::item::Ctx;
use super::resolve::{Entry, Model, Page, kind};

/// Kind sections in page order: (anchor kind, heading).
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

/// Renders every file, keyed by file name.
pub(crate) fn render(crates: &Crates, model: &Model) -> anyhow::Result<BTreeMap<String, String>> {
    let ctx = Ctx { crates, model };
    let version = crates
        .krate(super::json::Krate::Main)
        .crate_version
        .clone()
        .unwrap_or_default();
    let banner = format!(
        "> Generated from marcasite {version} (all features) by `just docs-md`. Do not edit."
    );
    let mut files = BTreeMap::new();
    let mut index = Vec::new();
    for page in &model.pages {
        let module = crates.item(page.module)?;
        index.push((
            page.file.clone(),
            page.path.join("::"),
            ctx.summary(page.module, module),
        ));
        files.insert(page.file.clone(), render_page(&ctx, page, &banner)?);
    }

    let mut readme = String::new();
    writeln!(readme, "# marcasite SDK reference\n\n{banner}\n")?;
    readme.push_str(
        "The public API of the [`marcasite`](https://crates.io/crates/marcasite) crate (an \
         unofficial Rust SDK for the Polymarket Predictions APIs) as plain markdown, generated \
         from rustdoc: every public module, type, method and signature, with its documentation. \
         Each file is one module; re-exports are followed, so a type is documented in full on \
         one page and linked from the others.\n\n\
         For the Polymarket APIs themselves see [`../polymarket/`](../polymarket/).\n\n",
    );
    writeln!(readme, "| File | Module | Summary |\n|---|---|---|")?;
    for (file, path, summary) in index {
        writeln!(
            readme,
            "| [`{file}`]({file}) | `{path}` | {} |",
            summary.replace('|', "\\|")
        )?;
    }
    files.insert("README.md".to_owned(), readme);
    Ok(files)
}

fn render_page(ctx: &Ctx<'_>, page: &Page, banner: &str) -> anyhow::Result<String> {
    let crates = ctx.crates;
    let module = crates.item(page.module)?;
    let mut out = String::new();
    let title = if page.path.len() == 1 {
        "Crate"
    } else {
        "Module"
    };
    writeln!(out, "# {title} `{}`\n\n{banner}\n", page.path.join("::"))?;
    let docs = ctx.docs(page.module, module, 1);
    if !docs.is_empty() {
        writeln!(out, "{docs}\n")?;
    }

    // Group entries.
    let mut modules = Vec::new();
    let mut reexports = Vec::new();
    let mut by_kind: BTreeMap<&str, Vec<(&str, super::json::ItemRef)>> = BTreeMap::new();
    for e in &page.entries {
        match e {
            Entry::Module { .. } => modules.push(e),
            Entry::Reexport { .. } | Entry::External { .. } => reexports.push(e),
            Entry::Item { name, item } => {
                by_kind
                    .entry(kind(&crates.item(*item)?.inner))
                    .or_default()
                    .push((name, *item));
            }
        }
    }

    // Index.
    writeln!(out, "## Index\n")?;
    if !modules.is_empty() {
        let list: Vec<String> = modules
            .iter()
            .map(|m| format!("[`{0}`]({0}.md)", m.name()))
            .collect();
        writeln!(out, "- **Modules:** {}", list.join(", "))?;
    }
    if !reexports.is_empty() {
        let list: Vec<String> = reexports
            .iter()
            .map(|r| format!("`{}`", r.name()))
            .collect();
        writeln!(out, "- **Re-exports:** {}", list.join(", "))?;
    }
    for (k, heading) in SECTIONS {
        if let Some(items) = by_kind.get(k) {
            let list: Vec<String> = items
                .iter()
                .map(|(n, _)| format!("[`{n}`](#{k}.{n})"))
                .collect();
            writeln!(out, "- **{heading}:** {}", list.join(", "))?;
        }
    }
    out.push('\n');

    if !modules.is_empty() {
        writeln!(out, "## Modules\n")?;
        for m in &modules {
            if let Entry::Module { name, item } = m {
                let loc = ctx.model.locs.get(item).context("module without a page")?;
                let summary = ctx.summary(*item, crates.item(*item)?);
                writeln!(out, "- [`{name}`]({}): {summary}", loc.file)?;
            }
        }
        out.push('\n');
    }

    if !reexports.is_empty() {
        writeln!(out, "## Re-exports\n")?;
        for r in &reexports {
            match r {
                Entry::Reexport { name, item } => {
                    let loc = ctx
                        .model
                        .locs
                        .get(item)
                        .context("re-export without a location")?;
                    writeln!(
                        out,
                        "- `{name}`: re-export of [`{}`]({}).",
                        loc.path,
                        loc.url()
                    )?;
                }
                Entry::External {
                    name,
                    source,
                    crate_name,
                    use_item,
                } => {
                    let docs = ctx.summary(*use_item, crates.item(*use_item)?);
                    let docs = if docs.is_empty() {
                        String::new()
                    } else {
                        format!(" {docs}")
                    };
                    writeln!(
                        out,
                        "- `{name}`: `pub use {source};` from the \
                         [`{crate_name}`](https://docs.rs/{crate_name}/latest/{crate_name}/) crate.{docs}"
                    )?;
                }
                _ => {}
            }
        }
        out.push('\n');
    }

    for (k, heading) in SECTIONS {
        let Some(items) = by_kind.get(k) else {
            continue;
        };
        writeln!(out, "## {heading}\n")?;
        for (name, item) in items {
            ctx.item(&mut out, *item, name)?;
        }
    }
    let unknown: Vec<&&str> = by_kind
        .keys()
        .filter(|k| !SECTIONS.iter().any(|(s, _)| s == *k))
        .collect();
    if !unknown.is_empty() {
        bail!("{}: no section for item kinds {unknown:?}", page.file);
    }
    Ok(format!("{}\n", out.trim_end()))
}

/// Replaces the contents of `dir` with `files`.
pub(crate) fn write(dir: &Path, files: &BTreeMap<String, String>) -> anyhow::Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).with_context(|| format!("removing {}", dir.display()))?;
    }
    std::fs::create_dir_all(dir)?;
    for (name, content) in files {
        std::fs::write(dir.join(name), content).with_context(|| format!("writing {name}"))?;
    }
    Ok(())
}

/// Lists the files in `dir` that differ from `files` (missing, changed or extra).
pub(crate) fn stale(dir: &Path, files: &BTreeMap<String, String>) -> anyhow::Result<Vec<String>> {
    let mut stale = Vec::new();
    for (name, content) in files {
        if std::fs::read_to_string(dir.join(name)).ok().as_deref() != Some(content.as_str()) {
            stale.push(name.clone());
        }
    }
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            if !files.contains_key(&name) {
                stale.push(name);
            }
        }
    }
    stale.sort();
    Ok(stale)
}
