//! A source excerpt of a finding's place, drawn with annotate-snippets: the span's lines with 2 lines of context
//! and its exact range marked with a label. The file is read when the report is printed.

use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use sl_core::Span;
use std::path::Path;

/// Lines shown above and below the span
const CONTEXT: u32 = 2;

/// The excerpt of `span` under `root`, the range marked and labelled; None when the span names no line or the
/// file cannot be read. A span over several lines is marked on its first line, to the end of it, and the label
/// says where it ends.
pub fn render(root: &Path, span: &Span, label: &str, deny: bool) -> Option<String> {
    if span.line == 0 {
        return None;
    }
    let text = std::fs::read_to_string(root.join(&span.file)).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let line = span.line as usize;
    let first_line = lines.get(line - 1)?;
    let from = line.saturating_sub(CONTEXT as usize).max(1);
    let to = (line + CONTEXT as usize).min(lines.len());
    let mut source = String::new();
    let mut start_of_line = 0;
    for (i, l) in lines.iter().enumerate().take(to).skip(from - 1) {
        if i + 1 == line {
            start_of_line = source.len();
        }
        source.push_str(l);
        source.push('\n');
    }
    let one_line = span.end_line <= span.line;
    let start = byte_of(first_line, span.column, 0);
    let end = if one_line {
        byte_of(first_line, span.end_column, first_line.len())
    } else {
        first_line.len()
    };
    // a span with no width (a column without an end) marks the rest of the line, a whole line its text
    let (start, end) = match (span.column, end <= start) {
        (0, _) => (indent(first_line), first_line.len()),
        (_, true) => (start, first_line.len()),
        _ => (start, end),
    };
    let label = if one_line {
        label.to_string()
    } else {
        format!("{label} (to line {})", span.end_line)
    };
    let level = if deny { Level::ERROR } else { Level::WARNING };
    let report = [Group::with_level(level).element(
        Snippet::source(source.as_str())
            .line_start(from)
            .path(span.file.as_str())
            .fold(false)
            .annotation(
                AnnotationKind::Primary
                    .span(start_of_line + start..start_of_line + end.max(start))
                    .label(label.as_str()),
            ),
    )];
    Some(
        Renderer::styled()
            .decor_style(DecorStyle::Ascii)
            .render(&report),
    )
}

/// The byte offset in `line` of a 1-based character column; `default` for column 0
fn byte_of(line: &str, column: u32, default: usize) -> usize {
    if column == 0 {
        return default;
    }
    line.char_indices()
        .nth(column as usize - 1)
        .map_or(line.len(), |(i, _)| i)
}

/// Where the text of a line starts after its indentation
fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(s: &str) -> String {
        anstream::adapter::strip_str(s).to_string()
    }

    fn scratch(text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sl_report_excerpt_{}_{}",
            std::process::id(),
            text.len()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.rs"), text).unwrap();
        dir
    }

    #[test]
    fn marks_the_range_with_context() {
        let dir = scratch("fn a() {}\n// x\n#[allow(dead_code)]\nfn b() {}\n// y\n// z\n");
        let s = render(
            &dir,
            &Span::range("a.rs", (3, 3), (3, 18)),
            "no reason",
            true,
        )
        .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let p = plain(&s);
        assert!(p.contains("--> a.rs:3:3"), "{p}");
        assert!(p.contains("1 | fn a() {}"), "{p}");
        assert!(p.contains("3 | #[allow(dead_code)]"), "{p}");
        assert!(p.contains("^^^^^^^^^^^^^^^ no reason"), "{p}");
        assert!(p.contains("5 | // y"), "{p}");
        assert!(!p.contains("// z"), "{p}");
    }

    #[test]
    fn a_whole_line_and_a_long_span() {
        let dir = scratch("fn a() {\n    let x = 1;\n}\n");
        let s = render(&dir, &Span::new("a.rs", 2), "here", false).unwrap();
        assert!(plain(&s).contains("^^^^^^^^^^ here"), "{}", plain(&s));
        let s = render(&dir, &Span::at("a.rs", 1, 1).to(3), "long", false).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(plain(&s).contains("long (to line 3)"), "{}", plain(&s));
    }

    #[test]
    fn nothing_without_a_line_or_a_file() {
        assert!(render(Path::new("/nonexistent"), &Span::new("a.rs", 1), "x", true).is_none());
        assert!(render(Path::new("."), &Span::new("Cargo.toml", 0), "x", true).is_none());
    }
}
