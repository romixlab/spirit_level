//! The cargo-shear adapter: `cargo shear --format json`, one finding per unused or misplaced dependency.
//!
//! cargo-shear prints one JSON document: `summary` (errors, warnings, fixed) and `findings`, each with a `code`
//! (`shear/unused_dependency`, `shear/unused_workspace_dependency`, `shear/misplaced_dependency`, ...), a
//! `severity`, a `message`, the `file` (relative to the project) and a byte `location` in it. It exits 1 when
//! it found anything, 2 on an error.

use serde_json::Value;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Need, Span};

lint!(SHEAR_UNUSED_DEPS, "X0030", "shear-unused-deps", "shear", "no crate declares a dependency it never uses, and every workspace dependency is used",
    tool: ("cargo-shear", Executable), params: [], needs: [Need::Tool("cargo-shear")], fix: None);
lint!(SHEAR_MISPLACED_DEPS, "X0029", "shear-misplaced-deps", "shear", "a dependency only tests use is a dev-dependency",
    tool: ("cargo-shear", Executable), params: [], needs: [Need::Tool("cargo-shear")], fix: None);

/// The cargo-shear analysis
pub struct Shear;

/// The registered instance
pub static SHEAR: Shear = Shear;

static LINTS: &[&LintMeta] = &[&SHEAR_UNUSED_DEPS, &SHEAR_MISPLACED_DEPS];

impl Analysis for Shear {
    fn name(&self) -> &'static str {
        "cargo-shear"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("cargo-shear", None) else {
            cx.not_run_all("tool:cargo-shear", &sl_tools::missing_reason("cargo-shear"));
            return;
        };
        let args = ["shear", "--format", "json", "--color", "never"];
        let out = match sl_tools::run_subcommand(&exe, &args, &p.root, &[]) {
            Ok(o) => o,
            Err(e) => {
                cx.not_run_all("run:cargo-shear", &format!("cannot run cargo-shear: {e}"));
                return;
            }
        };
        let Some(findings) = parse(&out.stdout) else {
            cx.not_run_all("run:cargo-shear", &format!("cargo shear {}", out.failure()));
            return;
        };
        for f in findings {
            let line = p.read(&f.file).map_or(0, |t| line_of(&t, f.offset));
            let krate = p
                .packages
                .iter()
                .find(|k| k.manifest.rel == f.file)
                .map(|k| k.name.clone());
            let mut d = Diagnostic::new(
                f.lint(),
                format!("{}:{}:{}", f.code, f.file, f.message),
                f.message.clone(),
            )
            .span(Span::new(&f.file, line))
            .help(if f.help.is_empty() {
                "see cargo shear's help".to_string()
            } else {
                f.help.clone()
            });
            if let Some(k) = krate {
                d = d.krate(k);
            }
            cx.emit(d);
        }
    }
}

/// One cargo-shear finding
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// `shear/unused_dependency`, ...
    pub code: String,
    /// The message
    pub message: String,
    /// The manifest, relative to the project
    pub file: String,
    /// Byte offset in it
    pub offset: usize,
    /// cargo-shear's help
    pub help: String,
}

impl Finding {
    /// The lint of its code: `misplaced` codes to `shear-misplaced-deps`, everything else (unused dependencies,
    /// unused workspace entries, ignore entries that ignore nothing) to `shear-unused-deps`
    pub fn lint(&self) -> &'static str {
        if self.code.contains("misplaced") {
            "shear-misplaced-deps"
        } else {
            "shear-unused-deps"
        }
    }
}

/// The findings of cargo-shear's JSON document; None when stdout is not one (a failed run)
pub fn parse(stdout: &str) -> Option<Vec<Finding>> {
    let v: Value = serde_json::from_str(stdout.trim()).ok()?;
    let s = |f: &Value, k: &str| {
        f.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    Some(
        v.get("findings")?
            .as_array()?
            .iter()
            .map(|f| Finding {
                code: s(f, "code"),
                message: s(f, "message"),
                file: s(f, "file"),
                offset: f
                    .get("location")
                    .and_then(|l| l.get("offset"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize,
                help: s(f, "help"),
            })
            .collect(),
    )
}

/// The 1-based line of a byte offset
fn line_of(text: &str, offset: usize) -> u32 {
    text.as_bytes()
        .iter()
        .take(offset)
        .filter(|b| **b == b'\n')
        .count() as u32
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings() {
        let f = parse(include_str!("../tests/samples/shear.json")).unwrap();
        assert_eq!(f.len(), 3);
        assert_eq!(f[0].file, "crates/a/Cargo.toml");
        assert_eq!(f[0].lint(), "shear-unused-deps");
        assert_eq!(f[2].code, "shear/unused_workspace_dependency");
        assert_eq!(line_of("a\nb\nc", 2), 2);
        assert!(parse("error: no manifest").is_none());
    }
}
