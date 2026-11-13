//! `mem-12` defmt in the dependencies without `-Tdefmt.x` (the cfg half; the `.defmt` section itself is elf, LINT-5)

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Confidence, Diagnostic, Span};
use sl_firmware::Firmware;

lint!(META, "mem-12", "mem", "a firmware with defmt links with -Tdefmt.x",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


const FIX: &str = "add `println!(\"cargo:rustc-link-arg=-Tdefmt.x\");` to build.rs, or call bedrock_build::common() \
    which does it (book/issues.md)";

/// `-Tdefmt.x` comes from a `cargo:rustc-link-arg` build script directive (the template's own, or
/// `bedrock_build::common()`), never from `.cargo/config.toml` rustflags
pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !fw.dependencies().iter().any(|d| d.package() == "defmt") {
        return Vec::new();
    }
    let Some(b) = &fw.build_script else {
        return vec![
            Diagnostic::new(
                "mem-12",
                "build.rs",
                "defmt is a dependency but the crate has no build.rs to add -Tdefmt.x: no .defmt section, defmt messages can't be decoded",
            )
            .span(Span::new(&fw.manifest.rel, 0))
            .help(FIX)
            .confidence(Confidence::Estimate),
        ];
    };
    let has_flag = b
        .with_key("rustc-link-arg")
        .any(|d| d.value.contains("defmt.x"))
        || b.file.text.contains("bedrock_build::common(");
    if has_flag {
        return Vec::new();
    }
    vec![
        Diagnostic::new(
            "mem-12",
            "build.rs",
            "defmt is a dependency but build.rs prints no cargo:rustc-link-arg=-Tdefmt.x (and doesn't call \
             bedrock_build::common()): no .defmt section, defmt messages can't be decoded",
        )
        .span(Span::new(b.file.rel.clone(), 0))
        .help(FIX)
        .confidence(Confidence::Estimate),
    ]
}
