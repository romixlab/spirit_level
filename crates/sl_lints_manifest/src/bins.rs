//! Binary target names: every `[[bin]]` and every implicit binary (`src/main.rs`, `src/bin/*.rs`,
//! `src/bin/*/main.rs`) checked against a configurable set of allowed characters.

use sl_analysis::Context;
use sl_core::{Diagnostic, Span};
use sl_model::TargetKind;

pub(crate) fn run(cx: &Context<'_>) {
    for pkg in &cx.project.packages {
        let params = cx.params_for("binary-name-chars", pkg);
        let spec = params.str("chars");
        for ((kind, name), root) in pkg.targets.iter().zip(&pkg.target_roots) {
            if *kind != TargetKind::Bin {
                continue;
            }
            let bad = disallowed(spec, name);
            if bad.is_empty() {
                continue;
            }
            cx.emit(
                Diagnostic::new(
                    "binary-name-chars",
                    format!("crate:{}:bin:{name}", pkg.name),
                    format!(
                        "binary {name} has a character the allowed set ({spec}) excludes: {bad}"
                    ),
                )
                .span(Span::new(pkg.path(root), 1))
                .krate(&pkg.name)
                .help("rename it: an explicit [[bin]] takes `name = \"...\"`; src/main.rs takes the package name; src/bin/*.rs and src/bin/*/main.rs take the file or folder name"),
            );
        }
    }
}

/// The characters of `name` the set `spec` (literals and `a-z`-style ranges) does not allow, each listed once
fn disallowed(spec: &str, name: &str) -> String {
    let mut bad = String::new();
    for c in name.chars() {
        if !in_set(spec, c) && !bad.contains(c) {
            bad.push(c);
        }
    }
    bad
}

/// Whether `c` is one of `spec`'s literal characters or inside one of its `a-z`-style ranges
fn in_set(spec: &str, c: char) -> bool {
    let chars: Vec<char> = spec.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            if (chars[i]..=chars[i + 2]).contains(&c) {
                return true;
            }
            i += 3;
        } else if chars[i] == c {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_literals() {
        assert!(in_set("a-z0-9", 'm'));
        assert!(in_set("a-z0-9", '7'));
        assert!(!in_set("a-z0-9", '-'));
        assert!(!in_set("a-z0-9", '_'));
        assert!(in_set("a-z_", '_'));
    }

    #[test]
    fn disallowed_lists_each_bad_character_once() {
        assert_eq!(disallowed("a-z0-9", "cs-node"), "-");
        assert_eq!(disallowed("a-z0-9", "snake_case"), "_");
        assert_eq!(disallowed("a-z0-9", "changedtests"), "");
    }
}
