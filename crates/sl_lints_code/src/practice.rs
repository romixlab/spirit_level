//! Practices from storage and kernel projects: sized integers in contract types (`sized-ints-in-contracts`), units
//! in the names of numeric quantities (`unit-suffix`), and reads of a whole stream without a bound
//! (`unbounded-read`).

use sl_analysis::Context;
use sl_analysis::syntax::{range, type_idents, without_tests};
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::{FileKind, Package};
use syn::visit::Visit;

const CONTRACTS: &str = "sized-ints-in-contracts";
const UNITS: &str = "unit-suffix";
const READS: &str = "unbounded-read";

const NUMERIC: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64",
];

pub(crate) fn run(cx: &Context<'_>, pkg: &Package) {
    let contract = cx
        .map()
        .and_then(|m| m.crates.get(&pkg.name))
        .is_some_and(|c| c.contract)
        || cx.params(CONTRACTS).list("also").contains(&pkg.name);
    let want_contract = contract && cx.enabled(CONTRACTS);
    if !(want_contract || cx.enabled(UNITS) || cx.enabled(READS)) {
        return;
    }
    let p = cx.project;
    let units = cx.params_for(UNITS, pkg);
    let reads = cx.params_for(READS, pkg);
    for f in p.sources(pkg) {
        if f.kind != FileKind::Src || p.is_test_file(pkg, f) {
            continue;
        }
        let Ok(ast) = f.ast() else { continue };
        let ast = without_tests(ast);
        let mut v = V {
            contract: want_contract,
            words: units.list("words"),
            units: units.list("units"),
            methods: reads.list("methods"),
            bounds: reads.list("bounds"),
            found: Vec::new(),
        };
        v.visit_file(&ast);
        for (lint, r, item, msg, help, conf) in v.found {
            cx.emit(
                Diagnostic::new(lint, format!("{}:{}:{item}", pkg.name, f.rel), msg)
                    .span(Span::range(f.rel.clone(), r.0, r.1))
                    .krate(&pkg.name)
                    .confidence(conf)
                    .help(help),
            );
        }
    }
}

type Found = (
    &'static str,
    sl_analysis::syntax::Range,
    String,
    String,
    &'static str,
    Confidence,
);

struct V<'p> {
    contract: bool,
    words: &'p [String],
    units: &'p [String],
    methods: &'p [String],
    bounds: &'p [String],
    found: Vec<Found>,
}

/// Whether the attributes derive serde's Serialize or Deserialize
fn derives_serde(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("derive")
            && a.meta.require_list().is_ok_and(|l| {
                let t = l.tokens.to_string();
                t.contains("Serialize") || t.contains("Deserialize")
            })
    })
}

/// The name's words that say a quantity, when none of its words names a unit
fn missing_unit(name: &str, words: &[String], units: &[String]) -> Option<String> {
    let low = name.to_ascii_lowercase();
    let parts: Vec<&str> = low.split('_').filter(|s| !s.is_empty()).collect();
    if parts.iter().any(|p| units.iter().any(|u| u == p)) {
        return None;
    }
    parts
        .iter()
        .find(|p| words.iter().any(|w| w == *p))
        .map(|p| (*p).to_owned())
}

fn is_numeric(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => {
            p.qself.is_none()
                && p.path
                    .get_ident()
                    .is_some_and(|i| NUMERIC.iter().any(|n| i == n))
        }
        syn::Type::Reference(r) => is_numeric(&r.elem),
        _ => false,
    }
}

impl V<'_> {
    fn unit(&mut self, name: &syn::Ident, ty: &syn::Type, what: &str) {
        if !is_numeric(ty) {
            return;
        }
        let n = name.to_string();
        if let Some(word) = missing_unit(&n, self.words, self.units) {
            self.found.push((
                UNITS,
                range(name),
                n.clone(),
                format!("{what} `{n}` is a {} quantity (`{word}`) with no unit in its name", quote(ty)),
                "put the unit last in the name: `timeout_ms`, `frame_size_bytes`, `latency_ms_max`; or use a typed unit (Duration)",
                Confidence::Estimate,
            ));
        }
    }

    fn fields(&mut self, fields: &syn::Fields, serde: bool) {
        for f in fields {
            if let Some(name) = &f.ident {
                self.unit(name, &f.ty, "field");
            }
            if serde && self.contract {
                let ids = type_idents(&f.ty);
                if let Some(t) = ids.iter().find(|i| *i == "usize" || *i == "isize") {
                    let name = f
                        .ident
                        .as_ref()
                        .map_or_else(|| "_".to_owned(), ToString::to_string);
                    self.found.push((
                        CONTRACTS,
                        range(&f.ty),
                        name.clone(),
                        format!("field `{name}` of a contract type is {t}, whose size depends on the platform"),
                        "use a sized integer (u32, u64, i64): every party of the contract reads the same width",
                        Confidence::Certain,
                    ));
                }
            }
        }
    }
}

fn quote(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .get_ident()
            .map_or_else(String::new, ToString::to_string),
        syn::Type::Reference(r) => quote(&r.elem),
        _ => String::new(),
    }
}

impl<'ast> Visit<'ast> for V<'_> {
    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        self.fields(&s.fields, derives_serde(&s.attrs));
    }

    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        let serde = derives_serde(&e.attrs);
        for v in &e.variants {
            self.fields(&v.fields, serde);
        }
    }

    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.unit(&c.ident, &c.ty, "constant");
        syn::visit::visit_item_const(self, c);
    }

    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        self.unit(&s.ident, &s.ty, "static");
        syn::visit::visit_item_static(self, s);
    }

    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        self.unit(&c.ident, &c.ty, "constant");
        syn::visit::visit_impl_item_const(self, c);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        if self.methods.contains(&name) && !bounded(&m.receiver, self.bounds) {
            self.found.push((
                READS,
                range(&m.method),
                name.clone(),
                format!("`{name}` reads until the end with no bound; a peer or a file from outside sets how much memory it takes"),
                "bound the read: `reader.take(MAX_BYTES).read_to_end(&mut buf)`, then refuse input that reached the limit",
                Confidence::Candidate,
            ));
        }
        syn::visit::visit_expr_method_call(self, m);
    }
}

/// Whether the receiver chain passes through a bounding call (`take`)
fn bounded(e: &syn::Expr, bounds: &[String]) -> bool {
    match e {
        syn::Expr::MethodCall(m) => {
            bounds.contains(&m.method.to_string()) || bounded(&m.receiver, bounds)
        }
        syn::Expr::Reference(r) => bounded(&r.expr, bounds),
        syn::Expr::Paren(p) => bounded(&p.expr, bounds),
        syn::Expr::Try(t) => bounded(&t.expr, bounds),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::missing_unit;

    #[test]
    fn units() {
        let w = ["timeout".to_owned(), "size".to_owned()];
        let u = ["ms".to_owned(), "bytes".to_owned()];
        assert_eq!(missing_unit("timeout", &w, &u).as_deref(), Some("timeout"));
        assert_eq!(missing_unit("READ_TIMEOUT_MS", &w, &u), None);
        assert_eq!(missing_unit("frame_size_bytes_max", &w, &u), None);
        assert_eq!(missing_unit("timeouts", &w, &u), None);
    }
}
