//! `cargo xtask docs`: generates `docs/sdk/`, markdown API docs for LLMs, from rustdoc JSON.
//!
//! The JSON is built by the `docs-md` recipe in the justfile with the pinned nightly
//! (`docs-nightly`), so this tool itself runs on the workspace's stable toolchain.

mod json;
mod render;
mod resolve;
mod write;

use std::path::Path;

use anyhow::bail;

/// Generates the docs from `<root>/target/sdk-docs/doc/*.json` into `<root>/docs/sdk/`, or
/// with `check`, fails if `docs/sdk/` is out of date.
pub(crate) fn run(root: &Path, check: bool) -> anyhow::Result<()> {
    let crates = json::Crates::load(&root.join("target/sdk-docs/doc"))?;
    let model = resolve::build(&crates)?;
    let files = write::render(&crates, &model)?;
    let out = root.join("docs/sdk");
    if check {
        let stale = write::stale(&out, &files)?;
        if !stale.is_empty() {
            bail!(
                "docs/sdk/ is out of date ({}). Run `just docs-md` and commit the result.",
                stale.join(", ")
            );
        }
        println!("docs/sdk/ is up to date ({} files)", files.len());
    } else {
        write::write(&out, &files)?;
        println!("wrote {} files to docs/sdk/", files.len());
    }
    Ok(())
}
