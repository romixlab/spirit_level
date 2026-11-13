//! `fls-2` More than one owner of the flash peripheral

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use super::fls_1::flash_drivers;
use sl_core::Diagnostic;
use sl_firmware::Firmware;
use sl_firmware::scan;

lint!(META, "F0045", "fls-2", "fls", "the flash peripheral has one owner",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[FwNeed::Sources];


/// More than one flash driver built (`Flash::new*` twice: the updater and the settings store each take
/// `FLASH`, one through `steal()`), or the `FLASH` singleton stolen: two drivers interleave erase and program
/// sequences on one controller. One driver behind a blocking mutex, shared, is the fix (FEATURES TPL-24).
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    let drivers = flash_drivers(fw);
    let mut out = Vec::new();
    if let [(_, first), rest @ ..] = drivers.as_slice() {
        for (path, at) in rest {
            out.push(
                Diagnostic::new(
                    "fls-2",
                    path.clone(),
                    "a second flash driver: two owners of the flash controller interleave their erase and program sequences",
                )
                .span(at.span().label("second driver"))
                .span(first.span().label("first driver"))
                .help("build one Flash, put it in a blocking Mutex<RefCell<..>> and share it (as FirmwareUpdaterConfig::from_linkerfile_blocking does)"),
            );
        }
    }
    for f in scan::functions(fw) {
        scan::each(f.block, |n| {
            if let scan::Node::Expr(e @ syn::Expr::Call(_)) = n
                && let Some(name) = scan::call_name(e)
                && name.ends_with("FLASH::steal")
            {
                out.push(
                    Diagnostic::new(
                        "fls-2",
                        format!("{}:steal", f.path()),
                        "the FLASH peripheral is stolen: whoever owns it already gets a second owner",
                    )
                    .span(scan::loc(f.file, scan::head_span(e)).span())
                    .help("pass the one shared flash driver instead of stealing the peripheral"),
                );
            }
        });
    }
    out
}
