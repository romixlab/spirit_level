//! Checks driven by cargo itself: the MSRV build, the no_std and wasm builds of the crates the map marks, and
//! `cargo doc` without warnings.
//!
//! Each runs cargo with `--message-format json` and reports the compiler's messages ([`sl_tools::cargo_json`]).
//! A build that fails without a compiler message (a dependency that needs a newer compiler, a resolver error) is
//! one finding with cargo's error lines. A toolchain or target that is not installed makes the lint not run, with
//! the command that installs it. The builds of other toolchains and targets go to their own target directories
//! under `target/spirit-level/`, so they never invalidate the normal build.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};
use sl_model::Package;
use sl_tools::Output;
use sl_tools::cargo_json::{self, Message};
use std::collections::BTreeMap;

lint!(CARGO_MSRV, "cargo-msrv", "build", "every crate builds with the toolchain of its declared rust-version",
    tool: ("cargo", Executable),
    params: [("args", StrList, "[]", "more arguments to `cargo check` (features, targets)")],
    needs: [Need::Tool("cargo")], fix: None);
lint!(CARGO_NO_STD, "cargo-no-std", "build", "every crate the map marks no_std builds for targets without std",
    tool: ("cargo", Executable),
    params: [("targets", StrList, "[\"thumbv7em-none-eabihf\"]", "the targets without std each such crate builds for"),
             ("args", StrList, "[\"--no-default-features\"]", "more arguments to `cargo check`")],
    needs: [Need::Tool("cargo"), Need::Map], fix: None);
lint!(CARGO_WASM, "cargo-wasm", "build", "every crate the map marks wasm builds for wasm32-unknown-unknown",
    tool: ("cargo", Executable),
    params: [("target", Str, "\"wasm32-unknown-unknown\"", "the wasm target"),
             ("args", StrList, "[]", "more arguments to `cargo check`")],
    needs: [Need::Tool("cargo"), Need::Map], fix: None);
lint!(CARGO_DOC, "cargo-doc", "build", "cargo doc builds the workspace's documentation without a warning",
    tool: ("cargo", Executable),
    params: [("args", StrList, "[\"--workspace\", \"--no-deps\"]", "arguments to `cargo doc`")],
    needs: [Need::Tool("cargo")], fix: None);

/// The cargo checks
pub struct CargoChecks;

/// The registered instance
pub static CARGO_CHECKS: CargoChecks = CargoChecks;

static LINTS: &[&LintMeta] = &[&CARGO_MSRV, &CARGO_NO_STD, &CARGO_WASM, &CARGO_DOC];

impl Analysis for CargoChecks {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let Some(cargo) = sl_tools::find("cargo", None) else {
            cx.not_run_all("tool:cargo", "cargo is not on PATH");
            return;
        };
        let run = Runner { cx, cargo };
        if cx.enabled("cargo-msrv") {
            run.msrv();
        }
        if cx.enabled("cargo-no-std") {
            run.no_std();
        }
        if cx.enabled("cargo-wasm") {
            run.wasm();
        }
        if cx.enabled("cargo-doc") {
            run.doc();
        }
    }
}

struct Runner<'a, 'b> {
    cx: &'a Context<'b>,
    cargo: std::path::PathBuf,
}

/// What one cargo run gave
enum Run {
    /// It ran: its messages, and the error lines of a failed build
    Done(Vec<Message>, Option<String>),
    /// It could not run: need and reason
    NotRun(String, String),
}

impl Runner<'_, '_> {
    fn cargo(&self, args: &[String], toolchain: Option<&str>) -> Run {
        // rustup may be set to install a missing toolchain by itself: never download one behind the user's back
        let env: Vec<(&str, &str)> = toolchain
            .map(|t| vec![("RUSTUP_TOOLCHAIN", t), ("RUSTUP_AUTO_INSTALL", "0")])
            .unwrap_or_default();
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let out: Output = match sl_tools::run(&self.cargo, &argv, &self.cx.project.root, &env) {
            Ok(o) => o,
            Err(e) => return Run::NotRun("run:cargo".into(), format!("cannot run cargo: {e}")),
        };
        if let Some(t) = toolchain
            && is_missing_toolchain(&out.stderr)
        {
            return Run::NotRun(
                format!("toolchain:{t}"),
                format!(
                    "the toolchain {t} is not installed: `rustup toolchain install {t} --profile minimal`"
                ),
            );
        }
        let msgs = cargo_json::parse(&out.stdout);
        let failed = msgs.success != Some(true);
        let errors = failed.then(|| cargo_errors(&out.stderr).unwrap_or_else(|| out.failure()));
        Run::Done(msgs.messages, errors)
    }

    /// Reports a run: compiler messages at `levels`, and a failed build without an error message as one finding
    fn report(
        &self,
        lint: &str,
        what: &str,
        run: Run,
        levels: &[&str],
        anchor: Span,
        krate: Option<&str>,
    ) {
        let cx = self.cx;
        let (msgs, errors) = match run {
            Run::Done(m, e) => (m, e),
            Run::NotRun(need, reason) => {
                cx.not_run(lint, need, reason);
                return;
            }
        };
        let mut reported = false;
        for m in msgs.iter().filter(|m| levels.contains(&m.level.as_str())) {
            let Some(span) = m.span_in(&cx.project.root) else {
                continue;
            };
            reported |= m.level == "error";
            let code = m
                .code
                .as_deref()
                .map_or(String::new(), |c| format!(" [{c}]"));
            let mut d = Diagnostic::new(
                lint,
                format!("{what}:{}", m.item()),
                format!("{what}: {}{code}", m.message),
            )
            .span(span)
            .help(
                m.help
                    .clone()
                    .unwrap_or_else(|| "fix the code the compiler points at".into()),
            );
            if let Some(p) = m.package(cx.project) {
                d = d.krate(&p.name);
            }
            cx.emit(d);
        }
        if let Some(e) = errors
            && !reported
        {
            let mut d = Diagnostic::new(lint, format!("{what}:build"), format!("{what}: the build failed: {e}"))
                .span(anchor)
                .help("cargo's error says what is missing: a dependency that needs a newer compiler, a feature that pulls std, a target without a crate");
            if let Some(k) = krate {
                d = d.krate(k);
            }
            cx.emit(d);
        }
    }

    fn target_dir(&self, name: &str) -> String {
        self.cx
            .project
            .root
            .join("target/spirit-level")
            .join(name)
            .display()
            .to_string()
    }

    fn msrv(&self) {
        let p = self.cx.project;
        let mut by_version: BTreeMap<&str, Vec<&Package>> = BTreeMap::new();
        for pkg in &p.packages {
            if let Some(v) = pkg.rust_version.as_deref() {
                by_version.entry(v).or_default().push(pkg);
            }
        }
        let extra = self.cx.params("cargo-msrv").list("args").to_vec();
        for (version, pkgs) in by_version {
            let mut args: Vec<String> = vec![
                "check".into(),
                "--message-format".into(),
                "json".into(),
                "--target-dir".into(),
                self.target_dir(&format!("msrv-{version}")),
            ];
            for k in &pkgs {
                args.extend(["-p".into(), k.name.clone()]);
            }
            args.extend(extra.iter().cloned());
            let first = pkgs[0];
            let anchor = Span::new(
                p.root_manifest.rel.clone(),
                p.root_manifest
                    .line(&["workspace", "package", "rust-version"]),
            );
            let anchor = if anchor.line == 0 {
                Span::new(
                    first.manifest.rel.clone(),
                    first.manifest.line(&["package", "rust-version"]),
                )
            } else {
                anchor
            };
            if let Some((need, reason)) = self.wrong_toolchain(version) {
                self.cx.not_run("cargo-msrv", need, reason);
                return;
            }
            let run = self.cargo(&args, Some(version));
            let krate = (pkgs.len() == 1).then_some(first.name.as_str());
            self.report(
                "cargo-msrv",
                &format!("rust {version}"),
                run,
                &["error"],
                anchor,
                krate,
            );
        }
    }

    /// Why the toolchain `version` cannot be selected: not installed, or no rustup (a distribution's cargo
    /// ignores RUSTUP_TOOLCHAIN and would build with itself, so the check would pass on the wrong compiler)
    fn wrong_toolchain(&self, version: &str) -> Option<(String, String)> {
        let env = [("RUSTUP_TOOLCHAIN", version), ("RUSTUP_AUTO_INSTALL", "0")];
        let out = sl_tools::run(
            std::path::Path::new("rustc"),
            &["-V"],
            &self.cx.project.root,
            &env,
        )
        .ok()?;
        if is_missing_toolchain(&out.stderr) {
            return Some((
                format!("toolchain:{version}"),
                format!(
                    "the toolchain {version} is not installed: `rustup toolchain install {version} --profile minimal`"
                ),
            ));
        }
        let got = out.stdout.trim();
        (!got.starts_with(&format!("rustc {version}"))).then(|| {
            (
                format!("toolchain:{version}"),
                format!("asked for rust {version}, got `{got}`: the MSRV build needs rustup to select the toolchain"),
            )
        })
    }

    fn marked(&self, pick: fn(&sl_config::MapCrate) -> bool) -> Vec<&Package> {
        let Some(map) = self.cx.map() else {
            return Vec::new();
        };
        self.cx
            .project
            .packages
            .iter()
            .filter(|k| map.crates.get(&k.name).is_some_and(pick))
            .collect()
    }

    fn map_anchor(&self, krate: &str) -> Span {
        self.cx.map().map_or_else(
            || Span::new("spirit_level.toml", 0),
            |m| {
                Span::new(
                    m.file.clone(),
                    m.crates.get(krate).map_or(m.line, |c| c.line),
                )
            },
        )
    }

    fn installed_targets(&self) -> Option<Vec<String>> {
        let out = sl_tools::run(
            std::path::Path::new("rustup"),
            &["target", "list", "--installed"],
            &self.cx.project.root,
            &[],
        )
        .ok()?;
        (out.code == Some(0)).then(|| out.stdout.lines().map(|l| l.trim().to_string()).collect())
    }

    fn cross(&self, lint: &str, crates: Vec<&Package>, targets: &[String], extra: &[String]) {
        if crates.is_empty() {
            return;
        }
        // without rustup (a distribution's toolchain) cargo says itself when a target is missing
        let installed = self.installed_targets();
        for t in targets {
            if let Some(i) = &installed
                && !i.contains(t)
            {
                self.cx.not_run(
                    lint,
                    format!("target:{t}"),
                    format!("the target {t} is not installed: `rustup target add {t}`"),
                );
                return;
            }
            for k in &crates {
                let mut args: Vec<String> = vec![
                    "check".into(),
                    "--message-format".into(),
                    "json".into(),
                    "--target-dir".into(),
                    self.target_dir(lint),
                    "-p".into(),
                    k.name.clone(),
                    "--target".into(),
                    t.clone(),
                ];
                args.extend(extra.iter().cloned());
                let run = self.cargo(&args, None);
                self.report(
                    lint,
                    &format!("{} for {t}", k.name),
                    run,
                    &["error"],
                    self.map_anchor(&k.name),
                    Some(&k.name),
                );
            }
        }
    }

    fn no_std(&self) {
        let params = self.cx.params("cargo-no-std");
        let crates = self.marked(|c| c.no_std);
        self.cross(
            "cargo-no-std",
            crates,
            params.list("targets"),
            params.list("args"),
        );
    }

    fn wasm(&self) {
        let params = self.cx.params("cargo-wasm");
        let crates = self.marked(|c| c.wasm);
        let target = vec![params.str("target").to_string()];
        self.cross("cargo-wasm", crates, &target, params.list("args"));
    }

    fn doc(&self) {
        let p = self.cx.project;
        let mut args: Vec<String> = vec!["doc".into(), "--message-format".into(), "json".into()];
        args.extend(self.cx.params("cargo-doc").list("args").iter().cloned());
        let run = self.cargo(&args, None);
        let anchor = Span::new(p.root_manifest.rel.clone(), 0);
        self.report(
            "cargo-doc",
            "cargo doc",
            run,
            &["warning", "error"],
            anchor,
            None,
        );
    }
}

/// Whether rustup refused because the toolchain is not installed
fn is_missing_toolchain(stderr: &str) -> bool {
    stderr.lines().any(|l| {
        let l = l.to_lowercase();
        (l.contains("toolchain") && l.contains("is not installed"))
            || l.contains("no such toolchain")
    })
}

/// The `error:` lines cargo wrote to stderr (outside the JSON messages), with what follows each on the same
/// topic, in one line
fn cargo_errors(stderr: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    let mut take = false;
    for l in stderr.lines() {
        if let Some(e) = l.strip_prefix("error: ") {
            out.push(e.trim().to_string());
            take = true;
        } else if take && l.starts_with("  ") && !l.trim().is_empty() {
            if let Some(last) = out.last_mut() {
                last.push(' ');
                last.push_str(l.trim());
            }
        } else {
            take = false;
        }
    }
    (!out.is_empty()).then(|| out.join(" | "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolchain_missing() {
        assert!(is_missing_toolchain(include_str!(
            "../tests/samples/msrv-missing.txt"
        )));
        assert!(!is_missing_toolchain("error: could not compile `x`"));
    }

    #[test]
    fn errors_outside_json() {
        let e = cargo_errors(include_str!("../tests/samples/msrv-too-new.txt")).unwrap();
        assert!(e.contains("requires rustc"), "{e}");
    }

    #[test]
    fn rustdoc_warnings() {
        let m = cargo_json::parse(include_str!("../tests/samples/doc.jsonl"));
        let w: Vec<&Message> = m
            .messages
            .iter()
            .filter(|m| m.level == "warning" && m.span.is_some())
            .collect();
        assert_eq!(w.len(), 1);
        assert_eq!(
            w[0].code.as_deref(),
            Some("rustdoc::broken_intra_doc_links")
        );
        assert_eq!(m.success, Some(true));
    }
}
