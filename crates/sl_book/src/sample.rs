//! A picture of a check for a README, where Markdown shows no colour: the coloured text report of a sample project
//! as SVG (`spl book sample`).

use std::path::Path;
use vc_ui::Paint;

/// The coloured report of `spl check` on `project`, as SVG. The summary names `project` as given, so the picture
/// does not depend on where the repo is checked out.
pub fn svg(project: &Path, opts: &spirit_level::Options) -> Result<String, String> {
    let (_, mut report) = spirit_level::check(project, opts).map_err(|e| e.to_string())?;
    report.root = project.display().to_string();
    let text = sl_report::text(
        &report,
        &spirit_level::lints(),
        sl_report::TextOptions {
            paint: Paint::ON,
            ..sl_report::TextOptions::default()
        },
    );
    Ok(sl_report::svg(&text))
}

/// Writes the picture of `project` to `out`, or with `check` only compares; whether `out` differed
pub fn write(
    project: &Path,
    out: &Path,
    opts: &spirit_level::Options,
    check: bool,
) -> Result<bool, String> {
    let new = svg(project, opts)?;
    if std::fs::read_to_string(out).ok().as_deref() == Some(new.as_str()) {
        return Ok(false);
    }
    if !check {
        std::fs::write(out, new).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    Ok(true)
}
