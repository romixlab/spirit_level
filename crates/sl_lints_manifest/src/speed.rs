//! Build speed: no LTO in the profiles of the edit-build-test loop, one integration-test binary per crate.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::{TargetKind, TomlFile, toml_edit};
use std::collections::BTreeSet;

/// The profile a profile inherits from: its `inherits`, or cargo's built-in parent
fn parent(files: &[&TomlFile], name: &str) -> Option<String> {
    let set = files.iter().find_map(|f| {
        f.get(&["profile", name, "inherits"])
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    });
    set.or_else(|| match name {
        "test" => Some("dev".to_owned()),
        "bench" => Some("release".to_owned()),
        _ => None,
    })
}

/// Whether an `lto` value turns LTO on (`true`, `"fat"`, `"thin"`); `false` and `"off"` keep it off
fn lto_on(v: &toml_edit::Item) -> bool {
    v.as_bool()
        .unwrap_or_else(|| v.as_str().is_some_and(|s| s != "off"))
}

pub(crate) fn run(cx: &Context<'_>) {
    no_lto(cx);
    test_binaries(cx);
}

fn no_lto(cx: &Context<'_>) {
    if !cx.enabled("profile-no-lto") {
        return;
    }
    let p = cx.project;
    let config = p
        .read(".cargo/config.toml")
        .and_then(|t| TomlFile::parse(".cargo/config.toml", t).ok());
    // cargo's configuration overrides the manifest's profiles
    let mut files: Vec<&TomlFile> = config.iter().collect();
    files.push(&p.root_manifest);
    for want in cx.params("profile-no-lto").list("profiles") {
        let mut name = want.clone();
        let mut seen = BTreeSet::new();
        while seen.insert(name.clone()) {
            let set = files.iter().find_map(|f| {
                f.get(&["profile", &name, "lto"])
                    .map(|v| (*f, lto_on(v), v.to_string()))
            });
            if let Some((file, on, value)) = set {
                if on {
                    let from = if name == *want {
                        String::new()
                    } else {
                        format!(", inherited from [profile.{name}]")
                    };
                    cx.emit(
                        Diagnostic::new("profile-no-lto", format!("profile.{want}.lto"), format!("[profile.{want}] builds with lto = {}{from}", value.trim()))
                            .span(file.span(&["profile", &name, "lto"]))
                            .help(format!("set `lto = false` under [profile.{want}]; LTO belongs in a release profile for shipped artifacts, if anywhere")),
                    );
                }
                break;
            }
            match parent(&files, &name) {
                Some(up) => name = up,
                None => break,
            }
        }
    }
}

fn test_binaries(cx: &Context<'_>) {
    if !cx.enabled("test-binaries") {
        return;
    }
    let max = cx.params("test-binaries").count("max-binaries");
    for pkg in &cx.project.packages {
        let tests: Vec<&str> = pkg
            .targets
            .iter()
            .filter(|(k, _)| *k == TargetKind::Test)
            .map(|(_, n)| n.as_str())
            .collect();
        if tests.len() <= max {
            continue;
        }
        cx.emit(
            Diagnostic::new("test-binaries", format!("{}:tests", pkg.name), format!("{} has {} integration-test binaries: {}", pkg.name, tests.len(), tests.join(", ")))
                .span(Span::new(pkg.path("tests"), 0))
                .krate(&pkg.name)
                .metric("test-binaries", tests.len() as f64, Some(max as f64))
                .help("make them modules of one binary, `tests/it/main.rs` with `mod a;` per file: each binary links the library again"),
        );
    }
}
