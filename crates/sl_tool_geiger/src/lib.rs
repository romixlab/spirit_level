//! The geiger adapter: `unsafe` in the dependencies, counted with the geiger library.
//!
//! Every package the workspace members reach through normal and build edges is scanned: each `.rs` file under
//! its directory (tests left out), parsed with syn by geiger. A package whose library root declares
//! `#![forbid(unsafe_code)]` and counts nothing is skipped.
//!
//! The library is linked only with the `link` feature; without it the lint reports that it did not run.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need};

lint!(GEIGER_UNSAFE_DEPS, "X0023", "geiger-unsafe-deps", "geiger", "every dependency that uses unsafe code is one the project accepted",
    tool: ("geiger", Library),
    params: [("allow", StrList, "[]", "dependencies accepted with their unsafe code"),
             ("max-uses", Int, "0", "unsafe uses a dependency may have without being reported")],
    needs: [Need::Resolve], fix: None);

/// The geiger analysis
pub struct Geiger;

/// The registered instance
pub static GEIGER: Geiger = Geiger;

static LINTS: &[&LintMeta] = &[&GEIGER_UNSAFE_DEPS];

impl Analysis for Geiger {
    fn name(&self) -> &'static str {
        "geiger"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        run(cx);
    }
}

#[cfg(not(feature = "link"))]
fn run(cx: &Context<'_>) {
    cx.not_run_all(
        "feature:geiger",
        "spl was built without the `geiger` feature (`cargo install spirit_level_cli --features geiger`)",
    );
}

#[cfg(feature = "link")]
fn run(cx: &Context<'_>) {
    use sl_core::{Confidence, Diagnostic, Span};
    use std::collections::BTreeSet;

    let p = cx.project;
    let Ok(resolve) = p.resolve() else { return };
    let params = cx.params("geiger-unsafe-deps");
    let allow = params.list("allow");
    let max = params.int("max-uses").max(0) as u64;
    let mut reached = BTreeSet::new();
    for (i, pkg) in resolve.packages.iter().enumerate() {
        if pkg.member {
            reached.extend(resolve.closure(i, false));
        }
    }
    for i in reached {
        let pkg = &resolve.packages[i];
        if pkg.member || allow.contains(&pkg.name) {
            continue;
        }
        let Some(dir) = pkg.manifest_path.parent() else {
            continue;
        };
        let (uses, files) = scan(dir);
        if uses > max {
            cx.emit(
                Diagnostic::new(
                    "geiger-unsafe-deps",
                    format!("{}@{}", pkg.name, pkg.version),
                    format!(
                        "{} {} has {uses} unsafe uses in {files} files (every file of the package counted, used in this build or not)",
                        pkg.name, pkg.version
                    ),
                )
                .span(Span::new("Cargo.lock", sl_tools::lock_line(p, &pkg.name, &pkg.version)))
                .metric("unsafe-uses", uses as f64, Some(max as f64))
                .confidence(Confidence::Estimate)
                .help("replace it with a crate that forbids unsafe code, or accept it in geiger-unsafe-deps' allow"),
            );
        }
    }
}

/// Unsafe uses in the `.rs` files under `dir`, and the number of files that have any
#[cfg(feature = "link")]
fn scan(dir: &std::path::Path) -> (u64, usize) {
    let mut uses = 0;
    let mut files = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !matches!(
                    name.as_str(),
                    "target" | "tests" | "benches" | "examples" | ".git"
                ) {
                    stack.push(path);
                }
            } else if name.ends_with(".rs")
                && let Ok(m) = geiger::find_unsafe_in_file(&path, geiger::IncludeTests::No)
            {
                let c = &m.counters;
                let n = c.functions.unsafe_
                    + c.exprs.unsafe_
                    + c.item_impls.unsafe_
                    + c.item_traits.unsafe_
                    + c.methods.unsafe_;
                if n > 0 {
                    uses += n;
                    files += 1;
                }
            }
        }
    }
    (uses, files)
}
