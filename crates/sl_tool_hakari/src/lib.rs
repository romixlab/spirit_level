//! The hakari adapter: the workspace-hack package, checked through the hakari library.
//!
//! The configuration is `.config/hakari.toml` (or `.guppy/hakari.toml`), as cargo-hakari reads it. The package
//! graph comes from a full `cargo metadata`. `hakari-generate` computes the workspace-hack's dependencies and
//! compares them with the generated section of its `Cargo.toml` (`cargo hakari generate --diff`); its fix
//! writes the section. `hakari-manage-deps` lists the members whose dependency on the workspace-hack is missing
//! or should go (`cargo hakari manage-deps --dry-run`).
//!
//! The library is linked only with the `link` feature; without it the lints report that they did not run.

use sl_analysis::{Analysis, Context, lint};
use sl_core::{LintMeta, Need};

lint!(HAKARI_GENERATE, "hakari-generate", "hakari", "the workspace-hack package's dependencies are the ones hakari computes",
    tool: ("hakari", Library), params: [], needs: [Need::Resolve], fix: Exact);
lint!(HAKARI_MANAGE_DEPS, "hakari-manage-deps", "hakari", "every member hakari manages depends on the workspace-hack package, and no other",
    tool: ("hakari", Library), params: [], needs: [Need::Resolve], fix: None);

/// The hakari analysis
pub struct Hakari;

/// The registered instance
pub static HAKARI: Hakari = Hakari;

static LINTS: &[&LintMeta] = &[&HAKARI_GENERATE, &HAKARI_MANAGE_DEPS];

/// Where cargo-hakari looks for its configuration, in order
pub const CONFIG_PATHS: &[&str] = &[".config/hakari.toml", ".guppy/hakari.toml"];

impl Analysis for Hakari {
    fn name(&self) -> &'static str {
        "hakari"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        run(cx);
    }
}

#[cfg(not(feature = "link"))]
fn run(cx: &Context<'_>) {
    cx.not_run_all(
        "feature:hakari",
        "spirit-level was built without the `hakari` feature (`cargo install spirit_level_cli --features hakari`)",
    );
}

#[cfg(feature = "link")]
fn run(cx: &Context<'_>) {
    linked::run(cx);
}

#[cfg(feature = "link")]
mod linked {
    use super::CONFIG_PATHS;
    use guppy::graph::PackageGraph;
    use hakari::summaries::HakariConfig;
    use sl_analysis::Context;
    use sl_core::{Diagnostic, Edit, Fix, Span};

    pub(super) fn run(cx: &Context<'_>) {
        let p = cx.project;
        let Some(config_path) = CONFIG_PATHS.iter().find(|c| p.exists(c)) else {
            for lint in ["hakari-generate", "hakari-manage-deps"] {
                cx.emit(
                    Diagnostic::new(lint, "workspace", "the workspace has no workspace-hack package (no .config/hakari.toml)")
                        .span(Span::new(p.root_manifest.rel.clone(), 0))
                        .help("`cargo hakari init workspace-hack`, then `cargo hakari generate` and `cargo hakari manage-deps`"),
                );
            }
            return;
        };
        let fail = |reason: String| cx.not_run_all("library:hakari", &reason);
        let config: HakariConfig = match p.read(config_path).unwrap_or_default().parse() {
            Ok(c) => c,
            Err(e) => return fail(format!("{config_path} does not parse: {e}")),
        };
        let out = match sl_model::command("cargo", &p.root)
            .args(["metadata", "--format-version", "1"])
            .output()
        {
            Ok(o) if o.status.success() => o,
            Ok(o) => {
                return fail(format!(
                    "cargo metadata failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ));
            }
            Err(e) => return fail(format!("cannot run cargo metadata: {e}")),
        };
        let graph = match PackageGraph::from_json(String::from_utf8_lossy(&out.stdout)) {
            Ok(g) => g,
            Err(e) => return fail(format!("the package graph could not be built: {e}")),
        };
        let builder = match config.builder.to_hakari_builder(&graph) {
            Ok(b) => b,
            Err(e) => return fail(format!("{config_path}: {e}")),
        };
        let anchor = Span::new(config_path.to_string(), 0);

        if cx.enabled("hakari-manage-deps") {
            let members = graph.resolve_workspace();
            match builder.manage_dep_ops(&members) {
                None => cx.not_run(
                    "hakari-manage-deps",
                    "library:hakari",
                    format!("{config_path} names no hakari-package"),
                ),
                Some(ops) if ops.is_empty() => {}
                Some(ops) => {
                    let what = ops.display().to_string();
                    let what: Vec<&str> = what
                        .lines()
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .collect();
                    cx.emit(
                        Diagnostic::new(
                            "hakari-manage-deps",
                            "workspace-hack:deps",
                            format!(
                                "members' dependencies on the workspace-hack are out of date: {}",
                                what.join("; ")
                            ),
                        )
                        .span(anchor.clone())
                        .help("`cargo hakari manage-deps` adds and removes the dependency lines"),
                    );
                }
            }
        }

        if cx.enabled("hakari-generate") {
            let hakari = builder.compute();
            let toml = match hakari.to_toml_string(&config.output.to_options()) {
                Ok(t) => t,
                Err(e) => {
                    cx.not_run(
                        "hakari-generate",
                        "library:hakari",
                        format!("hakari could not write its TOML: {e}"),
                    );
                    return;
                }
            };
            let existing = match hakari.read_toml() {
                Some(Ok(t)) => t,
                Some(Err(e)) => {
                    cx.not_run(
                        "hakari-generate",
                        "library:hakari",
                        format!("the workspace-hack's Cargo.toml: {e}"),
                    );
                    return;
                }
                None => {
                    cx.not_run(
                        "hakari-generate",
                        "library:hakari",
                        format!("{config_path} names no hakari-package"),
                    );
                    return;
                }
            };
            if existing.is_changed(&toml) {
                let path = existing.toml_path();
                let rel = path
                    .strip_prefix(p.root.to_string_lossy().as_ref())
                    .map(|r| r.as_str().to_string())
                    .unwrap_or_else(|_| path.as_str().to_string());
                let (start, end) = existing.generated_offsets();
                let line = existing.contents()[..start].lines().count() as u32 + 1;
                let changed = existing
                    .diff_toml(&toml)
                    .hunks()
                    .iter()
                    .flat_map(|h| h.lines())
                    .filter(|l| !matches!(l, hakari::diffy::Line::Context(_)))
                    .count();
                let krate = hakari
                    .builder()
                    .hakari_package()
                    .map(|k| k.name().to_string());
                let mut d = Diagnostic::new(
                    "hakari-generate",
                    "workspace-hack:generated",
                    format!("the workspace-hack's dependencies are out of date ({changed} lines differ)"),
                )
                .span(Span::new(&rel, line))
                .help("`spirit-level check --fix` (or `cargo hakari generate`) rewrites the generated section")
                .fix(Fix {
                    description: "regenerate the workspace-hack's dependencies".into(),
                    edits: vec![Edit {
                        file: rel.clone(),
                        start,
                        end,
                        replacement: toml,
                    }],
                });
                if let Some(k) = krate {
                    d = d.krate(k);
                }
                cx.emit(d);
            }
        }
    }
}
