//! `spl commit-msg FILE`: the message-only lints (`commit-subject-length`, `commit-body-lines`,
//! `commit-tested-line`) on one message, before it is a commit. No cargo metadata and no git history: a commit-msg
//! hook can run it on every commit, fast. `commit-prose` (filler, narration, emoji) and `fixes-tag` still need the
//! branch's history and stay in `spl check`.

use crate::registry::{Registered, Run};
use sl_config::{Config, Env};
use sl_core::Diagnostic;
use sl_model::git::Commit;
use std::path::{Path, PathBuf};
use vc_ui::Exit;

/// Check one commit message against the project's message-only lints
#[derive(clap::Parser)]
#[command(
    name = "commit-msg",
    display_order = 14,
    after_help = "Examples:\n  spl commit-msg .git/COMMIT_EDITMSG\n\nExit codes: 0 clean, or no spirit_level.toml above the working directory; 1 a denied finding; 2 a configuration error"
)]
pub(crate) struct CommitMsg {
    /// The file holding the proposed commit message (git's own commit-msg hook passes it as `$1`)
    file: PathBuf,
    /// Today's date for expiries (YYYY-MM-DD); default: the system clock
    #[arg(long, env = "SPIRIT_LEVEL_TODAY")]
    today: Option<String>,
}

inventory::submit! { Registered::of::<CommitMsg>() }

impl Run for CommitMsg {
    fn run(self) -> Result<Exit, String> {
        commit_msg(&self.file, self.today)
    }
}

/// A commit, for a message that is not one yet: `file()` reads `commit:workdir` in a finding
const PENDING: &str = "workdir";

/// How a message-only lint builds its finding from the message and the configuration
type Check = fn(&Commit, &Config) -> Option<Diagnostic>;

/// Each message-only lint's id and its check
const CHECKS: &[(&str, Check)] = &[
    ("commit-subject-length", |c, cfg| {
        sl_lints_prose::subject_diagnostic(
            c,
            cfg.lint("commit-subject-length").params.count("max-chars"),
        )
    }),
    ("commit-body-lines", |c, cfg| {
        sl_lints_prose::body_diagnostic(c, cfg.lint("commit-body-lines").params.count("max-lines"))
    }),
    ("commit-tested-line", |c, cfg| {
        sl_lints_prose::tested_diagnostic(c, cfg.lint("commit-tested-line").params.str("trailer"))
    }),
];

/// `spirit_level.toml` at `dir` or the closest ancestor that has one
fn find_root(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .find(|d| d.join(sl_config::FILE).is_file())
        .map(Path::to_owned)
}

/// Strips git's own comment lines (`#`, the default `core.commentChar`): present when the message still carries
/// the editor's template (`cleanup=strip`), absent from one written with `-m`/`-F` (`cleanup=whitespace`)
fn strip_comments(text: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn commit_msg(file: &Path, today: Option<String>) -> Result<Exit, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("the working directory: {e}"))?;
    let Some(root) = find_root(&cwd) else {
        return Ok(Exit::SUCCESS);
    };
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let Some(commit) = sl_model::git::parse(PENDING, &strip_comments(&text)) else {
        return Ok(Exit::SUCCESS);
    };
    let lints = spirit_level::lints();
    let env = Env {
        lints: &lints,
        tier: &spirit_level::tier,
        today: crate::today(today)?,
        version: None,
        ladder: spirit_level::LADDER,
    };
    let config = Config::load(&root, None, &env).map_err(|errs| {
        errs.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let mut diagnostics = Vec::new();
    for &(id, check) in CHECKS {
        if config.enabled(id)
            && let Some(d) = check(&commit, &config)
        {
            diagnostics.push(d);
        }
    }
    if diagnostics.is_empty() {
        return Ok(Exit::SUCCESS);
    }
    sl_engine::apply_levels(&mut diagnostics, &config);
    for d in &diagnostics {
        let code = spirit_level::lint(&d.lint).map_or_else(String::new, |m| m.code.to_owned());
        println!("{code} {}: {}", d.lint, d.message);
        if let Some(h) = &d.help {
            println!("  {h}");
        }
    }
    if diagnostics.iter().any(Diagnostic::fails) {
        Ok(crate::DENIED)
    } else {
        Ok(Exit::SUCCESS)
    }
}
