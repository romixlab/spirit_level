//! Assertions that say what broke: one condition per assertion (`compound-assert`), a message on each
//! (`assert-message`), and float ordering that cannot panic on NaN (`float-partial-cmp-unwrap`).

use sl_analysis::Context;
use sl_analysis::syntax::{Range, range, without_tests};
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::{FileKind, Package};
use syn::punctuated::Punctuated;
use syn::visit::Visit;

const COMPOUND: &str = "compound-assert";
const MESSAGE: &str = "assert-message";
const PARTIAL: &str = "float-partial-cmp-unwrap";

pub(crate) fn run(cx: &Context<'_>, pkg: &Package) {
    if !(cx.enabled(COMPOUND) || cx.enabled(MESSAGE) || cx.enabled(PARTIAL)) {
        return;
    }
    let p = cx.project;
    for f in p.sources(pkg) {
        if f.kind != FileKind::Src || p.is_test_file(pkg, f) {
            continue;
        }
        let Ok(ast) = f.ast() else { continue };
        let ast = without_tests(ast);
        let mut v = V {
            cx,
            asserts: cx.params_for(COMPOUND, pkg).list("macros").to_vec(),
            messages: cx.params_for(MESSAGE, pkg).list("macros").to_vec(),
            methods: cx.params_for(PARTIAL, pkg).list("methods").to_vec(),
            in_const: 0,
            found: Vec::new(),
        };
        v.visit_file(&ast);
        for (lint, r, msg, help, conf) in v.found {
            let line = r.0.0;
            cx.emit(
                Diagnostic::new(lint, format!("{}:{}:{line}", pkg.name, f.rel), msg)
                    .span(Span::range(f.rel.clone(), r.0, r.1))
                    .krate(&pkg.name)
                    .confidence(conf)
                    .help(help),
            );
        }
    }
}

/// (lint, range, message, help, confidence)
type Found = Vec<(&'static str, Range, String, &'static str, Confidence)>;

struct V<'a> {
    cx: &'a Context<'a>,
    asserts: Vec<String>,
    messages: Vec<String>,
    methods: Vec<String>,
    /// Inside a `const` item or block: a failed assertion there is a compile error that shows the condition
    in_const: u32,
    found: Found,
}

impl<'ast> Visit<'ast> for V<'_> {
    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        self.in_const += 1;
        syn::visit::visit_item_const(self, i);
        self.in_const -= 1;
    }

    fn visit_expr_const(&mut self, e: &'ast syn::ExprConst) {
        self.in_const += 1;
        syn::visit::visit_expr_const(self, e);
        self.in_const -= 1;
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let compound = self.cx.enabled(COMPOUND) && self.asserts.contains(&name);
        let message = self.cx.enabled(MESSAGE) && self.messages.contains(&name);
        if self.in_const == 0
            && (compound || message)
            && let Ok(args) =
                m.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
            && let Some(cond) = args.first()
        {
            let at = (range(&m.path).0, range(&m.bang_token).1);
            if compound
                && let syn::Expr::Binary(b) = cond
                && matches!(b.op, syn::BinOp::And(_))
            {
                self.found.push((
                    COMPOUND,
                    at,
                    format!("{name}!(a && b) checks two conditions: a failure does not say which one broke"),
                    "split it into one assertion per condition, each with its own message; `a || b` stays one",
                    Confidence::Certain,
                ));
            }
            if message && args.len() == 1 {
                self.found.push((
                    MESSAGE,
                    at,
                    format!("{name}! without a message: a failure prints the expression, not what it means"),
                    "add a message naming the invariant in words: `assert!(n > 0, \"a frame has at least one byte\")`",
                    Confidence::Certain,
                ));
            }
        }
        syn::visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let name = e.method.to_string();
        if self.cx.enabled(PARTIAL)
            && self.methods.contains(&name)
            && let syn::Expr::MethodCall(r) = &*e.receiver
            && r.method == "partial_cmp"
        {
            self.found.push((
                PARTIAL,
                (range(&r.method).0, range(&e.paren_token.span.join()).1),
                format!("partial_cmp(..).{name}() panics on NaN (assuming the values are floats)"),
                "order floats with total_cmp; where a NaN means a broken invariant, assert is_finite() with a message first",
                Confidence::Estimate,
            ));
        }
        syn::visit::visit_expr_method_call(self, e);
    }
}
