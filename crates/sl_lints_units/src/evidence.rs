//! What the crate says about the range of a name: assertions, early returns, clamps and doc comments.

use crate::names::{Tables, parse_int, storage_range};
use sl_analysis::syntax::line;
use std::collections::BTreeMap;
use syn::visit::Visit;
use syn::{BinOp, Expr, Lit, UnOp};

/// One end of a range and where the crate says it
#[derive(Clone)]
struct End {
    value: i128,
    at: String,
}

/// The ends the crate gives one name; several places widen it to the loosest
#[derive(Default, Clone)]
struct Ends {
    lo: Option<End>,
    hi: Option<End>,
}

/// A range for a name and the places that gave it
pub(crate) struct Found {
    pub range: (i128, i128),
    pub from: String,
}

/// The ranges of names across one crate
pub(crate) struct Evidence {
    ranges: BTreeMap<String, Ends>,
}

impl Evidence {
    /// Reads every file twice: the integer constants first, then the checks that use them
    pub fn new<'a>(files: impl Iterator<Item = (&'a str, &'a syn::File)> + Clone) -> Self {
        let mut consts = Consts(BTreeMap::new());
        for (_, ast) in files.clone() {
            consts.visit_file(ast);
        }
        let mut c = Checks {
            consts: &consts.0,
            file: "",
            ranges: BTreeMap::new(),
        };
        for (file, ast) in files {
            c.file = file;
            c.visit_file(ast);
        }
        Self { ranges: c.ranges }
    }

    /// The range of a value of this name in this storage, when the crate gives one narrower than the storage
    pub fn range(
        &self,
        name: &str,
        attrs: &[syn::Attribute],
        storage: &str,
        tables: &Tables,
    ) -> Option<Found> {
        let found = doc_range(attrs)
            .map(|range| Found {
                range,
                from: "its doc comment".to_owned(),
            })
            .or_else(|| self.code_range(name, storage))
            .or_else(|| {
                tables.range(name).map(|(w, range)| Found {
                    range,
                    from: format!("the word `{w}`"),
                })
            })?;
        match storage_range(storage) {
            Some(full) if found.range == full => None,
            Some(full) if found.range.0 < full.0 || found.range.1 > full.1 => None,
            _ => Some(found),
        }
    }

    fn code_range(&self, name: &str, storage: &str) -> Option<Found> {
        let e = self.ranges.get(name)?;
        let full = storage_range(storage);
        let lo =
            e.lo.as_ref()
                .map(|x| x.value)
                .or_else(|| full.map(|f| f.0))?;
        let hi =
            e.hi.as_ref()
                .map(|x| x.value)
                .or_else(|| full.map(|f| f.1))?;
        let mut at: Vec<&str> = [&e.lo, &e.hi]
            .into_iter()
            .flatten()
            .map(|x| x.at.as_str())
            .collect();
        at.dedup();
        (lo <= hi).then(|| Found {
            range: (lo, hi),
            from: at.join(" and "),
        })
    }
}

/// `name: TYPE = <integer>` constants and statics
struct Consts(BTreeMap<String, i128>);

impl<'ast> Visit<'ast> for Consts {
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        if let Some(v) = int_lit(&c.expr) {
            self.0.insert(c.ident.to_string(), v);
        }
    }

    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        if let Some(v) = int_lit(&c.expr) {
            self.0.insert(c.ident.to_string(), v);
        }
    }
}

fn int_lit(e: &Expr) -> Option<i128> {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => parse_int(i.base10_digits()),
            _ => None,
        },
        Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => int_lit(&u.expr).map(|v| -v),
        Expr::Paren(p) => int_lit(&p.expr),
        Expr::Group(g) => int_lit(&g.expr),
        _ => None,
    }
}

/// The name a check is about: `x`, `self.x`, `cfg.x`, `x as u32`, `*x`, `&x`
pub(crate) fn name_of(e: &Expr) -> Option<String> {
    match e {
        Expr::Path(p) if p.qself.is_none() => p.path.get_ident().map(ToString::to_string),
        Expr::Field(f) => match &f.member {
            syn::Member::Named(n) => Some(n.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        Expr::Cast(c) => name_of(&c.expr),
        Expr::Paren(p) => name_of(&p.expr),
        Expr::Group(g) => name_of(&g.expr),
        Expr::Reference(r) => name_of(&r.expr),
        Expr::Unary(u) if matches!(u.op, UnOp::Deref(_)) => name_of(&u.expr),
        _ => None,
    }
}

struct Checks<'c> {
    consts: &'c BTreeMap<String, i128>,
    file: &'c str,
    ranges: BTreeMap<String, Ends>,
}

/// Which end a fact gives
#[derive(Clone, Copy)]
enum Side {
    Lo,
    Hi,
}

impl Checks<'_> {
    fn value(&self, e: &Expr) -> Option<i128> {
        int_lit(e).or_else(|| match e {
            Expr::Path(p) => p
                .path
                .segments
                .last()
                .and_then(|s| self.consts.get(&s.ident.to_string()).copied()),
            Expr::Cast(c) => self.value(&c.expr),
            Expr::Paren(p) => self.value(&p.expr),
            _ => None,
        })
    }

    fn add(&mut self, name: String, side: Side, value: i128, what: &str, ln: u32) {
        let end = End {
            value,
            at: format!("`{what}` at {}:{ln}", self.file),
        };
        let e = self.ranges.entry(name).or_default();
        match side {
            Side::Lo if e.lo.as_ref().is_none_or(|x| value < x.value) => e.lo = Some(end),
            Side::Hi if e.hi.as_ref().is_none_or(|x| value > x.value) => e.hi = Some(end),
            Side::Lo | Side::Hi => {}
        }
    }

    /// The facts a condition gives when it holds (`holds`) or when it is false
    fn facts(&mut self, e: &Expr, holds: bool, what: &str, ln: u32) {
        match e {
            Expr::Paren(p) => self.facts(&p.expr, holds, what, ln),
            Expr::Unary(u) if matches!(u.op, UnOp::Not(_)) => self.facts(&u.expr, !holds, what, ln),
            Expr::Binary(b) => match (b.op, holds) {
                (BinOp::And(_), true) | (BinOp::Or(_), false) => {
                    self.facts(&b.left, holds, what, ln);
                    self.facts(&b.right, holds, what, ln);
                }
                _ => self.compare(&b.left, b.op, &b.right, holds, what, ln),
            },
            Expr::MethodCall(m) if holds && m.method == "contains" => {
                let (Some(arg), Some((lo, hi))) = (m.args.first(), self.range_expr(&m.receiver))
                else {
                    return;
                };
                if let Some(n) = name_of(arg) {
                    self.add(n.clone(), Side::Lo, lo, what, ln);
                    self.add(n, Side::Hi, hi, what, ln);
                }
            }
            Expr::Macro(m) if holds && m.mac.path.is_ident("matches") => {
                let Ok(args) = m.mac.parse_body_with(
                    syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
                ) else {
                    return;
                };
                let (Some(n), Some((lo, hi))) = (
                    args.first().and_then(name_of),
                    args.iter().nth(1).and_then(|r| self.range_expr(r)),
                ) else {
                    return;
                };
                self.add(n.clone(), Side::Lo, lo, what, ln);
                self.add(n, Side::Hi, hi, what, ln);
            }
            _ => {}
        }
    }

    fn range_expr(&self, e: &Expr) -> Option<(i128, i128)> {
        match e {
            Expr::Paren(p) => self.range_expr(&p.expr),
            Expr::Range(r) => {
                let lo = self.value(r.start.as_deref()?)?;
                let hi = self.value(r.end.as_deref()?)?;
                let hi = match r.limits {
                    syn::RangeLimits::Closed(_) => hi,
                    syn::RangeLimits::HalfOpen(_) => hi - 1,
                };
                Some((lo, hi))
            }
            _ => None,
        }
    }

    fn compare(&mut self, l: &Expr, op: BinOp, r: &Expr, holds: bool, what: &str, ln: u32) {
        // name OP value, or value OP name read as name OP' value
        let (name, value, op) = match (name_of(l), self.value(r), self.value(l), name_of(r)) {
            (Some(n), Some(v), _, _) => (n, v, op),
            (_, _, Some(v), Some(n)) => match flip(op) {
                Some(op) => (n, v, op),
                None => return,
            },
            _ => return,
        };
        let Some(op) = (if holds { Some(op) } else { negate(op) }) else {
            return;
        };
        let (side, v) = match op {
            BinOp::Le(_) => (Side::Hi, value),
            BinOp::Lt(_) => (Side::Hi, value - 1),
            BinOp::Ge(_) => (Side::Lo, value),
            BinOp::Gt(_) => (Side::Lo, value + 1),
            _ => return,
        };
        self.add(name, side, v, what, ln);
    }

    /// `target = value.clamp(a, b)`, `.min(b)` or `.max(a)`
    fn clamp(&mut self, target: &str, e: &Expr) {
        let Expr::MethodCall(m) = e else { return };
        let ln = line(m);
        let args: Vec<_> = m.args.iter().map(|a| self.value(a)).collect();
        let what = format!(".{}(..)", m.method);
        match (m.method.to_string().as_str(), args.as_slice()) {
            ("clamp", [Some(lo), Some(hi)]) => {
                self.add(target.to_owned(), Side::Lo, *lo, &what, ln);
                self.add(target.to_owned(), Side::Hi, *hi, &what, ln);
            }
            ("min", [Some(hi)]) => self.add(target.to_owned(), Side::Hi, *hi, &what, ln),
            ("max", [Some(lo)]) => self.add(target.to_owned(), Side::Lo, *lo, &what, ln),
            _ => {}
        }
    }
}

fn flip(op: BinOp) -> Option<BinOp> {
    Some(match op {
        BinOp::Le(_) => BinOp::Ge(Default::default()),
        BinOp::Lt(_) => BinOp::Gt(Default::default()),
        BinOp::Ge(_) => BinOp::Le(Default::default()),
        BinOp::Gt(_) => BinOp::Lt(Default::default()),
        _ => return None,
    })
}

fn negate(op: BinOp) -> Option<BinOp> {
    Some(match op {
        BinOp::Le(_) => BinOp::Gt(Default::default()),
        BinOp::Lt(_) => BinOp::Ge(Default::default()),
        BinOp::Ge(_) => BinOp::Lt(Default::default()),
        BinOp::Gt(_) => BinOp::Le(Default::default()),
        _ => return None,
    })
}

/// Whether a block leaves early: a `return` or an `Err(..)`
fn leaves(b: &syn::Block) -> bool {
    struct L(bool);
    impl<'ast> Visit<'ast> for L {
        fn visit_expr_return(&mut self, _: &'ast syn::ExprReturn) {
            self.0 = true;
        }
        fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
            if let Expr::Path(p) = &*c.func
                && p.path.is_ident("Err")
            {
                self.0 = true;
            }
            syn::visit::visit_expr_call(self, c);
        }
        fn visit_macro(&mut self, m: &'ast syn::Macro) {
            if m.path.is_ident("bail") || m.path.is_ident("panic") {
                self.0 = true;
            }
        }
    }
    let mut l = L(false);
    l.visit_block(b);
    l.0
}

const ASSERTS: &[&str] = &["assert", "debug_assert"];

impl<'ast> Visit<'ast> for Checks<'_> {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if ASSERTS.iter().any(|a| m.path.is_ident(a))
            && let Ok(args) = m.parse_body_with(
                syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
            )
            && let Some(cond) = args.first()
        {
            let what = format!(
                "{}!",
                m.path
                    .get_ident()
                    .map_or_else(String::new, ToString::to_string)
            );
            self.facts(cond, true, &what, line(m));
            self.visit_expr(cond);
        }
    }

    fn visit_expr_if(&mut self, i: &'ast syn::ExprIf) {
        if i.else_branch.is_none() && leaves(&i.then_branch) {
            self.facts(&i.cond, false, "if .. return", line(i));
        }
        syn::visit::visit_expr_if(self, i);
    }

    fn visit_local(&mut self, l: &'ast syn::Local) {
        if let (syn::Pat::Ident(p), Some(init)) = (&l.pat, &l.init) {
            self.clamp(&p.ident.to_string(), &init.expr);
        }
        syn::visit::visit_local(self, l);
    }

    fn visit_expr_assign(&mut self, a: &'ast syn::ExprAssign) {
        if let Some(n) = name_of(&a.left) {
            self.clamp(&n, &a.right);
        }
        syn::visit::visit_expr_assign(self, a);
    }

    fn visit_field_value(&mut self, f: &'ast syn::FieldValue) {
        if let syn::Member::Named(n) = &f.member {
            self.clamp(&n.to_string(), &f.expr);
        }
        syn::visit::visit_field_value(self, f);
    }
}

/// A range written in a doc comment: `0..=4095`, `0..100`, `0 to 100`, `-40–125`
pub(crate) fn doc_range(attrs: &[syn::Attribute]) -> Option<(i128, i128)> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                Expr::Lit(l) => match &l.lit {
                    Lit::Str(s) => Some(s.value()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .find_map(|text| text_range(&text))
}

/// The first `A <sep> B` in prose, where a separator is `..=`, `..`, `to`, an en or em dash
fn text_range(text: &str) -> Option<(i128, i128)> {
    let toks = tokenize(text);
    toks.windows(3).find_map(|w| {
        let lo = parse_int(&w[0])?;
        let hi = parse_int(&w[2])?;
        let hi = match w[1].as_str() {
            "..=" | "to" | "–" | "—" | "through" => hi,
            ".." => hi - 1,
            _ => return None,
        };
        (lo < hi).then_some((lo, hi))
    })
}

/// Numbers, separators and lowercase words of a doc text
fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if let Some(sep) = ["..=", "..", "–", "—"]
            .iter()
            .find(|s| rest.starts_with(**s))
        {
            out.push((*sep).to_owned());
            rest = &rest[sep.len()..];
        } else if c.is_ascii_digit()
            || (c == '-' && rest[1..].starts_with(|d: char| d.is_ascii_digit()))
        {
            let end = rest[1..]
                .find(|d: char| !(d.is_ascii_alphanumeric() || d == '_'))
                .map_or(rest.len(), |i| i + 1);
            out.push(rest[..end].to_owned());
            rest = &rest[end..];
        } else if c.is_alphabetic() {
            let end = rest
                .find(|d: char| !d.is_alphanumeric())
                .unwrap_or(rest.len());
            out.push(rest[..end].to_lowercase());
            rest = &rest[end..];
        } else {
            rest = &rest[c.len_utf8()..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::text_range;

    #[test]
    fn doc_ranges() {
        assert_eq!(text_range("Raw ADC code, 0..=4095."), Some((0, 4095)));
        assert_eq!(text_range("Duty from 0 to 100 percent"), Some((0, 100)));
        assert_eq!(text_range("Die temperature, -40–125 °C"), Some((-40, 125)));
        assert_eq!(text_range("Slot in `0..8`"), Some((0, 7)));
        assert_eq!(text_range("Version 1.2 of the protocol"), None);
        assert_eq!(text_range("Sent 3 to the host"), None);
    }
}
