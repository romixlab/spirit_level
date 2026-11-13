//! `siz-6` Large `.data` initializer

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Need;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::elf::{SymKind, bytes};

lint!(META, "F0090", "siz-6", "siz", "no large .data initializer",
    params: [("data-init", Int, "1024", "bytes of a .data initializer from which it is reported")], needs: [Need::Elf], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Elf];


/// Objects in a section that is copied from flash at startup, of the `data-init` parameter bytes or more
pub(crate) fn run(fw: &Firmware<'_>, params: &Params) -> Vec<Diagnostic> {
    let Some(elf) = &fw.elf else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for s in &elf.symbols {
        if s.kind != SymKind::Object || s.size < crate::limit(params, "data-init") {
            continue;
        }
        let Some(sec) = s.section.as_ref().and_then(|n| elf.section(n)) else {
            continue;
        };
        if !sec.is_init_data() {
            continue;
        }
        out.push(
            Diagnostic::new(
                "siz-6",
                format!("elf:{}:{}", sec.name, s.name),
                format!(
                    "`{}` is {} of {}: its initial value is stored in flash and copied to RAM at startup",
                    s.name,
                    bytes(s.size),
                    sec.name
                ),
            )
            .span(fw.symbol_span(s))
            .help("make its initial value all zeros (it goes to .bss) and set it up in place at run time"),
        );
    }
    out
}
