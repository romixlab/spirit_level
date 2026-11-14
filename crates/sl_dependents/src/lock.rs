//! A `Cargo.lock` as a graph: every package with its source and the packages it depends on.

use std::collections::BTreeSet;

/// A package of the lock
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LockPackage {
    pub name: String,
    pub version: String,
    /// None for a path package (a member or a path dependency)
    pub source: Option<String>,
    /// Indexes of the packages it depends on, of every kind (the lock does not tell them apart)
    pub deps: Vec<usize>,
}

/// The packages of a lock
#[derive(Clone, Debug, Default)]
pub(crate) struct Lock {
    pub packages: Vec<LockPackage>,
}

impl Lock {
    /// Reads a lock's text
    pub fn parse(text: &str) -> Result<Self, toml_edit::TomlError> {
        let doc: toml_edit::DocumentMut = text.parse()?;
        let mut packages = Vec::new();
        let mut names: Vec<Vec<String>> = Vec::new();
        if let Some(arr) = doc.get("package").and_then(|i| i.as_array_of_tables()) {
            for t in arr {
                let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_owned);
                packages.push(LockPackage {
                    name: s("name").unwrap_or_default(),
                    version: s("version").unwrap_or_default(),
                    source: s("source"),
                    deps: Vec::new(),
                });
                names.push(
                    t.get("dependencies")
                        .and_then(|d| d.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .map(str::to_owned)
                                .collect()
                        })
                        .unwrap_or_default(),
                );
            }
        }
        let resolved: Vec<Vec<usize>> = names
            .iter()
            .map(|ds| ds.iter().filter_map(|d| find(&packages, d)).collect())
            .collect();
        for (p, deps) in packages.iter_mut().zip(resolved) {
            p.deps = deps;
        }
        Ok(Self { packages })
    }

    /// The packages reachable from `start` (itself included)
    pub fn reach(&self, start: usize) -> BTreeSet<usize> {
        self.reach_until(start, &|_| false)
    }

    /// The packages reachable from `start` (itself included) without going past a package `stop` accepts
    pub fn reach_until(&self, start: usize, stop: &dyn Fn(usize) -> bool) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            if seen.insert(i)
                && (i == start || !stop(i))
                && let Some(p) = self.packages.get(i)
            {
                stack.extend(&p.deps);
            }
        }
        seen
    }
}

/// The package a `dependencies` entry names: `name`, `name version` or `name version (source)`
fn find(packages: &[LockPackage], entry: &str) -> Option<usize> {
    let mut parts = entry.splitn(3, ' ');
    let name = parts.next()?;
    let version = parts.next();
    let source = parts
        .next()
        .map(|s| s.trim_start_matches('(').trim_end_matches(')'));
    packages.iter().position(|p| {
        p.name == name
            && version.is_none_or(|v| p.version == v)
            && source.is_none_or(|s| p.source.as_deref() == Some(s))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = r#"
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = ["lib_io", "rand 0.9.0"]

[[package]]
name = "lib_io"
version = "0.2.0"
source = "git+https://example.invalid/lib?branch=main#0123abcd"
dependencies = ["rand 0.8.5 (registry+https://github.com/rust-lang/crates.io-index)"]

[[package]]
name = "rand"
version = "0.8.5"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "rand"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    #[test]
    fn entries_name_their_package() {
        let l = Lock::parse(LOCK).unwrap();
        assert_eq!(l.packages[0].deps, vec![1, 3]);
        assert_eq!(l.packages[1].deps, vec![2]);
        assert_eq!(l.reach(0), BTreeSet::from([0, 1, 2, 3]));
        assert_eq!(l.reach(1), BTreeSet::from([1, 2]));
        assert_eq!(l.reach_until(0, &|i| i == 1), BTreeSet::from([0, 1, 3]));
    }
}
