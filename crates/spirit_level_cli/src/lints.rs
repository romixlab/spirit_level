//! `spl lints` and `spl tiers`: the shipped lints with their levels, and the shipped tiers

use crate::Format;
use sl_core::Level;
use vc_ui::Exit;

pub(crate) fn list_lints(format: Format) -> Result<Exit, String> {
    let tiers = sl_book::tier_levels()?;
    let mut lints = spirit_level::lints();
    lints.sort_by_key(|m| m.code);
    match format {
        Format::Json => {
            let v: Vec<serde_json::Value> = lints
                .iter()
                .map(|m| {
                    let levels: serde_json::Map<String, serde_json::Value> = tiers
                        .iter()
                        .map(|(n, c)| (n.to_string(), c.lint(m.id).level.name().into()))
                        .collect();
                    serde_json::json!({"code": m.code, "id": m.id, "category": m.category, "summary": m.summary, "levels": levels, "meta": m})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
        Format::Text => {
            let w = lints.iter().map(|m| m.id.len()).max().unwrap_or(10);
            let names: Vec<String> = tiers.iter().map(|(n, _)| format!("{n:>6}")).collect();
            println!("code   {:w$}  {}  rule", "lint", names.join(" "));
            for m in &lints {
                let lv: Vec<String> = tiers
                    .iter()
                    .map(|(n, c)| {
                        let l = c.lint(m.id).level;
                        let s = if l == Level::Allow { "-" } else { l.name() };
                        format!("{s:>width$}", width = n.len().max(6))
                    })
                    .collect();
                println!("{}  {:w$}  {}  {}", m.code, m.id, lv.join(" "), m.summary);
            }
        }
    }
    Ok(Exit::SUCCESS)
}

pub(crate) fn tiers() -> Exit {
    for (n, t) in spirit_level::TIERS {
        println!("{n:16} {}", spirit_level::tier_description(t));
    }
    Exit::SUCCESS
}
