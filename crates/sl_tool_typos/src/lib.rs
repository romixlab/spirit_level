//! The typos adapter: `typos --format json`, one finding per misspelling.
//!
//! typos prints one JSON object per line; `"type": "typo"` objects carry `path`, `line_num`, `byte_offset` (in
//! the line), `typo` and `corrections`. It honours the project's `typos.toml` (or `_typos.toml`, `.typos.toml`)
//! and `.gitignore`. It exits 0 when clean, 2 when it found typos, and another code on an error.

use serde_json::Value;
use sl_analysis::{Analysis, Context, lint};
use sl_core::{Diagnostic, Edit, Fix, LintMeta, Need, Span};

lint!(TYPOS_MISSPELLING, "X0032", "typos-misspelling", "typos", "no misspelled word in code, comments, docs or file names",
    tool: ("typos", Executable), params: [], needs: [Need::Tool("typos")], fix: Exact);

/// The typos analysis
pub struct Typos;

/// The registered instance
pub static TYPOS: Typos = Typos;

static LINTS: &[&LintMeta] = &[&TYPOS_MISSPELLING];

impl Analysis for Typos {
    fn name(&self) -> &'static str {
        "typos"
    }

    fn lints(&self) -> &'static [&'static LintMeta] {
        LINTS
    }

    fn run(&self, cx: &Context<'_>) {
        let p = cx.project;
        let Some(exe) = sl_tools::find("typos", None) else {
            cx.not_run_all("tool:typos", &sl_tools::missing_reason("typos"));
            return;
        };
        let out = match sl_tools::run(&exe, &["--format", "json"], &p.root, &[]) {
            Ok(o) if matches!(o.code, Some(0 | 2)) => o,
            Ok(o) => {
                cx.not_run_all("run:typos", &format!("typos {}", o.failure()));
                return;
            }
            Err(e) => {
                cx.not_run_all("run:typos", &format!("cannot run typos: {e}"));
                return;
            }
        };
        for t in parse(&out.stdout) {
            let file = t.path.trim_start_matches("./").to_owned();
            let fix = match t.corrections.as_slice() {
                [one] => p.read(&file).and_then(|text| t.fix(&file, &text, one)),
                _ => None,
            };
            let help = match t.corrections.as_slice() {
                [] => "the word is known to be wrong and has no correction: reword it".to_owned(),
                c => format!(
                    "write {}; a word that is right here goes into typos.toml's [default.extend-words]",
                    c.join(" or ")
                ),
            };
            let mut d = Diagnostic::new(
                "typos-misspelling",
                format!("{file}:{}", t.typo),
                format!("`{}` is misspelled", t.typo),
            )
            .span(Span::at(&file, t.line, t.column()))
            .help(help);
            if let Some(f) = fix {
                d = d.fix(f);
            }
            cx.emit(d);
        }
    }
}

/// One misspelling
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typo {
    /// As typos prints it (`./src/lib.rs`)
    pub path: String,
    /// 1-based
    pub line: u32,
    /// Byte offset in the line
    pub offset: usize,
    /// The word
    pub typo: String,
    /// What it should be
    pub corrections: Vec<String>,
}

impl Typo {
    /// The 1-based column (in bytes)
    pub fn column(&self) -> u32 {
        self.offset as u32 + 1
    }

    /// The edit replacing the word in `text` (the file's content), when it is where typos said
    pub fn fix(&self, file: &str, text: &str, correction: &str) -> Option<Fix> {
        let start = text
            .split_inclusive('\n')
            .take(self.line.saturating_sub(1) as usize)
            .map(str::len)
            .sum::<usize>()
            + self.offset;
        let end = start + self.typo.len();
        (text.get(start..end) == Some(self.typo.as_str())).then(|| Fix {
            description: format!("{} -> {correction}", self.typo),
            edits: vec![Edit {
                file: file.to_owned(),
                start,
                end,
                replacement: correction.to_owned(),
            }],
        })
    }
}

/// The typos of typos's JSON lines
pub fn parse(stdout: &str) -> Vec<Typo> {
    stdout
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v.get("type").and_then(Value::as_str) == Some("typo"))
        .map(|v| Typo {
            path: v["path"].as_str().unwrap_or_default().to_owned(),
            line: v["line_num"].as_u64().unwrap_or(0) as u32,
            offset: v["byte_offset"].as_u64().unwrap_or(0) as usize,
            typo: v["typo"].as_str().unwrap_or_default().to_owned(),
            corrections: v["corrections"]
                .as_array()
                .map(|c| {
                    c.iter()
                        .filter_map(|s| s.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_typos() {
        let t = parse(include_str!("../tests/samples/typos.jsonl"));
        assert_eq!(t.len(), 9);
        assert_eq!(t[1].path, "./src/lib.rs");
        assert_eq!((t[1].line, t[1].column()), (1, 7));
        assert_eq!(t[1].corrections, vec!["sample".to_owned()]);
    }

    #[test]
    fn exact_fix() {
        let t = &parse(include_str!("../tests/samples/typos.jsonl"))[2];
        let text = "//! A sampel crate with recieve typos.\n"; // spellchecker:disable-line
        let f = t.fix("src/lib.rs", text, "receive").unwrap();
        assert_eq!(&text[f.edits[0].start..f.edits[0].end], "recieve"); // spellchecker:disable-line
        // a file changed since typos ran gets no fix
        assert!(t.fix("src/lib.rs", "something else\n", "receive").is_none());
    }
}
