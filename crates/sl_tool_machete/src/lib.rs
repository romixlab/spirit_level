//! The cargo-machete adapter: `cargo machete --skip-target-dir`, one finding per unused dependency.
//!
//! cargo-machete prints, per crate with unused dependencies, a line `<crate> -- <path>/Cargo.toml:` followed by
//! one tab-indented line per dependency, and exits 1 when it found any, 2 on an error.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};

lint!(MACHETE_UNUSED_DEPS, "X0026", "machete-unused-deps", "machete", "no crate declares a dependency its sources never use",
    tool: ("cargo-machete", Executable), params: [], needs: [Need::Tool("cargo-machete")], fix: None);

/// The cargo-machete analysis
pub struct Machete;

/// The registered instance
pub static MACHETE: Machete = Machete;

static LINTS: &[&LintMeta] = &[&MACHETE_UNUSED_DEPS];

impl Analysis for Machete {
    fn name(&self) -> &'static str {
        "cargo-machete"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("cargo-machete", None) else {
            cx.not_run_all(
                "tool:cargo-machete",
                &sl_tools::missing_reason("cargo-machete"),
            );
            return;
        };
        let out = match sl_tools::run_subcommand(
            &exe,
            &["machete", "--skip-target-dir", "."],
            &p.root,
            &[],
        ) {
            Ok(o) if matches!(o.code, Some(0 | 1)) => o,
            Ok(o) => {
                cx.not_run_all(
                    "run:cargo-machete",
                    &format!("cargo machete {}", o.failure()),
                );
                return;
            }
            Err(e) => {
                cx.not_run_all(
                    "run:cargo-machete",
                    &format!("cannot run cargo-machete: {e}"),
                );
                return;
            }
        };
        for (manifest, dep) in parse(&out.stdout) {
            let manifest = manifest.trim_start_matches("./").to_owned();
            let dir = manifest
                .strip_suffix("Cargo.toml")
                .unwrap_or("")
                .trim_end_matches('/');
            let Some(pkg) = p.packages.iter().find(|k| k.dir == dir) else {
                continue;
            };
            let line = pkg
                .deps
                .iter()
                .find(|d| d.key() == dep)
                .map_or(0, |d| pkg.manifest.line(&d.manifest_path()));
            cx.emit(
                Diagnostic::new(
                    "machete-unused-deps",
                    format!("crate:{}:{dep}", pkg.name),
                    format!("{} depends on {dep} but its sources never use it", pkg.name),
                )
                .span(Span::new(manifest, line))
                .krate(&pkg.name)
                .help("remove the dependency; a false positive goes into [package.metadata.cargo-machete] ignored"),
            );
        }
    }
}

/// `(manifest path as printed, dependency)` of each unused dependency
pub fn parse(stdout: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut manifest: Option<String> = None;
    for line in stdout.lines() {
        if let Some(dep) = line.strip_prefix('\t') {
            if let Some(m) = &manifest {
                out.push((m.clone(), dep.trim().to_owned()));
            }
        } else if let Some((_, m)) = line.split_once(" -- ")
            && let Some(m) = m.strip_suffix(':')
        {
            manifest = Some(m.to_owned());
        } else {
            manifest = None;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn unused_per_crate() {
        let got = super::parse(include_str!("../tests/samples/machete.txt"));
        assert_eq!(
            got,
            vec![
                ("./crates/a/Cargo.toml".to_owned(), "itoa".to_owned()),
                ("./crates/b/Cargo.toml".to_owned(), "ryu".to_owned()),
            ]
        );
    }
}
