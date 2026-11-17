//! Text and JSON output of a report: [`text`] groups the findings for people, [`json`] is complete and ungrouped;
//! [`dependents`] and [`dependents_json`] show `spl dependents`; [`svg`] draws coloured text as a picture.
#![expect(
    clippy::let_underscore_must_use,
    reason = "fmt::Write into a String cannot fail"
)]

mod dependents;
mod excerpt;
mod svg;
mod text;

pub use dependents::{dependents, dependents_json};
pub use svg::svg;

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
