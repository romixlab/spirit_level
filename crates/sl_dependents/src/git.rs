//! The few git questions asked of the library's checkout.

use std::path::Path;

/// git could not be run, or answered with an error
#[derive(Debug)]
pub enum GitError {
    /// git did not start
    Spawn {
        /// The arguments
        args: String,
        /// Why
        source: std::io::Error,
    },
    /// git exited with an error
    Failed {
        /// The arguments
        args: String,
        /// Its message
        stderr: String,
    },
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn { args, source } => write!(f, "git {args}: cannot run git: {source}"),
            Self::Failed { args, stderr } => write!(f, "git {args}: {stderr}"),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::Failed { .. } => None,
        }
    }
}

/// Runs git in `dir`; its stdout
pub(crate) fn run(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let joined = args.join(" ");
    let out = sl_model::command("git", dir)
        .args(args)
        .output()
        .map_err(|source| GitError::Spawn {
            args: joined.clone(),
            source,
        })?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(GitError::Failed {
            args: joined,
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        })
    }
}

/// The commit a revision names
pub(crate) fn commit(dir: &Path, rev: &str) -> Result<String, GitError> {
    Ok(run(
        dir,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ],
    )?
    .trim()
    .to_owned())
}

/// Whether `a` is an ancestor of `b`
pub(crate) fn is_ancestor(dir: &Path, a: &str, b: &str) -> bool {
    run(dir, &["merge-base", "--is-ancestor", a, b]).is_ok()
}

/// The commits in `to` that `from` lacks
pub(crate) fn count(dir: &Path, from: &str, to: &str) -> Option<u32> {
    run(dir, &["rev-list", "--count", &format!("{from}..{to}")])
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// NUL-separated paths of a git command
pub(crate) fn paths(dir: &Path, args: &[&str]) -> Result<Vec<String>, GitError> {
    Ok(run(dir, args)?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect())
}

/// The library's default branch: the local branch `origin/HEAD` names, else `origin/HEAD`'s target, else `main`
pub(crate) fn default_branch(dir: &Path) -> String {
    let Ok(remote) = run(
        dir,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) else {
        return "main".to_owned();
    };
    let remote = remote.trim();
    match remote.strip_prefix("origin/") {
        Some(local) if commit(dir, &format!("refs/heads/{local}")).is_ok() => local.to_owned(),
        _ => remote.to_owned(),
    }
}
