//! One generated page per Clippy lint, group and `clippy.toml` setting the shipped tiers name.
//!
//! A page is titled with the name alone, so the book's search ranks it first for that name; its first paragraph is
//! the tier's reason, which the search shows as the excerpt.

use crate::cell;
use sl_config::Config;
use std::collections::BTreeMap;
use std::fmt::Write;

/// What a name of a tier's `[clippy]` table is
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// One of Clippy's groups (`pedantic`)
    Group,
    /// A single lint (`use_self`)
    Lint,
    /// A `clippy.toml` setting (`allow-unwrap-in-tests`)
    Setting,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Group => "Clippy group",
            Self::Lint => "Clippy lint",
            Self::Setting => "`clippy.toml` setting",
        }
    }
}

/// Every name any shipped tier's Clippy set holds, with its kind, in name order
pub fn entries(tiers: &[(&str, Config)]) -> BTreeMap<String, Kind> {
    let mut out = BTreeMap::new();
    for (_, c) in tiers {
        for name in c.clippy.lints.keys() {
            let kind = if sl_config::clippy::is_group(name) {
                Kind::Group
            } else {
                Kind::Lint
            };
            out.insert(name.clone(), kind);
        }
        for key in c.clippy.config.keys() {
            out.insert(key.clone(), Kind::Setting);
        }
    }
    out
}

/// The page of a name, relative to the docs dir
pub fn path(name: &str) -> String {
    format!("clippy/{name}.md")
}

/// The name's entry on Clippy's own website
pub fn url(name: &str, kind: Kind) -> String {
    match kind {
        Kind::Lint => format!("https://rust-lang.github.io/rust-clippy/master/index.html#{name}"),
        Kind::Group if name == "all" => "https://doc.rust-lang.org/clippy/lints.html".to_owned(),
        Kind::Group => format!("https://doc.rust-lang.org/clippy/lints.html#{name}"),
        Kind::Setting => format!("https://doc.rust-lang.org/clippy/lint_configuration.html#{name}"),
    }
}

/// One tier's row for a name: level or value, reason, the tier file that sets it; `None` when the tier does not name it
fn row(name: &str, kind: Kind, c: &Config) -> Option<(String, String, String)> {
    let set_in = |o: &sl_config::Origin| o.file.trim_start_matches("tier:").to_owned();
    if kind == Kind::Setting {
        let v = c.clippy.config.get(name)?;
        Some((
            format!("`{}`", v.value),
            v.reason.clone(),
            set_in(&v.origin),
        ))
    } else {
        let l = c.clippy.lints.get(name)?;
        Some((
            format!("`{}`", l.level),
            l.reason.clone(),
            set_in(&l.origin),
        ))
    }
}

/// The line a project writes for a name, with the reason as its comment
fn config_line(name: &str, kind: Kind, c: &Config) -> String {
    match kind {
        Kind::Setting => c.clippy.config.get(name).map_or_else(String::new, |v| {
            format!("# clippy.toml\n{name} = {} # {}", v.value, v.reason)
        }),
        _ => c.clippy.lints.get(name).map_or_else(String::new, |l| {
            let level = if kind == Kind::Group {
                format!("{{ level = \"{}\", priority = -1 }}", l.level)
            } else {
                format!("\"{}\"", l.level)
            };
            format!("[workspace.lints.clippy]\n{name} = {level} # {}", l.reason)
        }),
    }
}

/// One name's page
pub fn page(name: &str, kind: Kind, tiers: &[(&str, Config)]) -> String {
    let first = tiers.iter().find(|(_, c)| row(name, kind, c).is_some());
    let reason = first
        .and_then(|(_, c)| row(name, kind, c))
        .map_or_else(String::new, |(_, r, _)| r);
    let mut s = String::new();
    let _ = writeln!(s, "# {name}\n");
    let _ = writeln!(s, "**{}:** {}.\n", kind.name(), crate::capital(&reason));
    let _ = writeln!(s, "| | |\n|---|---|");
    let _ = writeln!(s, "| Clippy's docs | [`{name}`]({}) |", url(name, kind));
    let written = match kind {
        Kind::Setting => "a key of `clippy.toml`",
        Kind::Group => "a group in `[workspace.lints.clippy]`, with `priority = -1`",
        Kind::Lint => "a lint in `[workspace.lints.clippy]`",
    };
    let _ = writeln!(s, "| Written as | {written} |");
    let _ = writeln!(
        s,
        "| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |"
    );
    let head = if kind == Kind::Setting {
        "Value"
    } else {
        "Level"
    };
    let _ = writeln!(
        s,
        "\n## {head} per tier\n\n| Tier | {head} | Reason | Set in |\n|---|---|---|---|"
    );
    for (tier, c) in tiers {
        let (v, r, at) = row(name, kind, c)
            .unwrap_or_else(|| ("not in the set".to_owned(), String::new(), String::new()));
        let _ = writeln!(
            s,
            "| [{tier}](../tiers/{tier}.md) | {v} | {} | {at} |",
            cell(&r)
        );
    }
    if let Some((_, c)) = first {
        let _ = writeln!(
            s,
            "\n## Configure\n\n```toml\n{}\n```",
            config_line(name, kind, c)
        );
    }
    s
}

/// The Lints list's section naming every Clippy entry, with its level in each tier
pub fn index_section(tiers: &[(&str, Config)]) -> String {
    let mut s = String::from(
        "\n## Clippy lints of the tiers\n\nThe Clippy lints, groups and `clippy.toml` settings the tiers expect in a project's own Clippy configuration ([tier-clippy-set](M0025.md) checks them; `allow`: an accepted opt-out).\n\n",
    );
    let mut head = String::from("| Name | Kind |");
    let mut sep = String::from("|---|---|");
    for (n, _) in tiers {
        let _ = write!(head, " {n} |");
        sep.push_str("---|");
    }
    let _ = writeln!(s, "{head}\n{sep}");
    for (name, kind) in entries(tiers) {
        let mut r = format!("| [{name}](../{}) | {} |", path(&name), kind.name());
        for (_, c) in tiers {
            let v = row(&name, kind, c).map_or("-".to_owned(), |(v, _, _)| v.replace('`', ""));
            let _ = write!(r, " {v} |");
        }
        let _ = writeln!(s, "{r}");
    }
    s
}

/// The links M0025's page adds under its own docs: every entry by kind
pub fn m0025_section(tiers: &[(&str, Config)]) -> String {
    let all = entries(tiers);
    let mut s = String::from(
        "\n## The Clippy set of the tiers\n\nEach name has a page with its level in every tier and the reason ([all in one table](index.md#clippy-lints-of-the-tiers)).\n",
    );
    for (kind, title) in [
        (Kind::Group, "Groups"),
        (Kind::Lint, "Lints"),
        (Kind::Setting, "`clippy.toml` settings"),
    ] {
        let names: Vec<String> = all
            .iter()
            .filter(|(_, k)| **k == kind)
            .map(|(n, _)| format!("[{n}](../{})", path(n)))
            .collect();
        if !names.is_empty() {
            let _ = writeln!(s, "\n{title}: {}.", names.join(", "));
        }
    }
    s
}
