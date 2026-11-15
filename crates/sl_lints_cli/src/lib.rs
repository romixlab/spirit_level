//! CLI lints, one analysis: [`CLI`]. It builds every CLI binary of the workspace and runs it with arguments that
//! only print: `-h` and `--help` of every command, `--version`, an unknown flag and `COMPLETE=<shell>`. The lints
//! read what it printed, so a derive struct, the builder API and self-registering commands look the same.

mod checks;
mod help;
mod probe;
mod run;
mod version;

use probe::{Tool, Wants};
use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need};
use sl_model::{DepKind, Package};
use std::path::Path;
use std::time::Duration;

lint!(HELP_EXAMPLES, "C0020", "cli-help-examples", "cli", "--help of a CLI and of each of its leaf commands ends in an Examples block",
    params: [
        ("headings", StrList, "[\"Examples:\", \"Example:\"]", "headings that open an examples block, compared without case"),
        ("exempt", StrList, "[]", "subcommand paths without examples, words separated by spaces (`* add`; `*` one word, `**` any)"),
    ],
    needs: [Need::Tool("cargo")], fix: None);
lint!(HELP_TEXT, "C0021", "cli-help-text", "cli", "every command and argument of a CLI has a help line",
    params: [], needs: [Need::Tool("cargo")], fix: None);
lint!(HELP_CLEAN, "C0022", "cli-help-clean", "cli", "a CLI's help holds no rustdoc leftovers",
    params: [("markers", StrList, "[\"```\", \"[`\", \"`](\"]", "text that only rustdoc source has: code fences, intra-doc links")],
    needs: [Need::Tool("cargo")], fix: None);
lint!(COMPLETION, "C0023", "cli-completion", "cli", "a CLI prints its shell completion registration for bash, zsh and fish",
    params: [
        ("shells", StrList, "[\"bash\", \"zsh\", \"fish\"]", "the shells a registration is asked for"),
        ("env", Str, "\"COMPLETE\"", "the variable that asks the binary for its registration (clap_complete's CompleteEnv)"),
    ],
    needs: [Need::Tool("cargo")], fix: None);
lint!(PLAIN_WHEN_PIPED, "C0024", "cli-plain-when-piped", "cli", "a CLI writes no colour escapes into a pipe",
    params: [], needs: [Need::Tool("cargo")], fix: None);
lint!(COLOR_FLAG, "C0025", "cli-color-flag", "cli", "a CLI takes a global --color flag with auto, always and never",
    params: [
        ("flag", Str, "\"--color\"", "the colour flag"),
        ("values", StrList, "[\"auto\", \"always\", \"never\"]", "the values it takes"),
    ],
    needs: [Need::Tool("cargo")], fix: None);
lint!(USAGE_ERROR, "C0026", "cli-usage-error", "cli", "a CLI answers an unknown flag with exit code 2 and a message on stderr only",
    params: [("code", Int, "2", "the exit code of a usage error")],
    needs: [Need::Tool("cargo")], fix: None);
lint!(VERSION_LINE, "C0027", "cli-version-line", "cli", "--version prints the binary's name, version, git SHA and commit time",
    params: [("format", Str, "\"{name} {version} ({sha}, commit {time})\"", "the first line of --version: {name} the binary, {version}, {sha} short SHA with -dirty, {time} like `3 Oct 2026 18:20 UTC`")],
    needs: [Need::Tool("cargo")], fix: None);
lint!(JSON_OUTPUT, "C0028", "cli-json-output", "cli", "every leaf command of a CLI has a machine-readable output flag",
    params: [
        ("flags", StrList, "[\"--json\", \"--format\"]", "flags that select a machine-readable output"),
        ("exempt", StrList, "[]", "subcommand paths that print no data, words separated by spaces (`* add`; `*` one word, `**` any)"),
    ],
    needs: [Need::Tool("cargo")], fix: None);
lint!(STANDARD_FLAGS, "C0029", "cli-standard-flags", "cli", "a CLI's short flags keep their standard meaning",
    params: [("pairs", StrList, "[\"-h=--help\", \"-V=--version\", \"-v=--verbose\", \"-q=--quiet\", \"-n=--dry-run\", \"-a=--all\", \"-o=--output\", \"-y=--yes\"]", "short flags and the long flag each must stand for")],
    needs: [Need::Tool("cargo")], fix: None);
lint!(SECRET_FLAGS, "C0037", "cli-secret-flags", "cli", "no option takes a secret as its value",
    params: [
        ("words", StrList, "[\"token\", \"password\", \"secret\", \"api-key\"]", "words that mark a flag's long name as holding a secret"),
        ("safe-suffixes", StrList, "[\"file\", \"path\", \"env\", \"stdin\"]", "a word right after a match that means the flag points at the secret, not holds it"),
    ],
    needs: [Need::Tool("cargo")], fix: None);

/// The CLI analysis
pub struct Cli;

/// The registered instance
pub static CLI: Cli = Cli;

static LINTS: &[&LintMeta] = &[
    &HELP_EXAMPLES,
    &HELP_TEXT,
    &HELP_CLEAN,
    &COMPLETION,
    &PLAIN_WHEN_PIPED,
    &COLOR_FLAG,
    &USAGE_ERROR,
    &VERSION_LINE,
    &JSON_OUTPUT,
    &STANDARD_FLAGS,
    &SECRET_FLAGS,
];

impl Analysis for Cli {
    fn name(&self) -> &'static str {
        "cli"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let clis: Vec<&Package> = cx
            .project
            .packages
            .iter()
            .filter(|p| is_cli(cx, p))
            .collect();
        if clis.is_empty() {
            return;
        }
        let Some(cargo) = sl_tools::find("cargo", None) else {
            cx.not_run_all("tool:cargo", "cargo is not on PATH");
            return;
        };
        for pkg in clis {
            let exes = match build(cx, &cargo, pkg) {
                Ok(e) => e,
                Err(why) => {
                    cx.not_run_all("tool:cargo", &why);
                    return;
                }
            };
            for exe in exes {
                let mut tool = Tool {
                    krate: pkg.name.clone(),
                    bin: exe.name.clone(),
                    file: rel(&cx.project.root, &exe.src_path),
                    commands: Vec::new(),
                    version: None,
                    usage: None,
                    completions: Vec::new(),
                    cut: false,
                };
                if let Err(e) = probe_tool(cx, Path::new(&exe.path), &mut tool) {
                    cx.not_run_all("tool:cargo", &format!("cannot run {}: {e}", exe.path));
                    return;
                }
                checks::run(cx, &tool);
            }
        }
    }
}

/// Whether a member's binaries are CLIs: the map's `cli` key, else a binary target and a dependency on clap
fn is_cli(cx: &Context<'_>, pkg: &Package) -> bool {
    if !pkg.has_bin() {
        return false;
    }
    let marked = cx
        .map()
        .and_then(|m| m.crates.get(&pkg.name))
        .and_then(|c| c.cli);
    marked.unwrap_or_else(|| {
        pkg.deps
            .iter()
            .any(|d| d.name == "clap" && d.kind == DepKind::Normal)
    })
}

/// Builds a member's binaries; Err says why the build failed
fn build(
    cx: &Context<'_>,
    cargo: &Path,
    pkg: &Package,
) -> Result<Vec<sl_tools::cargo_json::Executable>, String> {
    let probe = &cx.config.cli;
    let target_dir = probe
        .target_dir
        .as_ref()
        .map(|d| cx.project.root.join(d).display().to_string());
    let mut args = vec![
        "build",
        "--message-format",
        "json",
        "-p",
        &pkg.name,
        "--bins",
    ];
    if let Some(d) = &target_dir {
        args.extend(["--target-dir", d.as_str()]);
    }
    args.extend(probe.build_args.iter().map(String::as_str));
    let out = sl_tools::run(cargo, &args, &cx.project.root, &[])
        .map_err(|e| format!("cannot run cargo: {e}"))?;
    let m = sl_tools::cargo_json::parse(&out.stdout);
    if m.success != Some(true) {
        return Err(format!(
            "cargo build -p {} failed: {}",
            pkg.name,
            out.failure()
        ));
    }
    Ok(m.executables)
}

/// Runs the probes the enabled lints need
fn probe_tool(cx: &Context<'_>, exe: &Path, tool: &mut Tool) -> std::io::Result<()> {
    let probe = &cx.config.cli;
    let help = [
        "cli-help-examples",
        "cli-help-text",
        "cli-help-clean",
        "cli-plain-when-piped",
        "cli-color-flag",
        "cli-json-output",
        "cli-standard-flags",
        "cli-secret-flags",
    ]
    .iter()
    .any(|id| cx.enabled(id));
    let p = cx.params("cli-completion");
    let wants = Wants {
        help,
        version: cx.enabled("cli-version-line") || cx.enabled("cli-plain-when-piped"),
        usage: cx.enabled("cli-usage-error") || cx.enabled("cli-plain-when-piped"),
        completion: cx
            .enabled("cli-completion")
            .then(|| (p.str("env"), p.list("shells"))),
        depth_max: probe.depth_max,
        commands_max: probe.commands_max,
        commands_heading: &probe.commands_heading,
    };
    let sb = run::Sandbox::new(
        &format!("{}-{}", tool.krate, tool.bin),
        Duration::from_secs(probe.timeout_secs),
    )?;
    probe::probe(&sb, exe, &wants, tool)
}

/// An absolute path relative to the project root, with `/`
fn rel(root: &Path, p: &str) -> String {
    Path::new(p)
        .strip_prefix(root)
        .map_or_else(|_| p.to_owned(), |r| r.to_string_lossy().replace('\\', "/"))
}
