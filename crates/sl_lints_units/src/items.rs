//! Declarations: fields, constants, statics, function parameters and results.

use crate::evidence::Evidence;
use crate::names::{Tables, is_float, render, with_bounds, words};
use crate::{BOUNDS, CRATE, Found, PLAIN, QUANTITY};
use quote::ToTokens;
use sl_analysis::Context;
use sl_analysis::syntax::range;
use sl_core::Confidence;
use syn::visit::Visit;

const NUMERIC: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64",
];

/// `exact_units`' aliases written out, so bounds can be added to them
const ALIASES: &[(&str, &str)] = &[
    ("Nanos", "Fixed<Second, Nano, T>"),
    ("Micros", "Fixed<Second, Micro, T>"),
    ("Millis", "Fixed<Second, Milli, T>"),
    ("Secs", "Fixed<Second, One, T>"),
    ("Minutes", "Fixed<Minute, One, T>"),
    ("Hours", "Fixed<Hour, One, T>"),
    ("Days", "Fixed<Day, One, T>"),
    ("Hz", "Fixed<Hertz, One, T>"),
    ("Bytes", "Fixed<Byte, One, T>"),
    ("Kibibytes", "Fixed<Byte, Kibi, T>"),
    ("Mebibytes", "Fixed<Byte, Mebi, T>"),
    ("Gibibytes", "Fixed<Byte, Gibi, T>"),
    ("Percent", "Fixed<Ratio, Centi, T>"),
    ("Permille", "Fixed<Ratio, Milli, T>"),
    ("Ppm", "Fixed<Ratio, Micro, T>"),
    ("Ppb", "Fixed<Ratio, Nano, T>"),
];

/// A type as people write it: `Option<Fixed<Second, Milli, u32>>`
pub(crate) fn tidy(ty: &syn::Type) -> String {
    ty.to_token_stream()
        .to_string()
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace("& ", "&")
        .replace("[ ", "[")
        .replace(" ]", "]")
}

/// The numeric type inside a plain number, an `Option`, a `Vec`, an array, a slice or a reference
pub(crate) fn numeric(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => {
            let seg = p.path.segments.last()?;
            if p.path.segments.len() == 1 && NUMERIC.iter().any(|n| seg.ident == n) {
                return Some(seg.ident.to_string());
            }
            if seg.ident != "Option" && seg.ident != "Vec" {
                return None;
            }
            match &seg.arguments {
                syn::PathArguments::AngleBracketed(a) if a.args.len() == 1 => {
                    match a.args.first()? {
                        syn::GenericArgument::Type(t) => numeric(t),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        syn::Type::Array(a) => numeric(&a.elem),
        syn::Type::Slice(s) => numeric(&s.elem),
        syn::Type::Reference(r) => numeric(&r.elem),
        syn::Type::Paren(p) => numeric(&p.elem),
        _ => None,
    }
}

/// The text with its first whole-word `word` replaced
fn replace_first(text: &str, word: &str, with: &str) -> String {
    let b = text.as_bytes();
    let w = |j: usize| {
        b.get(j)
            .is_some_and(|x| x.is_ascii_alphanumeric() || *x == b'_')
    };
    let mut from = 0;
    while let Some(i) = text[from..].find(word).map(|i| i + from) {
        let end = i + word.len();
        if (i == 0 || !w(i - 1)) && !w(end) {
            return format!("{}{with}{}", &text[..i], &text[end..]);
        }
        from = end;
    }
    text.to_owned()
}

/// The name without the matched word (`timeout_ms` to `timeout`), when something is left
fn without_word(name: &str, word: &str) -> Option<String> {
    let screaming = name.chars().any(|c| c.is_ascii_uppercase());
    let rest: Vec<String> = words(name).into_iter().filter(|w| w != word).collect();
    (!rest.is_empty()).then(|| {
        let n = rest.join("_");
        if screaming { n.to_ascii_uppercase() } else { n }
    })
}

/// The type arguments of the last path segment
fn type_args(p: &syn::TypePath) -> Vec<&syn::Type> {
    match p.path.segments.last().map(|s| &s.arguments) {
        Some(syn::PathArguments::AngleBracketed(a)) => a
            .args
            .iter()
            .filter_map(|g| match g {
                syn::GenericArgument::Type(t) => Some(t),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) struct Items<'a> {
    cx: &'a Context<'a>,
    tables: &'a Tables,
    ev: &'a Evidence,
    has_dep: bool,
    /// The struct, enum, impl or function the walk is in
    owner: Vec<String>,
    pub found: Vec<Found>,
}

/// What is being declared
struct Decl<'d> {
    what: &'static str,
    name: &'d str,
    ty: &'d syn::Type,
    attrs: &'d [syn::Attribute],
}

impl<'a> Items<'a> {
    pub fn new(cx: &'a Context<'a>, tables: &'a Tables, ev: &'a Evidence, has_dep: bool) -> Self {
        Self {
            cx,
            tables,
            ev,
            has_dep,
            owner: Vec::new(),
            found: Vec::new(),
        }
    }

    fn item(&self, name: &str) -> String {
        let mut path = self.owner.join("::");
        if !path.is_empty() {
            path.push_str("::");
        }
        path.push_str(name);
        path
    }

    fn dep_note(&self) -> &'static str {
        if self.has_dep {
            ""
        } else {
            "; the crate needs `exact_units` as a dependency, with the unit group's feature"
        }
    }

    fn decl(&mut self, d: &Decl<'_>) {
        if let Some(storage) = numeric(d.ty) {
            self.plain_number(d, &storage);
        } else if let syn::Type::Path(p) = d.ty {
            self.typed(d, p);
        }
    }

    /// A plain number: a unit word ends the name, else a word of a position or count, else a quantity word
    fn plain_number(&mut self, d: &Decl<'_>, storage: &str) {
        let t = self.tables;
        let picked = if let Some(m) = t.unit(d.name) {
            (QUANTITY, m, "unit")
        } else if let Some(m) = t.plain(d.name) {
            (PLAIN, m, "word")
        } else if t.untyped(d.name) {
            return;
        } else if let Some(m) = t.quantity(d.name) {
            (QUANTITY, m, "word")
        } else {
            return;
        };
        let (lint, (word, template), kind) = picked;
        if !self.cx.enabled(lint) {
            return;
        }
        let confidence = Confidence::Estimate;
        let mut ty = render(template, storage);
        let mut bounds = String::new();
        if let Some(r) = self.ev.range(d.name, d.attrs, storage, t) {
            ty = with_bounds(&ty, r.range, is_float(storage));
            bounds = format!(", bounded to {}..={} as {}", r.range.0, r.range.1, r.from);
        }
        let whole = replace_first(&tidy(d.ty), storage, &ty);
        let rename = without_word(d.name, &word)
            .filter(|_| kind == "unit")
            .map(|n| format!("; the unit then lives in the type, so the name can drop it: `{n}`"))
            .unwrap_or_default();
        let holes = holes(&ty);
        self.found.push(Found {
            lint,
            range: range(d.ty),
            item: self.item(d.name),
            message: format!(
                "{} `{}` holds {} in a plain {storage} (the {kind} `{word}`)",
                d.what,
                d.name,
                if lint == PLAIN {
                    "a position or a count"
                } else {
                    "a quantity"
                }
            ),
            help: format!(
                "use `{whole}` from {CRATE}{bounds}{holes}{rename}{}",
                self.dep_note()
            ),
            confidence,
        });
    }

    /// A value of an `exact_units` type that could carry bounds; or a `Duration`, which the strict lint flags
    fn typed(&mut self, d: &Decl<'_>, p: &syn::TypePath) {
        let Some(seg) = p.path.segments.last() else {
            return;
        };
        let head = seg.ident.to_string();
        let args = type_args(p);
        if self.tables.durations.contains(&head) {
            if self.cx.enabled(PLAIN) {
                self.found.push(Found {
                    lint: PLAIN,
                    range: range(d.ty),
                    item: self.item(d.name),
                    message: format!("{} `{}` is a `{head}`, 12 or 16 bytes for a time span", d.what, d.name),
                    help: format!(
                        "use `Fixed<Second, Milli, u32>` (`Millis<u32>`, 4 bytes, 49 days) or the scale and storage the span needs, from {CRATE}{}",
                        self.dep_note()
                    ),
                    confidence: Confidence::Estimate,
                });
            }
            return;
        }
        if !self.cx.enabled(BOUNDS) {
            return;
        }
        let storage_of = |i: usize| args.get(i).and_then(|t| numeric(t));
        let (storage, full) = match head.as_str() {
            "Fixed" if args.len() == 3 => (storage_of(2), tidy(d.ty)),
            "Index" | "IndexFrom1" if args.len() == 2 => (storage_of(1), tidy(d.ty)),
            "Count" if args.len() == 2 => (
                storage_of(1),
                format!("Fixed<{}, One, {}>", tidy(args[0]), tidy(args[1])),
            ),
            h if args.len() == 1 && self.tables.unbounded.iter().any(|u| u == h) => {
                let s = storage_of(0);
                let tpl = ALIASES
                    .iter()
                    .find(|(a, _)| *a == h)
                    .map_or_else(|| format!("{h}<T>"), |(_, t)| (*t).to_owned());
                (s.clone(), render(&tpl, s.as_deref().unwrap_or("T")))
            }
            _ => return,
        };
        let Some(storage) = storage else { return };
        let Some(r) = self.ev.range(d.name, d.attrs, &storage, self.tables) else {
            return;
        };
        let bounded = with_bounds(&full, r.range, is_float(&storage));
        self.found.push(Found {
            lint: BOUNDS,
            range: range(d.ty),
            item: self.item(d.name),
            message: format!(
                "{} `{}` is `{}` without bounds, but stays in {}..={} by {}",
                d.what,
                d.name,
                tidy(d.ty),
                r.range.0,
                r.range.1,
                r.from
            ),
            help: format!(
                "use `{bounded}`: the type then refuses a value out of range where it is made, and the range is in the API"
            ),
            confidence: Confidence::Estimate,
        });
    }
}

/// What the suggested type leaves to choose
fn holes(ty: &str) -> String {
    let mut h = Vec::new();
    let has = |w: &str| replace_first(ty, w, "\u{0}") != ty;
    if has("P") {
        h.push("P is the scale: One, Milli, Micro or a Rational");
    }
    if has("Of") {
        h.push("Of is what is counted, a unit of the crate made with `counted!`");
    }
    if has("TICK_HZ") {
        h.push("TICK_HZ is the timer's rate");
    }
    if h.is_empty() {
        String::new()
    } else {
        format!("; {}", h.join("; "))
    }
}

fn is_ffi(attrs: &[syn::Attribute], abi: Option<&syn::Abi>) -> bool {
    abi.is_some()
        || attrs
            .iter()
            .any(|a| a.path().is_ident("no_mangle") || a.path().is_ident("unsafe"))
}

impl<'ast> Visit<'ast> for Items<'_> {
    fn visit_item_struct(&mut self, s: &'ast syn::ItemStruct) {
        self.owner.push(s.ident.to_string());
        for (i, f) in s.fields.iter().enumerate() {
            let name = f
                .ident
                .as_ref()
                .map_or_else(|| i.to_string(), ToString::to_string);
            self.decl(&Decl {
                what: "field",
                name: &name,
                ty: &f.ty,
                attrs: &f.attrs,
            });
        }
        syn::visit::visit_item_struct(self, s);
        self.owner.pop();
    }

    fn visit_item_enum(&mut self, e: &'ast syn::ItemEnum) {
        for v in &e.variants {
            self.owner.push(format!("{}::{}", e.ident, v.ident));
            for f in v.fields.iter().filter(|f| f.ident.is_some()) {
                let name = f
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                self.decl(&Decl {
                    what: "field",
                    name: &name,
                    ty: &f.ty,
                    attrs: &f.attrs,
                });
            }
            self.owner.pop();
        }
        syn::visit::visit_item_enum(self, e);
    }

    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        let name = c.ident.to_string();
        self.decl(&Decl {
            what: "constant",
            name: &name,
            ty: &c.ty,
            attrs: &c.attrs,
        });
        syn::visit::visit_item_const(self, c);
    }

    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        let name = s.ident.to_string();
        self.decl(&Decl {
            what: "static",
            name: &name,
            ty: &s.ty,
            attrs: &s.attrs,
        });
        syn::visit::visit_item_static(self, s);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        // a trait's signatures are the trait's to change
        if i.trait_.is_some() {
            return;
        }
        self.owner.push(sl_analysis::syntax::type_name(&i.self_ty));
        syn::visit::visit_item_impl(self, i);
        self.owner.pop();
    }

    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        let name = c.ident.to_string();
        self.decl(&Decl {
            what: "constant",
            name: &name,
            ty: &c.ty,
            attrs: &c.attrs,
        });
        syn::visit::visit_impl_item_const(self, c);
    }

    fn visit_item_foreign_mod(&mut self, _: &'ast syn::ItemForeignMod) {}

    fn visit_signature(&mut self, s: &'ast syn::Signature) {
        if s.abi.is_some() {
            return;
        }
        let fname = s.ident.to_string();
        self.owner.push(fname.clone());
        for a in &s.inputs {
            if let syn::FnArg::Typed(t) = a
                && let syn::Pat::Ident(p) = &*t.pat
            {
                let name = p.ident.to_string();
                self.decl(&Decl {
                    what: "parameter",
                    name: &name,
                    ty: &t.ty,
                    attrs: &t.attrs,
                });
            }
        }
        self.owner.pop();
        if let syn::ReturnType::Type(_, ty) = &s.output {
            self.decl(&Decl {
                what: "result of",
                name: &fname,
                ty,
                attrs: &[],
            });
        }
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        if is_ffi(&f.attrs, f.sig.abi.as_ref()) {
            return;
        }
        syn::visit::visit_item_fn(self, f);
    }
}
