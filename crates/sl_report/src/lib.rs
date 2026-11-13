//! Text and JSON output of a report: [`text`] groups the findings for people, [`json`] is complete and ungrouped.

mod excerpt;
mod text;

pub use text::{GroupBy, TextOptions, text};

use sl_config::ConfigError;
use sl_engine::Report;
use std::fmt::Write;
use vc_ui::{ERR, Paint};

/// The report as JSON
pub fn json(r: &Report) -> String {
    serde_json::to_string_pretty(r).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

/// Configuration errors for people, coloured as `p` says
pub fn config_errors(errs: &[ConfigError], p: Paint) -> String {
    let mut s = String::new();
    for e in errs {
        let _ = writeln!(s, "{} {e}", p.paint(ERR, "config error"));
    }
    let _ = writeln!(
        s,
        "{}: the configuration has {} error{}; no lint ran",
        p.paint(ERR, "failed"),
        errs.len(),
        if errs.len() == 1 { "" } else { "s" }
    );
    s
}

/// Configuration errors as JSON
pub fn config_errors_json(errs: &[ConfigError]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "spirit_level.config-errors/1",
        "errors": errs,
    }))
    .unwrap_or_default()
}
