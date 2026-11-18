//! `cls-3` A `service` module holds a call that panics

use crate::FwNeed;
use sl_analysis::lint;
use sl_analysis::syntax::{Range, range, without_tests};
use sl_core::{Diagnostic, Params, Span, TaskClass};
use sl_firmware::Firmware;
use syn::visit::{self, Visit};

lint!(META, "F0111", "cls-3", "cls", "a service module holds no call that panics in release",
    params: [
        ("methods", StrList, "[\"unwrap\", \"expect\"]", "methods that panic on None or Err"),
        ("macros", StrList, "[\"panic\", \"todo\", \"unimplemented\", \"unreachable\", \"assert\", \"assert_eq\", \"assert_ne\"]", "macros that panic in a release build"),
    ],
    needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];

/// Each call of `methods` and each use of `macros` in a file of class `service`, test code left out. A message does
/// not excuse one: the module never reaches the panic handler in release, whatever the panic would say.
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for f in &fw.sources {
        if fw.classes.class_of(&f.in_package) != Some(TaskClass::Service) {
            continue;
        }
        let mut calls = Calls {
            methods: params.list("methods"),
            macros: params.list("macros"),
            found: Vec::new(),
        };
        calls.visit_file(&without_tests(&f.ast));
        for (r, what) in calls.found {
            let line = r.0.0;
            out.push(
                Diagnostic::new("cls-3", format!("{}:{line}", f.rel), format!("{what} in service module {}: a panic in it resets the whole MCU", f.rel))
                    .span(Span::range(&f.rel, r.0, r.1))
                    .help("check with debug_assert!, count the miss (cnt!) and fall back: skip the sample, clamp, reset the task's own state"),
            );
        }
    }
    out
}

struct Calls<'a> {
    methods: &'a [String],
    macros: &'a [String],
    found: Vec<(Range, String)>,
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(name) = m.path.segments.last().map(|s| s.ident.to_string())
            && self.macros.contains(&name)
        {
            // the macro's name and its `!`
            self.found
                .push(((range(&m.path).0, range(&m.bang_token).1), format!("{name}!")));
        }
        visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let name = e.method.to_string();
        if self.methods.contains(&name) {
            // `.name(args)`
            self.found.push((
                (range(&e.dot_token).0, range(&e.paren_token.span.join()).1),
                format!(".{name}()"),
            ));
        }
        visit::visit_expr_method_call(self, e);
    }
}
