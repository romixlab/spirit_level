//! `spl tools needed`: the external tools, toolchains and targets a project's lints need, and how to install them
#![expect(
    clippy::let_underscore_must_use,
    reason = "writing to a String cannot fail"
)]

use crate::registry::{Registered, Run};
use clap::{ArgGroup, Args, Subcommand};
use serde_json::{Value, json};
use sl_tools::{Detection, PinKind, State};
use std::fmt::Write;
use std::path::PathBuf;
use vc_ui::{Exit, Paint};

const NEEDED_HELP: &str = "Examples:
  spl tools needed
  spl tools needed --all
  spl tools needed --spec --missing | xargs -r cargo binstall -y
  spl tools needed --spec --missing | xargs -r -n1 cargo install --locked
  spl tools needed --components --missing | xargs -r rustup component add
  spl tools needed --toolchains --missing | xargs -r -n1 rustup toolchain install --profile minimal
  spl tools needed --targets --missing | xargs -r rustup target add
  spl tools needed --json

A tool is found on PATH and its version read. One outside its tested range still runs; `tool-version` reports
its results as untested.

Exit codes: 0 everything needed is there and in range, 1 something needed is missing or outside its tested range,
2 usage or configuration error";

/// The external tools of the adapters: what the project needs, the tested versions, what is found on PATH
#[derive(clap::Parser)]
#[command(name = "tools", display_order = 10)]
pub(crate) struct Tools {
    #[command(subcommand)]
    cmd: ToolsCmd,
}

inventory::submit! { Registered::of::<Tools>() }

impl Run for Tools {
    fn run(self) -> Result<Exit, String> {
        run(self.cmd)
    }
}

#[derive(Subcommand)]
enum ToolsCmd {
    /// The external tools the project's enabled lints run, with their tested versions and what is found on PATH;
    /// the toolchains and targets of `cargo-msrv`, `cargo-no-std` and `cargo-wasm`
    #[command(after_help = NEEDED_HELP)]
    Needed(NeededArgs),
}

#[derive(Args)]
#[command(group(ArgGroup::new("output").args(["spec", "components", "toolchains", "targets", "json"])))]
pub struct NeededArgs {
    /// The project directory (its tier and `spirit_level.toml` decide the lints)
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Every tool that has an adapter, not only the project's; no project is read
    #[arg(long, conflicts_with_all = ["toolchains", "targets"])]
    all: bool,
    /// Only what is missing, outside its tested range or prints no version
    #[arg(long)]
    missing: bool,
    /// One `crate@version` per line, for `cargo binstall` or `cargo install --locked`
    #[arg(long)]
    spec: bool,
    /// One rustup component per line, for `rustup component add`
    #[arg(long)]
    components: bool,
    /// One toolchain per line (the members' rust-versions, for `cargo-msrv`), for `rustup toolchain install`
    #[arg(long)]
    toolchains: bool,
    /// One target per line (`cargo-no-std`, `cargo-wasm`), for `rustup target add`
    #[arg(long)]
    targets: bool,
    /// Every record as JSON, for scripts and packagers
    #[arg(long)]
    json: bool,
    /// Today's date for expiries in the configuration (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

/// A tool as found, with the lints that run it
struct Tool {
    found: Detection,
    lints: Vec<&'static str>,
}

/// A toolchain the MSRV build selects, and why it cannot be selected
struct Toolchain {
    version: String,
    missing: Option<String>,
}

/// A target the cross builds need; `installed` is None without rustup
struct Target {
    name: String,
    installed: Option<bool>,
}

/// Everything `needed` lists
struct Needed {
    tools: Vec<Tool>,
    /// A listed tool runs on nightly, and no nightly toolchain is installed
    nightly_missing: bool,
    /// The project's pinned nightly channel ([`sl_tools::rustup::nightly_channel`]), for the install hint
    nightly_channel: String,
    toolchains: Vec<Toolchain>,
    targets: Vec<Target>,
}

impl Tool {
    fn wanted(&self) -> bool {
        self.found.state() != State::InRange
    }
}

impl Target {
    fn state(&self) -> &'static str {
        match self.installed {
            Some(true) => "installed",
            Some(false) => "missing",
            None => "unknown (no rustup)",
        }
    }
}

impl Needed {
    fn load(a: &NeededArgs) -> Result<Self, String> {
        if a.all {
            let dir = std::env::current_dir().map_err(|e| e.to_string())?;
            return Ok(Self::tools(spirit_level::tool_lints(), &dir));
        }
        let project = sl_model::Project::load(&a.path).map_err(|e| e.to_string())?;
        let opts = spirit_level::Options::new(crate::today(a.today.clone())?);
        let config = spirit_level::config(&project, &opts).map_err(|e| e.to_string())?;
        let needs = spirit_level::tool_needs(&project, &config);
        let mut n = Self::tools(needs.tools, &project.root);
        n.toolchains = needs
            .toolchains
            .into_iter()
            .map(|v| Toolchain {
                missing: sl_tools::rustup::toolchain_missing(&v, &project.root),
                version: v,
            })
            .collect();
        let installed = sl_tools::rustup::installed_targets(&project.root);
        n.targets = needs
            .targets
            .into_iter()
            .map(|t| Target {
                installed: installed.as_ref().map(|i| i.contains(&t)),
                name: t,
            })
            .collect();
        Ok(n)
    }

    /// The tools of `by_tool`, in the order of `tools.toml`, found on PATH
    fn tools(
        mut by_tool: std::collections::BTreeMap<&'static str, Vec<&'static str>>,
        dir: &std::path::Path,
    ) -> Self {
        let tools: Vec<Tool> = sl_tools::pins()
            .iter()
            .filter_map(|p| {
                by_tool.remove(p.name.as_str()).map(|lints| Tool {
                    found: sl_tools::detect(p, None, dir),
                    lints,
                })
            })
            .collect();
        let nightly_missing = tools.iter().any(|t: &Tool| t.found.pin.nightly)
            && !sl_tools::rustup::nightly_installed(dir);
        Self {
            tools,
            nightly_missing,
            nightly_channel: sl_tools::rustup::nightly_channel(dir),
            toolchains: Vec::new(),
            targets: Vec::new(),
        }
    }

    /// Keeps only what is missing, out of range or unreadable
    fn only_wanted(&mut self) {
        self.tools.retain(Tool::wanted);
        self.toolchains.retain(|t| t.missing.is_some());
        self.targets.retain(|t| t.installed == Some(false));
        self.nightly_missing &= self.tools.iter().any(|t| t.found.pin.nightly);
    }

    /// Whether something needed is not there at all
    fn missing(&self) -> bool {
        self.tools.iter().any(|t| t.found.state() == State::Missing)
            || self.toolchains.iter().any(|t| t.missing.is_some())
            || self.targets.iter().any(|t| t.installed == Some(false))
            || self.nightly_missing
    }

    /// Whether a tool is there but its output untested: outside its tested range, or its version unreadable
    fn untested(&self) -> bool {
        self.tools
            .iter()
            .any(|t| matches!(t.found.state(), State::OutOfRange | State::Unreadable))
    }
}

fn run(cmd: ToolsCmd) -> Result<Exit, String> {
    match cmd {
        ToolsCmd::Needed(a) => needed(&a),
    }
}

fn needed(a: &NeededArgs) -> Result<Exit, String> {
    let mut n = Needed::load(a)?;
    if a.missing {
        n.only_wanted();
    }
    let out = if a.spec {
        lines(not_shadowed(&n).filter_map(|t| t.found.pin.spec()))
    } else if a.components {
        lines(not_shadowed(&n).filter_map(|t| match &t.found.pin.kind {
            PinKind::Component(c) => Some(c.clone()),
            PinKind::Crate { .. } => None,
        }))
    } else if a.toolchains {
        lines(n.toolchains.iter().map(|t| t.version.clone()))
    } else if a.targets {
        lines(n.targets.iter().map(|t| t.name.clone()))
    } else if a.json {
        let v = to_json(&n);
        format!(
            "{}\n",
            serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?
        )
    } else {
        table(&n, Paint::stdout())
    };
    print!("{out}");
    Ok(if n.missing() || n.untested() {
        Exit::FAILURE
    } else {
        Exit::SUCCESS
    })
}

fn lines(items: impl Iterator<Item = String>) -> String {
    items.map(|l| l + "\n").collect()
}

/// `n`'s tools that are not shadowed: an in-range tool further down PATH needs no install, only PATH fixed
fn not_shadowed(n: &Needed) -> impl Iterator<Item = &Tool> {
    n.tools.iter().filter(|t| t.found.shadowed.is_none())
}

fn to_json(n: &Needed) -> Value {
    let tools: Vec<Value> = n
        .tools
        .iter()
        .map(|t| {
            let d = &t.found;
            let p = d.pin;
            let (kind, krate, component) = match &p.kind {
                PinKind::Crate { krate, .. } => ("crate", Some(krate.as_str()), None),
                PinKind::Component(c) => ("component", None, Some(c.as_str())),
            };
            json!({
                "name": p.name,
                "kind": kind,
                "crate": krate,
                "version": krate.map(|_| p.version()),
                "spec": p.spec(),
                "component": component,
                "tested": p.tested.as_ref().map(ToString::to_string),
                "exe": p.exe,
                "nightly": p.nightly,
                "path": d.exe.as_ref().map(|e| e.display().to_string()),
                "found_version": d.version.as_ref().ok().map(ToString::to_string),
                "state": d.state().as_str(),
                "reason": d.missing_reason().or_else(|| d.untested_reason()),
                "install": p.install_help(),
                "shadow": d.shadowed.as_ref().map(|s| json!({
                    "path": s.exe.display().to_string(),
                    "version": s.version.to_string(),
                })),
                "lints": t.lints,
            })
        })
        .collect();
    let toolchains: Vec<Value> = n
        .toolchains
        .iter()
        .map(|t| {
            json!({
                "version": t.version,
                "state": if t.missing.is_some() { "missing" } else { "installed" },
                "reason": t.missing,
            })
        })
        .collect();
    let targets: Vec<Value> = n
        .targets
        .iter()
        .map(|t| json!({ "target": t.name, "state": t.state() }))
        .collect();
    json!({
        "schema": "spirit_level.tools/1",
        "tools": tools,
        "toolchains": toolchains,
        "targets": targets,
        "nightly_missing": n.nightly_missing,
        "missing": n.missing(),
        "untested": n.untested(),
    })
}

fn state_cell(p: Paint, s: State) -> String {
    match s {
        State::InRange => p.good(s.as_str()),
        State::Missing => p.err(s.as_str()),
        State::OutOfRange | State::Unreadable => p.warn(s.as_str()),
    }
}

/// Columns padded to their widest cell, colour codes not counted
fn columns(rows: &[Vec<String>]) -> String {
    let n = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..n)
        .map(|i| {
            rows.iter()
                .filter_map(|r| r.get(i))
                .map(|c| vc_ui::visible_width(c))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut s = String::new();
    for r in rows {
        let mut line = String::new();
        for (i, c) in r.iter().enumerate() {
            line.push_str(c);
            if i + 1 < r.len() {
                let pad = widths[i].saturating_sub(vc_ui::visible_width(c)) + 2;
                line.push_str(&" ".repeat(pad));
            }
        }
        s.push_str(line.trim_end());
        s.push('\n');
    }
    s
}

fn table(n: &Needed, p: Paint) -> String {
    let mut s = String::new();
    if n.tools.is_empty() && n.toolchains.is_empty() && n.targets.is_empty() {
        return "nothing to list\n".to_owned();
    }
    if !n.tools.is_empty() {
        let mut rows = vec![
            [
                "tool",
                "tested",
                "found",
                "state",
                "nightly",
                "path",
                "shadowed by",
                "lints",
            ]
            .map(|h| p.paint(vc_ui::HEADING, h))
            .to_vec(),
        ];
        for t in &n.tools {
            let d = &t.found;
            rows.push(vec![
                p.id(&d.pin.name),
                d.pin.tested_text(),
                d.version
                    .as_ref()
                    .map_or_else(|_| "-".to_owned(), ToString::to_string),
                state_cell(p, d.state()),
                if d.pin.nightly { "nightly" } else { "" }.to_owned(),
                d.exe
                    .as_ref()
                    .map_or_else(|| "-".to_owned(), |e| e.display().to_string()),
                d.shadowed.as_ref().map_or_else(
                    || "-".to_owned(),
                    |s| format!("{} {}", s.version, s.exe.display()),
                ),
                t.lints.join(", "),
            ]);
        }
        s.push_str(&columns(&rows));
    }
    if !n.toolchains.is_empty() {
        let _ = writeln!(
            s,
            "\n{}",
            p.paint(vc_ui::HEADING, "toolchains (cargo-msrv)")
        );
        let rows: Vec<Vec<String>> = n
            .toolchains
            .iter()
            .map(|t| {
                vec![
                    format!("  {}", p.id(&t.version)),
                    t.missing
                        .as_ref()
                        .map_or_else(|| p.good("installed"), |m| p.err(m)),
                ]
            })
            .collect();
        s.push_str(&columns(&rows));
    }
    if !n.targets.is_empty() {
        let _ = writeln!(
            s,
            "\n{}",
            p.paint(vc_ui::HEADING, "targets (cargo-no-std, cargo-wasm)")
        );
        let rows: Vec<Vec<String>> = n
            .targets
            .iter()
            .map(|t| {
                let st = match t.installed {
                    Some(true) => p.good(t.state()),
                    Some(false) => p.err(t.state()),
                    None => p.warn(t.state()),
                };
                vec![format!("  {}", p.id(&t.name)), st]
            })
            .collect();
        s.push_str(&columns(&rows));
    }
    s.push_str(&next_hints(n, p));
    s
}

/// The commands that install what is missing or untested
fn next_hints(n: &Needed, p: Paint) -> String {
    let wanted: Vec<&Tool> = n.tools.iter().filter(|t| t.wanted()).collect();
    let mut hints = Vec::new();
    if wanted
        .iter()
        .any(|t| t.found.shadowed.is_none() && matches!(t.found.pin.kind, PinKind::Crate { .. }))
    {
        hints.push("spl tools needed --spec --missing | xargs -r cargo binstall -y".to_owned());
    }
    let components: Vec<&str> = wanted
        .iter()
        .filter(|t| t.found.shadowed.is_none())
        .filter_map(|t| match &t.found.pin.kind {
            PinKind::Component(c) => Some(c.as_str()),
            PinKind::Crate { .. } => None,
        })
        .collect();
    if !components.is_empty() {
        hints.push(format!("rustup component add {}", components.join(" ")));
    }
    for t in wanted.iter().filter(|t| t.found.shadowed.is_some()) {
        hints.push(t.found.fix_help());
    }
    for t in n.toolchains.iter().filter(|t| t.missing.is_some()) {
        hints.push(format!(
            "rustup toolchain install {} --profile minimal",
            t.version
        ));
    }
    let targets: Vec<&str> = n
        .targets
        .iter()
        .filter(|t| t.installed == Some(false))
        .map(|t| t.name.as_str())
        .collect();
    if !targets.is_empty() {
        hints.push(format!("rustup target add {}", targets.join(" ")));
    }
    if n.nightly_missing {
        hints.push(format!(
            "rustup toolchain install {} --profile minimal",
            n.nightly_channel
        ));
    }
    let mut s = String::new();
    if !hints.is_empty() {
        s.push('\n');
    }
    for h in hints {
        let _ = writeln!(s, "{}", p.next_hint(&format!("next: {h}")));
    }
    s
}
