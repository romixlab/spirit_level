//! The panic sites of a release ELF ([`sl_firmware::panics`]): the report `spl panics` prints, and `pan-1`, which
//! holds every site to an entry with a reason in the firmware's acknowledgement file.
//!
//! ```toml
//! # panics.toml, beside the firmware's Cargo.toml
//! [[site]]
//! function = "app::power::duty"   # the function path without generic arguments
//! kind = "panic"                  # unwrap, expect, panic, assert, bounds, slice, overflow, division, refcell, alloc, resumed
//! index = 0                       # among the sites of this kind in this function, in source order
//! reason = "a zero period is a broken config table; the stage is off until the reset"
//!
//! [[crate]]                       # every site of a dependency, at once
//! name = "embassy_stm32"
//! reason = "the HAL's own invariants; its config errors fire at init, before any output"
//! ```

use sl_analysis::{Context, lint};
use sl_core::{Diagnostic, Need, Params, Span};
use sl_firmware::Firmware;
use sl_firmware::panics::{self, Inventory, Kind, NotRun, Site};
use sl_model::TomlFile;
use std::collections::BTreeSet;

lint!(PAN_1, "F0110", "pan-1", "pan", "every panic site of the release ELF is acknowledged with a reason",
    params: [
        ("file", Str, "\"panics.toml\"", "the acknowledgement file, relative to the firmware crate directory"),
        ("panic-fns", StrList, "[\"defmt::export::panic\"]", "functions outside the standard library that panic for their caller: a call to one is a site"),
        ("zero-in", StrList, "[\"service\"]", "task classes whose modules have no panic site in release: an entry does not acknowledge one there"),
    ],
    needs: [Need::Elf], fix: None);

/// The panic sites of one firmware's ELF
#[derive(Debug, serde::Serialize)]
pub struct FirmwarePanics {
    /// The firmware crate
    pub krate: String,
    /// The ELF, relative to the crate directory when inside it
    pub elf: String,
    /// The inventory: its sites and the count of calls inside the standard library
    #[serde(flatten)]
    pub inventory: Option<Inventory>,
    /// Why there is no inventory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_run: Option<String>,
}

/// The panic sites of each firmware crate of the project whose ELF is among `elfs`, placed in the project
pub fn report(
    project: &sl_model::Project,
    config: &sl_config::Config,
    elfs: &[std::path::PathBuf],
) -> Vec<FirmwarePanics> {
    let mut out = Vec::new();
    for mut fw in Firmware::load_all(project, elfs) {
        fw.classes = config.embedded.classes.clone();
        let Some(elf) = &fw.elf else { continue };
        let params = config.lint(PAN_1.id).params.clone();
        let result = inventory(&fw, &params);
        out.push(FirmwarePanics {
            krate: fw.name.clone(),
            elf: elf.rel.clone(),
            not_run: result.as_ref().err().map(ToString::to_string),
            inventory: result.ok(),
        });
    }
    out
}

fn inventory(fw: &Firmware<'_>, params: &Params) -> Result<Inventory, NotRun> {
    let elf = fw.elf.as_ref().ok_or(NotRun::NoDebugInfo)?;
    let mut inv = panics::inventory(elf, params.list("panic-fns"))?;
    panics::locate(&mut inv, fw);
    Ok(inv)
}

/// An entry of the acknowledgement file
struct Entry {
    /// `function kind index` of a `[[site]]`, or the crate of a `[[crate]]`
    target: Target,
    reason: String,
    line: u32,
}

#[derive(PartialEq, Eq)]
enum Target {
    Site(String, Kind, u32),
    Crate(String),
}

/// `pan-1` on one firmware with an ELF
pub(crate) fn run(cx: &Context<'_>, fw: &Firmware<'_>) {
    let params = cx.params_for(PAN_1.id, fw.package);
    let inv = match inventory(fw, &params) {
        Ok(i) => i,
        Err(e) => {
            cx.not_run(PAN_1.id, "dwarf", e.to_string());
            return;
        }
    };
    let file_name = params.str("file");
    let rel = fw.package.path(file_name);
    let (entries, mut out) = read_entries(fw, &rel);
    let zero_in = params.list("zero-in");
    let project_crates: BTreeSet<String> = fw
        .project
        .packages
        .iter()
        .map(|p| p.name.replace('-', "_"))
        .collect();
    let mut used = vec![false; entries.len()];
    for s in &inv.sites {
        let krate = crate_of(&s.function);
        let span = site_span(fw, s);
        let what = describe(s);
        let hit = entries.iter().position(|e| match &e.target {
            Target::Site(f, k, i) => *f == s.function && *k == s.kind && *i == s.index,
            Target::Crate(c) => *c == krate && !project_crates.contains(c),
        });
        if let Some(i) = hit {
            used[i] = true;
        }
        let zero = s.kind != Kind::Resumed
            && s.class
                .is_some_and(|c| zero_in.iter().any(|z| z == c.name()));
        if let Some(c) = s.class.filter(|_| zero) {
            out.push(
                Diagnostic::new(PAN_1.id, key(s), format!("{what} is in a {} module: it resets the whole MCU in release, and an entry does not acknowledge it", c.name()))
                    .span(span)
                    .help("handle it without a panic: a Result, `.get(i)`, `checked_*`, a debug_assert! with a counter and a fallback"),
            );
            continue;
        }
        if hit.is_none() {
            out.push(
                Diagnostic::new(PAN_1.id, key(s), format!("{what} is not acknowledged in {file_name}"))
                    .span(span)
                    .help(format!("remove the panic, or add it to {file_name}: [[site]] function = \"{}\", kind = \"{}\", index = {}, reason = \"why it cannot fire, or why a reset is the right answer\"", s.function, s.kind.name(), s.index)),
            );
        }
    }
    out.extend(entry_findings(
        &entries,
        &used,
        &project_crates,
        file_name,
        &rel,
    ));
    for mut d in out {
        d.item = format!("{}:{}", fw.name, d.item);
        d.krate = Some(fw.name.clone());
        cx.emit(d);
    }
}

/// The entries that match no site, acknowledge a project crate at once, or give no reason
fn entry_findings(
    entries: &[Entry],
    used: &[bool],
    project_crates: &BTreeSet<String>,
    file_name: &str,
    rel: &str,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (e, &used) in entries.iter().zip(used) {
        let (item, msg, help) = match &e.target {
            Target::Site(f, k, i) if !used => (
                format!("{f}#{}#{i}", k.name()),
                format!(
                    "{file_name} acknowledges {} {i} in {f}, which the ELF no longer has",
                    k.name()
                ),
                "remove the entry, or point it at the site `spl panics` lists",
            ),
            Target::Crate(c) if project_crates.contains(c) => (
                format!("crate:{c}"),
                format!(
                    "{file_name} acknowledges all of {c} at once: the project's own crates are acknowledged site by site"
                ),
                "list the crate's sites as [[site]] entries (`spl panics --ack` prints them)",
            ),
            Target::Crate(c) if !used => (
                format!("crate:{c}"),
                format!(
                    "{file_name} acknowledges the crate {c}, which has no panic site in the ELF"
                ),
                "remove the entry",
            ),
            _ if e.reason.trim().is_empty() => (
                format!("reason:{}", e.line),
                format!("an entry of {file_name} gives no reason"),
                "say why the site cannot fire, or why a reset is the right answer there",
            ),
            _ => continue,
        };
        out.push(
            Diagnostic::new(PAN_1.id, item, msg)
                .span(Span::new(rel, e.line))
                .help(help),
        );
    }
    out
}

/// The entries of the acknowledgement file, and what is wrong with it; no file is no entry
fn read_entries(fw: &Firmware<'_>, rel: &str) -> (Vec<Entry>, Vec<Diagnostic>) {
    let Ok(text) = std::fs::read_to_string(fw.root.join(rel)) else {
        return (Vec::new(), Vec::new());
    };
    let bad = |line: u32, msg: String| {
        Diagnostic::new(PAN_1.id, format!("file:{line}"), msg)
            .span(Span::new(rel, line))
            .help("each [[site]] has function, kind, index and reason; each [[crate]] has name and reason")
    };
    let file = match TomlFile::parse(rel, text) {
        Ok(f) => f,
        Err(e) => return (Vec::new(), vec![bad(1, format!("{rel} is not TOML: {e}"))]),
    };
    let mut entries = Vec::new();
    let mut errs = Vec::new();
    for (key, item) in file.root() {
        let Some(aot) = item
            .as_array_of_tables()
            .filter(|_| key == "site" || key == "crate")
        else {
            errs.push(bad(
                file.line(&[key]),
                format!("unknown key {key} in {rel}: [[site]] and [[crate]] only"),
            ));
            continue;
        };
        for t in aot {
            let line = t.span().map_or(1, |r| file.line_of_offset(r.start));
            let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_owned);
            let reason = s("reason").unwrap_or_default();
            let target = if key == "crate" {
                s("name").map(Target::Crate)
            } else {
                let kind = s("kind").and_then(|k| Kind::parse(&k));
                let index = t
                    .get("index")
                    .and_then(toml_edit_int)
                    .and_then(|i| u32::try_from(i).ok());
                match (s("function"), kind, index) {
                    (Some(f), Some(k), Some(i)) => Some(Target::Site(f, k, i)),
                    _ => None,
                }
            };
            match target {
                Some(target) => entries.push(Entry { target, reason, line }),
                None => errs.push(bad(line, format!("an entry of {rel} is incomplete: a [[site]] names function, kind (one of {}) and index; a [[crate]] names name", Kind::ALL.map(Kind::name).join(", ")))),
            }
        }
    }
    (entries, errs)
}

fn toml_edit_int(v: &sl_model::toml_edit::Item) -> Option<i64> {
    v.as_integer()
}

/// The crate a function path is in (`app` of `app::led::blink`, of `<app::Foo as core::fmt::Debug>::fmt`)
fn crate_of(function: &str) -> String {
    function
        .trim_start_matches('<')
        .split("::")
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn key(s: &Site) -> String {
    format!("{}#{}#{}", s.function, s.kind.name(), s.index)
}

/// `assert! (panic 1) in app::power::duty`
fn describe(s: &Site) -> String {
    let what = match s.macro_name {
        Some(m) if s.kind == Kind::Panic => format!("{m}! ({} {})", s.kind.name(), s.index),
        _ => format!("{} {}", s.kind.name(), s.index),
    };
    format!("{what} in {}", s.function)
}

/// The site's place in the project, else the ELF
fn site_span(fw: &Firmware<'_>, s: &Site) -> Span {
    match (&s.rel, &s.place) {
        (Some(rel), Some(p)) => Span::at(rel, p.line, p.column.max(1)),
        _ => fw.elf_span(format!(
            "{} at {:#x}",
            s.symbol,
            s.addrs.first().copied().unwrap_or_default()
        )),
    }
}
