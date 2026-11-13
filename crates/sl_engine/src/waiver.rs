//! Waivers written in the project's own files:
//!
//! - Rust: `// spirit-level: allow(lint-a, lint-b) the reason`
//! - TOML: `# spirit-level: allow(lint) the reason`
//! - optionally an expiry right after the list: `allow(lint) expires=2027-01-31 the reason` (or `expires=v0.5.0`)
//!
//! At the end of a line a waiver covers that line. On a line of its own it covers the next item: the first line
//! after it that is not blank, a comment or an attribute, and in Rust the whole item, field or statement that
//! starts there; in TOML a table header covers its table.

use sl_core::{Date, Expiry, Version};
use sl_model::Project;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// The marker that starts a waiver
pub const MARK: &str = "spirit-level:";

/// One waiver comment
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Waiver {
    /// File relative to the project root
    pub file: String,
    /// The line of the comment
    pub line: u32,
    /// The lints it allows
    pub lints: Vec<String>,
    /// Why
    pub reason: String,
    /// When it stops holding
    pub expires: Option<Expiry>,
    /// The lines it covers, inclusive
    pub covers: (u32, u32),
}

/// A waiver written wrong
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// File
    pub file: String,
    /// Line
    pub line: u32,
    /// What is wrong
    pub message: String,
}

/// The waivers of every Rust file and manifest of the project, and the problems found reading them. `resolve` turns
/// a lint's id or code into its id; a waiver keeps the ids.
pub fn collect(
    project: &Project,
    resolve: &dyn Fn(&str) -> Option<&'static str>,
    today: Date,
    version: Option<Version>,
) -> (Vec<Waiver>, Vec<Problem>) {
    let mut ws = Vec::new();
    let mut ps = Vec::new();
    for f in project.all_sources() {
        if f.text.contains(MARK) {
            let extents = f.ast().map(extents).unwrap_or_default();
            parse(
                &f.rel, &f.text, true, &extents, resolve, today, version, &mut ws, &mut ps,
            );
        }
    }
    let mut tomls: Vec<&sl_model::TomlFile> =
        project.packages.iter().map(|p| &p.manifest).collect();
    if !project.packages.iter().any(|p| p.dir.is_empty()) {
        tomls.push(&project.root_manifest);
    }
    for t in tomls {
        if t.text.contains(MARK) {
            parse(
                &t.rel,
                &t.text,
                false,
                &[],
                resolve,
                today,
                version,
                &mut ws,
                &mut ps,
            );
        }
    }
    (ws, ps)
}

#[allow(
    clippy::too_many_arguments,
    reason = "one internal call site; a struct would only rename the arguments"
)]
fn parse(
    rel: &str,
    text: &str,
    rust: bool,
    extents: &[(u32, u32)],
    resolve: &dyn Fn(&str) -> Option<&'static str>,
    today: Date,
    version: Option<Version>,
    ws: &mut Vec<Waiver>,
    ps: &mut Vec<Problem>,
) {
    let lines: Vec<&str> = text.lines().collect();
    let comment = if rust { "//" } else { "#" };
    for (i, l) in lines.iter().enumerate() {
        let Some(at) = l.find(MARK) else { continue };
        let before = &l[..at];
        let Some(cpos) = before.find(comment) else {
            continue;
        };
        // a plain comment only: not a doc comment (`///`, `//!`: documentation shows waivers as examples), not
        // inside a string literal, and the marker right after the comment start
        if !before[cpos + comment.len()..].trim().is_empty()
            || (rust && (l[cpos..].starts_with("///") || l[cpos..].starts_with("//!")))
            || in_string(&l[..cpos])
        {
            continue;
        }
        let line = i as u32 + 1;
        let own_line = before[..cpos].trim().is_empty();
        let rest = l[at + MARK.len()..].trim();
        let problem = |m: String| Problem {
            file: rel.to_string(),
            line,
            message: m,
        };
        let Some(inner) = rest.strip_prefix("allow(") else {
            ps.push(problem(format!(
                "a waiver reads `{MARK} allow(lint) reason`, found `{rest}`"
            )));
            continue;
        };
        let Some(close) = inner.find(')') else {
            ps.push(problem(
                "the waiver's lint list is not closed with `)`".into(),
            ));
            continue;
        };
        let named: Vec<&str> = inner[..close]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let mut tail = inner[close + 1..].trim();
        let mut expires = None;
        if let Some(e) = tail.strip_prefix("expires=") {
            let (val, r) = e.split_once(char::is_whitespace).unwrap_or((e, ""));
            match Expiry::parse(val) {
                Some(x) => expires = Some(x),
                None => ps.push(problem(format!(
                    "expires={val} is neither a date (2027-01-31) nor a release (v0.5.0)"
                ))),
            }
            tail = r.trim();
        }
        if named.is_empty() {
            ps.push(problem("the waiver names no lint".into()));
            continue;
        }
        let mut lints = Vec::new();
        for name in named {
            match resolve(name) {
                Some(id) => lints.push(id.to_string()),
                None => ps.push(problem(format!(
                    "the waiver names {name}, which is not a lint"
                ))),
            }
        }
        if tail.is_empty() {
            ps.push(problem(format!(
                "the waiver of {} gives no reason",
                lints.join(", ")
            )));
            continue;
        }
        if let Some(e) = expires
            && e.expired(today, version)
        {
            ps.push(problem(format!(
                "the waiver of {} expired ({e}): fix the finding and remove it, or renew it with a new reason",
                lints.join(", ")
            )));
            continue;
        }
        let covers = if own_line {
            coverage(&lines, i, line, rust, comment, extents)
        } else {
            (line, line)
        };
        ws.push(Waiver {
            file: rel.to_string(),
            line,
            lints,
            reason: tail.to_string(),
            expires,
            covers,
        });
    }
}

/// The lines an own-line waiver at index `i` (line `line`) covers: the next item
fn coverage(
    lines: &[&str],
    i: usize,
    line: u32,
    rust: bool,
    comment: &str,
    extents: &[(u32, u32)],
) -> (u32, u32) {
    let next = (i + 1..lines.len()).find(|&j| {
        let t = lines[j].trim();
        !(t.is_empty()
            || t.starts_with(comment)
            || (rust && t.starts_with("#["))
            || (rust && t.starts_with("#![")))
    });
    match next {
        None => (line, line),
        Some(j) => {
            let n = j as u32 + 1;
            // the run of comment and attribute lines the waiver sits in: an item's span starts at its first
            // doc comment or attribute, which may be above the waiver
            let top = (0..i)
                .rev()
                .take_while(|&k| {
                    let t = lines[k].trim();
                    t.starts_with(comment) || (rust && t.starts_with("#["))
                })
                .last()
                .map_or(line, |k| k as u32 + 1);
            let end = if rust {
                extents
                    .iter()
                    .filter(|(s, _)| *s >= top && *s <= n)
                    .map(|(_, e)| *e)
                    .max()
                    .unwrap_or(n)
                    .max(n)
            } else if lines[j].trim().starts_with('[') {
                (j + 1..lines.len())
                    .find(|&k| lines[k].trim().starts_with('['))
                    .map_or(lines.len() as u32, |k| k as u32)
            } else {
                n
            };
            (n, end)
        }
    }
}

/// Whether a position after `prefix` is inside a string literal (an odd count of unescaped quotes before it)
fn in_string(prefix: &str) -> bool {
    let mut n = 0;
    let mut esc = false;
    for c in prefix.chars() {
        match c {
            '\\' if !esc => esc = true,
            '"' if !esc => n += 1,
            _ => esc = false,
        }
        if c != '\\' {
            esc = false;
        }
    }
    n % 2 == 1
}

/// (first line, last line) of every item, impl item, trait item, field and statement
fn extents(file: &syn::File) -> Vec<(u32, u32)> {
    struct V(Vec<(u32, u32)>);
    impl V {
        fn add<T: Spanned>(&mut self, n: &T) {
            let s = n.span();
            self.0.push((s.start().line as u32, s.end().line as u32));
        }
    }
    impl<'ast> Visit<'ast> for V {
        fn visit_item(&mut self, i: &'ast syn::Item) {
            self.add(i);
            syn::visit::visit_item(self, i);
        }
        fn visit_impl_item(&mut self, i: &'ast syn::ImplItem) {
            self.add(i);
            syn::visit::visit_impl_item(self, i);
        }
        fn visit_trait_item(&mut self, i: &'ast syn::TraitItem) {
            self.add(i);
            syn::visit::visit_trait_item(self, i);
        }
        fn visit_field(&mut self, f: &'ast syn::Field) {
            self.add(f);
            syn::visit::visit_field(self, f);
        }
        fn visit_variant(&mut self, f: &'ast syn::Variant) {
            self.add(f);
            syn::visit::visit_variant(self, f);
        }
        fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
            self.add(s);
            syn::visit::visit_stmt(self, s);
        }
    }
    let mut v = V(Vec::new());
    v.visit_file(file);
    v.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lints the tests know, and one code
    fn resolve(name: &str) -> Option<&'static str> {
        ["a", "x", "x-y", "god-function"]
            .into_iter()
            .find(|id| *id == name)
            .or((name == "S0001").then_some("crate-lines"))
    }

    fn run(rel: &str, text: &str) -> (Vec<Waiver>, Vec<Problem>) {
        let rust = rel.ends_with(".rs");
        let ext = if rust {
            extents(&syn::parse_file(text).unwrap())
        } else {
            Vec::new()
        };
        let (mut w, mut p) = (Vec::new(), Vec::new());
        parse(
            rel,
            text,
            rust,
            &ext,
            &resolve,
            Date::parse("2026-11-13").unwrap(),
            None,
            &mut w,
            &mut p,
        );
        (w, p)
    }

    #[test]
    fn rust_waivers_cover_items() {
        let (w, p) = run(
            "a.rs",
            "// spirit-level: allow(god-function) parser state machine, split tracked\n#[inline]\nfn f() {\n    let x = 1;\n}\nstatic A: u8 = 0; // spirit-level: allow(x-y) set once\n",
        );
        assert!(p.is_empty(), "{p:?}");
        assert_eq!(w[0].covers, (3, 5));
        assert_eq!(w[0].lints, vec!["god-function"]);
        assert_eq!(w[1].covers, (6, 6));
        assert_eq!(w[1].reason, "set once");
    }

    #[test]
    fn waiver_inside_a_doc_block_covers_the_item() {
        let (w, _) = run(
            "a.rs",
            "/// Doc\n// spirit-level: allow(x) because\nfn f() {\n    g();\n}\n",
        );
        assert_eq!(w[0].covers, (3, 5));
    }

    #[test]
    fn toml_waiver_covers_table() {
        let (w, _) = run(
            "Cargo.toml",
            "# spirit-level: allow(a) why\n[dependencies]\nx = 1\ny = 2\n[features]\n",
        );
        assert_eq!(w[0].covers, (2, 4));
    }

    #[test]
    fn problems() {
        let (w, p) = run(
            "a.rs",
            "// spirit-level: allow(a)\nfn a() {}\n// spirit-level: allow(nope) r\nfn b() {}\n// spirit-level: allow(a) expires=2026-01-01 old\nfn c() {}\n// spirit-level: deny(a)\n",
        );
        let m: Vec<&str> = p.iter().map(|p| p.message.as_str()).collect();
        assert!(m[0].contains("gives no reason"));
        assert!(m[1].contains("nope, which is not a lint"));
        assert!(m[2].contains("expired"));
        assert!(m[3].contains("a waiver reads"));
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn a_code_names_its_lint() {
        let (w, p) = run(
            "a.rs",
            "// spirit-level: allow(S0001) big on purpose
fn f() {}
",
        );
        assert!(p.is_empty(), "{p:?}");
        assert_eq!(w[0].lints, vec!["crate-lines"]);
    }
}
