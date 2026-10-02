//! Renders one item (struct, enum, function, ...) as markdown.

use std::fmt::Write as _;

use anyhow::Context;
use rustdoc_types::{Attribute, Item, ItemEnum, StructKind, VariantKind, Visibility};

use super::doc;
use super::ty;
use crate::docs::json::{Crates, ItemRef, Resolved};
use crate::docs::resolve::{Model, inherent_impls, kind};

/// Traits left out of the `Implements` line (compiler plumbing).
const HIDDEN_TRAITS: &[&str] = &["StructuralPartialEq"];

pub(crate) struct Ctx<'a> {
    pub(crate) crates: &'a Crates,
    pub(crate) model: &'a Model,
}

impl Ctx<'_> {
    /// The item's docs with links rewritten and headings pushed down `shift` levels.
    pub(crate) fn docs(&self, r: ItemRef, item: &Item, shift: usize) -> String {
        let Some(docs) = item.docs.as_deref() else {
            return String::new();
        };
        let is_link = |k: &str| item.links.contains_key(k);
        let resolve = |k: &str| {
            let id = item.links.get(k)?;
            match self.crates.resolve(r.krate, *id)? {
                Resolved::Local(target) => self.model.locs.get(&target).map(|l| l.url()),
                Resolved::External { .. } => None,
            }
        };
        doc::rewrite(docs, &is_link, &resolve, shift)
    }

    /// The first paragraph of an item's docs, on one line.
    pub(crate) fn summary(&self, r: ItemRef, item: &Item) -> String {
        let docs = self.docs(r, item, 0);
        docs.split("\n\n")
            .next()
            .unwrap_or_default()
            .replace('\n', " ")
    }

    /// Renders a top-level item at heading level 3.
    pub(crate) fn item(&self, out: &mut String, r: ItemRef, name: &str) -> anyhow::Result<()> {
        let item = self.crates.item(r)?;
        let k = kind(&item.inner);
        let label = match k {
            "type" => "type",
            "constant" => "const",
            other => other,
        };
        writeln!(out, "### <a id=\"{k}.{name}\"></a>`{label} {name}`\n")?;
        writeln!(out, "```rust\n{}\n```\n", self.definition(r, item, name)?)?;
        deprecation(out, item)?;
        push_block(out, &self.docs(r, item, 3));

        let impls = match &item.inner {
            ItemEnum::Struct(s) => s.impls.as_slice(),
            ItemEnum::Enum(e) => e.impls.as_slice(),
            _ => &[],
        };
        let traits = self.trait_impls(r, impls)?;
        if !traits.is_empty() {
            let list: Vec<String> = traits.iter().map(|t| format!("`{t}`")).collect();
            writeln!(out, "**Implements:** {}\n", list.join(", "))?;
        }
        self.inherent(out, r, name, impls)?;
        if let ItemEnum::Trait(t) = &item.inner {
            let members = self.members(r, name, true, &t.items)?;
            if !members.is_empty() {
                writeln!(out, "#### Trait items\n\n{members}")?;
            }
        }
        Ok(())
    }

    /// The item's definition as Rust source (bodies and private fields elided).
    fn definition(&self, r: ItemRef, item: &Item, name: &str) -> anyhow::Result<String> {
        let mut s = attrs(item);
        match &item.inner {
            ItemEnum::Struct(st) => {
                let generics = ty::generic_params(&st.generics);
                let wc = ty::where_clause(&st.generics);
                match &st.kind {
                    StructKind::Unit => write!(s, "pub struct {name}{generics}{wc};")?,
                    StructKind::Tuple(fields) => {
                        let fields = self.tuple_fields(r, fields)?;
                        write!(s, "pub struct {name}{generics}({fields}){wc};")?;
                    }
                    StructKind::Plain {
                        fields,
                        has_stripped_fields,
                    } => {
                        write!(s, "pub struct {name}{generics}{wc} ")?;
                        s.push_str(&self.plain_fields(
                            r,
                            fields,
                            *has_stripped_fields,
                            "pub ",
                            "",
                        )?);
                    }
                }
            }
            ItemEnum::Enum(e) => {
                let wc = ty::where_clause(&e.generics);
                writeln!(
                    s,
                    "pub enum {name}{}{wc} {{",
                    ty::generic_params(&e.generics)
                )?;
                for v in &e.variants {
                    let vr = ItemRef {
                        krate: r.krate,
                        id: *v,
                    };
                    let var = self.crates.item(vr)?;
                    let ItemEnum::Variant(variant) = &var.inner else {
                        continue;
                    };
                    s.push_str(&doc_comment(&self.docs(vr, var, 0), "    "));
                    s.push_str(
                        &attrs(var)
                            .lines()
                            .map(|l| format!("    {l}\n"))
                            .collect::<String>(),
                    );
                    let vname = var.name.as_deref().unwrap_or_default();
                    write!(s, "    {vname}")?;
                    match &variant.kind {
                        VariantKind::Plain => {}
                        VariantKind::Tuple(fields) => {
                            write!(s, "({})", self.tuple_fields(r, fields)?)?
                        }
                        VariantKind::Struct {
                            fields,
                            has_stripped_fields,
                        } => {
                            s.push(' ');
                            s.push_str(&self.plain_fields(
                                r,
                                fields,
                                *has_stripped_fields,
                                "",
                                "    ",
                            )?);
                        }
                    }
                    if let Some(d) = &variant.discriminant {
                        write!(s, " = {}", d.expr)?;
                    }
                    s.push_str(",\n");
                }
                if e.has_stripped_variants {
                    s.push_str("    // some variants omitted\n");
                }
                s.push('}');
            }
            ItemEnum::Function(f) => s.push_str(&ty::fn_sig("pub ", name, f)),
            ItemEnum::TypeAlias(t) => {
                write!(
                    s,
                    "pub type {name}{} = {};",
                    ty::generic_params(&t.generics),
                    ty::ty(&t.type_)
                )?;
            }
            ItemEnum::Constant { type_, const_ } => {
                write!(s, "pub const {name}: {} = {};", ty::ty(type_), const_.expr)?;
            }
            ItemEnum::Static(st) => {
                let m = if st.is_mutable { "mut " } else { "" };
                write!(s, "pub static {m}{name}: {};", ty::ty(&st.type_))?;
            }
            ItemEnum::Trait(t) => {
                let b = if t.bounds.is_empty() {
                    String::new()
                } else {
                    format!(": {}", ty::bounds(&t.bounds))
                };
                let u = if t.is_unsafe { "unsafe " } else { "" };
                write!(
                    s,
                    "pub {u}trait {name}{}{b}{} {{ /* items below */ }}",
                    ty::generic_params(&t.generics),
                    ty::where_clause(&t.generics)
                )?;
            }
            ItemEnum::Macro(m) => s.push_str(m),
            other => write!(s, "/* {} {name} */", kind(other))?,
        }
        Ok(s)
    }

    fn tuple_fields(
        &self,
        r: ItemRef,
        fields: &[Option<rustdoc_types::Id>],
    ) -> anyhow::Result<String> {
        if fields.iter().all(Option::is_none) && !fields.is_empty() {
            return Ok("/* private fields */".to_owned());
        }
        let mut parts = Vec::new();
        for f in fields {
            match f {
                Some(id) => {
                    let fi = self.crates.item(ItemRef {
                        krate: r.krate,
                        id: *id,
                    })?;
                    let vis = if matches!(fi.visibility, Visibility::Public) {
                        "pub "
                    } else {
                        ""
                    };
                    if let ItemEnum::StructField(t) = &fi.inner {
                        parts.push(format!("{vis}{}", ty::ty(t)));
                    }
                }
                None => parts.push("/* private field */".to_owned()),
            }
        }
        Ok(parts.join(", "))
    }

    fn plain_fields(
        &self,
        r: ItemRef,
        fields: &[rustdoc_types::Id],
        stripped: bool,
        vis: &str,
        indent: &str,
    ) -> anyhow::Result<String> {
        if fields.is_empty() {
            return Ok(if stripped {
                "{ /* private fields */ }".to_owned()
            } else {
                "{}".to_owned()
            });
        }
        let inner = format!("{indent}    ");
        let mut s = "{\n".to_owned();
        for f in fields {
            let fr = ItemRef {
                krate: r.krate,
                id: *f,
            };
            let fi = self.crates.item(fr)?;
            let ItemEnum::StructField(t) = &fi.inner else {
                continue;
            };
            s.push_str(&doc_comment(&self.docs(fr, fi, 0), &inner));
            let fname = fi.name.as_deref().unwrap_or_default();
            writeln!(s, "{inner}{vis}{fname}: {},", ty::ty(t))?;
        }
        if stripped {
            writeln!(s, "{inner}/* private fields */")?;
        }
        write!(s, "{indent}}}")?;
        Ok(s)
    }

    /// Trait impls as `Trait<Args>` (with associated types, e.g. `Stream<Item = T>`), sorted.
    fn trait_impls(&self, r: ItemRef, impls: &[rustdoc_types::Id]) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        for id in impls {
            let ItemEnum::Impl(imp) = &self
                .crates
                .item(ItemRef {
                    krate: r.krate,
                    id: *id,
                })?
                .inner
            else {
                continue;
            };
            let Some(t) = &imp.trait_ else { continue };
            if imp.blanket_impl.is_some() || imp.is_synthetic {
                continue;
            }
            let mut s = ty::path(t);
            if HIDDEN_TRAITS.contains(&s.as_str()) {
                continue;
            }
            let mut assoc = Vec::new();
            for i in &imp.items {
                let it = self.crates.item(ItemRef {
                    krate: r.krate,
                    id: *i,
                })?;
                if let ItemEnum::AssocType { type_: Some(t), .. } = &it.inner {
                    assoc.push(format!(
                        "{} = {}",
                        it.name.as_deref().unwrap_or_default(),
                        ty::ty(t)
                    ));
                }
            }
            if !assoc.is_empty() {
                s = format!("{s}<{}>", assoc.join(", "));
            }
            if imp.is_negative {
                s = format!("!{s}");
            }
            out.push(s);
        }
        out.sort();
        out.dedup();
        Ok(out)
    }

    /// Public methods and associated items of the inherent impls.
    fn inherent(
        &self,
        out: &mut String,
        r: ItemRef,
        name: &str,
        impls: &[rustdoc_types::Id],
    ) -> anyhow::Result<()> {
        let impls = inherent_impls(self.crates, r.krate, impls)?;
        let items: Vec<(&rustdoc_types::Impl, rustdoc_types::Id)> = impls
            .iter()
            .flat_map(|imp| imp.items.iter().map(move |i| (*imp, *i)))
            .collect();
        let has_consts = items.iter().any(|(_, i)| {
            self.crates
                .item(ItemRef {
                    krate: r.krate,
                    id: *i,
                })
                .is_ok_and(|it| matches!(it.inner, ItemEnum::AssocConst { .. }))
        });
        let heading = if has_consts {
            "Associated items"
        } else {
            "Methods"
        };
        let mut rendered = String::new();
        for imp in impls {
            // Show the impl header only when it adds information (generics or bounds).
            let header =
                if imp.generics.params.is_empty() && imp.generics.where_predicates.is_empty() {
                    None
                } else {
                    Some(format!(
                        "impl{} {}{}",
                        ty::generic_params(&imp.generics),
                        ty::ty(&imp.for_),
                        ty::where_clause(&imp.generics)
                    ))
                };
            let members = self.members(r, name, false, &imp.items)?;
            if let (Some(h), false) = (header, members.is_empty()) {
                writeln!(rendered, "```rust\n{h}\n```\n")?;
            }
            rendered.push_str(&members);
        }
        if !rendered.is_empty() {
            writeln!(out, "#### {heading}\n")?;
            out.push_str(&rendered);
        }
        Ok(())
    }

    /// Renders public members (`fn`, `const`, `type`), or all members of a trait.
    fn members(
        &self,
        r: ItemRef,
        owner: &str,
        in_trait: bool,
        ids: &[rustdoc_types::Id],
    ) -> anyhow::Result<String> {
        let mut rendered = String::new();
        for id in ids {
            let mr = ItemRef {
                krate: r.krate,
                id: *id,
            };
            let m = self.crates.item(mr)?;
            if !in_trait && !matches!(m.visibility, Visibility::Public) {
                continue;
            }
            let mname = m.name.as_deref().context("unnamed associated item")?;
            let vis = if in_trait { "" } else { "pub " };
            let sig = match &m.inner {
                ItemEnum::Function(f) => ty::fn_sig(vis, mname, f),
                ItemEnum::AssocConst { type_, value, .. } => {
                    let v = value
                        .as_ref()
                        .map(|v| format!(" = {v}"))
                        .unwrap_or_default();
                    format!("{vis}const {mname}: {}{v};", ty::ty(type_))
                }
                ItemEnum::AssocType { bounds, type_, .. } => {
                    let b = if bounds.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", ty::bounds(bounds))
                    };
                    let t = type_
                        .as_ref()
                        .map(|t| format!(" = {}", ty::ty(t)))
                        .unwrap_or_default();
                    format!("type {mname}{b}{t};")
                }
                _ => continue,
            };
            let sig = format!("{}{sig}", attrs(m));
            writeln!(
                rendered,
                "##### <a id=\"{owner}.{}.{mname}\"></a>`{mname}`\n",
                kind(&m.inner)
            )?;
            writeln!(rendered, "```rust\n{sig}\n```\n")?;
            deprecation(&mut rendered, m)?;
            push_block(&mut rendered, &self.docs(mr, m, 5));
        }
        Ok(rendered)
    }
}

/// `#[non_exhaustive]` / `#[must_use]` lines.
fn attrs(item: &Item) -> String {
    let mut s = String::new();
    for a in &item.attrs {
        match a {
            Attribute::NonExhaustive => s.push_str("#[non_exhaustive]\n"),
            Attribute::MustUse { reason: None } => s.push_str("#[must_use]\n"),
            Attribute::MustUse { reason: Some(r) } => s.push_str(&format!("#[must_use = {r:?}]\n")),
            _ => {}
        }
    }
    s
}

/// Docs as `///` lines (for fields and variants inside a definition).
fn doc_comment(docs: &str, indent: &str) -> String {
    docs.lines()
        .map(|l| format!("{indent}///{}{l}\n", if l.is_empty() { "" } else { " " }))
        .collect()
}

fn deprecation(out: &mut String, item: &Item) -> anyhow::Result<()> {
    if let Some(d) = &item.deprecation {
        let since = d
            .since
            .as_ref()
            .map(|s| format!(" since {s}"))
            .unwrap_or_default();
        let note = d
            .note
            .as_ref()
            .map(|n| format!(": {n}"))
            .unwrap_or_default();
        writeln!(out, "**Deprecated{since}**{note}\n")?;
    }
    Ok(())
}

fn push_block(out: &mut String, block: &str) {
    if !block.is_empty() {
        out.push_str(block);
        out.push_str("\n\n");
    }
}
