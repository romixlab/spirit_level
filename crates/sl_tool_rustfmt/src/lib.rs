//! The rustfmt adapter: `cargo fmt --all --check --message-format short`, one finding per unformatted file.
//!
//! In the short format cargo fmt prints the absolute path of each file rustfmt would change, one per line, and
//! exits 1 when there is any; a file that does not parse is an error on stderr, also with exit 1.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};
use std::path::Path;

lint!(RUSTFMT_UNFORMATTED, "X0027", "rustfmt-unformatted", "rustfmt", "every Rust file is formatted the way cargo fmt writes it",
    tool: ("rustfmt", Executable), params: [], needs: [Need::Tool("rustfmt")], fix: None);

/// The rustfmt analysis
pub struct Rustfmt;

/// The registered instance
pub static RUSTFMT: Rustfmt = Rustfmt;

static LINTS: &[&LintMeta] = &[&RUSTFMT_UNFORMATTED];

impl Analysis for Rustfmt {
    fn name(&self) -> &'static str {
        "rustfmt"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let root = &cx.project.root;
        let Some(exe) = sl_tools::find("rustfmt", None) else {
            cx.not_run_all("tool:rustfmt", &sl_tools::missing_reason("rustfmt"));
            return;
        };
        let args = ["fmt", "--all", "--check", "--message-format", "short"];
        let out = match sl_tools::run_subcommand(&exe, &args, root, &[]) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:rustfmt", &format!("cannot run cargo fmt: {e}"));
                return;
            }
        };
        let files = parse(&out.stdout, root);
        if out.code != Some(0) && files.is_empty() {
            cx.not_run_all("run:rustfmt", &format!("cargo fmt {}", out.failure()));
            return;
        }
        for f in files {
            let krate = cx
                .project
                .packages
                .iter()
                .filter(|p| p.dir.is_empty() || f.starts_with(&format!("{}/", p.dir)))
                .max_by_key(|p| p.dir.len())
                .map(|p| p.name.clone());
            let mut d = Diagnostic::new(
                "rustfmt-unformatted",
                f.clone(),
                format!("{f} is not formatted"),
            )
            .span(Span::new(&f, 0))
            .help("run `cargo fmt --all`");
            if let Some(k) = krate {
                d = d.krate(k);
            }
            cx.emit(d);
        }
    }
}

/// The files cargo fmt would change, relative to `root`
pub fn parse(stdout: &str, root: &Path) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| l.ends_with(".rs"))
        .map(|l| {
            Path::new(l).strip_prefix(root).map_or_else(
                |_| l.to_string(),
                |r| r.to_string_lossy().replace('\\', "/"),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn files_relative_to_root() {
        let got = super::parse(
            include_str!("../tests/samples/fmt-short.txt"),
            std::path::Path::new("/work/sample"),
        );
        assert_eq!(got, vec!["src/lib.rs".to_string()]);
    }
}
