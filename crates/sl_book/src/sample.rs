//! Checks of sample projects, shown in colour where Markdown shows none.
//!
//! A README gets an SVG picture (`spl book sample`); the book's hand-written pages get generated blocks of coloured
//! HTML (`spl book docs`).

use std::path::Path;
use vc_ui::Paint;

/// The line before a generated block: `<!-- spirit-level:check PATH [--only LINT]... [--tier TIER]... -->`
const BLOCK_BEGIN: &str = "<!-- spirit-level:check ";
/// The line after it
const BLOCK_END: &str = "<!-- /spirit-level:check -->";

/// The coloured text report of `spl check` on `project`. The summary names the project `shown`, so the output does
/// not depend on where the repo is checked out.
pub fn text(project: &Path, shown: &str, opts: &spirit_level::Options) -> Result<String, String> {
    let (_, report) = spirit_level::check(project, opts).map_err(|e| e.to_string())?;
    let text = sl_report::text(
        &report,
        &spirit_level::lints(),
        sl_report::TextOptions {
            paint: Paint::ON,
            ..sl_report::TextOptions::default()
        },
    );
    // the root is the excerpts' base while they are read, and appears only in the summary line
    Ok(text.replace(&report.root, shown))
}

/// The report of `spl check` on `project`, as an SVG picture; `project` is shown as given
pub fn svg(project: &Path, opts: &spirit_level::Options) -> Result<String, String> {
    Ok(sl_report::svg(&text(
        project,
        &project.display().to_string(),
        opts,
    )?))
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

/// The options of a block's line: the project's path (relative to the repo root) and the check's options
fn block_args(args: &str, today: sl_core::Date) -> Result<(String, spirit_level::Options), String> {
    let mut words = args.split_whitespace();
    let path = words
        .next()
        .ok_or_else(|| format!("{BLOCK_BEGIN}-->: names no project"))?;
    let mut opts = spirit_level::Options::new(today);
    while let Some(w) = words.next() {
        let list = match w {
            "--only" => &mut opts.only,
            "--tier" => &mut opts.tiers,
            _ => {
                return Err(format!(
                    "{BLOCK_BEGIN}{args} -->: `{w}` is not --only or --tier"
                ));
            }
        };
        list.push(
            words
                .next()
                .ok_or_else(|| format!("{BLOCK_BEGIN}{args} -->: `{w}` has no value"))?
                .to_owned(),
        );
    }
    Ok((path.to_owned(), opts))
}

/// `page` with each generated block written anew: the coloured report of the check its line names, on a project
/// under `root`.
///
/// A block starts after a line `<!-- spirit-level:check PATH [--only LINT]... [--tier TIER]... -->` and ends before
/// a line `<!-- /spirit-level:check -->`.
pub fn blocks(page: &str, root: &Path, today: sl_core::Date) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = page;
    while let Some(a) = rest.find(BLOCK_BEGIN) {
        let line_end = rest[a..].find('\n').map_or(rest.len(), |i| a + i + 1);
        let line = &rest[a..line_end];
        let args = line[BLOCK_BEGIN.len()..]
            .trim_end()
            .strip_suffix("-->")
            .ok_or_else(|| format!("`{}`: the line does not end with -->", line.trim_end()))?;
        let end = rest[line_end..]
            .find(BLOCK_END)
            .map(|i| line_end + i)
            .ok_or_else(|| format!("`{}` has no `{BLOCK_END}` after it", line.trim_end()))?;
        let (path, opts) = block_args(args, today)?;
        let text = text(&root.join(&path), &path, &opts)?;
        out.push_str(&rest[..line_end]);
        out.push_str(&sl_report::html(&text));
        rest = &rest[end..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::block_args;
    use sl_core::Date;

    const TODAY: Date = Date {
        year: 2026,
        month: 11,
        day: 13,
    };

    #[test]
    fn a_block_line_names_the_project_and_the_options() {
        let (path, opts) = block_args(
            "assets/sample --only code --tier medium --only state ",
            TODAY,
        )
        .unwrap();
        assert_eq!(path, "assets/sample");
        assert_eq!(opts.only, ["code", "state"]);
        assert_eq!(opts.tiers, ["medium"]);
        assert!(block_args("", TODAY).is_err());
        assert!(block_args("x --fix", TODAY).is_err());
        assert!(block_args("x --only", TODAY).is_err());
    }
}
