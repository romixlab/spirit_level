//! The cargo-udeps adapter: `cargo +nightly udeps --output json --workspace --all-targets`.
//!
//! cargo-udeps prints one JSON document: `success`, and `unused_deps`, a map from package id to the package's
//! `manifest_path` and its unused `normal`, `development` and `build` dependencies. It exits 1 when it found
//! any; a failed build is an error on stderr.

use serde_json::Value;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};

lint!(UDEPS_UNUSED_DEPS, "X0033", "udeps-unused-deps", "udeps", "no crate declares a dependency the compiler never loads",
    tool: ("cargo-udeps", Executable, docs: "https://github.com/est31/cargo-udeps"), params: [], needs: [Need::Tool("cargo-udeps"), Need::Nightly], fix: None);

/// The cargo-udeps analysis
pub struct Udeps;

/// The registered instance
pub static UDEPS: Udeps = Udeps;

static LINTS: &[&LintMeta] = &[&UDEPS_UNUSED_DEPS];

impl Analysis for Udeps {
    fn name(&self) -> &'static str {
        "cargo-udeps"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("cargo-udeps", None) else {
            cx.not_run_all("tool:cargo-udeps", &sl_tools::missing_reason("cargo-udeps"));
            return;
        };
        let args = ["udeps", "--output", "json", "--workspace", "--all-targets"];
        let out = match sl_tools::run_subcommand(
            &exe,
            &args,
            &p.root,
            &[("RUSTUP_TOOLCHAIN", "nightly")],
        ) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:cargo-udeps", &format!("cannot run cargo-udeps: {e}"));
                return;
            }
        };
        let Some(found) = parse(&out.stdout) else {
            cx.not_run_all("run:cargo-udeps", &format!("cargo udeps {}", out.failure()));
            return;
        };
        for (manifest, kind, dep) in found {
            let Some(pkg) = sl_tools::cargo_json::package_of(p, &manifest) else {
                continue;
            };
            let table = match kind.as_str() {
                "development" => "dev-dependencies",
                "build" => "build-dependencies",
                _ => "dependencies",
            };
            let line = pkg.manifest.line(&[table, &dep]);
            cx.emit(
                Diagnostic::new(
                    "udeps-unused-deps",
                    format!("crate:{}:{table}:{dep}", pkg.name),
                    format!("{} never uses {dep} of its [{table}]", pkg.name),
                )
                .span(Span::new(pkg.manifest.rel.clone(), line))
                .krate(&pkg.name)
                .help("remove the dependency; one used only in doc tests goes into [package.metadata.cargo-udeps.ignore]"),
            );
        }
    }
}

/// `(manifest path, kind, dependency)` of each unused dependency; None when stdout is not cargo-udeps's document
pub fn parse(stdout: &str) -> Option<Vec<(String, String, String)>> {
    let v: Value = serde_json::from_str(stdout.trim()).ok()?;
    let mut out = Vec::new();
    for (_, pkg) in v.get("unused_deps")?.as_object()? {
        let manifest = pkg
            .get("manifest_path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        for kind in ["normal", "development", "build"] {
            for d in pkg
                .get(kind)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(d) = d.as_str() {
                    out.push((manifest.to_owned(), kind.to_owned(), d.to_owned()));
                }
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn unused() {
        let got = super::parse(include_str!("../tests/samples/udeps.json")).unwrap();
        assert_eq!(
            got,
            vec![(
                "/work/sample/Cargo.toml".to_owned(),
                "normal".to_owned(),
                "itoa".to_owned()
            )]
        );
        assert!(super::parse("error").is_none());
    }
}
