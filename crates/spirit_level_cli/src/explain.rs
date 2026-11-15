//! `spl explain`: a lint's page, or a tool's own rule and the adapter lints that report it

use crate::lint_candidates;
use crate::registry::{Registered, Run};
use clap_complete::engine::ArgValueCandidates;
use vc_ui::Exit;

/// Show a lint's page: its rule, why, parameters and level in every tier
#[derive(clap::Parser)]
#[command(
    name = "explain",
    display_order = 1,
    after_help = "Examples:\n  spl explain crate-lines\n  spl explain S0001"
)]
pub(crate) struct Explain {
    /// The lint: its code (`S0001`, `X-clippy:needless_return`) or its id (`crate-lines`)
    #[arg(add = ArgValueCandidates::new(lint_candidates))]
    lint: String,
}

inventory::submit! { Registered::of::<Explain>() }

impl Run for Explain {
    fn run(self) -> Result<Exit, String> {
        explain(&self.lint)
    }
}

fn explain(lint: &str) -> Result<Exit, String> {
    if spirit_level::lint(lint).is_none()
        && let Some((tool, rule)) = lint.strip_prefix("X-").and_then(|r| r.split_once(':'))
    {
        return adapter_rule(tool, rule);
    }
    let m = spirit_level::lint(lint)
        .ok_or_else(|| format!("no lint {lint} (`spl lints` lists them)"))?;
    let tiers = sl_book::tier_levels()?;
    print!("{}", sl_book::lint_text(m, &tiers));
    Ok(Exit::SUCCESS)
}

/// `spl explain X-<tool>:<rule>`: the tool's own rule, and the adapter lints that report it
fn adapter_rule(tool: &str, rule: &str) -> Result<Exit, String> {
    let lints: Vec<&sl_core::LintMeta> = spirit_level::lints()
        .into_iter()
        .filter(|m| {
            matches!(m.source, sl_core::Source::Tool { .. })
                && m.id.starts_with(&format!("{tool}-"))
        })
        .collect();
    if lints.is_empty() {
        return Err(format!("no adapter of {tool} (`spl lints` lists them)"));
    }
    println!(
        "# X-{tool}:{rule}\n\n{tool}'s own rule `{rule}`, reported through the {tool} adapter. Its level is the level of the adapter lint the rule falls under:\n"
    );
    for m in lints {
        println!("- {} {}: {}", m.code, m.id, m.summary);
    }
    Ok(Exit::SUCCESS)
}
