//! External tools for `spirit_level`'s adapters.
//!
//! - [`pins`]: every tool's pinned version, from `tools.toml` shipped in this crate.
//! - [`Cache`]: builds of the pinned versions, keyed on crate, version, features and `rustc -vV`.
//! - [`find`]: a tool's executable, the cache first, then PATH.
//! - [`run`]: a tool started in the project with the outer cargo's environment removed.
//! - [`cargo_json`]: compiler messages of `cargo ... --message-format json`.
//!
//! ```no_run
//! let exe = sl_tools::find("cargo-deny", None).expect("installed");
//! let out = sl_tools::run(&exe, &["deny", "--format", "json", "check"], std::path::Path::new("."), &[]).unwrap();
//! println!("exit {:?}, {} bytes of diagnostics", out.code, out.stderr.len());
//! ```

pub mod cache;
pub mod cargo_json;
mod pins;
pub mod rustup;

pub use cache::{Cache, InstallError};
pub use pins::{Pin, PinKind, pin, pins};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The executable of a tool: its pinned build in the tool cache first, then PATH.
///
/// `path` replaces the process's PATH when given. `name` is a tool of [`pins`] (`cargo-deny`, `clippy`) or any
/// executable (`cargo`).
pub fn find(name: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    let exe = pin(name).map_or(name, |p| p.exe.as_str());
    if let Some(p) = pin(name)
        && let Some(found) = Cache::from_env().find(p)
    {
        return Some(found);
    }
    on_path(exe, path)
}

/// An executable on PATH (or on `path`)
pub fn on_path(exe: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    let path = path
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var_os("PATH"))?;
    std::env::split_paths(&path)
        .map(|d| d.join(exe))
        .find(|p| p.is_file())
}

/// What a tool run gave
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The exit code; None when a signal ended it
    pub code: Option<i32>,
    /// Its standard output
    pub stdout: String,
    /// Its standard error
    pub stderr: String,
}

impl Output {
    /// The last lines of standard error, for a reason line when the tool failed
    pub fn stderr_tail(&self, lines: usize) -> String {
        let v: Vec<&str> = self
            .stderr
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        v[v.len().saturating_sub(lines)..].join(" | ")
    }

    /// `exit 2: <last lines of stderr>`, the reason of a failed run
    pub fn failure(&self) -> String {
        let code = self
            .code
            .map_or_else(|| "killed by a signal".to_owned(), |c| format!("exit {c}"));
        format!("{code}: {}", self.stderr_tail(3))
    }
}

/// Runs `exe` with `args` in `dir` and the outer cargo's environment removed ([`sl_model::command`]), plus `env`
pub fn run(exe: &Path, args: &[&str], dir: &Path, env: &[(&str, &str)]) -> std::io::Result<Output> {
    let mut cmd = sl_model::command(exe, dir);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output()?;
    Ok(Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// Runs a cargo subcommand the way a person does: `cargo <args>`.
///
/// `args[0]` is the subcommand (`deny`). The directory of its executable `exe` (`.../cargo-deny`, maybe in the tool
/// cache) comes first on PATH, so cargo finds that build; a subcommand started directly would not know it runs under
/// cargo.
pub fn run_subcommand(
    exe: &Path,
    args: &[&str],
    dir: &Path,
    env: &[(&str, &str)],
) -> std::io::Result<Output> {
    let mut paths: Vec<PathBuf> = exe.parent().map(Path::to_path_buf).into_iter().collect();
    if let Some(p) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&p));
    }
    let path = std::env::join_paths(paths).map_err(std::io::Error::other)?;
    let path = path.to_string_lossy().into_owned();
    let mut all: Vec<(&str, &str)> = vec![("PATH", path.as_str())];
    all.extend_from_slice(env);
    run(Path::new("cargo"), args, dir, &all)
}

/// Runs `exe` like [`run`], with `input` on its standard input
pub fn run_with_input(
    exe: &Path,
    args: &[&str],
    dir: &Path,
    input: &str,
) -> std::io::Result<Output> {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = sl_model::command(exe, dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("no stdin"))?;
    let text = input.to_owned();
    let writer = std::thread::spawn(move || stdin.write_all(text.as_bytes()));
    let out = child.wait_with_output()?;
    writer
        .join()
        .map_err(|_panic| std::io::Error::other("stdin writer panicked"))??;
    Ok(Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// `rustc -vV` of the toolchain a tool is built with (the default one, outside any project), read once
pub fn rustc_vv() -> Option<&'static str> {
    static VV: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    VV.get_or_init(|| {
        let dir = std::env::temp_dir();
        let out = sl_model::command("rustc", &dir).arg("-vV").output().ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    })
    .as_deref()
}

/// The `need` and `reason` of a lint whose tool is missing: how to install it
pub fn missing_reason(name: &str) -> String {
    match pin(name).map(|p| &p.kind) {
        Some(PinKind::Component(c)) => {
            format!(
                "{name} is not installed: `rustup component add {c}` (or `spl tools install {name}`)"
            )
        }
        Some(PinKind::Crate { .. }) => {
            format!("{name} is not installed: `spl tools install {name}` builds the pinned version")
        }
        None => format!("{name} is not on PATH"),
    }
}

/// The line of a package's `[[package]]` entry in the project's `Cargo.lock` (0 when it is not there): where a
/// finding about a dependency points
pub fn lock_line(project: &sl_model::Project, name: &str, version: &str) -> u32 {
    let Some(text) = project.read("Cargo.lock") else {
        return 0;
    };
    let lines: Vec<&str> = text.lines().collect();
    let want_name = format!("name = \"{name}\"");
    let want_version = format!("version = \"{version}\"");
    lines
        .iter()
        .enumerate()
        .find(|(i, l)| {
            l.trim() == want_name
                && (version.is_empty()
                    || lines.get(i + 1).is_some_and(|n| n.trim() == want_version))
        })
        .map_or(0, |(i, _)| i as u32 + 1)
}
