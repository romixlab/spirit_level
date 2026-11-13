//! Dead-code lints rustc can't report. One analysis, [`DEAD`], reports `dead-pub`: public items of workspace
//! libraries no other crate names.

mod pub_items;

use sl_analysis::{Analysis, Context, lint};
use sl_core::LintMeta;

lint!(DEAD_PUB, "dead-pub", "dead-code", "public items of workspace libraries are used outside their crate",
    params: [
        ("count-tests", Bool, "false", "uses in test code (tests/, benches/, examples/, #[cfg(test)]) of other crates count"),
        ("skip", StrList, "[]", "library crates whose public API is meant for users outside the workspace"),
        ("skip-published", Bool, "true", "skip crates that are published (no publish = false)"),
    ],
    needs: [], fix: None);

/// The dead-code analysis
pub struct Dead;

/// The registered instance
pub static DEAD: Dead = Dead;

static LINTS: &[&LintMeta] = &[&DEAD_PUB];

impl Analysis for Dead {
    fn name(&self) -> &'static str {
        "dead"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        pub_items::run(cx);
    }
}

/// Whether the crate map marks a crate as generated (its code is not written by hand)
fn generated(cx: &Context<'_>, krate: &str) -> bool {
    cx.map()
        .and_then(|m| m.crates.get(krate))
        .is_some_and(|c| c.generated)
}
