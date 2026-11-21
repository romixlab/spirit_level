//! Repo-level files: profiles, rule files, changelog, justfile, feature comments.

use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span};
use sl_model::{Justfile, TomlFile, toml_edit};

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    let root = &p.root_manifest;

    // dev-debug-off; profile-defaults' split-debuginfo check skips while debug is off: nothing to split
    let debug_off = dev_debug_off(cx, root);

    // profile-defaults
    if !debug_off
        && root
            .get(&["profile", "dev", "split-debuginfo"])
            .and_then(|v| v.as_str())
            != Some("unpacked")
    {
        cx.emit(
            Diagnostic::new("profile-defaults", "profile.dev.split-debuginfo", "the dev profile does not split debug info")
                .span(root.span(&["profile", "dev"]))
                .help("add `[profile.dev] split-debuginfo = \"unpacked\"` (and the same for [profile.test]): test binaries stop being I/O-bound"),
        );
    }
    let deploy = cx.params("profile-defaults").str("deploy-profile");
    if root
        .get(&["profile", deploy, "incremental"])
        .and_then(|v| v.as_bool())
        != Some(true)
    {
        cx.emit(
            Diagnostic::new("profile-defaults", format!("profile.{deploy}"), format!("there is no [profile.{deploy}] with incremental = true"))
                .span(root.span(&["profile", deploy]))
                .help(format!("add `[profile.{deploy}]` with `inherits = \"release\"` and `incremental = true`, and build installs with it")),
        );
    }

    // repo-files
    for f in cx.params("repo-files").list("files") {
        if !p.exists(f) {
            cx.emit(
                Diagnostic::new(
                    "repo-files",
                    format!("file:{f}"),
                    format!("the repo has no {f}"),
                )
                .span(Span::new(f.clone(), 0)),
            );
        }
    }

    // changelog-unreleased
    if let Some(cl) = p.read("CHANGELOG.md")
        && !cl
            .lines()
            .any(|l| l.trim().to_ascii_lowercase().starts_with("## [unreleased]"))
    {
        cx.emit(
            Diagnostic::new("changelog-unreleased", "CHANGELOG.md", "CHANGELOG.md has no `## [Unreleased]` section")
                .span(Span::new("CHANGELOG.md", 1))
                .help("add `## [Unreleased]` above the newest release; work commits put their entries there"),
        );
    }

    // justfile-recipes, justfile-simple
    let jf = Justfile::read(&p.root);
    let recipes = jf.as_ref().map_or(&[][..], |j| j.recipes.as_slice());
    let file = jf.as_ref().map_or("justfile", |j| j.rel.as_str());
    for want in cx.params("justfile-recipes").list("recipes") {
        if !recipes.iter().any(|r| &r.name == want) {
            cx.emit(
                Diagnostic::new(
                    "justfile-recipes",
                    format!("recipe:{want}"),
                    format!("the justfile has no `{want}` recipe"),
                )
                .span(Span::new(file, 0)),
            );
        }
    }
    let max = cx.params("justfile-simple").count("max-lines");
    for r in recipes {
        let commands = r.body.len();
        let why = if r.shebang {
            Some("is an inline script (a shebang recipe)".to_owned())
        } else if commands > max {
            Some(format!("has {commands} command lines (limit {max})"))
        } else {
            None
        };
        if let Some(why) = why {
            cx.emit(
                Diagnostic::new("justfile-simple", format!("recipe:{}", r.name), format!("recipe `{}` {why}", r.name))
                    .span(Span::new(file, r.line))
                    .metric("command-lines", commands as f64, Some(max as f64))
                    .help("move the logic into a Rust command (a subcommand or a small tool) and let the recipe call it"),
            );
        }
    }

    // feature-docs
    let skip = cx.params("feature-docs").list("skip");
    for pkg in &p.packages {
        let Some(t) = pkg.manifest.get(&["features"]).and_then(|t| t.as_table()) else {
            continue;
        };
        for (k, _) in t {
            if skip.iter().any(|s| s == k) {
                continue;
            }
            let text = &pkg.manifest.text;
            let documented = t
                .key(k)
                .and_then(|key| key.leaf_decor().prefix())
                .and_then(|r| r.as_str().or_else(|| r.span().and_then(|sp| text.get(sp))))
                .is_some_and(|s| s.contains('#'));
            if !documented {
                cx.emit(
                    Diagnostic::new(
                        "feature-docs",
                        format!("{}:feature:{k}", pkg.name),
                        format!("feature `{k}` of {} has no comment", pkg.name),
                    )
                    .span(pkg.manifest.span(&["features", k]))
                    .krate(&pkg.name)
                    .help("put a `# ...` line right above it: what it turns on and when to use it"),
                );
            }
        }
    }
}

/// `dev-debug-off`: whether `profile.dev.debug` is off (`false`, `0`, `"none"`), emitting a finding when it is
/// not and the value is not in the lint's `allow`
fn dev_debug_off(cx: &Context<'_>, root: &TomlFile) -> bool {
    let debug_item = root.get(&["profile", "dev", "debug"]);
    let debug_str = debug_item.and_then(debug_value_str);
    let debug_off = matches!(debug_str.as_deref(), Some("false" | "0" | "none"));
    if debug_off {
        return true;
    }
    let allow = cx.params("dev-debug-off").list("allow");
    let allowed = debug_str
        .as_deref()
        .is_some_and(|v| allow.iter().any(|a| a == v));
    if !allowed {
        let message = match &debug_str {
            Some(v) => format!("profile.dev.debug is {v}, not off and not in `allow`"),
            None if debug_item.is_some() => {
                "profile.dev.debug is set to a value this lint cannot read".to_owned()
            }
            None => "profile.dev does not set debug: cargo's default is full debug info".to_owned(),
        };
        cx.emit(
            Diagnostic::new("dev-debug-off", "profile.dev.debug", message)
                .span(root.span(&["profile", "dev"]))
                .help("add `[profile.dev] debug = false`: CARGO_PROFILE_DEV_DEBUG turns it on for one run when needed")
                .fix(dev_debug_off_fix(root)),
        );
    }
    false
}

/// A scalar `profile.dev.debug` value, as `cargo` would accept it, rendered for comparison (`"false"`, `"0"`,
/// `"none"`, `"line-tables-only"`); `None` for a missing key or a non-scalar one
fn debug_value_str(item: &toml_edit::Item) -> Option<String> {
    Some(match item.as_value()? {
        toml_edit::Value::Boolean(b) => b.value().to_string(),
        toml_edit::Value::Integer(n) => n.value().to_string(),
        toml_edit::Value::String(s) => s.value().to_owned(),
        _ => return None,
    })
}

/// The exact edit that sets `[profile.dev] debug = false`: in place when the key exists, right after the table
/// header when the table exists without it, appended as a new table otherwise
fn dev_debug_off_fix(root: &TomlFile) -> Fix {
    let edit = if let Some(span) = root.item_span(&["profile", "dev", "debug"]) {
        Edit {
            file: root.rel.clone(),
            start: span.start,
            end: span.end,
            replacement: "false".to_owned(),
        }
    } else if let Some(table) = root.item_span(&["profile", "dev"]) {
        let at = root.text[table.start..]
            .find('\n')
            .map_or(root.text.len(), |i| table.start + i + 1);
        let nl = if at == root.text.len() && !root.text.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        Edit {
            file: root.rel.clone(),
            start: at,
            end: at,
            replacement: format!("{nl}debug = false\n"),
        }
    } else {
        let text = &root.text;
        let sep = if text.ends_with("\n\n") || text.is_empty() {
            ""
        } else if text.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        Edit {
            file: root.rel.clone(),
            start: text.len(),
            end: text.len(),
            replacement: format!("{sep}[profile.dev]\ndebug = false\n"),
        }
    };
    Fix {
        description: "add [profile.dev] debug = false".into(),
        edits: vec![edit],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(text: &str) -> TomlFile {
        let root = TomlFile::parse("Cargo.toml", text.to_owned()).unwrap();
        let mut out = text.to_owned();
        for e in dev_debug_off_fix(&root).edits.iter().rev() {
            out.replace_range(e.start..e.end, &e.replacement);
        }
        TomlFile::parse("Cargo.toml", out).unwrap()
    }

    #[test]
    fn fix_sets_debug_off() {
        for text in [
            "[workspace]\n",
            "[workspace]\n\n[profile.dev]\nopt-level = 1\n",
            "[workspace]\n\n[profile.dev]\ndebug = true\n",
            "[workspace]\n\n[profile.dev.package.\"*\"]\nopt-level = 2\n",
            "[workspace]\n\n[profile.dev.package.\"*\"]\nopt-level = 2\n\n[profile.release]\nlto = true\n",
        ] {
            let after = fixed(text);
            assert_eq!(
                after
                    .get(&["profile", "dev", "debug"])
                    .and_then(toml_edit::Item::as_bool),
                Some(false),
                "{text:?} became {:?}",
                after.text
            );
            assert!(
                after.get(&["profile", "dev", "package", "debug"]).is_none(),
                "{:?}",
                after.text
            );
        }
    }
}
