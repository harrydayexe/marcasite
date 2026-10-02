//! Turns rustdoc JSON types back into Rust source text.
//!
//! Paths are printed by their last segment (`Result<Status>`, not
//! `marcasite_core::error::Result<Status>`), which is how the items read in source and keeps
//! signatures short.

use rustdoc_types::{
    AssocItemConstraint, AssocItemConstraintKind, Function, FunctionHeader, FunctionPointer,
    FunctionSignature, GenericArg, GenericArgs, GenericBound, GenericParamDef, GenericParamDefKind,
    Generics, Path, PreciseCapturingArg, Term, TraitBoundModifier, Type, WherePredicate,
};

/// Renders a type.
pub(crate) fn ty(t: &Type) -> String {
    match t {
        Type::ResolvedPath(p) => path(p),
        Type::DynTrait(d) => {
            let mut parts: Vec<String> = d
                .traits
                .iter()
                .map(|pt| format!("{}{}", hrtb(&pt.generic_params), path(&pt.trait_)))
                .collect();
            if let Some(l) = &d.lifetime {
                parts.push(l.clone());
            }
            format!("dyn {}", parts.join(" + "))
        }
        Type::Generic(name) | Type::Primitive(name) => name.clone(),
        Type::FunctionPointer(fp) => fn_pointer(fp),
        Type::Tuple(items) => match items.as_slice() {
            [one] => format!("({},)", ty(one)),
            _ => format!("({})", join(items.iter().map(ty))),
        },
        Type::Slice(inner) => format!("[{}]", ty(inner)),
        Type::Array { type_, len } => format!("[{}; {len}]", ty(type_)),
        Type::Pat { type_, .. } => ty(type_),
        Type::ImplTrait(b) => format!("impl {}", bounds(b)),
        Type::Infer => "_".to_owned(),
        Type::RawPointer { is_mutable, type_ } => {
            format!(
                "*{} {}",
                if *is_mutable { "mut" } else { "const" },
                ty(type_)
            )
        }
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => {
            let mut s = "&".to_owned();
            if let Some(l) = lifetime {
                s.push_str(l);
                s.push(' ');
            }
            if *is_mutable {
                s.push_str("mut ");
            }
            s.push_str(&ty(type_));
            s
        }
        Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => {
            let args = args.as_deref().map(generic_args).unwrap_or_default();
            match trait_ {
                Some(t) => format!("<{} as {}>::{name}{args}", ty(self_type), path(t)),
                None => format!("{}::{name}{args}", ty(self_type)),
            }
        }
    }
}

/// Renders a path by its last segment plus generic arguments.
pub(crate) fn path(p: &Path) -> String {
    let name = p.path.rsplit("::").next().unwrap_or(&p.path);
    let args = p.args.as_deref().map(generic_args).unwrap_or_default();
    format!("{name}{args}")
}

fn generic_args(a: &GenericArgs) -> String {
    match a {
        GenericArgs::AngleBracketed { args, constraints } => {
            let parts: Vec<String> = args
                .iter()
                .map(generic_arg)
                .chain(constraints.iter().map(constraint))
                .collect();
            if parts.is_empty() {
                String::new()
            } else {
                format!("<{}>", parts.join(", "))
            }
        }
        GenericArgs::Parenthesized { inputs, output } => {
            let out = output
                .as_ref()
                .map(|o| format!(" -> {}", ty(o)))
                .unwrap_or_default();
            format!("({}){out}", join(inputs.iter().map(ty)))
        }
        GenericArgs::ReturnTypeNotation => "(..)".to_owned(),
    }
}

fn generic_arg(a: &GenericArg) -> String {
    match a {
        GenericArg::Lifetime(l) => l.clone(),
        GenericArg::Type(t) => ty(t),
        GenericArg::Const(c) => c.expr.clone(),
        GenericArg::Infer => "_".to_owned(),
    }
}

fn constraint(c: &AssocItemConstraint) -> String {
    let args = c.args.as_deref().map(generic_args).unwrap_or_default();
    match &c.binding {
        AssocItemConstraintKind::Equality(t) => format!("{}{args} = {}", c.name, term(t)),
        AssocItemConstraintKind::Constraint(b) => format!("{}{args}: {}", c.name, bounds(b)),
    }
}

fn term(t: &Term) -> String {
    match t {
        Term::Type(t) => ty(t),
        Term::Constant(c) => c.expr.clone(),
    }
}

/// Renders bounds joined by ` + `.
pub(crate) fn bounds(b: &[GenericBound]) -> String {
    b.iter().map(bound).collect::<Vec<_>>().join(" + ")
}

fn bound(b: &GenericBound) -> String {
    match b {
        GenericBound::TraitBound {
            trait_,
            generic_params,
            modifier,
        } => {
            let m = match modifier {
                TraitBoundModifier::None => "",
                TraitBoundModifier::Maybe => "?",
                TraitBoundModifier::MaybeConst => "~const ",
            };
            format!("{}{m}{}", hrtb(generic_params), path(trait_))
        }
        GenericBound::Outlives(l) => l.clone(),
        GenericBound::Use(args) => format!(
            "use<{}>",
            join(args.iter().map(|a| match a {
                PreciseCapturingArg::Lifetime(l) | PreciseCapturingArg::Param(l) => l.clone(),
            }))
        ),
    }
}

fn hrtb(params: &[GenericParamDef]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!("for<{}> ", join(params.iter().map(param)))
    }
}

fn param(p: &GenericParamDef) -> String {
    match &p.kind {
        GenericParamDefKind::Lifetime { outlives } if outlives.is_empty() => p.name.clone(),
        GenericParamDefKind::Lifetime { outlives } => {
            format!("{}: {}", p.name, outlives.join(" + "))
        }
        GenericParamDefKind::Type {
            bounds: b, default, ..
        } => {
            let mut s = p.name.clone();
            if !b.is_empty() {
                s.push_str(": ");
                s.push_str(&bounds(b));
            }
            if let Some(d) = default {
                s.push_str(" = ");
                s.push_str(&ty(d));
            }
            s
        }
        GenericParamDefKind::Const { type_, default } => {
            let d = default
                .as_ref()
                .map(|d| format!(" = {d}"))
                .unwrap_or_default();
            format!("const {}: {}{d}", p.name, ty(type_))
        }
    }
}

/// Renders `<T: Bound, 'a>` (empty if there are no visible parameters). Synthetic parameters
/// (from argument-position `impl Trait`) are skipped.
pub(crate) fn generic_params(g: &Generics) -> String {
    let params: Vec<String> = g
        .params
        .iter()
        .filter(|p| {
            !matches!(
                p.kind,
                GenericParamDefKind::Type {
                    is_synthetic: true,
                    ..
                }
            )
        })
        .map(param)
        .collect();
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
    }
}

/// Renders a `where` clause on its own lines (empty if there is none).
pub(crate) fn where_clause(g: &Generics) -> String {
    if g.where_predicates.is_empty() {
        return String::new();
    }
    let preds: Vec<String> = g
        .where_predicates
        .iter()
        .map(|w| match w {
            WherePredicate::BoundPredicate {
                type_,
                bounds: b,
                generic_params,
            } => {
                format!("{}{}: {}", hrtb(generic_params), ty(type_), bounds(b))
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                format!("{lifetime}: {}", outlives.join(" + "))
            }
            WherePredicate::EqPredicate { lhs, rhs } => format!("{} == {}", ty(lhs), term(rhs)),
        })
        .map(|p| format!("    {p},"))
        .collect();
    format!("\nwhere\n{}", preds.join("\n"))
}

fn header(h: &FunctionHeader) -> String {
    let mut s = String::new();
    if h.is_const {
        s.push_str("const ");
    }
    if h.is_async {
        s.push_str("async ");
    }
    if h.is_unsafe {
        s.push_str("unsafe ");
    }
    s
}

fn inputs(sig: &FunctionSignature) -> String {
    join(sig.inputs.iter().map(|(name, t)| {
        if name == "self" {
            match t {
                Type::Generic(g) if g == "Self" => return "self".to_owned(),
                Type::BorrowedRef {
                    lifetime,
                    is_mutable,
                    type_,
                } if matches!(type_.as_ref(), Type::Generic(g) if g == "Self") => {
                    let l = lifetime
                        .as_ref()
                        .map(|l| format!("{l} "))
                        .unwrap_or_default();
                    let m = if *is_mutable { "mut " } else { "" };
                    return format!("&{l}{m}self");
                }
                _ => {}
            }
        }
        format!("{name}: {}", ty(t))
    }))
}

fn output(sig: &FunctionSignature) -> String {
    sig.output
        .as_ref()
        .map(|o| format!(" -> {}", ty(o)))
        .unwrap_or_default()
}

/// Renders a function signature (no body), e.g. `pub async fn get(&self, id: &str) -> Result<T>`.
pub(crate) fn fn_sig(vis: &str, name: &str, f: &Function) -> String {
    format!(
        "{vis}{}fn {name}{}({}){}{}",
        header(&f.header),
        generic_params(&f.generics),
        inputs(&f.sig),
        output(&f.sig),
        where_clause(&f.generics),
    )
}

fn fn_pointer(fp: &FunctionPointer) -> String {
    format!(
        "{}{}fn({}){}",
        hrtb(&fp.generic_params),
        header(&fp.header),
        join(fp.sig.inputs.iter().map(|(_, t)| ty(t))),
        output(&fp.sig),
    )
}

fn join(it: impl Iterator<Item = String>) -> String {
    it.collect::<Vec<_>>().join(", ")
}
