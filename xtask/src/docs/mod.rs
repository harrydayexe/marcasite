//! `cargo xtask docs`: generates `docs/sdk/`, markdown API docs for LLMs, and the MCP server's
//! index (`crates/marcasite-mcp/data/index.json`) from rustdoc JSON.
//!
//! The JSON is built by the `docs-md` recipe in the justfile with the pinned nightly
//! (`docs-nightly`), so this tool itself runs on the workspace's stable toolchain.

mod endpoints;
mod index;
mod json;
mod render;
mod resolve;
mod write;

use std::path::Path;

use anyhow::{Context, bail};

/// Where the MCP index is written, relative to the workspace root.
pub(crate) const INDEX_PATH: &str = "crates/marcasite-mcp/data/index.json";

/// Generates the docs from `<root>/target/sdk-docs/doc/*.json` into `<root>/docs/sdk/` and the
/// index into [`INDEX_PATH`], or with `check`, fails if either is out of date.
pub(crate) fn run(root: &Path, check: bool) -> anyhow::Result<()> {
    let crates = json::Crates::load(&root.join("target/sdk-docs/doc"))?;
    let model = resolve::build(&crates)?;
    let files = write::render(&crates, &model)?;
    let endpoints_md =
        std::fs::read_to_string(root.join("ENDPOINTS.md")).context("reading ENDPOINTS.md")?;
    let index = index::build(&crates, &model, &endpoints_md)?;
    validate(&index)?;
    let index_json = index::to_json(&index)?;

    let out = root.join("docs/sdk");
    let index_path = root.join(INDEX_PATH);
    if check {
        let mut stale = write::stale(&out, &files)?;
        if std::fs::read_to_string(&index_path).ok().as_deref() != Some(index_json.as_str()) {
            stale.push(INDEX_PATH.to_owned());
        }
        if !stale.is_empty() {
            bail!(
                "docs/sdk/ or the MCP index is out of date ({}). Run `just docs-md` and commit \
                 the result.",
                stale.join(", ")
            );
        }
        println!(
            "docs/sdk/ ({} files) and {INDEX_PATH} are up to date",
            files.len()
        );
    } else {
        write::write(&out, &files)?;
        if let Some(dir) = index_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&index_path, &index_json)
            .with_context(|| format!("writing {INDEX_PATH}"))?;
        println!(
            "wrote {} files to docs/sdk/ and {} items to {INDEX_PATH}",
            files.len(),
            index.items.len()
        );
    }
    Ok(())
}

/// Checks what the MCP server relies on: the index round-trips through JSON, every item has a
/// signature and docs, and every endpoint row's methods are items of the index.
fn validate(index: &index::Index) -> anyhow::Result<()> {
    let json = index::to_json(index)?;
    let back: index::Index = serde_json::from_str(&json).context("re-reading the index")?;
    if &back != index {
        bail!("the index does not round-trip through JSON");
    }
    let mut paths = std::collections::HashSet::new();
    for item in &index.items {
        if !paths.insert(item.path.as_str()) {
            bail!("duplicate index item `{}`", item.path);
        }
        if item.signature.trim().is_empty() || item.docs.trim().is_empty() {
            bail!("index item `{}` has no signature or no docs", item.path);
        }
    }
    for e in &index.endpoints {
        for p in &e.items {
            if !paths.contains(p.as_str()) {
                bail!(
                    "endpoint {} resolves to `{p}`, which is not an item",
                    e.path
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed index (whatever nightly generated it) satisfies [`validate`] and is
    /// byte-for-byte what serializing it again gives.
    #[test]
    fn committed_index_round_trips() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let text = std::fs::read_to_string(root.join(INDEX_PATH)).unwrap();
        let index: index::Index = serde_json::from_str(&text).unwrap();
        assert_eq!(index.format_version, index::FORMAT_VERSION);
        validate(&index).unwrap();
        assert_eq!(index::to_json(&index).unwrap(), text);
        assert!(
            index
                .endpoints
                .iter()
                .filter(|e| e.status == endpoints::Status::Implemented)
                .all(|e| !e.items.is_empty())
        );
    }
}
