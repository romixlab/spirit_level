//! The compiler messages of `cargo check|clippy|doc --message-format json`: one JSON object per line on stdout;
//! the objects with `"reason": "compiler-message"` carry a rustc diagnostic, `"build-finished"` says whether the
//! build succeeded.

use serde_json::Value;
use sl_core::Span;
use sl_model::{Package, Project};
use std::path::Path;

/// One compiler diagnostic
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The manifest of the package being built (absolute)
    pub manifest_path: String,
    /// `warning`, `error`, ...
    pub level: String,
    /// The lint or error code (`clippy::needless_return`, `unused_variables`, `E0308`)
    pub code: Option<String>,
    /// The message's first line
    pub message: String,
    /// The primary span, when there is one
    pub span: Option<MessageSpan>,
    /// The first `help:` child, with its suggested replacement when it has one
    pub help: Option<String>,
}

/// The primary span of a [`Message`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSpan {
    /// As rustc wrote it: relative to the workspace root for members, absolute elsewhere
    pub file: String,
    /// 1-based
    pub line: u32,
    /// 1-based
    pub column: u32,
    /// The last line
    pub end_line: u32,
    /// The highlighted source text (first line)
    pub text: String,
}

/// Everything read from a cargo run
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Messages {
    /// The diagnostics, each once (a lib and its test target report the same warning twice)
    pub messages: Vec<Message>,
    /// `build-finished`'s `success`, when the run got that far
    pub success: Option<bool>,
}

/// Reads cargo's JSON lines; other lines (a tool's own output) are skipped
pub fn parse(stdout: &str) -> Messages {
    let mut out = Messages::default();
    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match v.get("reason").and_then(Value::as_str) {
            Some("build-finished") => out.success = v.get("success").and_then(Value::as_bool),
            Some("compiler-message") => {
                let Some(m) = v.get("message") else { continue };
                let msg = Message {
                    manifest_path: str_of(&v, "manifest_path"),
                    level: str_of(m, "level"),
                    code: m
                        .get("code")
                        .and_then(|c| c.get("code"))
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    message: str_of(m, "message"),
                    span: primary(m),
                    help: help(m),
                };
                if !out.messages.contains(&msg) {
                    out.messages.push(msg);
                }
            }
            _ => {}
        }
    }
    out
}

fn str_of(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn primary(m: &Value) -> Option<MessageSpan> {
    let spans = m.get("spans")?.as_array()?;
    let s = spans
        .iter()
        .find(|s| s.get("is_primary").and_then(Value::as_bool) == Some(true))?;
    let n = |k: &str| s.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
    let text = s
        .get("text")
        .and_then(Value::as_array)
        .and_then(|t| t.first())
        .map(|t| {
            let line = str_of(t, "text");
            let a = t
                .get("highlight_start")
                .and_then(Value::as_u64)
                .unwrap_or(1) as usize;
            let b = t.get("highlight_end").and_then(Value::as_u64).unwrap_or(1) as usize;
            line.chars()
                .skip(a.saturating_sub(1))
                .take(b.saturating_sub(a))
                .collect::<String>()
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    Some(MessageSpan {
        file: str_of(s, "file_name"),
        line: n("line_start"),
        column: n("column_start"),
        end_line: n("line_end"),
        text,
    })
}

fn help(m: &Value) -> Option<String> {
    // a help with a suggestion first, then any help but rustc's "for further information visit" link
    let helps: Vec<&Value> = m
        .get("children")?
        .as_array()?
        .iter()
        .filter(|c| c.get("level").and_then(Value::as_str) == Some("help"))
        .collect();
    let suggestion = |c: &&&Value| {
        c.get("spans")
            .and_then(Value::as_array)
            .is_some_and(|s| !s.is_empty())
    };
    let c = helps.iter().find(suggestion).or_else(|| {
        helps
            .iter()
            .find(|c| !str_of(c, "message").starts_with("for further information"))
    })?;
    let mut h = str_of(c, "message");
    if let Some(r) = c
        .get("spans")
        .and_then(Value::as_array)
        .and_then(|s| s.first())
        .and_then(|s| s.get("suggested_replacement"))
        .and_then(Value::as_str)
        && !r.is_empty()
        && !r.contains('\n')
    {
        h.push_str(&format!(": `{r}`"));
    }
    Some(h)
}

impl Message {
    /// The span relative to the project root; None when the place is outside it (a dependency's source)
    pub fn span_in(&self, root: &Path) -> Option<Span> {
        let s = self.span.as_ref()?;
        let p = Path::new(&s.file);
        let rel = if p.is_absolute() {
            p.strip_prefix(root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/")
        } else {
            s.file.clone()
        };
        let span = Span::at(rel, s.line, s.column);
        Some(if s.end_line > s.line {
            span.to(s.end_line)
        } else {
            span
        })
    }

    /// The workspace package being built
    pub fn package<'a>(&self, project: &'a Project) -> Option<&'a Package> {
        package_of(project, &self.manifest_path)
    }

    /// A fingerprint that survives edits around it: code, file and the highlighted text
    pub fn item(&self) -> String {
        let code = self.code.as_deref().unwrap_or(&self.level);
        match &self.span {
            Some(s) => format!("{code}@{}:{}", s.file, s.text),
            None => format!("{code}:{}", self.message),
        }
    }
}

/// The workspace package of an absolute manifest path
pub fn package_of<'a>(project: &'a Project, manifest_path: &str) -> Option<&'a Package> {
    let rel = Path::new(manifest_path).strip_prefix(&project.root).ok()?;
    let dir = rel.parent()?.to_string_lossy().replace('\\', "/");
    project.packages.iter().find(|p| p.dir == dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIPPY: &str = include_str!("../tests/samples/clippy.jsonl");

    #[test]
    fn reads_clippy_messages() {
        let m = parse(CLIPPY);
        assert_eq!(m.success, Some(true));
        let codes: Vec<&str> = m
            .messages
            .iter()
            .filter_map(|m| m.code.as_deref())
            .collect();
        assert!(codes.contains(&"clippy::needless_return"), "{codes:?}");
        let r = m
            .messages
            .iter()
            .find(|m| m.code.as_deref() == Some("clippy::needless_return"))
            .unwrap();
        let s = r.span.as_ref().unwrap();
        assert_eq!((s.file.as_str(), s.line), ("src/lib.rs", 4));
        assert_eq!(s.text, "return x");
        assert_eq!(r.item(), "clippy::needless_return@src/lib.rs:return x");
        assert!(r.help.as_deref().unwrap().contains("remove `return`"));
        assert_eq!(
            r.span_in(Path::new("/x")).unwrap().to_string(),
            "src/lib.rs:4:52"
        );
    }

    #[test]
    fn skips_other_lines() {
        let m = parse("not json\n{\"reason\":\"compiler-artifact\"}\n");
        assert!(m.messages.is_empty() && m.success.is_none());
    }
}
