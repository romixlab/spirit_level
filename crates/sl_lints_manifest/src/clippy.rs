//! tier-clippy-set: the project turns on the Clippy set its tiers expect, in its own `[lints.clippy]` table and
//! `clippy.toml`, and says why at every opt-out. `spirit_level` reads the files; it never runs Clippy with flags of
//! its own, so `cargo clippy` gives the same answer with or without it.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

use sl_analysis::Context;
use sl_config::clippy::{ClippySet, is_group};
use sl_core::{Diagnostic, Level, ParamValue, Span};
use sl_model::{TomlFile, toml_edit};
use std::fmt::Write;

const ID: &str = "tier-clippy-set";

/// One `[lints.clippy]` table to check
struct Table<'a> {
    file: &'a TomlFile,
    path: Vec<&'static str>,
    krate: Option<&'a str>,
}

pub(crate) fn run(cx: &Context<'_>) {
    let set = &cx.config.clippy;
    if !cx.enabled(ID) || set.is_empty() {
        return;
    }
    let p = cx.project;
    let mut tables = Vec::new();
    if p.is_workspace() {
        tables.push(Table {
            file: &p.root_manifest,
            path: vec!["workspace", "lints", "clippy"],
            krate: None,
        });
    }
    for pkg in &p.packages {
        let inherits = pkg
            .manifest
            .get(&["lints", "workspace"])
            .and_then(|v| v.as_bool())
            == Some(true);
        // a workspace member without any [lints] runs with no policy: workspace-lints reports it
        if !inherits && (!p.is_workspace() || pkg.manifest.get(&["lints"]).is_some()) {
            tables.push(Table {
                file: &pkg.manifest,
                path: vec!["lints", "clippy"],
                krate: Some(&pkg.name),
            });
        }
    }
    for t in &tables {
        check_table(cx, set, t);
    }
    check_config(cx, set);
}

/// A project entry: its level as rustc ranks it (allow 0, warn 1, deny 2, forbid 3) and the line it is on
struct Entry {
    name: String,
    rank: u8,
    key: Option<std::ops::Range<usize>>,
    end: usize,
}

fn rank(level: &str) -> Option<u8> {
    match level {
        "allow" => Some(0),
        "warn" | "expect" => Some(1),
        "deny" => Some(2),
        "forbid" => Some(3),
        _ => None,
    }
}

fn entries(t: &Table<'_>) -> Vec<Entry> {
    let Some(table) = t.file.get(&t.path).and_then(|i| i.as_table_like()) else {
        return Vec::new();
    };
    table
        .iter()
        .filter_map(|(k, item)| {
            let level = item
                .as_str()
                .or_else(|| item.get("level").and_then(|l| l.as_str()))?;
            let (key, _) = table.get_key_value(k)?;
            Some(Entry {
                name: k.replace('-', "_"),
                rank: rank(level)?,
                key: key.span(),
                end: item.span().map_or(0, |s| s.end),
            })
        })
        .collect()
}

fn check_table(cx: &Context<'_>, set: &ClippySet, t: &Table<'_>) {
    let header = format!("[{}]", t.path.join("."));
    let scope = t.krate.unwrap_or("workspace");
    let have = entries(t);
    let span = table_span(t);
    let mut missing = Vec::new();
    let mut low = Vec::new();
    for (name, want) in &set.lints {
        let need = if want.level == Level::Deny { 2 } else { 1 };
        if want.level == Level::Allow {
            continue;
        }
        match have.iter().find(|e| &e.name == name) {
            None => missing.push(name.as_str()),
            Some(e) if e.rank > 0 && e.rank < need => low.push(name.as_str()),
            Some(_) => {}
        }
    }
    if !missing.is_empty() || !low.is_empty() {
        let mut names: Vec<&str> = missing.iter().chain(&low).copied().collect();
        names.sort_unstable();
        let mut d = Diagnostic::new(
            ID,
            format!("{scope}:clippy-set"),
            format!(
                "{} of the {} Clippy lints the tier expects are not on in {header}: {}",
                names.len(),
                set.lints
                    .values()
                    .filter(|l| l.level > Level::Allow)
                    .count(),
                list(&names)
            ),
        )
        .span(span.clone())
        .help(paste(set, &header, &missing, &low, &have));
        if let Some(k) = t.krate {
            d = d.krate(k);
        }
        cx.emit(d);
    }
    for e in have.iter().filter(|e| e.rank == 0) {
        if reasoned(t.file, e) {
            continue;
        }
        let line = e.key.as_ref().map_or(0, |k| t.file.line_of_offset(k.start));
        let why = set.lints.get(&e.name).filter(|l| l.level == Level::Allow);
        let mut d = Diagnostic::new(
            ID,
            format!("{scope}:clippy.{}", e.name),
            format!(
                "clippy::{} is turned off in {header} without a reason",
                e.name
            ),
        )
        .span(Span::new(t.file.rel.clone(), line))
        .help(match why {
            Some(l) => format!(
                "say why after it, e.g. `{} = \"allow\" # {}`",
                e.name, l.reason
            ),
            None => format!(
                "say why after it: `{} = \"allow\" # why this lint does not fit here`",
                e.name
            ),
        });
        if let Some(k) = t.krate {
            d = d.krate(k);
        }
        cx.emit(d);
    }
}

/// The TOML that turns the missing lints on: groups first (priority -1), then single lints, then the tier's
/// accepted opt-outs when a group comes with the paste
fn paste(set: &ClippySet, header: &str, missing: &[&str], low: &[&str], have: &[Entry]) -> String {
    let mut s = String::new();
    if !missing.is_empty() {
        let _ = writeln!(
            s,
            "add to {header} (each line says why in a comment):\n\n{header}"
        );
        let (groups, single): (Vec<&str>, Vec<&str>) = missing.iter().partition(|n| is_group(n));
        for (name, prio) in groups
            .iter()
            .map(|g| (g, true))
            .chain(single.iter().map(|n| (n, false)))
        {
            let l = &set.lints[*name];
            let level = l.level.to_string();
            if prio {
                let _ = writeln!(
                    s,
                    "{name} = {{ level = \"{level}\", priority = -1 }} # {}",
                    l.reason
                );
            } else {
                let _ = writeln!(s, "{name} = \"{level}\" # {}", l.reason);
            }
        }
        if !groups.is_empty() {
            for (name, l) in &set.lints {
                if l.level == Level::Allow && !have.iter().any(|e| &e.name == name) {
                    let _ = writeln!(s, "{name} = \"allow\" # {}", l.reason);
                }
            }
        }
    }
    if !low.is_empty() {
        if !s.is_empty() {
            s.push('\n');
        }
        let _ = write!(s, "raise to deny in {header}: {}", low.join(", "));
    }
    s.trim_end().to_owned()
}

/// Whether an opt-out says why: a comment after it on its line, or a comment line right above it
fn reasoned(file: &TomlFile, e: &Entry) -> bool {
    let text = &file.text;
    let end = e.end.min(text.len());
    let eol = text[end..].find('\n').map_or(text.len(), |i| end + i);
    if text[end..eol].trim_start().starts_with('#') {
        return true;
    }
    let Some(k) = &e.key else {
        return false;
    };
    let bol = text[..k.start].rfind('\n').map_or(0, |i| i + 1);
    let above = text[..bol.saturating_sub(1)]
        .rsplit('\n')
        .next()
        .unwrap_or("");
    bol > 0 && above.trim_start().starts_with('#')
}

fn table_span(t: &Table<'_>) -> Span {
    (1..=t.path.len())
        .rev()
        .map(|n| t.file.span(&t.path[..n]))
        .find(|s| s.line > 0)
        .unwrap_or_else(|| Span::new(t.file.rel.clone(), 0))
}

fn list(names: &[&str]) -> String {
    const SHOWN: usize = 8;
    if names.len() <= SHOWN {
        names.join(", ")
    } else {
        format!(
            "{} and {} more",
            names[..SHOWN].join(", "),
            names.len() - SHOWN
        )
    }
}

fn check_config(cx: &Context<'_>, set: &ClippySet) {
    if set.config.is_empty() {
        return;
    }
    let p = cx.project;
    let (rel, text) = match p.read("clippy.toml") {
        Some(t) => ("clippy.toml", t),
        None => match p.read(".clippy.toml") {
            Some(t) => (".clippy.toml", t),
            None => ("clippy.toml", String::new()),
        },
    };
    let file = match TomlFile::parse(rel, text) {
        Ok(f) => f,
        Err(e) => {
            cx.emit(
                Diagnostic::new(
                    ID,
                    format!("{rel}:parse"),
                    format!("{rel} does not parse: {e}"),
                )
                .span(Span::new(rel, 0))
                .help("Clippy refuses it too; fix the TOML"),
            );
            return;
        }
    };
    for (key, want) in &set.config {
        let have = file.get(&[key]).and_then(|i| i.as_value());
        let msg = match (have, &want.value) {
            (None, _) => format!("{rel} does not set {key}"),
            (Some(v), ParamValue::Int(max)) => match v.as_integer() {
                Some(n) if n <= *max => continue,
                _ => format!(
                    "{rel} sets {key} = {}, above the tier's {max}",
                    v.to_string().trim()
                ),
            },
            (Some(v), w) if same(v, w) => continue,
            (Some(v), w) => format!(
                "{rel} sets {key} = {}; the tier expects {w}",
                v.to_string().trim()
            ),
        };
        let bound = if matches!(want.value, ParamValue::Int(_)) {
            " (or lower)"
        } else {
            ""
        };
        cx.emit(
            Diagnostic::new(ID, format!("{rel}:{key}"), msg)
                .span(file.span(&[key]))
                .help(format!(
                    "write in {rel}{bound}:\n\n{key} = {} # {}",
                    want.value, want.reason
                )),
        );
    }
}

fn same(v: &toml_edit::Value, w: &ParamValue) -> bool {
    match w {
        ParamValue::Int(i) => v.as_integer() == Some(*i),
        ParamValue::Float(f) => v.as_float() == Some(*f),
        ParamValue::Bool(b) => v.as_bool() == Some(*b),
        ParamValue::Str(s) => v.as_str() == Some(s.as_str()),
        ParamValue::StrList(l) => v.as_array().is_some_and(|a| {
            a.iter()
                .map(toml_edit::Value::as_str)
                .eq(l.iter().map(|s| Some(s.as_str())))
        }),
        ParamValue::StrTable(t) => v.as_inline_table().is_some_and(|i| {
            i.len() == t.len()
                && t.iter()
                    .all(|(k, s)| i.get(k).and_then(toml_edit::Value::as_str) == Some(s.as_str()))
        }),
    }
}
