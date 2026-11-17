//! The CI workflow and the checks it runs: the justfile's `lint` and `test` recipes when it has both (through `just`
//! or by running each command of the recipe), else Clippy and the tests through cargo.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::justfile::Recipe;
use sl_model::workflow::{commands, invokes};
use sl_model::{Justfile, Workflow};

/// One check the workflow must run: its fingerprint, what it is called in a message, the step to add, whether a
/// command runs it, and the recipe whose own commands run it too
struct Check {
    item: &'static str,
    name: &'static str,
    step: &'static str,
    runs: fn(&[String]) -> bool,
    recipe: Option<&'static str>,
}

const JUST: [Check; 2] = [
    Check {
        item: "ci:lint",
        name: "`just lint`",
        step: "just lint",
        runs: |c| invokes(c, "just", |r| r == "lint"),
        recipe: Some("lint"),
    },
    Check {
        item: "ci:test",
        name: "`just test`",
        step: "just test",
        runs: |c| invokes(c, "just", is_test_recipe),
        recipe: Some("test"),
    },
];

const CARGO: [Check; 2] = [
    Check {
        item: "ci:clippy",
        name: "Clippy",
        step: "cargo clippy --workspace --all-targets -- -D warnings",
        runs: |c| invokes(c, "cargo", |s| s == "clippy"),
        recipe: None,
    },
    Check {
        item: "ci:test",
        name: "the tests",
        step: "cargo test --workspace",
        runs: |c| invokes(c, "cargo", |s| s == "test" || s == "nextest"),
        recipe: None,
    },
];

/// `test`, or a variant of it (`test-all`, `test_ci`)
fn is_test_recipe(r: &str) -> bool {
    r.strip_prefix("test")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['-', '_']))
}

/// Whether the workflow runs every command of the recipe's body itself, as written there: a recipe that is only
/// `spl check` is run by a step `spl check`. A shebang recipe or one with `{{...}}` in its body never is.
fn runs_body(r: &Recipe, cmds: &[Vec<String>]) -> bool {
    !r.shebang
        && !r.body.is_empty()
        && r.body.iter().all(|line| {
            // `@` (quiet) and `-` (errors ignored) prefix a line for just, not for the shell
            let line = line.trim_start_matches(['@', '-']);
            !line.contains("{{") && commands(line).iter().all(|c| cmds.contains(c))
        })
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
        .filter(|j| j.recipe("lint").is_some() && j.recipe("test").is_some());
    let checks = if just.is_some() { &JUST } else { &CARGO };
    let cmds: Vec<Vec<String>> = wf.commands().map(|(_, c)| c).collect();
    for c in checks {
        let by_body = c
            .recipe
            .and_then(|name| just.as_ref()?.recipe(name))
            .is_some_and(|r| runs_body(r, &cmds));
        if by_body || cmds.iter().any(|cmd| (c.runs)(cmd)) {
            continue;
        }
        cx.emit(
            Diagnostic::new(
                id,
                c.item,
                format!("the CI workflow {path} does not run {}", c.name),
            )
            .span(Span::new(path, 1))
            .help(match c.recipe {
                Some(r) => format!(
                    "add a step `- run: {}` to one of its jobs, or steps that run each command of the `{r}` recipe as written",
                    c.step
                ),
                None => format!("add a step `- run: {}` to one of its jobs", c.step),
            }),
        );
    }
}
