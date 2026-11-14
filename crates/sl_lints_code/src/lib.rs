//! Source lints, one analysis: [`CODE`].

mod asserts;
mod build_info;
mod fakes;
mod practice;

use sl_analysis::syntax::{Range, range, without_tests};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Confidence, Diagnostic, LintMeta, Span};
use sl_model::{FileKind, Package, SourceFile};
use syn::visit::Visit;

lint!(TODO_MACROS, "C0008", "todo-macros", "code", "no todo!() or unimplemented!() outside tests",
    params: [], needs: [], fix: None);
lint!(PANIC_IN_LIBRARY, "C0005", "panic-in-library", "code", "library code returns errors for what can fail and says why where it panics",
    params: [
        ("methods", StrList, "[\"unwrap\"]", "methods that panic on None or Err without saying why"),
        ("macros", StrList, "[\"panic\", \"todo\", \"unimplemented\"]", "macros that panic"),
        ("bare-macros", StrList, "[\"unreachable\"]", "macros that count only without a message"),
        ("test-files", StrList, "[\"**/*_tests.rs\"]", "files of test code by name, relative to the crate, besides tests/, benches/, examples/ and #[cfg(test)] modules"),
        ("classes", StrList, "[]", "task classes ([embedded.classes]) whose modules are checked in every crate, binaries included, instead of library code"),
        ("max", Int, "0", "panicking calls allowed per crate before each is reported"),
    ],
    needs: [], fix: None);
lint!(COMPOUND_ASSERT, "C0030", "compound-assert", "code", "an assertion checks one condition, so a failure names the one that broke",
    params: [("macros", StrList, "[\"assert\", \"debug_assert\"]", "assertion macros whose condition is checked")],
    needs: [], fix: None);
lint!(ASSERT_MESSAGE, "C0031", "assert-message", "code", "every assertion says in words what it checks",
    params: [("macros", StrList, "[\"assert\", \"debug_assert\"]", "assertion macros that need a message; assert_eq! and assert_ne! print their values")],
    needs: [], fix: None);
lint!(FLOAT_PARTIAL_CMP_UNWRAP, "C0032", "float-partial-cmp-unwrap", "code", "floats are ordered with total_cmp, not partial_cmp that panics on NaN",
    params: [("methods", StrList, "[\"unwrap\", \"expect\"]", "methods on the partial_cmp result that panic on None")],
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
lint!(BUILD_INFO_ENV, "C0014", "build-info-env", "code", "library code reads build info at run time, not through env!",
    params: [("names", StrList, "[\"GIT_*\", \"VERGEN_*\", \"BUILD_*\", \"*_GIT_*\", \"*_SHA\", \"*_COMMIT\", \"*_COMMIT_*\", \"*_BUILD_*\", \"SOURCE_DATE_EPOCH\"]", "variable names that hold build info, besides those the crate's build script sets")],
    needs: [], fix: None);
lint!(TEST_SUPPORT_GATED, "C0015", "test-support-gated", "code", "test fakes and testing modules compile only for tests or behind the test-support feature",
    params: [
        ("feature", Str, "\"test-support\"", "the feature that turns test helpers on"),
        ("modules", StrList, "[\"testing\", \"test_utils\", \"test_support\", \"fakes\", \"mocks\"]", "module names that hold test helpers"),
        ("prefixes", StrList, "[\"Fake\", \"Mock\"]", "type name prefixes of test fakes"),
        ("test-crates", StrList, "[\"*_testkit\", \"*_test_support\", \"*_testing\"]", "crates that exist for tests and are exempt"),
        ("test-dirs", StrList, "[\"**/tests/**\"]", "directories whose crates exist for tests and are exempt"),
    ],
    needs: [], fix: None);
lint!(SIZED_INTS_IN_CONTRACTS, "C0017", "sized-ints-in-contracts", "code", "serde types of a contract crate use sized integers, not usize or isize",
    params: [("also", StrList, "[]", "crates held to it besides those the map marks contract")],
    needs: [], fix: None);
lint!(UNIT_SUFFIX, "C0018", "unit-suffix", "code", "a numeric field or constant that holds a time or a size names its unit",
    params: [
        ("words", StrList, "[\"timeout\", \"delay\", \"interval\", \"period\", \"latency\", \"ttl\", \"backoff\", \"elapsed\", \"deadline\", \"size\"]", "name words that say a time or a size"),
        ("units", StrList, "[\"ns\", \"us\", \"ms\", \"s\", \"sec\", \"secs\", \"seconds\", \"nanos\", \"micros\", \"millis\", \"min\", \"mins\", \"minutes\", \"h\", \"hours\", \"days\", \"ticks\", \"cycles\", \"bytes\", \"b\", \"kb\", \"kib\", \"mb\", \"mib\", \"gb\", \"gib\", \"bits\", \"words\", \"pages\", \"sectors\", \"blocks\", \"count\", \"items\", \"entries\", \"elements\", \"chars\", \"lines\", \"samples\", \"frames\", \"px\", \"pt\", \"mm\"]", "name words that are units; one anywhere in the name is enough"),
    ],
    needs: [], fix: None);
lint!(UNBOUNDED_READ, "C0019", "unbounded-read", "code", "a read to the end of a stream is bounded with take()",
    params: [
        ("methods", StrList, "[\"read_to_end\", \"read_to_string\"]", "methods that read until the end"),
        ("bounds", StrList, "[\"take\"]", "methods in the receiver chain that bound the read"),
    ],
    needs: [], fix: None);

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
    &BUILD_INFO_ENV,
    &TEST_SUPPORT_GATED,
    &SIZED_INTS_IN_CONTRACTS,
    &UNIT_SUFFIX,
    &UNBOUNDED_READ,
    &COMPOUND_ASSERT,
    &ASSERT_MESSAGE,
    &FLOAT_PARTIAL_CMP_UNWRAP,
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
        fakes::features(cx);
        for pkg in &p.packages {
            build_info::run(cx, pkg);
            fakes::run(cx, pkg);
            practice::run(cx, pkg);
            asserts::run(cx, pkg);
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
                    panics_checked: panics_checked(cx, pkg, f, lib),
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
            let max = cx.params_for("panic-in-library", pkg).count("max");
            if panics.len() > max {
                let n = panics.len();
                for (file, r, what) in panics {
                    let line = r.0.0;
                    cx.emit(
                        Diagnostic::new("panic-in-library", format!("{}:{file}:{line}", pkg.name), format!("{what} in library code ({n} in {})", pkg.name))
                            .span(Span::range(file, r.0, r.1))
                            .krate(&pkg.name)
                            .metric("panics", n as f64, Some(max as f64))
                            .help("return an error for what can fail; where it cannot, say why: `.expect(\"why\")`, `unreachable!(\"why\")`, or a waiver"),
                    );
                }
            }
        }
    }
}

/// Whether panic-in-library looks at a file: library code that is not test code by name, or, where the lint names
/// task classes, a module of one of them in any crate
fn panics_checked(cx: &Context<'_>, pkg: &Package, f: &SourceFile, lib: bool) -> bool {
    let params = cx.params_for("panic-in-library", pkg);
    if params
        .list("test-files")
        .iter()
        .any(|g| sl_core::glob::matches(g, &f.in_package))
    {
        return false;
    }
    let classes = params.list("classes");
    if classes.is_empty() {
        return lib;
    }
    f.kind == FileKind::Src
        && cx
            .config
            .embedded
            .classes
            .class_of(&f.in_package)
            .is_some_and(|c| classes.iter().any(|n| n == c.name()))
}

/// Library code: under src/ of a crate with a library target, not a binary root or src/bin/
pub(crate) fn is_library_file(pkg: &Package, f: &SourceFile) -> bool {
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
    /// panic-in-library looks at this file
    panics_checked: bool,
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
        if self.panics_checked && self.on("panic-in-library") {
            let params = self.cx.params("panic-in-library");
            if params.list("macros").contains(&name) {
                self.panics.push((line, format!("{name}!")));
            } else if params.list("bare-macros").contains(&name) && m.tokens.is_empty() {
                self.panics
                    .push((line, format!("{name}!() without a message")));
            }
        }
        if self.lib
            && self.on("print-in-library")
            && self
                .cx
                .params("print-in-library")
                .list("macros")
                .contains(&name)
        {
            self.found.push(("print-in-library", line, format!("{name}! in library code"), "return the text or log it through `log`/`tracing`; printing is the binary's choice".into(), Confidence::Certain));
        }
        syn::visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let name = e.method.to_string();
        // `.name(args)`
        let line = (range(&e.dot_token).0, range(&e.paren_token.span.join()).1);
        if self.panics_checked
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
                .to_owned();
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
            if segs.ends_with(&["Command".to_owned(), "new".to_owned()])
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
