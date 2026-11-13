//! `err-11` Silicon revision read from a register that can't tell all revisions apart

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0021", "err-11", "err", "the silicon revision is read from a source that tells every revision apart",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// On RP2350, `CHIP_ID.REVISION` read (`chip_id().read().revision()`) in a firmware that never reads the
/// bootrom's version byte: RP2350 datasheet appendix C, "Stepping A3 is identified by a CHIP_ID.REVISION value
/// of 0x3, and a bootrom version byte at offset 0x00000013 of 0x3", "Stepping A4 is identified by a
/// CHIP_ID.REVISION value of 0x3, and a bootrom version byte at offset 0x00000013 of 0x4".
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !scan::family_in(fw, &["rp235x"]) {
        return Vec::new();
    }
    let words = scan::words(fw);
    let rom_version = words.iter().any(|w| {
        let s = scan::segments(&w.text);
        s.iter().any(|x| x == "bootrom" || x == "chip_version")
            || (s.iter().any(|x| x == "rom") && s.iter().any(|x| x == "version"))
    });
    if rom_version {
        return Vec::new();
    }
    let mut out = Vec::new();
    for f in scan::functions(fw) {
        for c in scan::method_calls(&f) {
            if c.name == "revision" && c.receiver.contains("chip_id") {
                out.push(
                    Diagnostic::new(
                        "err-11",
                        format!("{}:revision", f.path()),
                        "the silicon revision is taken from CHIP_ID.REVISION, which is 3 on both RP2350 A3 and A4",
                    )
                    .span(c.loc.span())
                    .help("tell the revision by the bootrom version byte at 0x00000013 (3 on A3, 4 on A4), keep REVISION as the raw value"),
                );
            }
        }
    }
    out
}
