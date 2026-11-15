//! build-script-clock: build.rs reads no clock and resolves git through the real git dir.

use sl_analysis::Context;
use sl_analysis::syntax::{Range, range};
use sl_core::{Confidence, Diagnostic, Span};
use sl_model::{FileKind, Package};
use syn::visit::Visit;

const ID: &str = "build-script-clock";

/// The `.git` path components of a string literal: `.git` with no identifier character touching either side (so
/// `cargo:rerun-if-changed=.git/HEAD` matches and `.github`/`.gitignore` do not), each widened to its full path-like
/// token (`.git/HEAD`, not the whole cargo directive it sits in)
fn git_dir_components(s: &str) -> Vec<String> {
    let touches = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'-';
    let in_token = |c: u8| touches(c) || matches!(c, b'.' | b'/' | b'\\');
    let b = s.as_bytes();
    s.match_indices(".git")
        .filter(|(i, _)| {
            let before_ok = *i == 0 || !touches(b[i - 1]);
            let after = i + 4;
            let after_ok = after == b.len() || !touches(b[after]);
            before_ok && after_ok
        })
        .map(|(i, _)| {
            let mut start = i;
            while start > 0 && in_token(b[start - 1]) {
                start -= 1;
            }
            let mut end = i + 4;
            while end < b.len() && in_token(b[end]) {
                end += 1;
            }
            s[start..end].to_owned()
        })
        .collect()
}

struct Hits {
    clock: Vec<(Range, String)>,
    git_dir: Vec<(Range, String)>,
    clock_fns: Vec<Vec<String>>,
}

impl<'ast> Visit<'ast> for Hits {
    /// `println!`/`eprintln!` and friends hold an opaque token stream; cargo directives (`cargo:rerun-if-changed=`)
    /// live in their first argument, so it is parsed as an expression list and walked like any other code
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m.path.segments.last().map(|s| s.ident.to_string());
        if matches!(
            name.as_deref(),
            Some("println" | "print" | "eprintln" | "eprint" | "format" | "write" | "writeln")
        ) && let Ok(args) = m.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for a in &args {
                self.visit_expr(a);
            }
        }
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if let syn::Expr::Path(fp) = &*c.func {
            let segs: Vec<String> = fp
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if let Some(matched) = self.clock_fns.iter().find(|f| segs.ends_with(f.as_slice())) {
                self.clock.push((range(c), matched.join("::")));
            }
        }
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_expr_lit(&mut self, e: &'ast syn::ExprLit) {
        if let syn::Lit::Str(s) = &e.lit {
            for component in git_dir_components(&s.value()) {
                self.git_dir.push((range(e), component));
            }
        }
        syn::visit::visit_expr_lit(self, e);
    }
}

pub(crate) fn run(cx: &Context<'_>, pkg: &Package) {
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.project;
    let params = cx.params_for(ID, pkg);
    let clock_fns: Vec<Vec<String>> = params
        .list("clock-functions")
        .iter()
        .map(|f| f.split("::").map(str::to_owned).collect())
        .collect();
    let evidence = params.list("git-dir-evidence").to_vec();
    for f in p.sources(pkg) {
        if f.kind != FileKind::Build {
            continue;
        }
        let Ok(ast) = f.ast() else { continue };
        let mut v = Hits {
            clock: Vec::new(),
            git_dir: Vec::new(),
            clock_fns: clock_fns.clone(),
        };
        v.visit_file(ast);
        for (r, call) in v.clock {
            let line = r.0.0;
            cx.emit(
                Diagnostic::new(ID, format!("{}:{}:{line}:{call}", pkg.name, f.rel), format!("{} calls {call}() at build time: the output differs each run, so cargo never sees a cached build", f.rel))
                    .span(Span::range(f.rel.clone(), r.0, r.1))
                    .krate(&pkg.name)
                    .confidence(Confidence::Estimate)
                    .help("drop the clock read; read the git commit time instead, or leave the field unset until the binary that wants it looks it up"),
            );
        }
        if evidence.iter().any(|e| f.text.contains(e.as_str())) {
            continue;
        }
        for (r, lit) in v.git_dir {
            let line = r.0.0;
            cx.emit(
                Diagnostic::new(ID, format!("{}:{}:{line}:{lit}", pkg.name, f.rel), format!("{} treats `{lit}` as the git directory: `.git` is a file inside a worktree checkout, not the real git dir", f.rel))
                    .span(Span::range(f.rel.clone(), r.0, r.1))
                    .krate(&pkg.name)
                    .confidence(Confidence::Estimate)
                    .help("resolve the real git dir first (`git rev-parse --git-dir`, run from the build script's directory) and rerun-if-changed on its HEAD and index, not a hardcoded `.git` path"),
            );
        }
    }
}
