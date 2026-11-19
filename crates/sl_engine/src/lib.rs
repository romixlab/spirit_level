//! A `spirit_level` run: [`run`] turns a project, a configuration and the registered analyses into a [`Report`];
//! [`fix::apply`] applies the exact fixes of a report.

pub mod fix;
pub mod progress;
pub mod waiver;

pub use progress::{Event, Progress, Silent};

use serde::Serialize;
use sl_analysis::{Analysis, Context};
use sl_config::Config;
use sl_core::{Date, Diagnostic, FixKind, Level, LintMeta, Need, Source, Span};
use sl_model::Project;
use std::collections::{BTreeMap, BTreeSet};
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

/// `tool-version`: built into the engine, which finds the tools
pub static TOOL_VERSION: LintMeta = LintMeta {
    code: "X0034",
    id: "tool-version",
    category: "tools",
    summary: "every external tool an enabled lint runs is a version its output is tested against",
    docs: include_str!("../docs/tool-version.md"),
    params: &[],
    needs: &[],
    fix: FixKind::None,
    source: Source::Native,
};

/// The lints of the engine itself
pub static ENGINE_LINTS: &[&LintMeta] = &[&WAIVER_INVALID, &WAIVER_UNUSED, &TOOL_VERSION];

/// What to run
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Only these lint ids, codes or categories; empty for every enabled lint
    pub only: Vec<String>,
    /// Today
    pub today: Date,
    /// The PATH tools are found on instead of the process's (tests set a fake PATH through this)
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

/// A lint enabled but skipped: nothing in the project is of the kind it checks. Unlike [`NotRun`] it never fails
/// the run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Skipped {
    /// The lint
    pub lint: String,
    /// Its code
    pub code: String,
    /// Its configured level
    pub level: Level,
    /// The need it has nothing to apply to (`elf`)
    pub need: String,
    /// Why (`no firmware binary`)
    pub reason: String,
}

/// An `[[exclude]]` entry as a run applied it
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Excluded {
    /// The crates it leaves out
    pub crates: Vec<String>,
    /// The path patterns it leaves out
    pub paths: Vec<String>,
    /// Why
    pub reason: String,
    /// When it stops holding: a date or a release
    pub expires: String,
    /// Days until a date expiry; None for a release
    pub days_left: Option<i64>,
    /// Where it is written
    pub origin: String,
    /// The findings it left out of this run
    pub findings: usize,
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
    /// Lints enabled but skipped: the project has nothing they apply to
    pub skipped: Vec<Skipped>,
    /// The `[[exclude]]` entries and what each left out
    pub excluded: Vec<Excluded>,
    /// Time per analysis
    pub timings: Vec<Timing>,
}

impl Report {
    /// Findings that fail the run
    pub fn failing(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.fails()).count()
    }

    /// Whether an enabled lint that could not run fails the run: every one does, except a warn-level one when
    /// `allow_warn_not_run` is set (`spl check --allow-warn-not-run`)
    pub fn incomplete(&self, allow_warn_not_run: bool) -> bool {
        self.not_run
            .iter()
            .any(|n| n.level == Level::Deny || !allow_warn_not_run)
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

/// The findings of `waiver-unused`: each waiver that allowed nothing, when every lint it names ran
fn unused_waivers(
    waivers: &[waiver::Waiver],
    used: &[bool],
    ran: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    waivers
        .iter()
        .zip(used)
        // a waiver of a lint that did not run this time is not unused
        .filter(|(w, u)| !**u && w.lints.iter().all(|l| ran.contains(l)))
        .map(|(w, _)| {
            Diagnostic::new(
                "waiver-unused",
                format!("{}:{}", w.file, w.line),
                format!(
                    "the waiver of {} allows nothing: remove it",
                    w.lints.join(", ")
                ),
            )
            .span(Span::new(w.file.clone(), w.line))
        })
        .collect()
}

/// The tools of `pins` found on PATH, each detected once per run
type Tools = BTreeMap<&'static str, sl_tools::Detection>;

/// Why a need is missing, or None when it is met
fn missing(
    need: Need,
    project: &Project,
    config: &Config,
    opts: &RunOptions,
    tools: &Tools,
) -> Option<String> {
    match need {
        Need::Resolve => project.resolve().err().map(|e| {
            format!("the dependency graph could not be resolved (full cargo metadata): {e}")
        }),
        Need::Map => config.map.is_none().then(|| {
            "no crate map: add [map] or map-file to spirit_level.toml (see the book's map page)"
                .to_owned()
        }),
        Need::Tool(t) => match tools.get(t) {
            Some(d) => d.missing_reason(),
            None => sl_tools::find(t, opts.path.as_deref())
                .is_none()
                .then(|| sl_tools::missing_reason(t)),
        },
        Need::Nightly => (!sl_tools::rustup::nightly_installed(&project.root)).then(|| {
            let channel = sl_tools::rustup::nightly_channel(&project.root);
            format!(
                "no {channel} toolchain: `rustup toolchain install {channel} --profile minimal`"
            )
        }),
        Need::Elf => opts.elf.is_empty().then(|| {
            "no ELF given: build the firmware and pass --elf target/<triple>/release/<name>"
                .to_owned()
        }),
    }
}

/// Why a lint has nothing in the project to apply to, or None when it has: a lint that needs the linked ELF applies
/// to the firmware binaries ([`sl_firmware::firmware_binaries`]) it is enabled on, by the crate's own tiers and
/// exceptions
fn inapplicable<'f>(
    need: Need,
    lint: &str,
    config: &Config,
    firmware: impl FnOnce() -> &'f [String],
) -> Option<String> {
    match need {
        Need::Elf => {
            let firmware = firmware();
            if firmware.is_empty() {
                return Some("no firmware binary".to_owned());
            }
            firmware
                .iter()
                .all(|k| config.level(lint, Some(k), None).0 == Level::Allow)
                .then(|| {
                    format!(
                        "no firmware binary it is enabled on ({} allow it)",
                        firmware.join(", ")
                    )
                })
        }
        Need::Resolve | Need::Map | Need::Tool(_) | Need::Nightly => None,
    }
}

/// What the needs of one lint say about it
enum Verdict {
    /// Every need is met
    Run,
    /// The project has nothing it applies to
    Skip(Skipped),
    /// A need is missing
    NotRun(NotRun),
}

/// The needs of a run's lints, checked against the project and the options
struct Needs<'r> {
    project: &'r Project,
    config: &'r Config,
    opts: &'r RunOptions,
    tools: &'r Tools,
    /// The firmware binaries' names, read the first time a lint needs the ELF
    firmware: std::cell::OnceCell<Vec<String>>,
}

impl Needs<'_> {
    /// Each analysis of `analyses` with the enabled and selected lints whose needs are met, in order; the others go
    /// to `skipped` and `not_run`
    fn plan<'a>(
        &self,
        analyses: &[&'a dyn Analysis],
        selected: &dyn Fn(&LintMeta) -> bool,
        skipped: &mut Vec<Skipped>,
        not_run: &mut Vec<NotRun>,
    ) -> Vec<(&'a dyn Analysis, Vec<&'static str>)> {
        let mut plan = Vec::new();
        for a in analyses {
            let mut runnable = Vec::new();
            for m in a.lints() {
                if !self.config.enabled(m.id) || !selected(m) {
                    continue;
                }
                match self.check(m) {
                    Verdict::Run => runnable.push(m.id),
                    Verdict::Skip(s) => skipped.push(s),
                    Verdict::NotRun(n) => not_run.push(n),
                }
            }
            if !runnable.is_empty() {
                plan.push((*a, runnable));
            }
        }
        plan
    }

    fn check(&self, m: &LintMeta) -> Verdict {
        let firmware = || -> &[String] {
            self.firmware.get_or_init(|| {
                sl_firmware::firmware_binaries(self.project)
                    .into_iter()
                    .map(|p| p.name.clone())
                    .collect()
            })
        };
        let level = self.config.highest_level(m.id);
        if let Some((need, reason)) = m
            .needs
            .iter()
            .find_map(|n| inapplicable(*n, m.id, self.config, firmware).map(|r| (n, r)))
        {
            return Verdict::Skip(Skipped {
                lint: m.id.to_owned(),
                code: m.code.to_owned(),
                level,
                need: need.name(),
                reason,
            });
        }
        m.needs
            .iter()
            .find_map(|n| {
                missing(*n, self.project, self.config, self.opts, self.tools).map(|r| (n, r))
            })
            .map_or(Verdict::Run, |(need, reason)| {
                Verdict::NotRun(NotRun {
                    lint: m.id.to_owned(),
                    code: m.code.to_owned(),
                    level,
                    need: need.name(),
                    reason,
                })
            })
    }
}

/// Detects every tool of `pins` that a lint of `wanted` needs
fn detect_tools<'m>(
    wanted: impl Iterator<Item = &'m LintMeta>,
    project: &Project,
    opts: &RunOptions,
) -> Tools {
    let mut tools = Tools::new();
    for m in wanted {
        for need in m.needs {
            if let Need::Tool(t) = need
                && let Some(pin) = sl_tools::pin(t)
                && !tools.contains_key(t)
            {
                tools.insert(
                    t,
                    sl_tools::detect(pin, opts.path.as_deref(), &project.root),
                );
            }
        }
    }
    tools
}

/// The findings of `tool-version`: each tool that ran outside its tested range or printed no version, with the
/// lints whose results it makes untested
fn untested_tools(tools: &Tools, ran: &[String], lints: &[&'static LintMeta]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (name, d) in tools {
        let Some(why) = d.untested_reason() else {
            continue;
        };
        let used: Vec<&str> = lints
            .iter()
            .filter(|m| ran.iter().any(|r| r == m.id) && m.needs.contains(&Need::Tool(name)))
            .map(|m| m.id)
            .collect();
        if used.is_empty() {
            continue;
        }
        out.push(
            Diagnostic::new(
                "tool-version",
                format!("tool:{name}"),
                format!("{why}: the results of {} are untested", used.join(", ")),
            )
            .span(Span::new("spirit_level.toml", 0))
            .help(format!(
                "{}; `spl tools needed` lists every tool",
                d.fix_help()
            )),
        );
    }
    out
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
            d.level = config.highest_level(&d.lint);
        }
    }
}

/// Drops each finding an `[[exclude]]` entry covers: one about an excluded crate, or at a file an entry's patterns
/// match or that lies in an excluded crate's directory; each entry with the count it dropped
fn apply_excludes(
    diagnostics: &mut Vec<Diagnostic>,
    project: &Project,
    config: &Config,
    today: Date,
) -> Vec<Excluded> {
    let mut counts = vec![0; config.excludes.len()];
    // the deepest member directory holding a file names the crate a finding without one is about
    let crate_at = |path: &str| {
        project
            .packages
            .iter()
            .filter(|p| !p.dir.is_empty() && path.starts_with(&format!("{}/", p.dir)))
            .max_by_key(|p| p.dir.len())
            .map(|p| p.name.as_str())
    };
    diagnostics.retain(|d| {
        let path = d.primary().map(|s| s.file.as_str());
        let krate = d.krate.as_deref().or_else(|| path.and_then(crate_at));
        match config.excludes.iter().position(|e| e.covers(krate, path)) {
            Some(i) => {
                counts[i] += 1;
                false
            }
            None => true,
        }
    });
    config
        .excludes
        .iter()
        .zip(counts)
        .map(|(e, findings)| Excluded {
            crates: e.crates.clone(),
            paths: e.paths.clone(),
            reason: e.reason.clone(),
            expires: e.expires.to_string(),
            days_left: e.days_left(today),
            origin: e.origin.to_string(),
            findings,
        })
        .collect()
}

/// Allows each finding a waiver covers, with the waiver's reason; which waivers allowed something
fn apply_waivers(diagnostics: &mut [Diagnostic], waivers: &[waiver::Waiver]) -> Vec<bool> {
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
    used
}

/// The tools the lints of `runnable` run, each once
fn tools_of(lints: &[&LintMeta], runnable: &[&'static str]) -> Vec<&'static str> {
    let mut tools = Vec::new();
    for m in lints.iter().filter(|m| runnable.contains(&m.id)) {
        for need in m.needs {
            if let Need::Tool(t) = need
                && !tools.contains(t)
            {
                tools.push(*t);
            }
        }
    }
    tools
}

/// Runs every enabled lint of `analyses` on the project, reporting each step to `progress`
pub fn run(
    project: &Project,
    config: &Config,
    analyses: &[&dyn Analysis],
    opts: &RunOptions,
    progress: &dyn Progress,
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
    let mut skipped = Vec::new();
    let mut ran = Vec::new();
    let mut timings = Vec::new();
    progress.event(Event::FindingTools);
    let tools = detect_tools(
        lints
            .iter()
            .copied()
            .filter(|m| config.enabled(m.id) && selected(m)),
        project,
        opts,
    );
    let needs = Needs {
        project,
        config,
        opts,
        tools: &tools,
        firmware: std::cell::OnceCell::new(),
    };
    // each analysis with the lints it runs, known before the first one starts
    let plan = needs.plan(analyses, &selected, &mut skipped, &mut not_run);
    progress.event(Event::Planned {
        analyses: plan.len(),
    });
    for (a, runnable) in plan {
        let analysis_tools = tools_of(a.lints(), &runnable);
        progress.event(Event::Started {
            analysis: a.name(),
            tools: &analysis_tools,
        });
        let start = Instant::now();
        let cx = Context::new(project, config, opts.today, runnable.clone())
            .with_elf(&opts.elf)
            .with_landing(opts.landing);
        a.run(&cx);
        let out = cx.take();
        ran.extend(
            runnable
                .iter()
                .filter(|id| !out.not_run.iter().any(|s| s.lint == **id))
                .map(|s| s.to_string()),
        );
        not_run.extend(out.not_run.into_iter().map(|s| NotRun {
            level: config.highest_level(&s.lint),
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
        progress.event(Event::Finished { analysis: a.name() });
    }

    apply_levels(&mut diagnostics, config);

    // waivers in sources, manifests and the messages of the commits read
    progress.event(Event::Waivers);
    let resolve = |name: &str| lints.iter().find(|m| m.is(name)).map(|m| m.id);
    let (mut waivers, mut problems) =
        waiver::collect(project, &resolve, opts.today, project.version);
    let (cw, cp) = waiver::collect_commits(project, &resolve, opts.today, project.version);
    waivers.extend(cw);
    problems.extend(cp);
    let used = apply_waivers(&mut diagnostics, &waivers);
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
        engine.extend(unused_waivers(&waivers, &used, &ran_set));
    }
    if config.enabled("tool-version") && selected(&TOOL_VERSION) {
        ran.push("tool-version".into());
        engine.extend(untested_tools(&tools, &ran, &lints));
    }
    for mut d in engine {
        let path = d.primary().map(|s| s.file.clone());
        d.level = config.level(&d.lint, None, path.as_deref()).0;
        diagnostics.push(d);
    }

    let excluded = apply_excludes(&mut diagnostics, project, config, opts.today);
    for d in &mut diagnostics {
        d.code = code_of(&d.lint);
    }
    diagnostics.sort_by(|a, b| {
        (b.level, &a.lint, a.primary(), &a.item).cmp(&(a.level, &b.lint, b.primary(), &b.item))
    });
    not_run.sort_by(|a, b| a.lint.cmp(&b.lint));
    skipped.sort_by(|a, b| a.lint.cmp(&b.lint));
    ran.sort();
    Report {
        schema: "spirit_level.report/1",
        root: project.root.display().to_string(),
        extends: config.extends.clone(),
        diagnostics,
        ran,
        not_run,
        skipped,
        excluded,
        timings,
    }
}
