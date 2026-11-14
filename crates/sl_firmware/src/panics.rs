//! The panic sites of a linked Arm ELF.
//!
//! A site is a call from the firmware's code into the standard library's panic entry points
//! (`core::panicking::*`, `unwrap_failed`, `panic_bounds_check`, ...), placed in the source with the DWARF of the
//! call instruction.
//!
//! Its place is the innermost frame of the DWARF inline chain that is not standard
//! library code, so an `unwrap()` inlined from `Option::unwrap` is placed at the `.unwrap()` in the caller's
//! source, and its kind comes from the standard library frames between that place and the call. Calls that
//! inlining or monomorphization copied are one site with several addresses. A site is keyed by its function's path
//! without generic arguments, its kind and its index among the sites of that kind in that function, in source
//! order: a key survives edits elsewhere and new lines above it.
//!
//! What the ELF cannot tell: a check the optimizer proved never fails is not in the binary (and cannot panic); a
//! call several source sites were folded into has no line (DWARF line 0), and is listed under the function that
//! holds it; with `panic-immediate-abort` there are no calls to list, every panic is a trap; without DWARF there
//! are no places. Only Thumb code is read.

use crate::elf::{Elf, Machine, SymKind, demangle};
use addr2line::gimli;
use object::read::elf::ElfFile32;
use object::{Object, ObjectSection};
use std::collections::BTreeMap;
use std::fmt;

/// What kind of panic a site is
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// `.unwrap()` on an `Option` or a `Result`
    Unwrap,
    /// `.expect(..)`
    Expect,
    /// `panic!`, `assert!`, `unreachable!`, `todo!`, `unimplemented!` (the source tells which)
    Panic,
    /// `assert_eq!`, `assert_ne!`
    Assert,
    /// An index out of bounds (`x[i]`)
    Bounds,
    /// A slice or string range out of bounds, a length mismatch
    Slice,
    /// An arithmetic overflow (with `overflow-checks`), a shift too far
    Overflow,
    /// A division or remainder by zero
    Division,
    /// A `RefCell` already borrowed
    Refcell,
    /// An allocation that failed or overflowed
    Alloc,
}

impl Kind {
    /// The name in reports and in the acknowledgement file (`unwrap`)
    pub fn name(self) -> &'static str {
        match self {
            Self::Unwrap => "unwrap",
            Self::Expect => "expect",
            Self::Panic => "panic",
            Self::Assert => "assert",
            Self::Bounds => "bounds",
            Self::Slice => "slice",
            Self::Overflow => "overflow",
            Self::Division => "division",
            Self::Refcell => "refcell",
            Self::Alloc => "alloc",
        }
    }

    /// The kind of a name
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == s)
    }

    /// Every kind
    pub const ALL: [Self; 10] = [
        Self::Unwrap,
        Self::Expect,
        Self::Panic,
        Self::Assert,
        Self::Bounds,
        Self::Slice,
        Self::Overflow,
        Self::Division,
        Self::Refcell,
        Self::Alloc,
    ];
}

/// Where a site is in the source, as the DWARF says
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Place {
    /// The file as the DWARF names it (absolute, or `/rustc/<hash>/library/..`)
    pub file: String,
    /// 1-based line
    pub line: u32,
    /// 1-based column, 0 when the DWARF has none
    pub column: u32,
}

/// One panic site
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Site {
    /// The path of the function the site is written in, without generic arguments (`app::led::blink`)
    pub function: String,
    /// What panics
    pub kind: Kind,
    /// Its index among the sites of this kind in this function, in source order
    pub index: u32,
    /// Where it is written; `None` when the optimizer folded several sites into this call (DWARF line 0)
    pub place: Option<Place>,
    /// The panic entry it calls (`core::panicking::panic_fmt`)
    pub via: String,
    /// The ELF function that holds the call
    pub symbol: String,
    /// The call instructions: more than one where inlining or monomorphization copied the site
    pub addrs: Vec<u64>,
    /// The function is in a crate of the project, folded or not ([`locate`])
    pub in_project: bool,
    /// The file relative to the project root, when it is a file of the project ([`locate`])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    /// The task class of that file ([`locate`])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<sl_core::TaskClass>,
    /// The panicking macro written at the place, read from the source (`assert`, `unreachable`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_name: Option<&'static str>,
}

impl Site {
    /// The key that names the site in the acknowledgement file: `function kind index`
    pub fn key(&self) -> (String, Kind, u32) {
        (self.function.clone(), self.kind, self.index)
    }
}

/// The panic sites of an ELF
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Inventory {
    /// The sites, by function, kind and index
    pub sites: Vec<Site>,
    /// Calls into a panic entry from the standard library's own code, not listed
    pub in_std: usize,
}

/// Why the inventory cannot be made
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotRun {
    /// The ELF is not Arm: only Thumb code is read
    Machine(Machine),
    /// No `.debug_info` or `.debug_line`
    NoDebugInfo,
    /// Compressed debug sections
    Compressed,
    /// No panic handler and no `core::panicking` function: built with `panic-immediate-abort`, or nothing can panic
    NoPanicEntry,
    /// The DWARF could not be read
    Dwarf(String),
}

impl fmt::Display for NotRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Machine(m) => write!(f, "the ELF is for {m:?}: only Arm Thumb code is read"),
            Self::NoDebugInfo => write!(
                f,
                "the ELF has no DWARF (.debug_info, .debug_line): set `debug = 2` in the release profile; debug info does not reach the flash"
            ),
            Self::Compressed => write!(
                f,
                "the ELF's debug sections are compressed: link without --compress-debug-sections"
            ),
            Self::NoPanicEntry => write!(
                f,
                "the ELF has no panic handler and no core::panicking function: built with panic-immediate-abort (every panic is a trap no call names), or no panic path is left"
            ),
            Self::Dwarf(e) => write!(f, "the ELF's DWARF cannot be read: {e}"),
        }
    }
}

type Reader<'a> = gimli::EndianSlice<'a, gimli::RunTimeEndian>;

/// The panic sites of `elf`. `panic_fns` names functions outside the standard library that panic for their caller
/// (`defmt::export::panic`): a call to one is a site, and a frame of one is not the site's place.
pub fn inventory(elf: &Elf, panic_fns: &[String]) -> Result<Inventory, NotRun> {
    if elf.machine != Machine::Arm {
        return Err(NotRun::Machine(elf.machine));
    }
    let file = ElfFile32::<object::Endianness>::parse(&*elf.data.0)
        .map_err(|e| NotRun::Dwarf(e.to_string()))?;
    let cx = context(&file)?;
    let names: Vec<String> = elf.symbols.iter().map(|s| normalize(&s.name)).collect();
    if !elf.symbols.iter().any(|s| s.raw == "rust_begin_unwind")
        && !names.iter().any(|n| n.starts_with("core::panicking::"))
    {
        return Err(NotRun::NoPanicEntry);
    }
    let entries = entries(elf, &names, panic_fns);
    let mut out = Inventory::default();
    let mut found: Vec<Site> = Vec::new();
    for code in &elf.code {
        let holder = &names[code.symbol];
        for &(pc, callee) in &code.sites {
            let Some(&callee_kind) = entries.get(&callee) else {
                continue;
            };
            let frames = frames(&cx, pc).map_err(|e| NotRun::Dwarf(e.to_string()))?;
            let Some((function, place, kind)) = attribute(&frames, holder, callee_kind, panic_fns)
            else {
                out.in_std += 1;
                continue;
            };
            match found.iter_mut().find(|s| {
                place.is_some() && s.function == function && s.kind == kind && s.place == place
            }) {
                Some(s) => s.addrs.push(pc),
                None => found.push(Site {
                    function,
                    kind,
                    index: 0,
                    place,
                    via: names[callee].clone(),
                    symbol: holder.clone(),
                    addrs: vec![pc],
                    in_project: false,
                    rel: None,
                    class: None,
                    macro_name: None,
                }),
            }
        }
    }
    number(&mut found);
    out.sites = found;
    Ok(out)
}

/// The DWARF of the ELF, ready for lookups
fn context<'d>(
    file: &ElfFile32<'d, object::Endianness>,
) -> Result<addr2line::Context<Reader<'d>>, NotRun> {
    for name in [".debug_info", ".debug_line"] {
        let s = file.section_by_name(name).ok_or(NotRun::NoDebugInfo)?;
        if s.size() == 0 {
            return Err(NotRun::NoDebugInfo);
        }
        if !matches!(
            s.compressed_file_range().map(|r| r.format),
            Ok(object::CompressionFormat::None)
        ) {
            return Err(NotRun::Compressed);
        }
    }
    let endian = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    let dwarf = gimli::Dwarf::load(|id| -> Result<Reader<'d>, gimli::Error> {
        let bytes = file
            .section_by_name(id.name())
            .and_then(|s| s.data().ok())
            .unwrap_or(&[]);
        Ok(gimli::EndianSlice::new(bytes, endian))
    })
    .map_err(|e| NotRun::Dwarf(e.to_string()))?;
    addr2line::Context::from_dwarf(dwarf).map_err(|e| NotRun::Dwarf(e.to_string()))
}

/// The panic entries among the function symbols, with the kind their name stands for (`None` for `panic_fns`)
fn entries(elf: &Elf, names: &[String], panic_fns: &[String]) -> BTreeMap<usize, Option<Kind>> {
    names
        .iter()
        .enumerate()
        .filter(|(i, _)| elf.symbols[*i].kind == SymKind::Func)
        .filter_map(|(i, n)| {
            if panic_fns.iter().any(|p| p == n) {
                Some((i, None))
            } else if is_std(n) {
                classify(n).map(|k| (i, Some(k)))
            } else {
                None
            }
        })
        .collect()
}

/// Where a call is written and what kind of panic it is: the innermost frame that is neither the standard library
/// nor a `panic_fns` function, the kind of the library frames between it and the call (outermost first), else the
/// callee's. `None` for a call inside the standard library.
fn attribute(
    frames: &[(String, Option<Place>)],
    holder: &str,
    callee_kind: Option<Kind>,
    panic_fns: &[String],
) -> Option<(String, Option<Place>, Kind)> {
    let lib = |(f, p): &(String, Option<Place>)| {
        p.as_ref().is_some_and(|p| is_std_file(&p.file)) || is_std(f) || panic_fns.contains(f)
    };
    let split = frames.iter().position(|f| !lib(f));
    let (function, place) = match split {
        Some(i) => frames[i].clone(),
        None if frames.is_empty() && !is_std(holder) => (holder.to_owned(), None),
        None => return None,
    };
    let kind = frames[..split.unwrap_or(0)]
        .iter()
        .rev()
        .find_map(|(f, _)| classify(f))
        .or(callee_kind)
        .unwrap_or(Kind::Panic);
    Some((function, place.filter(|p| p.line > 0), kind))
}

/// Sorts the sites into source order within a function and kind (folded sites, without a place, after the
/// placed ones, by address) and numbers them
fn number(found: &mut [Site]) {
    found.sort_by(|a, b| {
        (&a.function, a.kind, a.place.is_none(), &a.place, &a.addrs).cmp(&(
            &b.function,
            b.kind,
            b.place.is_none(),
            &b.place,
            &b.addrs,
        ))
    });
    let mut last: Option<(String, Kind)> = None;
    let mut n = 0;
    for s in found {
        let k = (s.function.clone(), s.kind);
        n = if last.as_ref() == Some(&k) { n + 1 } else { 0 };
        s.index = n;
        s.addrs.sort_unstable();
        last = Some(k);
    }
}

/// Places the sites in the firmware's project.
///
/// A site in a function of a project crate gets its file relative to the project root (the DWARF path may come
/// from another checkout: the longest tail of it that is a file of that crate), the task class of that file, and
/// the panicking macro written at its place.
pub fn locate(inv: &mut Inventory, fw: &crate::Firmware<'_>) {
    let mut texts: BTreeMap<String, Option<String>> = BTreeMap::new();
    for s in &mut inv.sites {
        let krate = s.function.trim_start_matches('<');
        let krate = krate.split("::").next().unwrap_or_default();
        let Some(pkg) = fw
            .project
            .packages
            .iter()
            .find(|p| p.name.replace('-', "_") == krate)
        else {
            continue;
        };
        s.in_project = true;
        let Some(place) = &s.place else { continue };
        let parts: Vec<&str> = place
            .file
            .split(['/', '\\'])
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        let Some(in_pkg) = (0..parts.len())
            .map(|k| parts[k..].join("/"))
            .find(|tail| fw.root.join(&pkg.dir).join(tail).is_file())
        else {
            continue;
        };
        let rel = pkg.path(&in_pkg);
        s.class = fw.classes.class_of(&in_pkg);
        let text = texts
            .entry(rel.clone())
            .or_insert_with(|| std::fs::read_to_string(fw.root.join(&rel)).ok());
        if s.kind == Kind::Panic {
            s.macro_name = text
                .as_deref()
                .and_then(|t| t.lines().nth(place.line as usize - 1))
                .and_then(|l| macro_on(l, place.column));
        }
        s.rel = Some(rel);
    }
}

/// The inline chain at `pc`, innermost first: each frame's function (normalized) and its place
fn frames(
    cx: &addr2line::Context<Reader<'_>>,
    pc: u64,
) -> Result<Vec<(String, Option<Place>)>, gimli::Error> {
    let mut out = Vec::new();
    let mut it = cx.find_frames(pc).skip_all_loads()?;
    while let Some(f) = it.next()? {
        let function = f
            .function
            .as_ref()
            .and_then(|n| n.raw_name().ok())
            .map(|n| normalize(&demangle(&n)))
            .unwrap_or_default();
        let place = f.location.and_then(|l| {
            Some(Place {
                file: l.file?.to_owned(),
                line: l.line.unwrap_or(0),
                column: l.column.unwrap_or(0),
            })
        });
        out.push((function, place));
    }
    Ok(out)
}

/// A function of the standard library, by its path
fn is_std(name: &str) -> bool {
    let n = name.trim_start_matches('<');
    ["core::", "alloc::", "std::", "compiler_builtins::"]
        .iter()
        .any(|p| n.starts_with(p))
}

/// A source file of the standard library (`/rustc/<hash>/library/core/src/option.rs`, a `rust-src` copy)
fn is_std_file(file: &str) -> bool {
    ["/library/core/", "/library/alloc/", "/library/std/"]
        .iter()
        .any(|p| file.contains(p))
}

/// Name parts of standard library functions and the kind each stands for, checked in order
const KINDS: &[(&[&str], Kind)] = &[
    (&["expect_failed"], Kind::Expect),
    (&["unwrap_failed"], Kind::Unwrap),
    (
        &["RefCell", "already_borrowed", "already_mutably_borrowed"],
        Kind::Refcell,
    ),
    (&["panic_bounds_check"], Kind::Bounds),
    (
        &["handle_alloc_error", "capacity_overflow", "raw_vec"],
        Kind::Alloc,
    ),
    (&["div_by_zero", "rem_by_zero"], Kind::Division),
    (&["_overflow"], Kind::Overflow),
    (
        &[
            "_index_len_fail",
            "slice_index_fail",
            "slice_index_order_fail",
            "slice_error_fail",
            "str_index",
            "len_mismatch_fail",
        ],
        Kind::Slice,
    ),
    (&["assert_failed"], Kind::Assert),
    (&["panic", "unreachable"], Kind::Panic),
];

/// The kind of panic a standard library function stands for, by its path
fn classify(name: &str) -> Option<Kind> {
    match name.rsplit("::").next() {
        Some("expect") => return Some(Kind::Expect),
        Some("unwrap") => return Some(Kind::Unwrap),
        _ => {}
    }
    KINDS
        .iter()
        .find(|(parts, _)| parts.iter().any(|p| name.contains(p)))
        .map(|(_, k)| *k)
}

/// A function path without generic arguments: `<core::option::Option<u8>>::unwrap` is
/// `<core::option::Option>::unwrap`, `app::scaled::<u16>` is `app::scaled`. Monomorphizations of one function share
/// a path.
pub fn normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        let generic = c == '<'
            && out
                .chars()
                .last()
                .is_some_and(|p| p.is_alphanumeric() || p == '_' || p == ':');
        if !generic {
            out.push(c);
            continue;
        }
        if out.ends_with("::") {
            out.truncate(out.len() - 2);
        }
        let mut depth = 1;
        let mut prev = c;
        for c in chars.by_ref() {
            match c {
                '<' => depth += 1,
                '>' if prev != '-' => depth -= 1,
                _ => {}
            }
            prev = c;
            if depth == 0 {
                break;
            }
        }
    }
    out
}

/// The panicking macro written on a source line at a 1-based column, else the first on the line: `assert`,
/// `unreachable`, `panic`, ...
fn macro_on(line: &str, column: u32) -> Option<&'static str> {
    const MACROS: &[&str] = &[
        "debug_assert_eq",
        "debug_assert_ne",
        "debug_assert",
        "assert_eq",
        "assert_ne",
        "assert",
        "unreachable",
        "unimplemented",
        "todo",
        "panic",
        "unwrap",
        "expect",
    ];
    let from = (column as usize).saturating_sub(1).min(line.len());
    let find = |text: &str| {
        let mut best: Option<(usize, &'static str)> = None;
        for m in MACROS {
            let pat = format!("{m}!");
            let mut start = 0;
            while let Some(i) = text[start..].find(&pat) {
                let at = start + i;
                let word_start = at == 0
                    || !text[..at]
                        .chars()
                        .last()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_');
                if word_start && best.is_none_or(|(b, _)| at < b) {
                    best = Some((at, m));
                }
                start = at + pat.len();
            }
        }
        best.map(|(_, m)| m)
    };
    line.get(from..).and_then(find).or_else(|| find(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_drops_generic_arguments() {
        assert_eq!(
            normalize("<core::option::Option<usize>>::unwrap"),
            "<core::option::Option>::unwrap"
        );
        assert_eq!(normalize("app::power::scaled::<u16>"), "app::power::scaled");
        assert_eq!(
            normalize("<app::Foo<u8> as core::fmt::Debug>::fmt"),
            "<app::Foo as core::fmt::Debug>::fmt"
        );
        assert_eq!(
            normalize("app::run::<fn() -> u8>::{closure#0}"),
            "app::run::{closure#0}"
        );
    }

    #[test]
    fn kinds_by_name() {
        assert_eq!(
            classify("<core::option::Option>::expect"),
            Some(Kind::Expect)
        );
        assert_eq!(classify("core::result::unwrap_failed"), Some(Kind::Unwrap));
        assert_eq!(
            classify("core::panicking::panic_bounds_check"),
            Some(Kind::Bounds)
        );
        assert_eq!(
            classify("core::panicking::panic_const::panic_const_add_overflow"),
            Some(Kind::Overflow)
        );
        assert_eq!(
            classify("core::panicking::panic_const::panic_const_div_by_zero"),
            Some(Kind::Division)
        );
        assert_eq!(
            classify("core::slice::index::slice_end_index_len_fail"),
            Some(Kind::Slice)
        );
        assert_eq!(
            classify("<core::cell::RefCell>::borrow_mut"),
            Some(Kind::Refcell)
        );
        assert_eq!(
            classify("core::panicking::assert_failed"),
            Some(Kind::Assert)
        );
        assert_eq!(classify("core::panicking::panic_fmt"), Some(Kind::Panic));
        assert_eq!(classify("core::fmt::write"), None);
    }

    #[test]
    fn macro_at_the_column_first() {
        let l = "    assert!(a, \"x\"); panic!(\"y\")";
        assert_eq!(macro_on(l, 5), Some("assert"));
        assert_eq!(macro_on(l, 22), Some("panic"));
        assert_eq!(macro_on("    debug_assert!(x)", 1), Some("debug_assert"));
        assert_eq!(macro_on("    x.unwrap()", 1), None);
    }
}
