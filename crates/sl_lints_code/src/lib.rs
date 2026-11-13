//! Source lints, one analysis: [`CODE`].

use sl_analysis::syntax::{Range, range, without_tests};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Confidence, Diagnostic, LintMeta, Span};
use sl_model::{FileKind, Package, SourceFile};
use syn::visit::Visit;

lint!(TODO_MACROS, "C0008", "todo-macros", "code", "no todo!() or unimplemented!() outside tests",
    params: [], needs: [], fix: None);
lint!(PANIC_IN_LIBRARY, "C0005", "panic-in-library", "code", "library code returns errors instead of panicking",
    params: [
        ("methods", StrList, "[\"unwrap\", \"expect\"]", "methods that panic on None or Err"),
        ("macros", StrList, "[\"panic\", \"unreachable\"]", "macros that panic"),
        ("max", Int, "0", "panicking calls allowed per crate before each is reported"),
    ],
    needs: [], fix: None);
lint!(PRINT_IN_LIBRARY, "C0006", "print-in-library", "code", "library code does not print",
    params: [("macros", StrList, "[\"println\", \"eprintln\", \"print\", \"eprint\", \"dbg\"]", "macros that print")],
    needs: [], fix: None);
lint!(ALLOW_WITHOUT_REASON, "C0002", "allow-without-reason", "code", "every allow or expect of a lint says why",
    params: [], needs: [], fix: None);
lint!(FLOAT_DETERMINISM, "C0004", "float-determinism", "code", "deterministic crates compute float transcendentals through libm",
    params: [
        ("methods", StrList, "[\"sin\", \"cos\", \"tan\", \"asin\", \"acos\", \"atan\", \"atan2\", \"sinh\", \"cosh\", \"tanh\", \"asinh\", \"acosh\", \"atanh\", \"exp\", \"exp2\", \"exp_m1\", \"ln\", \"log\", \"log2\", \"log10\", \"ln_1p\", \"powf\", \"powi\", \"cbrt\", \"hypot\", \"sin_cos\", \"mul_add\"]", "float methods whose precision is unspecified"),
        ("also", StrList, "[]", "crates held to it besides those the map marks deterministic"),
        ("replacement", Str, "\"libm\"", "what to call instead, named in the message"),
    ],
    needs: [], fix: None);
lint!(RANDOM_STATE_HASH, "C0007", "random-state-hash", "code", "deterministic crates use no randomly seeded hash maps",
    params: [("also", StrList, "[]", "crates held to it besides those the map marks deterministic")],
    needs: [], fix: None);
lint!(CLI_SUBPROCESS, "C0003", "cli-subprocess", "code", "workspace crates call each other as libraries, never by running the workspace's own binaries",
    params: [], needs: [], fix: None);

/// The source analysis
pub struct Code;

/// The registered instance
pub static CODE: Code = Code;

static LINTS: &[&LintMeta] = &[
    &TODO_MACROS,
    &PANIC_IN_LIBRARY,
    &PRINT_IN_LIBRARY,
    &ALLOW_WITHOUT_REASON,
    &FLOAT_DETERMINISM,
    &RANDOM_STATE_HASH,
    &CLI_SUBPROCESS,
];

impl Analysis for Code {
    fn name(&self) -> &'static str {
        "code"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let bins: Vec<String> = p
            .packages
            .iter()
            .flat_map(|k| {
                k.targets
                    .iter()
                    .filter(|(t, _)| *t == sl_model::TargetKind::Bin)
                    .map(|(_, n)| n.clone())
            })
            .collect();
        for pkg in &p.packages {
            let det = cx
                .map()
                .and_then(|m| m.crates.get(&pkg.name))
                .is_some_and(|c| c.deterministic);
            let det_float = det
                || cx
                    .params("float-determinism")
                    .list("also")
                    .contains(&pkg.name);
            let det_hash = det
                || cx
                    .params("random-state-hash")
                    .list("also")
                    .contains(&pkg.name);
            let mut panics = Vec::new();
            for f in p.sources(pkg) {
                if f.kind == FileKind::Build {
                    continue;
                }
                let Ok(ast) = f.ast() else { continue };
                // allow-without-reason looks at test code too
                let mut a = Allows { found: Vec::new() };
                a.visit_file(ast);
                for (r, what) in a.found {
                    let line = r.0.0;
                    cx.emit(
                        Diagnostic::new("allow-without-reason", format!("{}:{}:{line}", pkg.name, f.rel), format!("#[{what}] gives no reason"))
                            .span(Span::range(f.rel.clone(), r.0, r.1))
                            .krate(&pkg.name)
                            .help("add `reason = \"...\"` (and prefer #[expect], which fails once the lint no longer fires)"),
                    );
                }
                if p.is_test_file(pkg, f) {
                    continue;
                }
                let ast = without_tests(ast);
                let lib = is_library_file(pkg, f);
                let mut v = Calls {
                    cx,
                    bins: &bins,
                    lib,
                    det_float,
                    det_hash,
                    found: Vec::new(),
                    panics: Vec::new(),
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
                panics.extend(
                    v.panics
                        .into_iter()
                        .map(|(l, what)| (f.rel.clone(), l, what)),
                );
            }
            let max = cx.params_for("panic-in-library", pkg).int("max");
            if panics.len() as i64 > max {
                let n = panics.len();
                for (file, r, what) in panics {
                    let line = r.0.0;
                    cx.emit(
                        Diagnostic::new("panic-in-library", format!("{}:{file}:{line}", pkg.name), format!("{what} in library code ({n} in {})", pkg.name))
                            .span(Span::range(file, r.0, r.1))
                            .krate(&pkg.name)
                            .metric("panics", n as f64, Some(max as f64))
                            .help("return an error the caller can handle; where the case truly cannot happen, say why in a waiver"),
                    );
                }
            }
        }
    }
}

/// Library code: under src/ of a crate with a library target, not a binary root or src/bin/
fn is_library_file(pkg: &Package, f: &SourceFile) -> bool {
    pkg.has_lib()
        && f.kind == FileKind::Src
        && f.in_package != "src/main.rs"
        && !f.in_package.starts_with("src/bin/")
}

struct Allows {
    found: Vec<(Range, String)>,
}

impl Allows {
    /// An `allow(...)` or `expect(...)` without a reason, also inside `cfg_attr(predicate, ...)`
    fn check(&mut self, meta: &syn::Meta) {
        let Ok(list) = meta.require_list() else {
            return;
        };
        if list.path.is_ident("cfg_attr") {
            type Metas = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;
            if let Ok(inner) = list.parse_args_with(Metas::parse_terminated) {
                // the first is the predicate (`not(test)`), the rest are the attributes it applies
                for m in inner.iter().skip(1) {
                    self.check(m);
                }
            }
            return;
        }
        let name = if list.path.is_ident("allow") {
            "allow"
        } else if list.path.is_ident("expect") {
            "expect"
        } else {
            return;
        };
        let t = list.tokens.to_string();
        if !t
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|w| w == "reason")
        {
            self.found.push((range(meta), format!("{name}({t})")));
        }
    }
}

impl<'ast> Visit<'ast> for Allows {
    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        self.check(&a.meta);
    }
}

/// (lint, range, message, help, confidence)
type Found = Vec<(&'static str, Range, String, String, Confidence)>;

struct Calls<'a> {
    cx: &'a Context<'a>,
    bins: &'a [String],
    lib: bool,
    det_float: bool,
    det_hash: bool,
    found: Found,
    panics: Vec<(Range, String)>,
}

impl Calls<'_> {
    fn on(&self, id: &str) -> bool {
        self.cx.enabled(id)
    }
}

fn mac_name(m: &syn::Macro) -> String {
    m.path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = mac_name(m);
        // the macro's name and its `!`
        let line = (range(&m.path).0, range(&m.bang_token).1);
        if (name == "todo" || name == "unimplemented") && self.on("todo-macros") {
            self.found.push((
                "todo-macros",
                line,
                format!("{name}!() defers the design: the code is not finished"),
                "write the real implementation, or return an error that says what is not supported"
                    .into(),
                Confidence::Certain,
            ));
        }
        if self.lib {
            if self.on("panic-in-library")
                && self
                    .cx
                    .params("panic-in-library")
                    .list("macros")
                    .contains(&name)
            {
                self.panics.push((line, format!("{name}!")));
            }
            if self.on("print-in-library")
                && self
                    .cx
                    .params("print-in-library")
                    .list("macros")
                    .contains(&name)
            {
                self.found.push(("print-in-library", line, format!("{name}! in library code"), "return the text or log it through `log`/`tracing`; printing is the binary's choice".into(), Confidence::Certain));
            }
        }
        syn::visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let name = e.method.to_string();
        // `.name(args)`
        let line = (range(&e.dot_token).0, range(&e.paren_token.span.join()).1);
        if self.lib
            && self.on("panic-in-library")
            && self
                .cx
                .params("panic-in-library")
                .list("methods")
                .contains(&name)
        {
            // .lock().unwrap() is lock-unwrap's
            let is_lock = matches!(&*e.receiver, syn::Expr::MethodCall(r) if r.args.is_empty() && ["lock", "read", "write"].contains(&r.method.to_string().as_str()));
            if !is_lock {
                self.panics.push((line, format!(".{name}()")));
            }
        }
        if self.det_float
            && self.on("float-determinism")
            && self
                .cx
                .params("float-determinism")
                .list("methods")
                .contains(&name)
        {
            let rep = self
                .cx
                .params("float-determinism")
                .str("replacement")
                .to_string();
            self.found.push(("float-determinism", line, format!(".{name}() has unspecified precision: results differ between platforms and Rust versions "), format!("call {rep}'s {name} instead, so every platform computes the same bits"), Confidence::Estimate));
        }
        syn::visit::visit_expr_method_call(self, e);
    }

    fn visit_path(&mut self, p: &'ast syn::Path) {
        if self.det_hash && self.on("random-state-hash") {
            let segs: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
            let last = segs.last().map(String::as_str);
            if matches!(last, Some("HashMap" | "HashSet"))
                && (segs.len() == 1 || segs.iter().any(|s| s == "std" || s == "collections"))
                && p.segments.last().is_some_and(|s| match &s.arguments {
                    syn::PathArguments::AngleBracketed(a) => {
                        a.args.len() < if last == Some("HashMap") { 3 } else { 2 }
                    }
                    _ => true,
                })
            {
                self.found.push((
                    "random-state-hash",
                    range(p),
                    format!(
                        "{} with the default random hasher: iteration order differs per run",
                        last.unwrap_or_default()
                    ),
                    "use a BTreeMap or a fixed-seed hasher where the order can reach an output"
                        .into(),
                    Confidence::Estimate,
                ));
            }
        }
        syn::visit::visit_path(self, p);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if self.on("cli-subprocess")
            && let syn::Expr::Path(fp) = &*c.func
        {
            let segs: Vec<String> = fp
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if segs.ends_with(&["Command".to_string(), "new".to_string()])
                && let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = c.args.first()
                && self.bins.contains(&s.value())
            {
                self.found.push(("cli-subprocess", range(c), format!("runs the workspace's own binary `{}`", s.value()), "call the library the binary is built on; the CLI is for people, agents and recipes".into(), Confidence::Certain));
            }
        }
        syn::visit::visit_expr_call(self, c);
    }
}
