//! Paths as written, resolved through the file's `use` items and matched against the patterns of the parameters,
//! and the literals of an expression.

use sl_core::glob;
use std::collections::BTreeMap;
use syn::visit::Visit;
use syn::{Expr, Lit, UseTree};

/// The `use` items of a file: each name brought into scope and the path it stands for
#[derive(Default)]
pub(crate) struct Uses(BTreeMap<String, Vec<String>>);

impl Uses {
    /// The `use` items anywhere in `file`
    pub(crate) fn of(file: &syn::File) -> Self {
        struct V(Uses);
        impl<'ast> Visit<'ast> for V {
            fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
                self.0.add(&mut Vec::new(), &u.tree);
            }
        }
        let mut v = V(Self::default());
        v.visit_file(file);
        v.0
    }

    fn add(&mut self, prefix: &mut Vec<String>, tree: &UseTree) {
        match tree {
            UseTree::Path(p) => {
                prefix.push(p.ident.to_string());
                self.add(prefix, &p.tree);
                prefix.pop();
            }
            UseTree::Name(n) if n.ident == "self" => {
                if let Some(last) = prefix.last() {
                    self.0.insert(last.clone(), prefix.clone());
                }
            }
            UseTree::Name(n) => {
                let mut full = prefix.clone();
                full.push(n.ident.to_string());
                self.0.insert(n.ident.to_string(), full);
            }
            UseTree::Rename(r) => {
                let mut full = prefix.clone();
                full.push(r.ident.to_string());
                self.0.insert(r.rename.to_string(), full);
            }
            UseTree::Group(g) => {
                for t in &g.items {
                    self.add(prefix, t);
                }
            }
            UseTree::Glob(_) => {}
        }
    }

    /// The segments of `path`, its first segment replaced by the path a `use` gives it
    pub(crate) fn resolve(&self, path: &syn::Path) -> Vec<String> {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        match segs.first().and_then(|f| self.0.get(f)) {
            Some(full) => full.iter().chain(&segs[1..]).cloned().collect(),
            None => segs,
        }
    }
}

/// Whether a path matches a pattern (`std::thread::spawn`, `Color32::from_*`): the shorter is a suffix of the
/// longer, segment by segment, and has two segments at least, or they have the same length; `*` matches within a
/// segment
pub(crate) fn path_matches(pattern: &str, segs: &[String]) -> bool {
    let pat: Vec<&str> = pattern.split("::").collect();
    let n = pat.len().min(segs.len());
    if n < 2 && pat.len() != segs.len() {
        return false;
    }
    pat[pat.len() - n..]
        .iter()
        .zip(&segs[segs.len() - n..])
        .all(|(p, s)| glob::matches(p, s))
}

/// The patterns of a list that name functions (`Type::name`) matching the path
pub(crate) fn path_in(list: &[String], segs: &[String]) -> bool {
    list.iter()
        .any(|p| p.contains("::") && path_matches(p, segs))
}

/// Whether a list names the method `name` (a pattern without `::`)
pub(crate) fn method_in(list: &[String], name: &str) -> bool {
    list.iter()
        .any(|p| !p.contains("::") && glob::matches(p, name))
}

/// The value of a number literal, also negated, cast or in parentheses
pub(crate) fn number(e: &Expr) -> Option<f64> {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => i.base10_digits().parse().ok(),
            Lit::Float(f) => f.base10_digits().parse().ok(),
            _ => None,
        },
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => number(&u.expr).map(|v| -v),
        Expr::Cast(c) => number(&c.expr),
        Expr::Paren(p) => number(&p.expr),
        Expr::Group(g) => number(&g.expr),
        _ => None,
    }
}

/// Whether an expression is a literal: a number, a string, a character or a bool
pub(crate) fn is_literal(e: &Expr) -> bool {
    match e {
        Expr::Lit(_) => true,
        Expr::Reference(r) => is_literal(&r.expr),
        _ => number(e).is_some(),
    }
}

/// The number literals an argument holds as a size: itself, the items of an array or tuple, the arguments of a
/// vector constructor (`vec2(8.0, 4.0)`, `Vec2::new`)
pub(crate) fn size_numbers(e: &Expr, out: &mut Vec<(f64, sl_analysis::syntax::Range)>) {
    if let Some(v) = number(e) {
        out.push((v, sl_analysis::syntax::range(e)));
        return;
    }
    match e {
        Expr::Array(a) => a.elems.iter().for_each(|x| size_numbers(x, out)),
        Expr::Tuple(t) => t.elems.iter().for_each(|x| size_numbers(x, out)),
        Expr::Reference(r) => size_numbers(&r.expr, out),
        Expr::Call(c) => {
            if let Expr::Path(p) = &*c.func {
                let segs: Vec<String> = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let last = segs.last().map(String::as_str);
                let ty = segs.len().checked_sub(2).map(|i| segs[i].as_str());
                if matches!(last, Some("vec2" | "pos2")) || matches!(ty, Some("Vec2" | "Pos2")) {
                    c.args.iter().for_each(|x| size_numbers(x, out));
                }
            }
        }
        _ => {}
    }
}

/// The arguments of a macro call, when they parse as expressions separated by commas (`format!`, `vec!`)
pub(crate) fn macro_args(m: &syn::Macro) -> Vec<Expr> {
    type Args = syn::punctuated::Punctuated<Expr, syn::Token![,]>;
    m.parse_body_with(Args::parse_terminated)
        .map(|a| a.into_iter().collect())
        .unwrap_or_default()
}

/// The string literals of the text an argument shows: a literal, the format string of a `format!`, the literals of
/// a builder chain (`RichText::new("x").strong()`)
pub(crate) fn texts(e: &Expr, out: &mut Vec<syn::LitStr>) {
    match e {
        Expr::Lit(l) => {
            if let Lit::Str(s) = &l.lit {
                out.push(s.clone());
            }
        }
        Expr::Macro(m) => {
            let name = m.mac.path.segments.last().map(|s| s.ident.to_string());
            if matches!(name.as_deref(), Some("format" | "format_args"))
                && let Some(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                })) = macro_args(&m.mac).first()
            {
                out.push(s.clone());
            }
        }
        Expr::Call(c) => c.args.iter().for_each(|x| texts(x, out)),
        Expr::MethodCall(m) => {
            texts(&m.receiver, out);
            m.args.iter().for_each(|x| texts(x, out));
        }
        Expr::Reference(r) => texts(&r.expr, out),
        Expr::Paren(p) => texts(&p.expr, out),
        Expr::Group(g) => texts(&g.expr, out),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segs(s: &str) -> Vec<String> {
        s.split("::").map(str::to_owned).collect()
    }

    #[test]
    fn suffixes_match_from_two_segments() {
        assert!(path_matches("std::thread::spawn", &segs("thread::spawn")));
        assert!(path_matches(
            "std::thread::spawn",
            &segs("std::thread::spawn")
        ));
        assert!(path_matches(
            "Color32::from_*",
            &segs("egui::Color32::from_rgb")
        ));
        assert!(!path_matches("tokio::spawn", &segs("spawn")));
        assert!(!path_matches(
            "std::process::Command::new",
            &segs("tokio::process::Command::new")
        ));
    }

    #[test]
    fn uses_resolve_the_first_segment() {
        let f: syn::File =
            syn::parse_str("use std::thread::{self, spawn as go}; use std::process::Command;")
                .unwrap();
        let u = Uses::of(&f);
        let p: syn::Path = syn::parse_str("go").unwrap();
        assert_eq!(u.resolve(&p), segs("std::thread::spawn"));
        let p: syn::Path = syn::parse_str("thread::sleep").unwrap();
        assert_eq!(u.resolve(&p), segs("std::thread::sleep"));
        let p: syn::Path = syn::parse_str("Command::new").unwrap();
        assert_eq!(u.resolve(&p), segs("std::process::Command::new"));
    }
}
