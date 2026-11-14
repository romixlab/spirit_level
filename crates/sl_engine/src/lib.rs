//! A `spirit_level` run: [`run`] turns a project, a configuration and the registered analyses into a [`Report`];
//! [`fix::apply`] applies the exact fixes of a report.

pub mod fix;
pub mod waiver;

use serde::Serialize;
use sl_analysis::{Analysis, Context};
use sl_config::Config;
use sl_core::{Date, Diagnostic, FixKind, Level, LintMeta, Need, Source, Span};
use sl_model::Project;
use std::collections::BTreeSet;
use std::time::Instant;

/// `waiver-invalid`: built into the engine
pub static WAIVER_INVALID: LintMeta = LintMeta {
    code: "C0012",
    id: "waiver-invalid",
    category: "waivers",
    summary: "a waiver comment names known lints, gives a reason and has not expired",
    docs: include_str!("../docs/waiver-invalid.md"),
    params: &[],
    needs: &[],
    fix: FixKind::None,
    source: Source::Native,
};

/// `waiver-unused`: built into the engine
pub static WAIVER_UNUSED: LintMeta = LintMeta {
    code: "C0013",
    id: "waiver-unused",
    category: "waivers",
    summary: "every waiver comment allows a finding",
    docs: include_str!("../docs/waiver-unused.md"),
    params: &[],
    needs: &[],
    fix: FixKind::None,
    source: Source::Native,
};

/// The lints of the engine itself
pub static ENGINE_LINTS: &[&LintMeta] = &[&WAIVER_INVALID, &WAIVER_UNUSED];

/// What to run
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Only these lint ids, codes or categories; empty for every enabled lint
    pub only: Vec<String>,
    /// Today
    pub today: Date,
    /// Whether a tool on PATH counts as present (tests set a fake PATH through this)
    pub path: Option<std::ffi::OsString>,
    /// Linked ELFs of the project's firmware (`--elf`), for the lints that need [`Need::Elf`]
    pub elf: Vec<std::path::PathBuf>,
    /// Whether this run lands a branch (`--landing`): lints that allow something during the work refuse it
    pub landing: bool,
}

/// A lint that did not run, and why
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NotRun {
    /// The lint
    pub lint: String,
    /// Its code
    pub code: String,
    /// Its configured level
    pub level: Level,
    /// The missing need (`resolve`, `map`, `tool:cargo-deny`)
    pub need: String,
    /// Why it is missing and what would provide it
    pub reason: String,
}

/// How long one analysis took
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Timing {
    /// The analysis
    pub analysis: String,
    /// Milliseconds
    pub ms: u128,
}

/// The result of a run
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    /// Format id of the JSON form
    pub schema: &'static str,
    /// The project root
    pub root: String,
    /// What the configuration extends
    pub extends: Vec<String>,
    /// Findings, worst first, then by lint and place
    pub diagnostics: Vec<Diagnostic>,
    /// Lints that ran
    pub ran: Vec<String>,
    /// Lints enabled but not run
    pub not_run: Vec<NotRun>,
    /// Time per analysis
    pub timings: Vec<Timing>,
}

impl Report {
    /// Findings that fail the run
    pub fn failing(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.fails()).count()
    }

    /// Whether a denied lint could not run (the run is incomplete)
    pub fn incomplete(&self) -> bool {
        self.not_run.iter().any(|n| n.level == Level::Deny)
    }

    /// Findings counted by level, allowed ones apart: (deny, warn, allowed)
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for d in &self.diagnostics {
            if d.allowed.is_some() {
                c.2 += 1;
            } else if d.level == Level::Deny {
                c.0 += 1;
            } else {
                c.1 += 1;
            }
        }
        c
    }
}

/// Every lint id of the analyses plus the engine's own
pub fn all_lints(analyses: &[&dyn Analysis]) -> Vec<&'static LintMeta> {
    let mut v: Vec<&'static LintMeta> = ENGINE_LINTS.to_vec();
    for a in analyses {
        v.extend(a.lints().iter().copied());
    }
    v
}

/// Why a need is missing, or None when it is met
pub fn missing(
    need: Need,
    project: &Project,
    config: &Config,
    opts: &RunOptions,
) -> Option<String> {
    let path = opts.path.as_deref();
    match need {
        Need::Resolve => project.resolve().err().map(|e| {
            format!("the dependency graph could not be resolved (full cargo metadata): {e}")
        }),
        Need::Map => config.map.is_none().then(|| {
            "no crate map: add [map] or map-file to spirit_level.toml (see the book's map page)"
                .to_owned()
        }),
        Need::Tool(t) => find_tool(t, path)
            .is_none()
            .then(|| sl_tools::missing_reason(t)),
        Need::Nightly => (!has_nightly())
            .then(|| "no nightly toolchain: `rustup toolchain install nightly`".to_owned()),
        Need::Elf => opts.elf.is_empty().then(|| {
            "no ELF given: build the firmware and pass --elf target/<triple>/release/<name>"
                .to_owned()
        }),
    }
}

/// A tool's executable: the tool cache's build of its pinned version, then PATH (or `path`)
pub fn find_tool(name: &str, path: Option<&std::ffi::OsStr>) -> Option<std::path::PathBuf> {
    sl_tools::find(name, path)
}

fn has_nightly() -> bool {
    std::process::Command::new("rustup")
        .args(["run", "nightly", "rustc", "-V"])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Sets each finding's level from the config and its scoped exceptions; an excepted finding keeps
/// the lint's level and records the exception's reason
fn apply_levels(diagnostics: &mut [Diagnostic], config: &Config) {
    for d in diagnostics {
        let path = d.primary().map(|s| s.file.clone());
        let (level, exc) = config.level(&d.lint, d.krate.as_deref(), path.as_deref());
        d.level = d.at_most.map_or(level, |cap| level.min(cap));
        if level == Level::Allow {
            d.allowed = Some(exc.map_or_else(|| "allowed".to_owned(), |e| e.reason.clone()));
            d.level = config.lint(&d.lint).level;
        }
    }
}

/// Runs every enabled lint of `analyses` on the project
pub fn run(
    project: &Project,
    config: &Config,
    analyses: &[&dyn Analysis],
    opts: &RunOptions,
) -> Report {
    let lints = all_lints(analyses);
    let code_of = |id: &str| {
        lints
            .iter()
            .find(|m| m.id == id)
            .map_or_else(String::new, |m| m.code.to_owned())
    };
    let selected =
        |m: &LintMeta| opts.only.is_empty() || opts.only.iter().any(|o| m.is(o) || o == m.category);
    let mut diagnostics = Vec::new();
    let mut not_run = Vec::new();
    let mut ran = Vec::new();
    let mut timings = Vec::new();
    for a in analyses {
        let mut runnable = Vec::new();
        for m in a.lints() {
            if !config.enabled(m.id) || !selected(m) {
                continue;
            }
            match m
                .needs
                .iter()
                .find_map(|n| missing(*n, project, config, opts).map(|r| (n, r)))
            {
                Some((need, reason)) => not_run.push(NotRun {
                    lint: m.id.to_owned(),
                    code: m.code.to_owned(),
                    level: config.lint(m.id).level,
                    need: need.name(),
                    reason,
                }),
                None => runnable.push(m.id),
            }
        }
        if runnable.is_empty() {
            continue;
        }
        let start = Instant::now();
        let cx = Context::new(project, config, opts.today, runnable.clone())
            .with_elf(&opts.elf)
            .with_landing(opts.landing);
        a.run(&cx);
        let out = cx.take();
        ran.extend(
            runnable
                .iter()
                .filter(|id| !out.skipped.iter().any(|s| s.lint == **id))
                .map(|s| s.to_string()),
        );
        not_run.extend(out.skipped.into_iter().map(|s| NotRun {
            level: config.lint(&s.lint).level,
            code: code_of(&s.lint),
            lint: s.lint,
            need: s.need,
            reason: s.reason,
        }));
        diagnostics.extend(out.diagnostics);
        timings.push(Timing {
            analysis: a.name().to_owned(),
            ms: start.elapsed().as_millis(),
        });
    }

    apply_levels(&mut diagnostics, config);

    // waivers in sources, manifests and the messages of the commits read
    let resolve = |name: &str| lints.iter().find(|m| m.is(name)).map(|m| m.id);
    let (mut waivers, mut problems) =
        waiver::collect(project, &resolve, opts.today, project.version);
    let (cw, cp) = waiver::collect_commits(project, &resolve, opts.today, project.version);
    waivers.extend(cw);
    problems.extend(cp);
    let mut used = vec![false; waivers.len()];
    for d in diagnostics.iter_mut().filter(|d| d.allowed.is_none()) {
        let Some(p) = d.primary() else { continue };
        if let Some(i) = waivers.iter().position(|w| {
            w.file == p.file
                && p.line >= w.covers.0
                && p.line <= w.covers.1
                && w.lints.contains(&d.lint)
        }) {
            used[i] = true;
            d.allowed = Some(waivers[i].reason.clone());
        }
    }
    let ran_set: BTreeSet<String> = ran.iter().cloned().collect();
    let mut engine = Vec::new();
    if config.enabled("waiver-invalid") && selected(&WAIVER_INVALID) {
        ran.push("waiver-invalid".into());
        for p in problems {
            engine.push(
                Diagnostic::new(
                    "waiver-invalid",
                    format!("{}:{}", p.file, p.line),
                    p.message,
                )
                .span(Span::new(p.file, p.line)),
            );
        }
    }
    if config.enabled("waiver-unused") && selected(&WAIVER_UNUSED) {
        ran.push("waiver-unused".into());
        for (w, u) in waivers.iter().zip(&used) {
            // a waiver of a lint that did not run this time is not unused
            if !u && w.lints.iter().all(|l| ran_set.contains(l)) {
                engine.push(
                    Diagnostic::new(
                        "waiver-unused",
                        format!("{}:{}", w.file, w.line),
                        format!(
                            "the waiver of {} allows nothing: remove it",
                            w.lints.join(", ")
                        ),
                    )
                    .span(Span::new(w.file.clone(), w.line)),
                );
            }
        }
    }
    for mut d in engine {
        let path = d.primary().map(|s| s.file.clone());
        d.level = config.level(&d.lint, None, path.as_deref()).0;
        diagnostics.push(d);
    }

    for d in &mut diagnostics {
        d.code = code_of(&d.lint);
    }
    diagnostics.sort_by(|a, b| {
        (b.level, &a.lint, a.primary(), &a.item).cmp(&(a.level, &b.lint, b.primary(), &b.item))
    });
    not_run.sort_by(|a, b| a.lint.cmp(&b.lint));
    ran.sort();
    Report {
        schema: "spirit_level.report/1",
        root: project.root.display().to_string(),
        extends: config.extends.clone(),
        diagnostics,
        ran,
        not_run,
        timings,
    }
}
