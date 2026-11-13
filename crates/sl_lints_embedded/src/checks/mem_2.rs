//! `mem-2` Extra RAM section with a tail `. = ALIGN(n)` or a symbol in it

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "F0070", "mem-2", "mem", "extra RAM sections end without a tail ALIGN or symbol",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::MemoryX];


const FIX: &str = "a NOLOAD section must hold only `*(.name .name.*);`: lld keeps an empty section that flip-link \
    counts as used RAM when the location counter advances or a symbol sits in it (flip-link#142)";

/// Walks every `.name (NOLOAD) : ALIGN(n) { ... }` section body and flags the first line in it that isn't
/// `*(.name .name.*);`
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let Some(mx) = &fw.memory_x else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut section: Option<(String, bool)> = None; // name, past the opening brace
    for (i, line) in mx.text.lines().enumerate() {
        let t = line.trim();
        let Some((name, in_body)) = &mut section else {
            if t.contains("(NOLOAD)") {
                let name = t.split_whitespace().next().unwrap_or("").to_string();
                section = Some((name, false));
            }
            continue;
        };
        if !*in_body {
            if t == "{" {
                *in_body = true;
            }
            continue;
        }
        if t.starts_with('}') {
            section = None;
            continue;
        }
        if t.is_empty() || t.starts_with("*(") {
            continue;
        }
        out.push(
            Diagnostic::new(
                "mem-2",
                format!("memory.x:{name}"),
                format!("{name} (NOLOAD) has a line besides `*(...)`: `{t}`"),
            )
            .span(Span::new(mx.rel.clone(), i as u32 + 1))
            .help(FIX),
        );
        section = None;
    }
    out
}
