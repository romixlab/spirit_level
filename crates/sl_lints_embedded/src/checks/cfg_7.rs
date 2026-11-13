//! `cfg-7` Dirty build flashed without a mark

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "cfg-7", "cfg", "the build info carries the git SHA and a dirty flag",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


/// No `bedrock_build` dependency: the firmware embeds no git SHA or dirty flag, so a dirty build flashed in a
/// hurry can't be told apart from a clean one once it is on the device
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if fw
        .dependencies()
        .iter()
        .any(|d| d.package() == "bedrock_build")
    {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "cfg-7",
            "bedrock_build",
            "no bedrock_build dependency: the build carries no git SHA or dirty flag, so a dirty build \
             flashed by mistake can't be told apart from a clean one by reading the device",
        )
        .span(Span::new(&fw.manifest.rel, 0))
        .help("add bedrock_build as a build-dependency and call serialize_build_info() from build.rs (book/build_info.md)")
        .confidence(Confidence::Estimate),
    ]
}
