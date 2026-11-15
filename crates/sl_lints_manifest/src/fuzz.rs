//! fuzz-corpus-test: every `fuzz/fuzz_targets/*.rs` has a corpus tracked in git and a `#[test]` that replays it.

use sl_analysis::Context;
use sl_analysis::syntax::is_test;
use sl_core::{Confidence, Diagnostic, Span, glob};
use sl_model::Project;
use syn::Item;

const ID: &str = "fuzz-corpus-test";
const TARGETS_GLOB: &str = "fuzz/fuzz_targets/*.rs";
const TARGETS_DIR: &str = "fuzz/fuzz_targets/";
const CORPUS_DIR: &str = "fuzz/corpus";

pub(crate) fn run(cx: &Context<'_>) {
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.project;
    let tracked = match p.tracked() {
        Ok(t) => t,
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    ID,
                    "git",
                    format!("fuzz targets not checked: tracked files unknown ({e})"),
                )
                .span(Span::new(".git", 0))
                .confidence(Confidence::Candidate)
                .help("run spl inside the project's git checkout"),
            );
            return;
        }
    };
    let targets: Vec<&str> = tracked
        .iter()
        .filter(|f| glob::matches(TARGETS_GLOB, f))
        .filter_map(|f| f.strip_prefix(TARGETS_DIR))
        .filter_map(|f| f.strip_suffix(".rs"))
        .collect();
    if targets.is_empty() {
        return;
    }
    let test_names = test_fn_names(p);
    for target in targets {
        let file = format!("{TARGETS_DIR}{target}.rs");
        let prefix = format!("{CORPUS_DIR}/{target}/");
        if !tracked.iter().any(|f| f.starts_with(&prefix)) {
            cx.emit(
                Diagnostic::new(ID, format!("{file}:corpus"), format!("fuzz target {target} has no corpus tracked at {prefix}"))
                    .span(Span::new(file.clone(), 0))
                    .help(format!("commit a few seed inputs under {prefix}: {target} replays them without a fuzzing engine")),
            );
        }
        let needle = target.replace('-', "_");
        if !test_names.iter().any(|n| n.contains(&needle)) {
            cx.emit(
                Diagnostic::new(ID, format!("{file}:test"), format!("fuzz target {target} has no #[test] whose name mentions {needle}"))
                    .span(Span::new(file, 0))
                    .confidence(Confidence::Estimate)
                    .help(format!("add a #[test] naming {needle} that runs its entry function over every file in {prefix}")),
            );
        }
    }
}

/// The names of every `#[test]` function in the project's own sources (fuzz/ is its own excluded workspace, so
/// its targets never appear here: the replaying test lives in the crate the target exercises)
fn test_fn_names(p: &Project) -> Vec<String> {
    let mut out = Vec::new();
    for f in p.all_sources() {
        let Ok(ast) = f.ast() else { continue };
        collect(&ast.items, &mut out);
    }
    out
}

fn collect(items: &[Item], out: &mut Vec<String>) {
    for item in items {
        match item {
            Item::Fn(f) if is_test(&f.attrs) => out.push(f.sig.ident.to_string()),
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect(inner, out);
                }
            }
            _ => {}
        }
    }
}
