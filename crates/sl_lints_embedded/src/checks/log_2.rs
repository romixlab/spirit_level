//! `log-2` `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` not set explicitly in `.cargo/config.toml`

use crate::FwNeed;
use sl_analysis::lint;
use sl_core::Params;
use sl_core::{Diagnostic, Span};
use sl_firmware::Firmware;
use sl_firmware::TomlExt;

lint!(META, "log-2", "log", "DEFMT_RTT_BUFFER_SIZE and DEFMT_LOG are set in .cargo/config.toml",
    params: [], needs: [], fix: None);

/// What the check needs from a firmware crate; a crate without it is not checked
pub(crate) const NEEDS: &[FwNeed] = &[];


fn env_var(fw: &Firmware<'_>, name: &str) -> bool {
    fw.cargo_config(&["env", name]).is_some()
}

pub(crate) fn run(fw: &Firmware<'_>, _params: &Params) -> Vec<Diagnostic> {
    if !fw.dependencies().iter().any(|d| d.package() == "defmt") {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (name, why) in [
        (
            "DEFMT_LOG",
            "the log level falls back to a built-in default",
        ),
        (
            "DEFMT_RTT_BUFFER_SIZE",
            "the RTT buffer falls back to defmt-rtt's default and can lose a burst (init especially)",
        ),
    ] {
        if env_var(fw, name) {
            continue;
        }
        let span = fw
            .cargo_configs
            .first()
            .map(|c| Span::new(&c.rel, c.line_of_table(&["env"])))
            .unwrap_or_else(|| Span::new(&fw.manifest.rel, 0));
        out.push(
            Diagnostic::new(
                "log-2",
                name,
                format!("{name} not set explicitly in [env] of .cargo/config.toml: {why}"),
            )
            .span(span)
            .help(format!("set {name} under [env] in .cargo/config.toml")),
        );
    }
    out
}
