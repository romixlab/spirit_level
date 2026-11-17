//! The project's justfile: its recipes, each with the line it starts on and the command lines of its body.

use std::path::Path;

/// The file names `just` looks for, in its order
pub const FILES: [&str; 3] = ["justfile", "Justfile", ".justfile"];

/// A justfile of the project
#[derive(Debug)]
pub struct Justfile {
    /// Path relative to the project root
    pub rel: String,
    /// Its recipes, in file order
    pub recipes: Vec<Recipe>,
}

/// One recipe
#[derive(Debug)]
pub struct Recipe {
    /// Its name
    pub name: String,
    /// 1-based line of its header
    pub line: u32,
    /// The command lines of its body, trimmed, without blank lines, comments and a shebang line
    pub body: Vec<String>,
    /// Whether its body is an inline script (a shebang recipe)
    pub shebang: bool,
}

impl Justfile {
    /// Reads the justfile at the project root; None when there is none
    pub fn read(root: &Path) -> Option<Self> {
        FILES.into_iter().find_map(|f| {
            let text = std::fs::read_to_string(root.join(f)).ok()?;
            Some(Self::parse(f, &text))
        })
    }

    /// Reads the recipes of a justfile's text
    pub fn parse(rel: impl Into<String>, text: &str) -> Self {
        Self {
            rel: rel.into(),
            recipes: recipes(text),
        }
    }

    /// The recipe of a name
    pub fn recipe(&self, name: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.name == name)
    }
}

/// The recipes of a justfile's text
fn recipes(text: &str) -> Vec<Recipe> {
    let mut out: Vec<Recipe> = Vec::new();
    // an assignment, setting or attribute between recipes ends the body of the one before it
    let mut open = false;
    for (i, l) in text.lines().enumerate() {
        if l.starts_with([' ', '\t']) {
            if let Some(r) = out.last_mut().filter(|_| open) {
                let t = l.trim();
                if t.starts_with("#!") {
                    r.shebang = true;
                } else if !t.is_empty() && !t.starts_with('#') {
                    r.body.push(t.to_owned());
                }
            }
            continue;
        }
        let t = l.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        open = false;
        if t.starts_with('[') || t.contains(":=") {
            continue;
        }
        let Some((head, _)) = t.split_once(':') else {
            continue;
        };
        let name = head
            .trim_start_matches('@')
            .split_whitespace()
            .next()
            .unwrap_or("");
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            || ["set", "export", "alias", "import", "mod"].contains(&name)
        {
            continue;
        }
        out.push(Recipe {
            name: name.to_owned(),
            line: i as u32 + 1,
            body: Vec::new(),
            shebang: false,
        });
        open = true;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recipes() {
        let j = Justfile::parse(
            "justfile",
            "set shell := [\"bash\"]\n# doc\ndefault:\n    @just --list\n\ntest *flags:\n    cargo nextest run {{flags}}\n    # comment\n\n[private]\n_x a=\"1\": test\n    #!/usr/bin/env bash\n    echo\n",
        );
        let names: Vec<&str> = j.recipes.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["default", "test", "_x"]);
        assert_eq!(j.recipes[1].body, ["cargo nextest run {{flags}}"]);
        assert!(j.recipes[2].shebang);
        assert_eq!(j.recipes[1].line, 6);
        assert_eq!(j.recipe("_x").map(|r| r.body.len()), Some(1));
    }
}
