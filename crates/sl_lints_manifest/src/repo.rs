//! Repo-level files: profiles, toolchain, rule files, changelog, justfile, feature comments.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::Justfile;

pub(crate) fn run(cx: &Context<'_>) {
    let p = cx.project;
    let root = &p.root_manifest;

    // profile-defaults
    if root
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

    // toolchain-pinned
    let tc = p
        .read("rust-toolchain.toml")
        .or_else(|| p.read("rust-toolchain"));
    let pinned = tc.as_deref().is_some_and(|t| {
        t.lines().any(|l| {
            let l = l.trim();
            l.starts_with("channel") || (!l.is_empty() && !l.starts_with('[') && !l.contains('='))
        })
    });
    if !pinned {
        cx.emit(
            Diagnostic::new(
                "toolchain-pinned",
                "rust-toolchain.toml",
                "the toolchain is not pinned",
            )
            .span(Span::new("rust-toolchain.toml", 0))
            .help("add rust-toolchain.toml with `[toolchain] channel = \"1.xx.y\"`"),
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
