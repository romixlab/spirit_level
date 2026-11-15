//! The lints, each reading what the probe of one tool printed.

use crate::probe::{Cmd, Tool};
use crate::run::Run;
use crate::version;
use sl_analysis::Context;
use sl_core::{Confidence, Diagnostic, Span, glob};
use std::collections::BTreeSet;
use std::path::Path;

/// Every lint over one probed tool
pub(crate) fn run(cx: &Context<'_>, tool: &Tool) {
    let heading = cx.config.cli.commands_heading.as_str();
    examples(cx, tool, heading);
    help_text(cx, tool, heading);
    help_clean(cx, tool);
    completion(cx, tool);
    plain(cx, tool);
    color_flag(cx, tool, heading);
    usage_error(cx, tool);
    version_line(cx, tool);
    json_output(cx, tool, heading);
    standard_flags(cx, tool);
    secret_flags(cx, tool);
}

/// A finding about `tool`, keyed on the binary and `what`, placed at its main file
fn diag(id: &str, tool: &Tool, what: &str, message: String) -> Diagnostic {
    Diagnostic::new(id, format!("{}:{what}", tool.bin), message)
        .span(Span::new(&tool.file, 0))
        .krate(&tool.krate)
}

/// Whether a command's path (words after the binary) matches one of the `exempt` patterns
fn exempt(cmd: &Cmd, patterns: &[String]) -> bool {
    let path = cmd.words.join("/");
    patterns
        .iter()
        .any(|p| glob::matches(&p.replace(' ', "/"), &path))
}

/// Why a run gave nothing to read, when it did not exit with 0
fn failed(run: &Run) -> Option<String> {
    if run.timed_out {
        Some("did not exit within the time limit ([cli] timeout-secs)".to_owned())
    } else if run.code != Some(0) {
        let code = run
            .code
            .map_or_else(|| "a signal".to_owned(), |c| c.to_string());
        match run.stderr.lines().map(str::trim).find(|l| !l.is_empty()) {
            Some(line) => Some(format!("exited with {code}: {line}")),
            None => Some(format!("exited with {code}")),
        }
    } else {
        None
    }
}

fn examples(cx: &Context<'_>, tool: &Tool, heading: &str) {
    const ID: &str = "cli-help-examples";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    let headings: Vec<String> = p
        .list("headings")
        .iter()
        .map(|h| h.to_lowercase())
        .collect();
    for cmd in &tool.commands {
        let wanted = cmd.words.is_empty() || cmd.is_leaf(heading);
        if !wanted || exempt(cmd, p.list("exempt")) || failed(&cmd.long).is_some() {
            continue;
        }
        let lines: Vec<&str> = cmd.long.stdout.lines().map(str::trim).collect();
        let block = lines.iter().enumerate().any(|(i, l)| {
            headings.contains(&l.to_lowercase()) && lines[i + 1..].iter().any(|n| !n.is_empty())
        });
        if !block {
            let name = cmd.name(&tool.bin);
            cx.emit(
                diag(ID, tool, &name, format!("`{name} --help` has no Examples block"))
                    .help("end the long help with `Examples:` and a few real command lines (clap: `after_long_help`)"),
            );
        }
    }
}

fn help_text(cx: &Context<'_>, tool: &Tool, heading: &str) {
    const ID: &str = "cli-help-text";
    if !cx.enabled(ID) {
        return;
    }
    if tool.cut {
        cx.emit(
            diag(
                ID,
                tool,
                "commands-max",
                format!(
                    "{} has more commands than [cli] commands-max; the rest were not probed",
                    tool.bin
                ),
            )
            .confidence(Confidence::Estimate)
            .help("raise commands-max in the [cli] table of spirit_level.toml"),
        );
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for cmd in &tool.commands {
        let name = cmd.name(&tool.bin);
        if let Some(why) = failed(&cmd.short) {
            cx.emit(
                diag(ID, tool, &name, format!("`{name} -h` {why}"))
                    .help("-h prints the help and exits with 0"),
            );
            continue;
        }
        let Some(h) = &cmd.short_help else {
            cx.emit(
                diag(ID, tool, &name, format!("`{name} -h` printed no help in clap's layout (no `Usage:` line)"))
                    .confidence(Confidence::Estimate)
                    .help("a binary that is not a clap CLI is left out with `cli = false` in the crate map"),
            );
            continue;
        };
        if cmd.words.is_empty() && h.about.is_empty() {
            cx.emit(
                diag(
                    ID,
                    tool,
                    &name,
                    format!("`{name} -h` says nothing before `Usage:`"),
                )
                .help("give the command an `about`"),
            );
        }
        if let Some(sec) = h.section(heading) {
            for e in sec.entries.iter().filter(|e| e.own_help().is_empty()) {
                let sub = format!("{name} {}", e.name);
                cx.emit(
                    diag(
                        ID,
                        tool,
                        &sub,
                        format!("subcommand `{sub}` has no help line in `{name} -h`"),
                    )
                    .help("give it a doc comment or `about`"),
                );
            }
        }
        for e in h.args().filter(|e| e.own_help().is_empty()) {
            // a global option shows in every subcommand's help; one finding is enough
            if seen.insert(e.name.clone()) {
                cx.emit(
                    diag(
                        ID,
                        tool,
                        &format!("{name} {}", e.name),
                        format!("`{}` has no help line in `{name} -h`", e.name),
                    )
                    .help("give the argument a doc comment or `help`"),
                );
            }
        }
    }
}

fn help_clean(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-help-clean";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    for cmd in &tool.commands {
        let name = cmd.name(&tool.bin);
        for (flag, run) in [("-h", &cmd.short), ("--help", &cmd.long)] {
            let hit = run.stdout.lines().find_map(|l| {
                p.list("markers")
                    .iter()
                    .find(|m| l.contains(m.as_str()))
                    .map(|m| (l.trim(), m))
            });
            if let Some((line, marker)) = hit {
                cx.emit(
                    diag(ID, tool, &format!("{name} {flag}"), format!("`{name} {flag}` shows rustdoc source ({marker:?}): {line}"))
                        .help("a doc comment of a flattened struct or enum becomes help text; set `about`/`long_about` (or `help`) explicitly, or move the rustdoc to a `//` comment"),
                );
                break;
            }
        }
    }
}

fn completion(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-completion";
    if !cx.enabled(ID) {
        return;
    }
    let var = cx.config.params_for(ID, &tool.krate).str("env").to_owned();
    for (shell, run) in &tool.completions {
        let bin = &tool.bin;
        let what = format!("completion {shell}");
        let call = format!("`{var}={shell} {bin}`");
        if let Some(why) = failed(run) {
            cx.emit(diag(ID, tool, &what, format!("{call} {why}")).help("call clap_complete's `CompleteEnv::with_factory(..).complete()` (or `vc_complete::complete`) first in main"));
            continue;
        }
        let printed_help = run.stdout.lines().any(|l| l.starts_with("Usage:"));
        let keyword = match shell.as_str() {
            "zsh" => "compdef",
            "bash" | "fish" => "complete ",
            _ => "",
        };
        if printed_help || run.stdout.trim().is_empty() || !run.stdout.contains(keyword) {
            cx.emit(
                diag(ID, tool, &what, format!("{call} prints no {shell} completion registration"))
                    .help("call clap_complete's `CompleteEnv::with_factory(..).complete()` (or `vc_complete::complete`) first in main, so `source <(COMPLETE=bash BIN)` registers live completion"),
            );
        } else if shell == "bash" && run.stdout.contains("bashdefault") {
            cx.emit(
                diag(ID, tool, &what, format!("{call} registers with `-o bashdefault`: a failed completion falls back to file names"))
                    .help("register without `-o bashdefault` (vc_complete's bash adapter does)"),
            );
        }
    }
}

fn plain(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-plain-when-piped";
    if !cx.enabled(ID) {
        return;
    }
    let mut runs: Vec<(String, &Run)> = Vec::new();
    for cmd in &tool.commands {
        let name = cmd.name(&tool.bin);
        runs.push((format!("{name} -h"), &cmd.short));
        runs.push((format!("{name} --help"), &cmd.long));
    }
    if let Some(r) = &tool.version {
        runs.push((format!("{} --version", tool.bin), r));
    }
    if let Some(r) = &tool.usage {
        runs.push((format!("{} {}", tool.bin, crate::probe::UNKNOWN_FLAG), r));
    }
    let hit = runs.iter().find_map(|(call, r)| {
        let stream = if r.stdout.contains('\x1b') {
            "stdout"
        } else if r.stderr.contains('\x1b') {
            "stderr"
        } else {
            return None;
        };
        Some((call, stream))
    });
    if let Some((call, stream)) = hit {
        cx.emit(
            diag(ID, tool, "escapes", format!("`{call}` writes colour escapes to {stream}, which is a pipe"))
                .help("decide colour per stream: off when it is not a terminal or NO_COLOR is set (anstream does; clap: `ColorChoice::Auto`)"),
        );
    }
}

fn color_flag(cx: &Context<'_>, tool: &Tool, heading: &str) {
    const ID: &str = "cli-color-flag";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    let flag = p.str("flag");
    let Some(root) = tool.commands.first() else {
        return;
    };
    let Some(h) = &root.short_help else { return };
    let has = |c: &Cmd| {
        c.short_help
            .as_ref()
            .and_then(|h| h.args().find(|e| e.flags().1 == Some(flag)).cloned())
    };
    let Some(entry) = has(root) else {
        cx.emit(
            diag(ID, tool, flag, format!("{} takes no {flag} flag", tool.bin))
                .help("flatten `vc_ui::ColorFlag` into the root parser, or add `--color <WHEN>` (auto, always, never) with `global = true`"),
        );
        return;
    };
    let missing: Vec<&String> = p
        .list("values")
        .iter()
        .filter(|v| !entry.help.contains(v.as_str()))
        .collect();
    if !missing.is_empty() {
        let list = missing
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        cx.emit(
            diag(
                ID,
                tool,
                &format!("{flag} values"),
                format!("{} {flag} does not list {list} in -h", tool.bin),
            )
            .help("take auto, always and never (a clap ValueEnum lists them)"),
        );
    }
    let first_sub = crate::probe::subcommands(Some(h), heading).next();
    if let Some(sub) = first_sub
        && let Some(cmd) = tool.commands.iter().find(|c| c.words == [sub])
        && cmd.short_help.is_some()
        && has(cmd).is_none()
    {
        cx.emit(
            diag(
                ID,
                tool,
                &format!("{flag} global"),
                format!(
                    "{flag} is not global: `{} {sub} -h` does not list it",
                    tool.bin
                ),
            )
            .help("mark the flag `global = true`, so it works after the subcommand too"),
        );
    }
}

fn usage_error(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-usage-error";
    if !cx.enabled(ID) {
        return;
    }
    let Some(r) = &tool.usage else { return };
    let want = cx.config.params_for(ID, &tool.krate).int("code");
    let call = format!("`{} {}`", tool.bin, crate::probe::UNKNOWN_FLAG);
    let mut wrong = Vec::new();
    if r.timed_out {
        wrong.push("did not exit within the time limit".to_owned());
    } else if r.code.map(i64::from) != Some(want) {
        let code = r
            .code
            .map_or_else(|| "a signal".to_owned(), |c| c.to_string());
        wrong.push(format!("exits with {code}, not {want}"));
    }
    if !r.stdout.trim().is_empty() {
        wrong.push("writes to stdout".to_owned());
    }
    if r.stderr.trim().is_empty() {
        wrong.push("says nothing on stderr".to_owned());
    }
    if !wrong.is_empty() {
        cx.emit(
            diag(ID, tool, "unknown flag", format!("{call} {}", wrong.join(", ")))
                .help("let clap report usage errors (exit 2, message on stderr); a scripted caller tells a usage error from a failure by the code"),
        );
    }
}

fn version_line(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-version-line";
    if !cx.enabled(ID) {
        return;
    }
    let Some(r) = &tool.version else { return };
    let call = format!("`{} --version`", tool.bin);
    let help = "print `NAME VERSION (SHA[-dirty], commit D Mon YYYY HH:MM UTC)` from a worktree-safe build.rs (vc_build::emit and vc_build::version!)";
    if let Some(why) = failed(r) {
        cx.emit(diag(ID, tool, "--version", format!("{call} {why}")).help(help));
        return;
    }
    let line = r.stdout.lines().next().unwrap_or("").trim();
    let format = cx
        .config
        .params_for(ID, &tool.krate)
        .str("format")
        .to_owned();
    match version::check(line, &format, &tool.bin) {
        Err(why) => {
            cx.emit(
                diag(
                    ID,
                    tool,
                    "--version",
                    format!("{call} prints `{line}`: {why}"),
                )
                .help(help),
            );
        }
        Ok(f)
            if in_git(&cx.project.root)
                && (f.sha.as_deref() == Some("unknown")
                    || f.time.as_deref() == Some("unknown")) =>
        {
            cx.emit(
                diag(ID, tool, "--version unknown", format!("{call} prints `{line}` though it was built in a git checkout: the build script did not find git"))
                    .help("find the git dir from the build script's directory (`git rev-parse --git-dir`, so a worktree works) and rerun on its HEAD and index"),
            );
        }
        Ok(_) => {}
    }
}

fn json_output(cx: &Context<'_>, tool: &Tool, heading: &str) {
    const ID: &str = "cli-json-output";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    for cmd in tool.commands.iter().filter(|c| c.is_leaf(heading)) {
        let Some(h) = &cmd.short_help else { continue };
        if exempt(cmd, p.list("exempt")) {
            continue;
        }
        let has = h.args().any(|e| {
            e.flags()
                .1
                .is_some_and(|l| p.list("flags").iter().any(|f| f == l))
        });
        if !has {
            let name = cmd.name(&tool.bin);
            let flags = p.list("flags").join(" or ");
            cx.emit(
                diag(ID, tool, &name, format!("`{name}` takes no {flags}; assumed: every leaf command prints something a script may read"))
                    .confidence(Confidence::Estimate)
                    .help("add `--json` (one JSON value on stdout), or list the command in `exempt` when it prints no data"),
            );
        }
    }
}

fn standard_flags(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-standard-flags";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    let pairs: Vec<(&str, &str)> = p
        .list("pairs")
        .iter()
        .filter_map(|s| s.split_once('='))
        .collect();
    let mut seen = BTreeSet::new();
    for cmd in &tool.commands {
        let Some(h) = &cmd.short_help else { continue };
        for e in h.args() {
            let (Some(short), Some(long)) = e.flags() else {
                continue;
            };
            let Some((_, want)) = pairs.iter().find(|(s, _)| *s == short) else {
                continue;
            };
            if long != *want && seen.insert((short, long)) {
                let name = cmd.name(&tool.bin);
                cx.emit(
                    diag(ID, tool, &format!("{short} {long}"), format!("`{name}` uses {short} for {long}; {short} stands for {want} in every CLI"))
                        .help(format!("give {long} another short flag, or none")),
                );
            }
        }
    }
}

/// Whether an option entry takes a value, as clap shows it: a placeholder after the flags (`<N>`, `[N]`)
fn takes_value(name: &str) -> bool {
    name.contains('<') || name.contains('[')
}

/// The index right after `word` in `tokens`, when it occurs there as a contiguous run
fn word_end(tokens: &[&str], word: &[String]) -> Option<usize> {
    if word.is_empty() || word.len() > tokens.len() {
        return None;
    }
    tokens
        .windows(word.len())
        .position(|w| w.iter().zip(word).all(|(a, b)| a == b))
        .map(|i| i + word.len())
}

fn secret_flags(cx: &Context<'_>, tool: &Tool) {
    const ID: &str = "cli-secret-flags";
    if !cx.enabled(ID) {
        return;
    }
    let p = cx.config.params_for(ID, &tool.krate);
    let words: Vec<Vec<String>> = p
        .list("words")
        .iter()
        .map(|w| w.split('-').map(str::to_owned).collect())
        .collect();
    let safe = p.list("safe-suffixes");
    let mut seen = BTreeSet::new();
    for cmd in &tool.commands {
        let Some(h) = &cmd.short_help else { continue };
        let name = cmd.name(&tool.bin);
        for e in h.args().filter(|e| takes_value(&e.name)) {
            let Some(long) = e.flags().1 else { continue };
            if !seen.insert(long.to_owned()) {
                continue;
            }
            let tokens: Vec<&str> = long.trim_start_matches('-').split('-').collect();
            let Some(word) = words.iter().find(|w| {
                word_end(&tokens, w).is_some_and(|end| {
                    tokens
                        .get(end)
                        .copied()
                        .is_none_or(|next| !safe.iter().any(|s| s.as_str() == next))
                })
            }) else {
                continue;
            };
            cx.emit(
                diag(ID, tool, long, format!("`{name} {long}` takes a secret as its value ({})", word.join("-")))
                    .confidence(Confidence::Candidate)
                    .help("read it from a file (a `--*-file` flag) or stdin, or an environment variable; never as a flag's value, where the shell history and the process list both see it"),
            );
        }
    }
}

/// Whether the project root is inside a git checkout
fn in_git(root: &Path) -> bool {
    root.ancestors().any(|d| d.join(".git").exists())
}

#[cfg(test)]
mod tests {
    use super::word_end;

    fn tokens(flag: &str) -> Vec<&str> {
        flag.trim_start_matches('-').split('-').collect()
    }

    fn word(s: &str) -> Vec<String> {
        s.split('-').map(str::to_owned).collect()
    }

    #[test]
    fn finds_a_word_at_either_end() {
        assert_eq!(word_end(&tokens("--token"), &word("token")), Some(1));
        assert_eq!(word_end(&tokens("--auth-token"), &word("token")), Some(2));
        assert_eq!(word_end(&tokens("--api-key"), &word("api-key")), Some(2));
    }

    #[test]
    fn no_match_without_the_whole_word() {
        assert_eq!(word_end(&tokens("--name"), &word("token")), None);
        assert_eq!(word_end(&tokens("--api-keys"), &word("api-key")), None);
        assert_eq!(word_end(&tokens("--token"), &word("api-key")), None);
    }
}
