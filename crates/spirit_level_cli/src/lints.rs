//! `spl lints` and `spl tiers`: the shipped lints with their levels and sets, and the shipped tiers and sets

use crate::Format;
use crate::registry::{Registered, Run};
use sl_core::Level;
use std::path::{Path, PathBuf};
use vc_ui::{DIM, Exit, HEADING, ID, NEXT, OK, Paint};

/// List every lint with its set and its level in each shipped tier
#[derive(clap::Parser)]
#[command(
    name = "lints",
    display_order = 3,
    after_help = "Examples:\n  spl lints\n  spl lints --format json"
)]
pub(crate) struct Lints {
    /// Output for people or for machines
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
}

inventory::submit! { Registered::of::<Lints>() }

impl Run for Lints {
    fn run(self) -> Result<Exit, String> {
        list_lints(self.format)
    }
}

/// Show the tier ladder, the add-on tiers and the sets, marking the ones this project extends
#[derive(clap::Parser)]
#[command(
    name = "tiers",
    display_order = 4,
    after_help = "Examples:\n  spl tiers\n  spl tiers ../other_repo\n\nThe ladder's tiers extend each other, small to big; a project extends one of them and any add-ons or sets beside it\n(extends = [\"medium\", \"ai\", \"layers\"]). Each ladder tier is built from sets: small groups of lints by what they guard. `spl check --next` checks a project at the next rung, `--at TIER` at any."
)]
pub(crate) struct Tiers {
    /// The project whose tiers to mark
    #[arg(default_value = ".")]
    path: PathBuf,
}

inventory::submit! { Registered::of::<Tiers>() }

impl Run for Tiers {
    fn run(self) -> Result<Exit, String> {
        tiers(&self.path)
    }
}

fn list_lints(format: Format) -> Result<Exit, String> {
    let tiers = sl_book::tier_levels()?;
    let set_of = sl_book::set_of(&sl_book::set_levels()?);
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
                    serde_json::json!({"code": m.code, "id": m.id, "category": m.category, "set": set_of.get(m.id), "summary": m.summary, "levels": levels, "meta": m})
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        }
        Format::Text => {
            let w = lints.iter().map(|m| m.id.len()).max().unwrap_or(10);
            let ws = set_of.values().map(|s| s.len()).max().unwrap_or(3);
            let names: Vec<String> = tiers.iter().map(|(n, _)| format!("{n:>6}")).collect();
            println!(
                "code   {:w$}  {:ws$}  {}  rule",
                "lint",
                "set",
                names.join(" ")
            );
            for m in &lints {
                let lv: Vec<String> = tiers
                    .iter()
                    .map(|(n, c)| {
                        let l = c.lint(m.id).level;
                        let s = if l == Level::Allow { "-" } else { l.name() };
                        format!("{s:>width$}", width = n.len().max(6))
                    })
                    .collect();
                let set = set_of.get(m.id).copied().unwrap_or("-");
                println!(
                    "{}  {:w$}  {set:ws$}  {}  {}",
                    m.code,
                    m.id,
                    lv.join(" "),
                    m.summary
                );
            }
        }
    }
    Ok(Exit::SUCCESS)
}

fn tiers(path: &Path) -> Result<Exit, String> {
    let levels = sl_book::tier_levels()?;
    let sets = sl_book::set_levels()?;
    let enabled = |name: &str| {
        levels
            .iter()
            .chain(&sets)
            .find(|(n, _)| *n == name)
            .map_or(0, |(_, c)| {
                c.lints.values().filter(|l| l.level != Level::Allow).count()
                    + c.clippy
                        .lints
                        .values()
                        .filter(|l| l.level != Level::Allow)
                        .count()
            })
    };
    let own = sl_model::Project::load(path)
        .map_err(|e| e.to_string())
        .and_then(|p| {
            spirit_level::config(&p, &spirit_level::Options::new(sl_core::Date::today_utc()))
                .map_err(|e| e.to_string())
        });
    let extends = own.as_ref().map(|c| c.extends.clone()).unwrap_or_default();
    let p = Paint::stdout();
    let row = |name: &str, text: &str| {
        let on = extends.iter().any(|e| e == name);
        let mark = if on { p.paint(OK, "*") } else { " ".to_owned() };
        let n = enabled(name);
        let lints = format!("{n:>3} lint{}", if n == 1 { " " } else { "s" });
        println!(
            "{mark} {} {}  {}",
            p.paint(ID, &format!("{name:16}")),
            p.paint(DIM, &lints),
            spirit_level::tier_description(text)
        );
    };
    println!(
        "{}",
        p.paint(HEADING, "ladder: each tier extends the one before")
    );
    for (n, t) in spirit_level::TIERS
        .iter()
        .filter(|(n, _)| spirit_level::LADDER.contains(n))
    {
        row(n, t);
    }
    println!(
        "{}",
        p.paint(
            HEADING,
            "add-ons: beside a ladder tier, extends = [\"medium\", \"ai\"]"
        )
    );
    for (n, t) in spirit_level::TIERS
        .iter()
        .filter(|(n, _)| !spirit_level::LADDER.contains(n))
    {
        row(n, t);
    }
    println!(
        "{}",
        p.paint(
            HEADING,
            "sets: what a tier is built from, or beside one, extends = [\"small-workspace\", \"msrv\"]"
        )
    );
    for rung in spirit_level::LADDER.iter().map(Some).chain([None]) {
        println!(
            "  {}",
            p.paint(
                DIM,
                &rung.map_or_else(|| "beside the tiers".to_owned(), |r| format!("in {r}"))
            )
        );
        for (n, t) in spirit_level::SETS
            .iter()
            .filter(|(n, _)| spirit_level::set_tier(n) == rung.copied())
        {
            row(n, t);
        }
    }
    match &own {
        Ok(c) => {
            let rung = spirit_level::ladder_tier(&c.extends);
            let next =
                rung.and_then(|r| spirit_level::LADDER.iter().skip_while(|l| **l != r).nth(1));
            println!(
                "\n{} extends {}{}",
                c.file.as_deref().unwrap_or(sl_config::FILE),
                c.extends.join(", "),
                next.map_or_else(String::new, |n| format!(
                    "; {} `spl check --next` checks it at {n}",
                    p.paint(NEXT, "next:")
                ))
            );
        }
        Err(e) => println!(
            "\n{}",
            p.paint(
                DIM,
                &format!("no tier marked: {}", e.lines().next().unwrap_or_default())
            )
        ),
    }
    Ok(Exit::SUCCESS)
}
