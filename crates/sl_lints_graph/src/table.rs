//! The crate table in the README, between `<!-- spirit-level:crates -->` and `<!-- /spirit-level:crates -->`.

use sl_analysis::Context;
use sl_core::{Diagnostic, Edit, Fix, Span};
use sl_model::Project;

/// The opening marker
pub const BEGIN: &str = "<!-- spirit-level:crates -->";
/// The closing marker
pub const END: &str = "<!-- /spirit-level:crates -->";

/// The generated table: crate, layer (when there is a map), description
pub fn render(cx: &Context<'_>, p: &Project) -> String {
    let map = cx.map();
    let mut s = String::new();
    if map.is_some() {
        s.push_str("| Crate | Layer | Description |\n|---|---|---|\n");
    } else {
        s.push_str("| Crate | Description |\n|---|---|\n");
    }
    let mut pkgs: Vec<_> = p.packages.iter().collect();
    pkgs.sort_by_key(|k| {
        let layer = map
            .and_then(|m| m.crates.get(&k.name))
            .and_then(|c| m_index(map, &c.layer))
            .unwrap_or(usize::MAX);
        (layer, k.name.clone())
    });
    for k in pkgs {
        let desc = k
            .description
            .as_deref()
            .unwrap_or("")
            .replace('|', "\\|")
            .replace('\n', " ");
        let link = if k.dir.is_empty() {
            format!("`{}`", k.name)
        } else {
            format!("[`{}`]({})", k.name, k.dir)
        };
        let row = match map {
            Some(m) => {
                let layer = m.crates.get(&k.name).map_or("", |c| c.layer.as_str());
                format!("| {link} | {layer} | {desc} |\n")
            }
            None => format!("| {link} | {desc} |\n"),
        };
        s.push_str(&row);
    }
    s
}

fn m_index(map: Option<&sl_config::Map>, layer: &str) -> Option<usize> {
    map?.layers.iter().position(|l| l.name == layer)
}

pub(crate) fn check(cx: &Context<'_>, p: &Project) {
    let file = cx.params("crate-table").str("file").to_owned();
    let Some(text) = p.read(&file) else {
        cx.emit(
            Diagnostic::new(
                "crate-table",
                format!("file:{file}"),
                format!("there is no {file} to hold the crate table"),
            )
            .span(Span::new(file.clone(), 0))
            .help(format!(
                "write {file} with a line `{BEGIN}` and a line `{END}`; --fix then fills the table"
            )),
        );
        return;
    };
    let table = render(cx, p);
    let (Some(a), Some(b)) = (text.find(BEGIN), text.find(END)) else {
        cx.emit(
            Diagnostic::new(
                "crate-table",
                format!("file:{file}"),
                format!("{file} has no crate table markers"),
            )
            .span(Span::new(file.clone(), 0))
            .help(format!(
                "add a line `{BEGIN}` and a line `{END}` where the table goes; --fix then fills it"
            )),
        );
        return;
    };
    let start = a + BEGIN.len();
    if b < start {
        return;
    }
    let want = format!("\n{table}");
    if text[start..b] != want {
        let line = text[..a].matches('\n').count() as u32 + 1;
        cx.emit(
            Diagnostic::new("crate-table", format!("file:{file}"), format!("the crate table in {file} is not the generated one"))
                .span(Span::new(file.clone(), line))
                .help("descriptions live in each crate's Cargo.toml and layers in the map; edit those, then --fix")
                .fix(Fix {
                    description: format!("regenerate the crate table in {file}"),
                    edits: vec![Edit { file: file.clone(), start, end: b, replacement: want }],
                }),
        );
    }
}
