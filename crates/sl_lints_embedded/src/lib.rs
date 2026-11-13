//! Lints on bare-metal firmware: every automatic check of embedded_bedrock's firmware check catalogue
//! (bedrock_lint), keeping its ids in kebab-case (`EXE-3` is `exe-3`). One analysis, [`EMBEDDED`], reads each
//! firmware crate of the project once ([`sl_firmware::Firmware`]) and runs the enabled checks on it.

mod checks;

use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, LintMeta, Params, Span};
use sl_firmware::Firmware;

lint!(FIRMWARE_READABLE, "F0020", "firmware-readable", "embedded", "every file of a firmware crate that the embedded lints read can be read",
    params: [], needs: [], fix: None);

/// What a check needs from one firmware crate; a crate without it is not checked by that lint
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FwNeed {
    /// Parsed Rust source under `src/`
    Sources,
    /// A build script
    BuildScript,
    /// `memory.x`
    MemoryX,
    /// A known chip (`bedrock_fw.json` or the HAL's chip feature)
    Chip,
    /// The linked ELF
    Elf,
    /// Arm Thumb functions in the ELF
    Thumb,
    /// `MEMORY` regions read from memory.x
    Regions,
}

impl FwNeed {
    fn met(self, fw: &Firmware<'_>) -> bool {
        match self {
            FwNeed::Sources => !fw.sources.is_empty(),
            FwNeed::BuildScript => fw.build_script.is_some(),
            FwNeed::MemoryX => fw.memory_x.is_some(),
            FwNeed::Chip => fw.chip().is_some(),
            FwNeed::Elf => fw.elf.is_some(),
            FwNeed::Thumb => fw.elf.as_ref().is_some_and(|e| !e.code.is_empty()),
            FwNeed::Regions => !fw.memory.is_empty(),
        }
    }
}

/// A check: its lint, what it needs from a crate, and the function that runs it
pub(crate) struct Check {
    pub(crate) meta: &'static LintMeta,
    pub(crate) needs: &'static [FwNeed],
    pub(crate) run: fn(&Firmware<'_>, &Params) -> Vec<Diagnostic>,
}

/// Declares the check modules and the lists made from them: [`LINTS`] and [`CHECKS`]
macro_rules! declare_checks {
    ($($m:ident),* $(,)?) => {
        $(pub(crate) mod $m;)*
        /// Every check, in catalogue order
        pub(crate) static CHECKS: &[$crate::Check] = &[$($crate::Check {
            meta: &$m::META,
            needs: $m::NEEDS,
            run: $m::run,
        }),*];
        /// The lints of the checks and of the analysis itself
        pub(crate) static LINTS: &[&sl_core::LintMeta] = &[&$crate::FIRMWARE_READABLE, $(&$m::META),*];
    };
}
pub(crate) use declare_checks;

/// The embedded analysis
pub struct Embedded;

/// The registered instance
pub static EMBEDDED: Embedded = Embedded;

impl Analysis for Embedded {
    fn name(&self) -> &'static str {
        "embedded"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        checks::LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let mut seen = std::collections::BTreeSet::new();
        for fw in Firmware::load_all(cx.project, cx.elf()) {
            for n in &fw.notes {
                cx.emit(
                    Diagnostic::new(
                        FIRMWARE_READABLE.id,
                        format!("{}:{}", fw.name, n.file),
                        format!("{}: {}", n.file, n.message),
                    )
                    .span(Span::new(n.file.clone(), 0))
                    .krate(fw.name.clone())
                    .help("fix the file: the embedded lints that read it find nothing in it until then"),
                );
            }
            for c in checks::CHECKS {
                if !cx.enabled(c.meta.id) || !c.needs.iter().all(|n| n.met(&fw)) {
                    continue;
                }
                let params = cx.params_for(c.meta.id, fw.package);
                for mut d in (c.run)(&fw, &params) {
                    // a finding in a path dependency shared by two firmware crates is reported once
                    let key = (d.lint.clone(), d.primary().cloned(), d.message.clone());
                    if !seen.insert(key) {
                        continue;
                    }
                    d.item = format!("{}:{}", fw.name, d.item);
                    if d.krate.is_none() {
                        d.krate = Some(fw.name.clone());
                    }
                    cx.emit(d);
                }
            }
        }
    }
}

/// A size or count parameter as a number of bytes or a percentage (a negative value counts as 0)
pub(crate) fn limit(params: &Params, name: &str) -> u64 {
    u64::try_from(params.int(name)).unwrap_or(0)
}
