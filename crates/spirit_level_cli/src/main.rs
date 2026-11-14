//! `spl`: a linter for Rust projects of every size. See the README for the commands.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the binary: printing its report and errors is its job"
)]

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use sl_core::{Date, Level};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vc_ui::{Exit, Paint};

mod dependents;
mod pages;
mod panics;
mod tools;

/// The tool's name, in front of its error lines
const TOOL: &str = "spl";
/// A denied finding
const DENIED: Exit = Exit::FAILURE;
/// A denied lint could not run
const NOT_RUN: Exit = Exit::category(3, "not_run");

#[derive(Parser)]
#[command(name = "spl", version = vc_build::version!(), about = "A linter for Rust projects of every size: tiers of lints in TOML, from one crate to a workspace of hundreds", long_about = None)]
#[command(
    after_help = "Examples:\n  spl check\n  spl check --all --group-by crate\n  spl check --format json --only S0001\n  spl explain workspace-lints\n  spl init --tier small-workspace\n  spl dependents --base main\n  spl pages"
)]
struct Cli {
    #[command(flatten)]
    color: vc_ui::ColorFlag,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum GroupByArg {
    /// One group per lint (an adapter's tool rule makes its own)
    Lint,
    /// One group per crate
    Crate,
    /// One group per file
    File,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check the project: run every lint its configuration enables
    #[command(
        after_help = "Examples:\n  spl check\n  spl check --all --group-by crate\n  spl check --tier small --only cli ../other_repo\n  spl check --format json --fix\n\nExit codes: 0 clean, 1 a denied finding, 2 a configuration or usage error, 3 a denied lint could not run"
    )]
    Check(CheckArgs),
    /// Show a lint's page: its rule, why, parameters and level in every tier
    #[command(after_help = "Examples:\n  spl explain crate-lines\n  spl explain S0001")]
    Explain {
        /// The lint: its code (`S0001`, `X-clippy:needless_return`) or its id (`crate-lines`)
        #[arg(add = ArgValueCandidates::new(lint_candidates))]
        lint: String,
    },
    /// List every lint code ever given, with its lint id, group and status; `--check` holds the registry to the
    /// lints (for `spirit_level`'s own `just lint`)
    #[command(
        after_help = "Examples:\n  spl codes\n  spl codes --check --base main\n\nA code is never reused or changed; a retired lint keeps its code reserved."
    )]
    Codes {
        /// Check the registry instead of listing it: every lint has its code, no code or id twice
        #[arg(long)]
        check: bool,
        /// With --check: the git revision whose registry every row must still match (rows only go from active to
        /// retired)
        #[arg(long, requires = "check", value_name = "REV")]
        base: Option<String>,
    },
    /// List every lint with its level in each shipped tier
    #[command(after_help = "Examples:\n  spl lints\n  spl lints --format json")]
    Lints {
        /// Output for people or for machines
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
    /// List the shipped tiers
    #[command(after_help = "Examples:\n  spl tiers")]
    Tiers,
    /// Show the project's resolved configuration: levels with where they were set, overrides and exceptions with
    /// their reasons and expiries
    #[command(after_help = "Examples:\n  spl config\n  spl config --all ../other_repo")]
    Config(ConfigArgs),
    /// Write a `spirit_level.toml` extending a tier
    #[command(after_help = "Examples:\n  spl init\n  spl init --tier medium --tier ai")]
    Init {
        /// The project directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The tier to extend
        #[arg(long, default_value = "small", add = ArgValueCandidates::new(tier_candidates))]
        tier: Vec<String>,
    },
    /// The repos that build on this library and the crates in them to retest: what changed against a base, the
    /// crates of the dependents' locks that reach it, the command to run in each
    #[command(after_help = dependents::AFTER_HELP)]
    Dependents(dependents::DependentsArgs),
    /// The reading metrics of every Markdown page: words, reading time, longest run of prose, paragraphs,
    /// sentences, figures, headings, links, code
    #[command(after_help = pages::AFTER_HELP)]
    Pages(pages::PagesArgs),
    /// What can panic in a firmware release: every panic site of the linked ELF with its function, kind, place
    /// and task class
    #[command(after_help = panics::AFTER_HELP)]
    Panics(panics::PanicsArgs),
    /// The external tools of the adapters: what the project needs, the tested versions, what is found on PATH
    #[command(subcommand)]
    Tools(tools::ToolsCmd),
    /// Write the book's generated pages (lints, tiers, nav) from the lints' metadata
    #[command(after_help = "Examples:\n  spl book\n  spl book docs --check")]
    Book {
        /// The docs directory
        #[arg(default_value = "docs")]
        docs: PathBuf,
        /// The zensical.toml whose nav block is generated
        #[arg(long, default_value = "zensical.toml")]
        zensical: PathBuf,
        /// Write nothing; fail when a page is stale
        #[arg(long)]
        check: bool,
    },
}

#[derive(clap::Args)]
struct CheckArgs {
    /// The project directory (cargo's search for the workspace root starts here)
    #[arg(default_value = ".")]
    path: PathBuf,
    /// A configuration file instead of `<root>/spirit_level.toml`
    #[arg(long)]
    config: Option<PathBuf>,
    /// Use shipped tiers instead of a configuration file (repeat to combine)
    #[arg(long, add = ArgValueCandidates::new(tier_candidates))]
    tier: Vec<String>,
    /// Only this lint or category (repeat for more)
    #[arg(long, add = ArgValueCandidates::new(lint_candidates))]
    only: Vec<String>,
    /// A linked ELF of the project's firmware, for the embedded lints that read it (repeat for several firmware crates)
    #[arg(long, value_name = "PATH")]
    elf: Vec<PathBuf>,
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Apply the exact fixes, then check again
    #[arg(long)]
    fix: bool,
    /// Also list findings allowed by waivers and exceptions
    #[arg(long)]
    show_allowed: bool,
    /// List every place of a group, not the first 5 (text output; JSON always has every finding)
    #[arg(long, short = 'a')]
    all: bool,
    /// Group the findings by lint, crate or file (text output)
    #[arg(long, value_enum, default_value = "lint", value_name = "BY")]
    group_by: GroupByArg,
    /// Source excerpts per group, from its first places; 0 turns them off (text output)
    #[arg(long, default_value_t = 1, value_name = "N")]
    excerpts: usize,
    /// Show how long each analysis took
    #[arg(long)]
    timings: bool,
    /// Today's date for expiries (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
    /// This run lands the branch: what is allowed while it is in work (a `FIXME`) fails here
    #[arg(long, env = "SPIRIT_LEVEL_LANDING")]
    landing: bool,
}

#[derive(clap::Args)]
struct ConfigArgs {
    /// The project directory
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Only lints that are on, and every override and exception
    #[arg(long)]
    all: bool,
    /// Today's date for expiries (YYYY-MM-DD)
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

fn lint_candidates() -> Vec<CompletionCandidate> {
    spirit_level::lints()
        .into_iter()
        .flat_map(|m| {
            [
                CompletionCandidate::new(m.code)
                    .help(Some(format!("{}: {}", m.id, m.summary).into())),
                CompletionCandidate::new(m.id)
                    .help(Some(format!("{}: {}", m.code, m.summary).into())),
            ]
        })
        .collect()
}

fn tier_candidates() -> Vec<CompletionCandidate> {
    spirit_level::TIERS
        .iter()
        .map(|(n, t)| {
            CompletionCandidate::new(*n).help(Some(spirit_level::tier_description(t).into()))
        })
        .collect()
}

fn today(s: Option<String>) -> Result<Date, String> {
    match s {
        None => Ok(Date::today_utc()),
        Some(s) => Date::parse(&s).ok_or_else(|| format!("--today {s}: not a date (YYYY-MM-DD)")),
    }
}

fn main() -> ExitCode {
    vc_ui::restore_sigpipe();
    vc_complete::complete(<Cli as clap::CommandFactory>::command);
    let cli = Cli::parse();
    cli.color.apply();
    // every error that ends a run is a configuration or usage error (exit 2)
    vc_ui::finish(TOOL, run(cli), |_| Some(Exit::BAD_ARGS)).into()
}

fn run(cli: Cli) -> Result<Exit, String> {
    match cli.cmd {
        Cmd::Check(a) => check(a),
        Cmd::Explain { lint } => explain(&lint),
        Cmd::Codes { check, base } => codes(check, base.as_deref()),
        Cmd::Lints { format } => list_lints(format),
        Cmd::Tiers => Ok(tiers()),
        Cmd::Config(a) => show_config(a),
        Cmd::Init { path, tier } => init(&path, &tier),
        Cmd::Dependents(a) => dependents::run(a),
        Cmd::Pages(a) => pages::run(a),
        Cmd::Panics(a) => panics::run(a),
        Cmd::Tools(cmd) => tools::run(cmd),
        Cmd::Book {
            docs,
            zensical,
            check,
        } => book(&docs, &zensical, check),
    }
}

fn check(a: CheckArgs) -> Result<Exit, String> {
    let CheckArgs {
        path,
        config,
        tier,
        only,
        elf,
        format,
        fix,
        show_allowed,
        all,
        group_by,
        excerpts,
        timings,
        today: t,
        landing,
    } = a;
    let mut opts = spirit_level::Options::new(today(t)?);
    opts.landing = landing;
    opts.config = config;
    opts.tiers = tier;
    opts.only = only;
    for e in &elf {
        if !e.is_file() {
            return Err(format!("--elf {}: no such file", e.display()));
        }
        opts.elf
            .push(std::path::absolute(e).map_err(|err| format!("--elf {}: {err}", e.display()))?);
    }
    let (project, mut report) = match spirit_level::check(&path, &opts) {
        Ok(r) => r,
        Err(spirit_level::CheckError::Config(errs)) => {
            match format {
                Format::Text => eprint!("{}", sl_report::config_errors(&errs, Paint::stderr())),
                Format::Json => println!("{}", sl_report::config_errors_json(&errs)),
            }
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    if fix {
        let applied =
            sl_engine::fix::apply(&project.root, &report.diagnostics).map_err(|e| e.to_string())?;
        for (file, what) in &applied.applied {
            eprintln!("fixed {file}: {what}");
        }
        for (file, what) in &applied.skipped {
            eprintln!("not fixed (overlaps another fix, run again) {file}: {what}");
        }
        if !applied.applied.is_empty() {
            report = spirit_level::check(&path, &opts)
                .map_err(|e| e.to_string())?
                .1;
        }
    }
    match format {
        Format::Text => print!(
            "{}",
            sl_report::text(
                &report,
                &spirit_level::lints(),
                sl_report::TextOptions {
                    show_allowed,
                    timings,
                    all,
                    excerpts,
                    group_by: match group_by {
                        GroupByArg::Lint => sl_report::GroupBy::Lint,
                        GroupByArg::Crate => sl_report::GroupBy::Crate,
                        GroupByArg::File => sl_report::GroupBy::File,
                    },
                    paint: Paint::stdout(),
                    ..sl_report::TextOptions::default()
                }
            )
        ),
        Format::Json => println!("{}", sl_report::json(&report)),
    }
    Ok(if report.failing() > 0 {
        DENIED
    } else if report.incomplete() {
        NOT_RUN
    } else {
        Exit::SUCCESS
    })
}

fn explain(lint: &str) -> Result<Exit, String> {
    if spirit_level::lint(lint).is_none()
        && let Some((tool, rule)) = lint.strip_prefix("X-").and_then(|r| r.split_once(':'))
    {
        return adapter_rule(tool, rule);
    }
    let m = spirit_level::lint(lint)
        .ok_or_else(|| format!("no lint {lint} (`spl lints` lists them)"))?;
    let tiers = sl_book::tier_levels()?;
    print!("{}", sl_book::lint_text(m, &tiers));
    Ok(Exit::SUCCESS)
}

/// `spl explain X-<tool>:<rule>`: the tool's own rule, and the adapter lints that report it
fn adapter_rule(tool: &str, rule: &str) -> Result<Exit, String> {
    let lints: Vec<&sl_core::LintMeta> = spirit_level::lints()
        .into_iter()
        .filter(|m| {
            matches!(m.source, sl_core::Source::Tool { .. })
                && m.id.starts_with(&format!("{tool}-"))
        })
        .collect();
    if lints.is_empty() {
        return Err(format!("no adapter of {tool} (`spl lints` lists them)"));
    }
    println!(
        "# X-{tool}:{rule}\n\n{tool}'s own rule `{rule}`, reported through the {tool} adapter. Its level is the level of the adapter lint the rule falls under:\n"
    );
    for m in lints {
        println!("- {} {}: {}", m.code, m.id, m.summary);
    }
    Ok(Exit::SUCCESS)
}

fn codes(check: bool, base: Option<&str>) -> Result<Exit, String> {
    use spirit_level::codes;
    if !check {
        for r in codes::rows() {
            println!("{} {:28} {:9} {}", r.code, r.id, r.group, r.status.name());
        }
        return Ok(Exit::SUCCESS);
    }
    let base_text = match &base {
        None => None,
        Some(rev) => {
            let out = std::process::Command::new("git")
                .args(["show", &format!("{rev}:crates/spirit_level/codes.toml")])
                .output()
                .map_err(|e| format!("--base {rev}: cannot run git: {e}"))?;
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).into_owned())
            } else {
                eprintln!("codes.toml is not in {rev} yet: nothing to compare with");
                None
            }
        }
    };
    let problems = codes::check(
        &spirit_level::lints(),
        codes::REGISTRY,
        base_text.as_deref(),
    );
    for p in &problems {
        let e = Exit::FAILURE.error(format!("codes.toml: {p}"));
        eprintln!("{}", vc_ui::error_line(TOOL, &e, Paint::stderr()));
    }
    Ok(if problems.is_empty() {
        Exit::SUCCESS
    } else {
        Exit::FAILURE
    })
}

fn list_lints(format: Format) -> Result<Exit, String> {
    let tiers = sl_book::tier_levels()?;
    let mut lints = spirit_level::lints();
    lints.sort_by_key(|m| m.code);
    match format {
        Format::Json => {
            let v: Vec<serde_json::Value> = lints
                .iter()
                .map(|m| {
                    let levels: serde_json::Map<String, serde_json::Value> = tiers
                        .iter()
                        .map(|(n, c)| (n.to_string(), c.lint(m.id).level.name().into()))
                        .collect();
                    serde_json::json!({"code": m.code, "id": m.id, "category": m.category, "summary": m.summary, "levels": levels, "meta": m})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
        Format::Text => {
            let w = lints.iter().map(|m| m.id.len()).max().unwrap_or(10);
            let names: Vec<String> = tiers.iter().map(|(n, _)| format!("{n:>6}")).collect();
            println!("code   {:w$}  {}  rule", "lint", names.join(" "));
            for m in &lints {
                let lv: Vec<String> = tiers
                    .iter()
                    .map(|(n, c)| {
                        let l = c.lint(m.id).level;
                        let s = if l == Level::Allow { "-" } else { l.name() };
                        format!("{s:>width$}", width = n.len().max(6))
                    })
                    .collect();
                println!("{}  {:w$}  {}  {}", m.code, m.id, lv.join(" "), m.summary);
            }
        }
    }
    Ok(Exit::SUCCESS)
}

fn tiers() -> Exit {
    for (n, t) in spirit_level::TIERS {
        println!("{n:16} {}", spirit_level::tier_description(t));
    }
    Exit::SUCCESS
}

fn show_config(a: ConfigArgs) -> Result<Exit, String> {
    let ConfigArgs {
        path,
        all,
        today: t,
    } = a;
    let project = sl_model::Project::load(&path).map_err(|e| e.to_string())?;
    let opts = spirit_level::Options::new(today(t)?);
    let c = match spirit_level::config(&project, &opts) {
        Ok(c) => c,
        Err(spirit_level::CheckError::Config(errs)) => {
            eprint!("{}", sl_report::config_errors(&errs, Paint::stderr()));
            return Ok(Exit::BAD_ARGS);
        }
        Err(e) => return Err(e.to_string()),
    };
    println!("extends: {}", c.extends.join(", "));
    for (id, l) in &c.lints {
        if l.level == Level::Allow && !all && l.origin.is_none() {
            continue;
        }
        let at = l
            .origin
            .as_ref()
            .map_or_else(|| "-".to_owned(), ToString::to_string);
        let exp = l
            .expires
            .map_or(String::new(), |e| format!(" (expires {e})"));
        println!("{:5} {id:32} {at:28} {}{exp}", l.level.name(), l.reason);
    }
    for e in &c.exceptions {
        let scope: Vec<String> = e.crates.iter().chain(&e.paths).cloned().collect();
        let params: Vec<String> = e.params.iter().map(|(k, v)| format!("{k} = {v}")).collect();
        println!(
            "exception {} for {}: {}{}{} — {} [{}]{}",
            e.lint,
            scope.join(", "),
            e.level.map_or(String::new(), |l| format!("level {l}")),
            if e.level.is_some() && !params.is_empty() {
                ", "
            } else {
                ""
            },
            params.join(", "),
            e.reason,
            e.origin,
            e.expires
                .map_or(String::new(), |x| format!(" (expires {x})"))
        );
    }
    if let Some(m) = &c.map {
        println!(
            "map: {} ({} layers, {} crates)",
            m.file,
            m.layers.len(),
            m.crates.len()
        );
    }
    Ok(Exit::SUCCESS)
}

fn init(path: &Path, tier: &[String]) -> Result<Exit, String> {
    let file = path.join(sl_config::FILE);
    if file.exists() {
        return Err(format!("{} exists already", file.display()));
    }
    for t in tier {
        if spirit_level::tier(t).is_none() {
            return Err(format!("no tier {t} (`spl tiers` lists them)"));
        }
    }
    let ext = if tier.len() == 1 {
        format!("{:?}", tier[0])
    } else {
        format!(
            "[{}]",
            tier.iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let text = format!(
        "# spirit_level configuration: https://github.com/romixlab/spirit_level (book: configuration)\nextends = {ext}\n\n# Override a lint: a level and/or parameters, always with a reason, optionally an expiry.\n# [lints.crate-lines]\n# level = \"deny\"\n# reason = \"why\"\n# expires = \"2027-06-30\"\n"
    );
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    println!("wrote {}", file.display());
    Ok(Exit::SUCCESS)
}

fn book(docs: &Path, zensical: &Path, check: bool) -> Result<Exit, String> {
    let changed = sl_book::write(docs, zensical, check).map_err(|e| e.to_string())?;
    for c in &changed {
        println!("{} {c}", if check { "stale" } else { "wrote" });
    }
    Ok(if check && !changed.is_empty() {
        Exit::FAILURE
    } else {
        Exit::SUCCESS
    })
}
