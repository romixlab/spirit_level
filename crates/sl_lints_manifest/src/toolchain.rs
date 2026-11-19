//! The pinned toolchain: every `rust-toolchain.toml` pins the expected exact release or dated nightly, and
//! justfiles and CI workflows name one of the two wherever they choose a toolchain.

use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span, glob};
use sl_model::toml_file::line_of;
use sl_model::{TomlFile, justfile};
use std::collections::BTreeMap;
use std::path::Path;

const PINNED: &str = "toolchain-pinned";
const OVERRIDES: &str = "toolchain-overrides-pinned";
const TOOLCHAIN_FILES: [&str; 2] = ["rust-toolchain.toml", "rust-toolchain"];

/// The expected pair of `toolchain-pinned`'s parameters
struct Pair<'a> {
    stable: &'a str,
    nightly: &'a str,
}

impl Pair<'_> {
    /// The pinned toolchain that replaces `name`: the nightly for any nightly, else the stable release
    fn expected(&self, name: &str) -> &str {
        if name.starts_with("nightly") {
            self.nightly
        } else {
            self.stable
        }
    }

    fn is_expected(&self, name: &str) -> bool {
        name == self.stable || name == self.nightly
    }
}

/// How a toolchain name pins: exactly (`1.100.0`, `nightly-2026-11-19`) or not (`stable`, `nightly`, `1.100`)
fn is_exact(name: &str) -> bool {
    is_release(name) || name.strip_prefix("nightly-").is_some_and(is_date)
}

fn is_release(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn is_date(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
}

/// Whether a word names a toolchain at all: a channel, a release or a dated channel (a ref such as `master` or
/// `v1` does not)
fn is_toolchain_name(w: &str) -> bool {
    ["stable", "beta", "nightly"]
        .iter()
        .any(|c| w == *c || w.strip_prefix(c).is_some_and(|r| r.starts_with('-')))
        || w.split('.').count() >= 2
            && w.split('.')
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// What is wrong with a pin: None when it is the expected one
fn problem(name: &str, pair: &Pair<'_>) -> Option<String> {
    if pair.is_expected(name) {
        None
    } else if is_exact(name) {
        Some(format!(
            "pins {name}, not the expected {}",
            pair.expected(name)
        ))
    } else {
        Some(format!(
            "`{name}` is not an exact toolchain: it moves between PCs and days"
        ))
    }
}

/// The files of the project named one of `names`, relative to the root: every tracked one outside `skip`, or the
/// root's own when the tracked files are unknown
fn files(cx: &Context<'_>, names: &[&str], skip: &[String]) -> Vec<String> {
    let named = |f: &str| names.iter().any(|n| f.rsplit('/').next() == Some(*n));
    let mut out: Vec<String> = match cx.project.tracked() {
        Ok(t) => t
            .iter()
            .filter(|f| named(f) && !skip.iter().any(|s| glob::matches(s, f)))
            .cloned()
            .collect(),
        Err(_) => names
            .iter()
            .filter(|n| cx.project.exists(n))
            .map(|n| (*n).to_owned())
            .collect(),
    };
    out.sort();
    out
}

pub(crate) fn run(cx: &Context<'_>) {
    let params = cx.params(PINNED);
    let pair = Pair {
        stable: params.str("stable"),
        nightly: params.str("nightly"),
    };
    let mut bad = false;
    for (name, value, want) in [
        ("stable", pair.stable, "an exact release (`1.100.0`)"),
        (
            "nightly",
            pair.nightly,
            "a dated nightly (`nightly-2026-11-19`)",
        ),
    ] {
        let ok = if name == "stable" {
            is_release(value)
        } else {
            value.strip_prefix("nightly-").is_some_and(is_date)
        };
        if !ok {
            bad = true;
            for lint in [PINNED, OVERRIDES] {
                cx.emit(
                    Diagnostic::new(
                        lint,
                        format!("param:{name}"),
                        format!("toolchain-pinned's `{name} = \"{value}\"` is not {want}"),
                    )
                    .span(Span::new(sl_config::FILE, 0))
                    .help(format!(
                        "set `{name}` in [lints.toolchain-pinned] to {want}"
                    )),
                );
            }
        }
    }
    if bad {
        return;
    }
    if cx.enabled(PINNED) {
        toolchain_files(cx, &pair);
    }
    if cx.enabled(OVERRIDES) {
        let skip = cx.params(OVERRIDES).list("skip");
        for f in files(cx, &justfile::FILES, skip) {
            if let Some(text) = cx.project.read(&f) {
                overrides(cx, &f, &text, &pair, true);
            }
        }
        let workflows: Vec<String> = match cx.project.tracked() {
            Ok(t) => t
                .iter()
                .filter(|f| is_workflow(f) && !skip.iter().any(|s| glob::matches(s, f)))
                .cloned()
                .collect(),
            Err(_) => sl_model::Workflow::all(&cx.project.root)
                .into_iter()
                .flatten()
                .map(|w| w.rel)
                .collect(),
        };
        for f in workflows {
            if let Some(text) = cx.project.read(&f) {
                overrides(cx, &f, &text, &pair, false);
            }
        }
    }
}

fn is_workflow(f: &str) -> bool {
    f.strip_prefix(sl_model::workflow::DIR)
        .and_then(|r| r.strip_prefix('/'))
        .is_some_and(|n| {
            !n.contains('/')
                && Path::new(n).extension().is_some_and(|x| {
                    x.eq_ignore_ascii_case("yml") || x.eq_ignore_ascii_case("yaml")
                })
        })
}

/// `toolchain-pinned`: every toolchain file pins the expected toolchain exactly; the root has one
fn toolchain_files(cx: &Context<'_>, pair: &Pair<'_>) {
    let found = files(cx, &TOOLCHAIN_FILES, cx.params(PINNED).list("skip"));
    let at_root = |f: &&String| TOOLCHAIN_FILES.contains(&f.as_str());
    if !found.iter().any(|f| at_root(&f)) {
        cx.emit(
            Diagnostic::new(
                PINNED,
                "rust-toolchain.toml",
                "the toolchain is not pinned: the root has no rust-toolchain.toml",
            )
            .span(Span::new("rust-toolchain.toml", 0))
            .help(format!(
                "add rust-toolchain.toml with `[toolchain] channel = \"{}\"`",
                pair.stable
            )),
        );
    }
    for f in &found {
        let Some(text) = cx.project.read(f) else {
            continue;
        };
        if let Some(d) = check_file(f, text, pair) {
            cx.emit(d);
        }
    }
}

/// The finding of one toolchain file, if it does not pin the expected toolchain
fn check_file(rel: &str, text: String, pair: &Pair<'_>) -> Option<Diagnostic> {
    let legacy = Path::new(rel)
        .file_name()
        .is_some_and(|n| n == "rust-toolchain");
    let first = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    if legacy && !first.starts_with('[') {
        let start = text.find(first).unwrap_or(0);
        return finding(rel, &text, first, start..start + first.len(), pair);
    }
    let m = match TomlFile::parse(rel, text) {
        Ok(m) => m,
        Err(e) => {
            return Some(
                Diagnostic::new(PINNED, rel, format!("{rel} does not parse: {e}"))
                    .span(Span::new(rel, 0))
                    .help("fix the TOML; rustup reads `[toolchain] channel`"),
            );
        }
    };
    let Some(channel) = m.get(&["toolchain", "channel"]) else {
        let edit = m.item_span(&["toolchain"]).map(|s| {
            let at = m.text[s.start..]
                .find('\n')
                .map_or(m.text.len(), |i| s.start + i + 1);
            let nl = if at == m.text.len() && !m.text.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            Edit {
                file: rel.to_owned(),
                start: at,
                end: at,
                replacement: format!("{nl}channel = \"{}\"\n", pair.stable),
            }
        });
        let mut d = Diagnostic::new(
            PINNED,
            rel,
            format!("{rel} names no channel: rustup takes whatever is installed"),
        )
        .span(m.span(&["toolchain"]))
        .help(format!(
            "add `channel = \"{}\"` under [toolchain]",
            pair.stable
        ));
        if let Some(e) = edit {
            d = d.fix(Fix {
                description: format!("pin {}", pair.stable),
                edits: vec![e],
            });
        }
        return Some(d);
    };
    let name = channel.as_str().unwrap_or_default().to_owned();
    let span = m.item_span(&["toolchain", "channel"])?;
    let value = m.text[span.clone()].to_owned();
    let start = span.start + value.find(&name).unwrap_or(0);
    finding(rel, &m.text, &name, start..start + name.len(), pair)
}

/// The finding of a pin `name` at `range` of a toolchain file, with the fix that writes the expected one
fn finding(
    rel: &str,
    text: &str,
    name: &str,
    range: std::ops::Range<usize>,
    pair: &Pair<'_>,
) -> Option<Diagnostic> {
    let why = problem(name, pair)?;
    let want = pair.expected(name);
    Some(
        Diagnostic::new(PINNED, format!("{rel}:{name}"), format!("{rel} {why}"))
            .span(Span::new(rel, line_of(text, range.start)))
            .help(format!(
                "pin `{want}`: every PC and the gate build with it (`spl check --fix`)"
            ))
            .fix(Fix {
                description: format!("pin {want}"),
                edits: vec![Edit {
                    file: rel.to_owned(),
                    start: range.start,
                    end: range.end,
                    replacement: want.to_owned(),
                }],
            }),
    )
}

/// One toolchain a justfile or workflow chooses: its name and where the name is written
struct Choice {
    name: String,
    start: usize,
}

/// `toolchain-overrides-pinned`: the toolchains a justfile (`just`) or a workflow chooses
fn overrides(cx: &Context<'_>, rel: &str, text: &str, pair: &Pair<'_>, just: bool) {
    let vars = if just {
        variables(text)
    } else {
        BTreeMap::new()
    };
    let mut seen = Vec::new();
    for c in choices(text, &vars, just) {
        if seen.contains(&c.start) {
            continue;
        }
        seen.push(c.start);
        let Some(why) = problem(&c.name, pair) else {
            continue;
        };
        let want = pair.expected(&c.name);
        cx.emit(
            Diagnostic::new(OVERRIDES, format!("{rel}:{}", c.name), format!("{rel} chooses a toolchain that {why}"))
                .span(Span::new(rel, line_of(text, c.start)))
                .help(format!(
                    "name `{want}`{}",
                    if just { " (a variable `nightly := \"...\"` and `cargo +{{ nightly }}` keep it in one place)" } else { "" }
                ))
                .fix(Fix {
                    description: format!("choose {want}"),
                    edits: vec![Edit { file: rel.to_owned(), start: c.start, end: c.start + c.name.len(), replacement: want.to_owned() }],
                }),
        );
    }
}

/// A justfile's string assignments (`nightly := "nightly-2026-11-19"`): name, value and the value's offset
fn variables(text: &str) -> BTreeMap<String, (String, usize)> {
    let mut out = BTreeMap::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if let Some((name, value)) = line.split_once(":=") {
            let name = name.trim().trim_start_matches("export ").trim();
            let v = value.trim();
            if let Some(inner) = v
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            {
                let at = offset + line.find(inner).unwrap_or(0);
                out.insert(name.to_owned(), (inner.to_owned(), at));
            }
        }
        offset += line.len();
    }
    out
}

/// Every toolchain a file chooses: `cargo +X` (and rustc, rustdoc), `rustup run X`, `--toolchain X`, and in a
/// workflow `toolchain: X` and `dtolnay/rust-toolchain@X`. A justfile's `{{ var }}` is read from its assignment.
fn choices<'a>(text: &'a str, vars: &BTreeMap<String, (String, usize)>, just: bool) -> Vec<Choice> {
    let mut out = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let code = line.split(" #").next().unwrap_or(line);
        if code.trim_start().starts_with('#') {
            offset += line.len();
            continue;
        }
        let words = words(code);
        for (i, (w, at)) in words.iter().enumerate() {
            let prev = i.checked_sub(1).map(|p| words[p].0);
            let base = |p: Option<&'a str>| p.map(|p| p.rsplit('/').next().unwrap_or(p));
            let target = if let Some(rest) = w.strip_prefix('+')
                && matches!(base(prev), Some("cargo" | "rustc" | "rustdoc"))
            {
                Some((rest, at + 1))
            } else if matches!(prev, Some("--toolchain"))
                || (base(i.checked_sub(2).map(|p| words[p].0)) == Some("rustup")
                    && prev == Some("run"))
            {
                Some((*w, *at))
            } else if let Some(rest) = w.strip_prefix("--toolchain=") {
                Some((rest, at + "--toolchain=".len()))
            } else if !just && *w == "toolchain:" {
                words.get(i + 1).map(|(n, a)| (*n, *a))
            } else if !just && let Some(r) = w.split_once("dtolnay/rust-toolchain@") {
                Some((r.1, at + r.0.len() + "dtolnay/rust-toolchain@".len()))
            } else {
                None
            };
            let Some((name, start)) = target else {
                continue;
            };
            let (name, start) = unquote(name, offset + start);
            if let Some(var) = name
                .strip_prefix("{{")
                .and_then(|v| v.strip_suffix("}}"))
                .map(str::trim)
            {
                if let Some((value, at)) = vars.get(var)
                    && is_toolchain_name(value)
                {
                    out.push(Choice {
                        name: value.clone(),
                        start: *at,
                    });
                }
            } else if is_toolchain_name(name) {
                out.push(Choice {
                    name: name.to_owned(),
                    start,
                });
            }
        }
        offset += line.len();
    }
    out
}

/// A name without its quotes, and its offset moved past an opening one
fn unquote(name: &str, start: usize) -> (&str, usize) {
    let q = name.starts_with(['"', '\'']);
    let name = name.trim_matches(['"', '\'']);
    (name, start + usize::from(q))
}

/// The words of a line with their offsets; `{{ x }}` is one word
fn words(line: &str) -> Vec<(&str, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    let b = line.as_bytes();
    while i < b.len() {
        if b[i].is_ascii_whitespace() || b[i] == b';' {
            i += 1;
            continue;
        }
        let start = i;
        let mut depth = 0usize;
        while i < b.len() {
            if line[i..].starts_with("{{") {
                depth += 1;
                i += 2;
                continue;
            }
            if depth > 0 && line[i..].starts_with("}}") {
                depth -= 1;
                i += 2;
                continue;
            }
            if depth == 0 && (b[i].is_ascii_whitespace() || b[i] == b';') {
                break;
            }
            i += 1;
        }
        out.push((&line[start..i], start));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAIR: Pair<'static> = Pair {
        stable: "1.100.0",
        nightly: "nightly-2026-11-19",
    };

    /// The file after the finding's fix
    fn fixed(rel: &str, text: &str) -> Option<String> {
        let d = check_file(rel, text.to_owned(), &PAIR)?;
        let mut out = text.to_owned();
        for e in d
            .fix
            .expect("a toolchain finding has a fix")
            .edits
            .iter()
            .rev()
        {
            out.replace_range(e.start..e.end, &e.replacement);
        }
        Some(out)
    }

    #[test]
    fn the_fix_writes_the_expected_toolchain() {
        let f = |t: &str| fixed("rust-toolchain.toml", t);
        assert_eq!(f("[toolchain]\nchannel = \"1.100.0\"\n"), None);
        assert_eq!(f("[toolchain]\nchannel = \"nightly-2026-11-19\"\n"), None);
        assert_eq!(
            f("[toolchain]\nchannel = \"stable\"\ncomponents = [\"clippy\"]\n").as_deref(),
            Some("[toolchain]\nchannel = \"1.100.0\"\ncomponents = [\"clippy\"]\n")
        );
        assert_eq!(
            f("[toolchain]\nchannel = 'nightly'\n").as_deref(),
            Some("[toolchain]\nchannel = 'nightly-2026-11-19'\n")
        );
        assert_eq!(
            f("[toolchain]\nchannel = \"nightly-2026-01-05\"\n").as_deref(),
            Some("[toolchain]\nchannel = \"nightly-2026-11-19\"\n")
        );
        assert_eq!(
            f("[toolchain]\ncomponents = [\"clippy\"]\n").as_deref(),
            Some("[toolchain]\nchannel = \"1.100.0\"\ncomponents = [\"clippy\"]\n")
        );
        assert_eq!(
            f("[toolchain]").as_deref(),
            Some("[toolchain]\nchannel = \"1.100.0\"\n")
        );
        assert_eq!(
            fixed("rust-toolchain", "1.99\n").as_deref(),
            Some("1.100.0\n")
        );
    }

    #[test]
    fn exact_names() {
        for n in ["1.100.0", "nightly-2026-11-19"] {
            assert!(is_exact(n), "{n}");
        }
        for n in [
            "stable",
            "beta",
            "nightly",
            "1.100",
            "nightly-2026-11",
            "1.100.0.1",
        ] {
            assert!(!is_exact(n), "{n}");
        }
        for n in ["master", "v1", "main", "{{nightly}}", "x86_64"] {
            assert!(!is_toolchain_name(n), "{n}");
        }
    }

    #[test]
    fn choices_of_a_justfile_and_a_workflow() {
        let just = "n := \"nightly\"\nr:\n    cargo +{{ n }} fuzz run # cargo +stable\n    cargo +beta t; rustc +1.99 x.rs\n    cargo --toolchain=stable x\n";
        let names: Vec<(String, usize)> = choices(just, &variables(just), true)
            .into_iter()
            .map(|c| (c.name, c.start))
            .collect();
        assert_eq!(
            names,
            [
                ("nightly".to_owned(), 6),
                ("beta".to_owned(), just.find("beta").unwrap()),
                ("1.99".to_owned(), just.find("1.99").unwrap()),
                ("stable".to_owned(), just.rfind("stable").unwrap()),
            ]
        );
        let wf = "steps:\n  - uses: dtolnay/rust-toolchain@nightly\n    with:\n      toolchain: \"1.99.0\"\n  - uses: actions/checkout@v4\n";
        let names: Vec<String> = choices(wf, &BTreeMap::new(), false)
            .into_iter()
            .map(|c| wf[c.start..c.start + c.name.len()].to_owned())
            .collect();
        assert_eq!(names, ["nightly", "1.99.0"]);
    }
}
