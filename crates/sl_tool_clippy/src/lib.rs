//! The clippy adapter: `cargo clippy --message-format json`, each finding under the lint of its clippy group.
//!
//! Which clippy lints run, and at which rustc level, is the project's own `[lints.clippy]` table: the adapter
//! passes no `-W` flags, so it never overrides an `allow` the project wrote. The `spirit_level` level of a group
//! decides whether its findings warn or fail the run, and waivers, exceptions and expiries apply as to any lint.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need};
use sl_tools::cargo_json::{self, Message};
use std::collections::BTreeMap;

lint!(CLIPPY_BUILD, "X0009", "clippy-build", "clippy", "the workspace builds under clippy without compiler errors or warnings",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/usage.html"),
    params: [("args", StrList, "[\"--workspace\", \"--all-targets\"]", "arguments to `cargo clippy` (packages, targets, features); every clippy lint reads them")],
    needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_CORRECTNESS, "X0012", "clippy-correctness", "clippy", "no finding of clippy's correctness group: code that is outright wrong",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#correctness"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_SUSPICIOUS, "X0018", "clippy-suspicious", "clippy", "no finding of clippy's suspicious group: code that is most likely wrong",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#suspicious"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_STYLE, "X0017", "clippy-style", "clippy", "no finding of clippy's style group: code written the idiomatic way",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#style"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_COMPLEXITY, "X0011", "clippy-complexity", "clippy", "no finding of clippy's complexity group: simple things done simply",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#complexity"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_PERF, "X0015", "clippy-perf", "clippy", "no finding of clippy's perf group: no needlessly slow code",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#perf"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_PEDANTIC, "X0014", "clippy-pedantic", "clippy", "no finding of the pedantic clippy lints the project turned on",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#pedantic"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_NURSERY, "X0013", "clippy-nursery", "clippy", "no finding of the nursery clippy lints the project turned on",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#nursery"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_CARGO, "X0010", "clippy-cargo", "clippy", "no finding of the cargo-group clippy lints the project turned on",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#cargo"), params: [], needs: [Need::Tool("clippy")], fix: None);
lint!(CLIPPY_RESTRICTION, "X0016", "clippy-restriction", "clippy", "no finding of the restriction clippy lints the project turned on",
    tool: ("clippy", Executable, docs: "https://doc.rust-lang.org/clippy/lints.html#restriction"), params: [], needs: [Need::Tool("clippy")], fix: None);

/// The clippy analysis
pub struct Clippy;

/// The registered instance
pub static CLIPPY: Clippy = Clippy;

static LINTS: &[&LintMeta] = &[
    &CLIPPY_BUILD,
    &CLIPPY_CORRECTNESS,
    &CLIPPY_SUSPICIOUS,
    &CLIPPY_STYLE,
    &CLIPPY_COMPLEXITY,
    &CLIPPY_PERF,
    &CLIPPY_PEDANTIC,
    &CLIPPY_NURSERY,
    &CLIPPY_CARGO,
    &CLIPPY_RESTRICTION,
];

/// clippy's groups as `clippy-driver -W help` names them, and the lint of each
const GROUPS: &[(&str, &str)] = &[
    ("correctness", "clippy-correctness"),
    ("suspicious", "clippy-suspicious"),
    ("style", "clippy-style"),
    ("complexity", "clippy-complexity"),
    ("perf", "clippy-perf"),
    ("pedantic", "clippy-pedantic"),
    ("nursery", "clippy-nursery"),
    ("cargo", "clippy-cargo"),
    ("restriction", "clippy-restriction"),
];

impl Analysis for Clippy {
    fn name(&self) -> &'static str {
        "clippy"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let root = &cx.project.root;
        let Some(exe) = sl_tools::find("clippy", None) else {
            cx.not_run_all("tool:clippy", &sl_tools::missing_reason("clippy"));
            return;
        };
        let driver = exe.with_file_name("clippy-driver");
        let groups = match sl_tools::run(&driver, &["-W", "help"], root, &[]) {
            Ok(o) if o.code == Some(0) => parse_groups(&o.stdout),
            Ok(o) => {
                cx.not_run_all(
                    "run:clippy",
                    &format!("`clippy-driver -W help` failed, {}", o.failure()),
                );
                return;
            }
            Err(e) => {
                cx.not_run_all("run:clippy", &format!("cannot run clippy-driver: {e}"));
                return;
            }
        };
        let args = cx.params("clippy-build").list("args");
        let excludes = cx.cargo_excludes(args);
        let mut argv = vec!["clippy", "--message-format", "json"];
        argv.extend(args.iter().chain(&excludes).map(String::as_str));
        let out = match sl_tools::run_subcommand(&exe, &argv, root, &[]) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:clippy", &format!("cannot run cargo clippy: {e}"));
                return;
            }
        };
        let msgs = cargo_json::parse(&out.stdout);
        if msgs.success.is_none() && msgs.messages.is_empty() {
            cx.not_run_all("run:clippy", &format!("cargo clippy {}", out.failure()));
            return;
        }
        for m in &msgs.messages {
            if let Some(d) = diagnostic(m, &groups, cx) {
                cx.emit(d);
            }
        }
    }
}

/// The lint a message belongs to
fn lint_of(m: &Message, groups: &BTreeMap<String, &'static str>) -> &'static str {
    match m.code.as_deref().and_then(|c| c.strip_prefix("clippy::")) {
        // a clippy lint in no group (renamed, or newer than the table): restriction, as it is not on by default
        Some(name) => groups.get(name).copied().unwrap_or("clippy-restriction"),
        None => "clippy-build",
    }
}

fn diagnostic(
    m: &Message,
    groups: &BTreeMap<String, &'static str>,
    cx: &Context<'_>,
) -> Option<Diagnostic> {
    if !matches!(m.level.as_str(), "warning" | "error") {
        return None;
    }
    // a summary line (`3 warnings emitted`, `aborting due to ...`) has no place
    let span = m.span_in(&cx.project.root)?;
    let lint = lint_of(m, groups);
    let help = m.help.clone().unwrap_or_else(|| match &m.code {
        Some(c) if c.starts_with("clippy::") => format!(
            "see https://rust-lang.github.io/rust-clippy/master/index.html#{}",
            &c["clippy::".len()..]
        ),
        _ => "fix the code the compiler points at".to_owned(),
    });
    let code = m.code.as_deref().unwrap_or(&m.level);
    // clippy's own lint (`clippy::needless_return`), or rustc's (`unused_variables`, `E0308`)
    let rule = match code.strip_prefix("clippy::") {
        Some(name) => format!("clippy:{name}"),
        None => format!("rustc:{code}"),
    };
    let mut d = Diagnostic::new(lint, m.item(), format!("{}: {code}", m.message))
        .span(span)
        .help(help)
        .rule(rule);
    if let Some(p) = m.package(cx.project) {
        d = d.krate(&p.name);
    }
    Some(d)
}

/// The clippy lint names (with `_`) of each group the adapter maps, from `clippy-driver -W help`
fn parse_groups(help: &str) -> BTreeMap<String, &'static str> {
    let mut map = BTreeMap::new();
    for line in help.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("clippy::") else {
            continue;
        };
        let Some((group, members)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        let Some((_, lint)) = GROUPS.iter().find(|(g, _)| *g == group) else {
            continue;
        };
        for m in members.split(',') {
            if let Some(name) = m.trim().strip_prefix("clippy::") {
                map.insert(name.replace('-', "_"), *lint);
            }
        }
    }
    map
}

/// Reads captured clippy output the way [`Clippy`] does: `(lint, item)` of each finding; for tests
#[doc(hidden)]
pub fn classify(help: &str, jsonl: &str) -> Vec<(&'static str, String)> {
    let groups = parse_groups(help);
    cargo_json::parse(jsonl)
        .messages
        .iter()
        .filter(|m| m.span.is_some() && matches!(m.level.as_str(), "warning" | "error"))
        .map(|m| (lint_of(m, &groups), m.item()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GROUPS_SAMPLE: &str = include_str!("../tests/samples/groups.txt");
    const CLIPPY_SAMPLE: &str = include_str!("../tests/samples/clippy.jsonl");

    #[test]
    fn groups_table() {
        let g = parse_groups(GROUPS_SAMPLE);
        assert_eq!(g.get("approx_constant"), Some(&"clippy-correctness"));
        assert_eq!(g.get("needless_return"), Some(&"clippy-style"));
        assert_eq!(g.get("inline_always"), Some(&"clippy-pedantic"));
        assert_eq!(g.get("absolute_paths"), Some(&"clippy-restriction"));
        assert_eq!(g.get("multiple_crate_versions"), Some(&"clippy-cargo"));
        // `clippy::all` is a union, not a group of its own
        assert!(g.len() > 700, "{}", g.len());
    }

    #[test]
    fn findings_by_group() {
        let got = classify(GROUPS_SAMPLE, CLIPPY_SAMPLE);
        let lints: Vec<&str> = got.iter().map(|(l, _)| *l).collect();
        for want in [
            "clippy-correctness",
            "clippy-style",
            "clippy-perf",
            "clippy-build",
        ] {
            assert!(lints.contains(&want), "{want} missing in {got:?}");
        }
        assert!(
            got.iter()
                .any(|(l, i)| *l == "clippy-build" && i == "unused_variables@src/lib.rs:unused"),
            "{got:?}"
        );
    }
}
