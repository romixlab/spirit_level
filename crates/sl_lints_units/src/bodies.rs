//! Function bodies: unit-named locals, literals passed to unit-named parameters, conversions by hand.

use crate::evidence::name_of;
use crate::items::{numeric, tidy};
use crate::names::{Tables, render};
use crate::{BODIES, CRATE, Found};
use sl_analysis::syntax::range;
use sl_core::{Confidence, Params};
use std::collections::BTreeMap;
use syn::visit::Visit;
use syn::{BinOp, Expr, Lit};

/// A parameter's unit word and the type that holds it
type Param = Option<(String, String)>;

/// The crate's functions by name, with each parameter's unit; a name declared twice with other units is dropped
pub(crate) fn signatures<'a>(
    files: impl Iterator<Item = &'a syn::File>,
    tables: &Tables,
) -> BTreeMap<String, Vec<Param>> {
    struct S<'t> {
        tables: &'t Tables,
        fns: BTreeMap<String, Option<Vec<Param>>>,
    }
    impl<'ast> Visit<'ast> for S<'_> {
        fn visit_signature(&mut self, s: &'ast syn::Signature) {
            let params: Vec<Param> = s
                .inputs
                .iter()
                .filter_map(|a| match a {
                    syn::FnArg::Typed(t) => Some(t),
                    syn::FnArg::Receiver(_) => None,
                })
                .map(|t| {
                    let syn::Pat::Ident(p) = &*t.pat else {
                        return None;
                    };
                    let storage = numeric(&t.ty)?;
                    let (w, tpl) = self.tables.unit(&p.ident.to_string())?;
                    Some((w, render(tpl, &storage)))
                })
                .collect();
            let e = self
                .fns
                .entry(s.ident.to_string())
                .or_insert_with(|| Some(params.clone()));
            if e.as_ref().is_some_and(|p| *p != params) {
                *e = None;
            }
        }
    }
    let mut s = S {
        tables,
        fns: BTreeMap::new(),
    };
    for f in files {
        s.visit_file(f);
    }
    s.fns
        .into_iter()
        .filter_map(|(k, v)| v.filter(|p| p.iter().any(Option::is_some)).map(|v| (k, v)))
        .collect()
}

pub(crate) struct Bodies<'a> {
    tables: &'a Tables,
    fns: &'a BTreeMap<String, Vec<Param>>,
    factors: Vec<String>,
    escapes: &'a [String],
    ignore: Vec<String>,
    /// Whether the crate uses `exact_units`, so a bare `.get()` is likely its way out of a typed value
    has_dep: bool,
    owner: Vec<String>,
    pub found: Vec<Found>,
}

fn literal(e: &Expr) -> Option<String> {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => Some(i.base10_digits().to_owned()),
            Lit::Float(f) => Some(f.base10_digits().to_owned()),
            _ => None,
        },
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => {
            literal(&u.expr).map(|v| format!("-{v}"))
        }
        Expr::Paren(p) => literal(&p.expr),
        _ => None,
    }
}

/// The literal's type when it says one (`500u32`), else the default integer or float
fn literal_storage(e: &Expr) -> &'static str {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Float(_) => "f64",
            _ => "i32",
        },
        Expr::Unary(u) => literal_storage(&u.expr),
        Expr::Paren(p) => literal_storage(&p.expr),
        _ => "i32",
    }
}

impl<'a> Bodies<'a> {
    pub fn new(
        tables: &'a Tables,
        fns: &'a BTreeMap<String, Vec<Param>>,
        params: &'a Params,
        has_dep: bool,
    ) -> Self {
        let plain = |k: &str| params.list(k).iter().map(|f| f.replace('_', "")).collect();
        Self {
            tables,
            fns,
            factors: plain("factors"),
            escapes: params.list("escapes"),
            ignore: plain("ignore-literals"),
            has_dep,
            owner: Vec::new(),
            found: Vec::new(),
        }
    }

    fn item(&self, what: &str) -> String {
        let mut s = self.owner.join("::");
        if !s.is_empty() {
            s.push_str("::");
        }
        s.push_str(what);
        s
    }

    /// The unit word of a value's name: `x_ms`, `self.x_ms`, `x_ms as u64`
    fn unit_of(&self, e: &Expr) -> Option<(String, String)> {
        let name = name_of(e)?;
        self.tables.unit(&name).map(|(w, _)| (name, w))
    }

    /// `x_ms * 1000`, `x_mv * 0.001`, `t.get() + 3`
    fn arithmetic(&mut self, b: &syn::ExprBinary, op: &str) {
        for (v, f) in [(&*b.left, &*b.right), (&*b.right, &*b.left)] {
            let lit = literal(f).map(|l| l.replace('_', ""));
            if let (Some(lit), Some((name, word))) = (&lit, self.unit_of(v)) {
                let (item, at) = (self.item(&format!("{name}{op}{lit}")), range(b));
                if self.factors.contains(lit) && (op == "×" || op == "÷") {
                    self.push(at, item,
                        format!("`{name}` (in `{word}`) is converted by hand, {op} {lit}"),
                        "hold it as a Fixed and convert with `.to::<Prefix, Storage>()`: exact or refused, the factor worked out by the compiler".to_owned());
                    return;
                }
                // a cast to a float is reported as the cast
                if matches!(f, Expr::Lit(l) if matches!(l.lit, Lit::Float(_)))
                    && !matches!(v, Expr::Cast(_))
                {
                    self.push(at, item,
                        format!("`{name}` (in `{word}`) goes into float maths with `{lit}`"),
                        "keep the quantity a Fixed: scales and products stay exact; a float only through a named call (`to_si_f64`)".to_owned());
                    return;
                }
            }
            if self.has_dep
                && let Expr::MethodCall(m) = v
                && m.args.is_empty()
                && self.escapes.iter().any(|e| m.method == e)
            {
                let method = m.method.to_string();
                self.push(range(b), self.item(&format!(".{method}(){op}")),
                    format!("arithmetic {op} on `.{method}()`, the plain number out of a typed value"),
                    "do the arithmetic on the typed values: the result keeps its unit and scale, and a product or quotient its dimension".to_owned());
                return;
            }
        }
    }

    /// `temp_c > 85`
    fn comparison(&mut self, b: &syn::ExprBinary) {
        for (v, f) in [(&*b.left, &*b.right), (&*b.right, &*b.left)] {
            let (Some(lit), Some((name, word))) = (literal(f), self.unit_of(v)) else {
                continue;
            };
            if self.ignore.contains(&lit.replace('_', "")) {
                return;
            }
            self.push(range(b), self.item(&format!("{name}~{lit}")),
                format!("`{name}` (in `{word}`) is compared with the bare literal `{lit}`"),
                "name the limit as a typed constant (`const OVERHEAT: Point<Celsius, ..> = ..`): the literal then carries its unit and scale".to_owned());
            return;
        }
    }

    /// `MV_PER_LSB: u32 = 805`: a scale between two units kept as a plain number
    fn scale(&mut self, ident: &syn::Ident, ty: &syn::Type) {
        let name = ident.to_string();
        let ws = crate::names::words(&name);
        let Some(i) = ws.iter().position(|w| w == "per") else {
            return;
        };
        let (Some(storage), true) = (numeric(ty), i > 0 && i + 1 < ws.len()) else {
            return;
        };
        let unit = |w: &[String]| w.iter().any(|x| self.tables.unit(x).is_some());
        if !(unit(&ws[..i]) || unit(&ws[i + 1..])) {
            return;
        }
        self.push(range(ident), self.item(&name),
            format!("constant `{name}` is a scale between units in a plain {storage}"),
            "make the scale part of a type: a reading in `Fixed<Volt, Rational<805, 1000>, u16>` is in mV by its type, and its conversions are exact".to_owned());
    }

    fn push(
        &mut self,
        at: sl_analysis::syntax::Range,
        item: String,
        message: String,
        help: String,
    ) {
        self.found.push(Found {
            lint: BODIES,
            range: at,
            item,
            message,
            help,
            confidence: Confidence::Estimate,
        });
    }

    fn args(&mut self, callee: &str, args: &syn::punctuated::Punctuated<Expr, syn::Token![,]>) {
        let Some(params) = self.fns.get(callee) else {
            return;
        };
        for (i, (arg, param)) in args.iter().zip(params).enumerate() {
            let (Some(lit), Some((word, ty))) = (literal(arg), param) else {
                continue;
            };
            let (item, at) = (self.item(&format!("{callee}#{i}")), range(arg));
            self.push(
                at,
                item,
                format!("literal `{lit}` is passed to `{callee}` as a plain number in `{word}`"),
                format!("make the parameter `{ty}` and pass a typed value, so the unit is checked at the call"),
            );
        }
    }
}

impl<'ast> Visit<'ast> for Bodies<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.owner.push(f.sig.ident.to_string());
        syn::visit::visit_item_fn(self, f);
        self.owner.pop();
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.owner.push(f.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, f);
        self.owner.pop();
    }

    fn visit_local(&mut self, l: &'ast syn::Local) {
        let (pat, ty) = match &l.pat {
            syn::Pat::Type(t) => (&*t.pat, Some(&*t.ty)),
            p => (p, None),
        };
        if let syn::Pat::Ident(p) = pat
            && !self.owner.is_empty()
        {
            let name = p.ident.to_string();
            let init = l.init.as_ref().map(|i| &*i.expr);
            let storage = match (ty, init) {
                (Some(t), _) => numeric(t),
                (None, Some(e)) => literal(e).map(|_| literal_storage(e).to_owned()),
                (None, None) => None,
            };
            if let (Some(storage), Some((word, tpl))) = (storage, self.tables.unit(&name)) {
                let ty = render(tpl, &storage);
                let shown = ty_or(ty.clone(), l);
                self.push(
                    range(&p.ident),
                    self.item(&name),
                    format!(
                        "local `{name}` holds a quantity in a plain {storage} (the unit `{word}`)"
                    ),
                    format!("make it `{shown}` from {CRATE}, so what it meets is checked"),
                );
            }
        }
        syn::visit::visit_local(self, l);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if let Expr::Path(p) = &*c.func
            && let Some(seg) = p.path.segments.last()
        {
            self.args(&seg.ident.to_string(), &c.args);
        }
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        self.args(&m.method.to_string(), &m.args);
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_binary(&mut self, b: &'ast syn::ExprBinary) {
        if !self.owner.is_empty() {
            match b.op {
                BinOp::Mul(_) | BinOp::MulAssign(_) => self.arithmetic(b, "×"),
                BinOp::Div(_) | BinOp::DivAssign(_) => self.arithmetic(b, "÷"),
                BinOp::Add(_) | BinOp::AddAssign(_) => self.arithmetic(b, "+"),
                BinOp::Sub(_) | BinOp::SubAssign(_) => self.arithmetic(b, "−"),
                BinOp::Rem(_) | BinOp::RemAssign(_) => self.arithmetic(b, "%"),
                BinOp::Lt(_)
                | BinOp::Le(_)
                | BinOp::Gt(_)
                | BinOp::Ge(_)
                | BinOp::Eq(_)
                | BinOp::Ne(_) => {
                    self.comparison(b);
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_binary(self, b);
    }

    fn visit_expr_cast(&mut self, c: &'ast syn::ExprCast) {
        if !self.owner.is_empty()
            && let Some((name, word)) = self.unit_of(&c.expr)
            && let Some(to) = numeric(&c.ty)
        {
            let (message, help) = if crate::names::is_float(&to) {
                (
                    format!("`{name}` (in `{word}`) becomes a float with `as {to}`"),
                    "keep the quantity a Fixed: scales and products stay exact; a float only through a named call (`to_si_f64`)",
                )
            } else {
                (
                    format!(
                        "`{name}` (in `{word}`) is cast with `as {to}`, which truncates or wraps without a word"
                    ),
                    "convert the Fixed with `.to::<Prefix, Storage>()` or `.convert()`: exact or `None`, never cut",
                )
            };
            self.push(
                range(c),
                self.item(&format!("{name} as {to}")),
                message,
                help.to_owned(),
            );
        }
        syn::visit::visit_expr_cast(self, c);
    }

    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.scale(&c.ident, &c.ty);
        syn::visit::visit_item_const(self, c);
    }

    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        self.scale(&c.ident, &c.ty);
        syn::visit::visit_impl_item_const(self, c);
    }
}

/// The suggested type, or the annotated one rewritten, for a `let`
fn ty_or(suggested: String, l: &syn::Local) -> String {
    match &l.pat {
        syn::Pat::Type(t) => {
            let shown = tidy(&t.ty);
            numeric(&t.ty).map_or(shown.clone(), |s| shown.replacen(&s, &suggested, 1))
        }
        _ => suggested,
    }
}
