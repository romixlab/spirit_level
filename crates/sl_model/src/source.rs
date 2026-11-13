//! Rust source files of a package, with their module path, kind and syn tree.

use crate::lines::{LineCount, count_lines};
use std::path::Path;
use std::sync::OnceLock;

/// Where a file sits in its package
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// Under `src/` (library and binaries)
    Src,
    /// Under `tests/`
    Test,
    /// Under `benches/`
    Bench,
    /// Under `examples/`
    Example,
    /// `build.rs`
    Build,
}

impl FileKind {
    /// Test, bench or example code, not shipped
    pub fn is_test_like(self) -> bool {
        matches!(self, FileKind::Test | FileKind::Bench | FileKind::Example)
    }
}

/// One `.rs` file
#[derive(Debug)]
pub struct SourceFile {
    /// Path relative to the project root
    pub rel: String,
    /// Path relative to the package directory (`src/a/b.rs`)
    pub in_package: String,
    /// The package's name
    pub package: String,
    /// Where it sits
    pub kind: FileKind,
    /// The module path inside its crate for `src/` files (`a::b` for `src/a/b.rs` or `src/a/b/mod.rs`, empty for
    /// `lib.rs` and `main.rs`); derived from the file path, so a `#[path]` module is placed by its file
    pub module: String,
    /// The text
    pub text: String,
    /// Its lines
    pub lines: LineCount,
    ast: OnceLock<Result<syn::File, String>>,
}

impl SourceFile {
    /// Reads and parses one file
    pub fn read(
        root: &Path,
        rel: &str,
        package: &str,
        in_package: &str,
    ) -> std::io::Result<SourceFile> {
        let text = std::fs::read_to_string(root.join(rel))?;
        Ok(SourceFile::from_text(rel, package, in_package, text))
    }

    /// A file from its text (fixtures and tests)
    pub fn from_text(rel: &str, package: &str, in_package: &str, text: String) -> SourceFile {
        let kind = kind_of(in_package);
        let module = if kind == FileKind::Src {
            module_of(in_package)
        } else {
            String::new()
        };
        SourceFile {
            rel: rel.to_string(),
            in_package: in_package.to_string(),
            package: package.to_string(),
            kind,
            module,
            lines: count_lines(&text),
            text,
            ast: OnceLock::new(),
        }
    }

    /// The syn tree, parsed on first use, or why it could not be parsed (`file:line:column: message`)
    pub fn ast(&self) -> Result<&syn::File, &str> {
        self.ast
            .get_or_init(|| {
                syn::parse_file(&self.text).map_err(|e| {
                    let at = e.span().start();
                    format!("{}:{}:{}: {e}", self.rel, at.line, at.column + 1)
                })
            })
            .as_ref()
            .map_err(String::as_str)
    }

    /// Whether it is a crate root (`src/lib.rs`, `src/main.rs`, `src/bin/*.rs`)
    pub fn is_crate_root(&self) -> bool {
        matches!(self.in_package.as_str(), "src/lib.rs" | "src/main.rs")
            || (self.in_package.starts_with("src/bin/")
                && self.in_package.matches('/').count() == 2)
    }
}

fn kind_of(in_package: &str) -> FileKind {
    let first = in_package.split('/').next().unwrap_or("");
    match first {
        "tests" => FileKind::Test,
        "benches" => FileKind::Bench,
        "examples" => FileKind::Example,
        "build.rs" => FileKind::Build,
        _ => FileKind::Src,
    }
}

/// `src/a/b.rs` -> `a::b`; `src/a/mod.rs` -> `a`; `src/lib.rs` -> ``; `src/bin/x.rs` -> `` (a crate of its own)
fn module_of(in_package: &str) -> String {
    let p = in_package.strip_prefix("src/").unwrap_or(in_package);
    if let Some(rest) = p.strip_prefix("bin/") {
        // src/bin/x/main.rs and src/bin/x.rs are binary roots; deeper files are their modules
        let parts: Vec<&str> = rest.trim_end_matches(".rs").split('/').collect();
        return if parts.len() <= 1 || (parts.len() == 2 && parts[1] == "main") {
            String::new()
        } else {
            parts[1..]
                .iter()
                .filter(|s| **s != "mod")
                .copied()
                .collect::<Vec<_>>()
                .join("::")
        };
    }
    let p = p.trim_end_matches(".rs");
    if p == "lib" || p == "main" {
        return String::new();
    }
    p.split('/')
        .filter(|s| *s != "mod")
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modules() {
        assert_eq!(module_of("src/lib.rs"), "");
        assert_eq!(module_of("src/a/b.rs"), "a::b");
        assert_eq!(module_of("src/a/mod.rs"), "a");
        assert_eq!(module_of("src/bin/x.rs"), "");
        assert_eq!(module_of("src/bin/x/main.rs"), "");
        assert_eq!(module_of("src/bin/x/y.rs"), "y");
    }

    #[test]
    fn kinds() {
        let f = SourceFile::from_text("t/tests/a.rs", "t", "tests/a.rs", "fn main(){}".into());
        assert_eq!(f.kind, FileKind::Test);
        assert!(f.ast().is_ok());
        let f = SourceFile::from_text("t/src/lib.rs", "t", "src/lib.rs", "fn (".into());
        assert!(f.ast().unwrap_err().starts_with("t/src/lib.rs:1:"));
    }
}
