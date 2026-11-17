//! The CI workflow and the checks it runs: the justfile's `lint` and `test` recipes when it has both, else Clippy
//! and the tests through cargo.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::workflow::invokes;
use sl_model::{Justfile, Workflow};

/// One check the workflow must run: its fingerprint, what it is called in a message, the step to add, and whether
/// a command runs it
struct Check {
    item: &'static str,
    name: &'static str,
    step: &'static str,
    runs: fn(&[String]) -> bool,
}

const JUST: [Check; 2] = [
    Check {
        item: "ci:lint",
        name: "`just lint`",
        step: "just lint",
        runs: |c| invokes(c, "just", |r| r == "lint"),
    },
    Check {
        item: "ci:test",
        name: "`just test`",
        step: "just test",
        runs: |c| invokes(c, "just", is_test_recipe),
    },
];

const CARGO: [Check; 2] = [
    Check {
        item: "ci:clippy",
        name: "Clippy",
        step: "cargo clippy --workspace --all-targets -- -D warnings",
        runs: |c| invokes(c, "cargo", |s| s == "clippy"),
    },
    Check {
        item: "ci:test",
        name: "the tests",
        step: "cargo test --workspace",
        runs: |c| invokes(c, "cargo", |s| s == "test" || s == "nextest"),
    },
];

/// `test`, or a variant of it (`test-all`, `test_ci`)
fn is_test_recipe(r: &str) -> bool {
    r.strip_prefix("test")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['-', '_']))
}

pub(crate) fn run(cx: &Context<'_>) {
    let id = "ci-workflow";
    if !cx.enabled(id) {
        return;
    }
    let p = cx.project;
    let path = cx.params(id).str("path");
    let wf = match Workflow::read(&p.root, path) {
        None => {
            cx.emit(
                Diagnostic::new(id, format!("file:{path}"), format!("the repo has no CI workflow at {path}"))
                    .span(Span::new(path, 0))
                    .help(format!("add {path}: on pushes and pull requests it runs the checks a person runs before landing")),
            );
            return;
        }
        Some(Err(e)) => {
            cx.not_run(id, format!("file:{path}"), format!("cannot read it: {e}"));
            return;
        }
        Some(Ok(wf)) => wf,
    };
    let just = Justfile::read(&p.root)
        .is_some_and(|j| j.recipe("lint").is_some() && j.recipe("test").is_some());
    let checks = if just { &JUST } else { &CARGO };
    for c in checks {
        if wf.commands().any(|(_, cmd)| (c.runs)(&cmd)) {
            continue;
        }
        cx.emit(
            Diagnostic::new(
                id,
                c.item,
                format!("the CI workflow {path} does not run {}", c.name),
            )
            .span(Span::new(path, 1))
            .help(format!("add a step `- run: {}` to one of its jobs", c.step)),
        );
    }
}
