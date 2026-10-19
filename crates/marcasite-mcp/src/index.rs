//! The documentation index embedded in the binary.
//!
//! `cargo xtask docs` (run by `just docs-md`) writes `data/index.json` from rustdoc JSON and
//! `ENDPOINTS.md`; these types mirror the generator's (`xtask/src/docs/index.rs`). A change to
//! the shape bumps [`FORMAT_VERSION`] on both sides.

use std::fmt;

use serde::Deserialize;

/// The index format this server reads.
pub(crate) const FORMAT_VERSION: u32 = 1;

/// The index generated from the `marcasite` sources this crate was released with.
const EMBEDDED: &str = include_str!("../data/index.json");

/// The whole index.
#[derive(Debug, Deserialize)]
pub(crate) struct Index {
    pub(crate) format_version: u32,
    /// The `marcasite` version the index was generated from.
    pub(crate) marcasite_version: String,
    /// Root first, then by path.
    pub(crate) modules: Vec<Module>,
    /// Sorted by path.
    pub(crate) items: Vec<Item>,
    /// In `ENDPOINTS.md` order.
    pub(crate) endpoints: Vec<Endpoint>,
}

/// A public module.
#[derive(Debug, Deserialize)]
pub(crate) struct Module {
    /// `marcasite` or `marcasite::gamma`.
    pub(crate) path: String,
    pub(crate) summary: String,
    /// Markdown, headings starting at `#`.
    pub(crate) docs: String,
    pub(crate) submodules: Vec<String>,
    /// Paths of the items documented in this module, by name.
    pub(crate) items: Vec<String>,
    pub(crate) reexports: Vec<Reexport>,
}

/// A `pub use` in a module.
#[derive(Debug, Deserialize)]
pub(crate) struct Reexport {
    pub(crate) name: String,
    /// The canonical path of a workspace item.
    pub(crate) target: Option<String>,
    pub(crate) external: Option<External>,
}

/// A re-export of another crate or one of its items.
#[derive(Debug, Deserialize)]
pub(crate) struct External {
    pub(crate) source: String,
    pub(crate) crate_name: String,
    pub(crate) summary: String,
}

/// A documented item: a top-level item or an associated item.
#[derive(Debug, Deserialize)]
pub(crate) struct Item {
    /// Canonical public path, e.g. `marcasite::gamma::GammaClient::get_tag`.
    pub(crate) path: String,
    pub(crate) name: String,
    /// `struct`, `enum`, `trait`, `type`, `constant`, `static`, `fn`, `macro`, `union`,
    /// `method`, `associated_const` or `associated_type`.
    pub(crate) kind: String,
    pub(crate) module: String,
    /// The owning type or trait, for associated items.
    pub(crate) parent: Option<String>,
    /// The definition or signature as Rust source.
    pub(crate) signature: String,
    pub(crate) summary: String,
    /// Markdown, headings starting at `#`, intra-doc links pointing at item paths.
    pub(crate) docs: String,
    pub(crate) deprecated: Option<String>,
    pub(crate) implements: Vec<String>,
    /// Paths of the associated items, in source order.
    pub(crate) members: Vec<String>,
    pub(crate) reexported_as: Vec<String>,
    /// Polymarket documentation pages cited in the docs.
    pub(crate) doc_urls: Vec<String>,
    /// Items the docs link to or the signature mentions.
    pub(crate) related: Vec<String>,
}

/// Implementation status of an endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Status {
    Implemented,
    NotImplemented,
    /// Needs authentication, which marcasite does not support yet.
    OutOfScope,
}

/// A row of the repository's `ENDPOINTS.md`.
#[derive(Debug, Deserialize)]
pub(crate) struct Endpoint {
    /// Section heading, e.g. `Gamma API`.
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
    /// The table's `Rust` cell, as written.
    pub(crate) rust: String,
    /// Paths of the items implementing it.
    pub(crate) items: Vec<String>,
}

/// The embedded index could not be read.
#[derive(Debug)]
pub(crate) enum LoadError {
    Json(serde_json::Error),
    Version { found: u32 },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "the embedded docs index is malformed: {e}"),
            Self::Version { found } => write!(
                f,
                "the embedded docs index has format {found}, but this server reads format \
                 {FORMAT_VERSION}"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

impl Index {
    /// Parses the index embedded at build time.
    pub(crate) fn embedded() -> Result<Self, LoadError> {
        Self::parse(EMBEDDED)
    }

    /// Parses an index, checking its format version.
    pub(crate) fn parse(json: &str) -> Result<Self, LoadError> {
        let index: Self = serde_json::from_str(json).map_err(LoadError::Json)?;
        if index.format_version != FORMAT_VERSION {
            return Err(LoadError::Version {
                found: index.format_version,
            });
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_index_loads() {
        let index = Index::embedded().unwrap();
        assert_eq!(index.marcasite_version, env!("CARGO_PKG_VERSION"));
        assert!(!index.items.is_empty());
        assert!(!index.endpoints.is_empty());
        assert_eq!(index.modules[0].path, "marcasite");
    }

    #[test]
    fn rejects_bad_input() {
        assert!(matches!(Index::parse("{"), Err(LoadError::Json(_))));
        let other = r#"{"format_version": 999, "marcasite_version": "0", "modules": [],
                        "items": [], "endpoints": []}"#;
        assert!(matches!(
            Index::parse(other),
            Err(LoadError::Version { found: 999 })
        ));
    }
}
