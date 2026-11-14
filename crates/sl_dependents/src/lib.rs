//! The repos that build on a library, and the crates in them to retest when it changes.
//!
//! [`find`] takes the library's [`Project`] and its `[dependents]` table: it measures what changed against the
//! base revision, adds the library's crates that depend on those, then reads the `Cargo.lock` of every checkout
//! the table names. A crate of a dependent is listed when its lock graph reaches a changed crate, directly or
//! through others. Nothing is built and nothing is written.
//!
//! ```
//! fn commands(
//!     project: &sl_model::Project,
//!     table: &sl_config::Dependents,
//! ) -> Result<Vec<String>, sl_dependents::GitError> {
//!     let report = sl_dependents::find(project, table, &sl_dependents::Options::default())?;
//!     Ok(report.retest.iter().map(|d| format!("{}: {}", d.name, d.command)).collect())
//! }
//! ```

mod changed;
mod git;
mod lock;

pub use git::GitError;

use lock::Lock;
use sl_config::Dependents;
use sl_model::Project;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The JSON schema of a [`Report`]
pub const SCHEMA: &str = "spirit_level.dependents/1";

/// How a run measures change
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The base revision instead of the table's (whose default is the library's default branch)
    pub base: Option<String>,
    /// Count every crate of the library as changed (no git needed)
    pub all: bool,
    /// The home directory, for `~/` in the table
    pub home: Option<PathBuf>,
}

/// What to retest, and where
#[derive(Clone, Debug, serde::Serialize)]
pub struct Report {
    /// [`SCHEMA`]
    pub schema: &'static str,
    /// The library's folder name
    pub library: String,
    /// The library's root
    pub root: PathBuf,
    /// The revision changes were measured against; None with `--all`
    pub base: Option<String>,
    /// The crates whose files changed
    pub changed: Vec<String>,
    /// The file that made every crate count as changed
    pub workspace_wide: Option<String>,
    /// The changed crates and the library's crates that depend on them
    pub affected: Vec<String>,
    /// The dependents with crates to retest
    pub retest: Vec<Dependent>,
    /// The dependents that use the library but none of the affected crates
    pub unaffected: Vec<Unaffected>,
    /// Checkouts that could not be read
    pub errors: Vec<RepoError>,
}

/// A dependent repo with crates to retest
#[derive(Clone, Debug, serde::Serialize)]
pub struct Dependent {
    /// Its folder name
    pub name: String,
    /// Its root
    pub root: PathBuf,
    /// The crates to retest
    pub crates: Vec<RetestCrate>,
    /// The retest command, run at `root`
    pub command: String,
    /// Where its lock takes the library from
    pub sources: Vec<LibSource>,
}

/// A crate of a dependent to retest
#[derive(Clone, Debug, serde::Serialize)]
pub struct RetestCrate {
    /// The crate
    pub name: String,
    /// The affected crates of the library it reaches
    pub uses: Vec<String>,
    /// Its direct dependencies the affected crates are reached through; empty when it uses them directly
    pub through: Vec<String>,
}

/// A dependent repo none of whose crates reach an affected crate
#[derive(Clone, Debug, serde::Serialize)]
pub struct Unaffected {
    /// Its folder name
    pub name: String,
    /// Its root
    pub root: PathBuf,
    /// The library's crates its lock holds
    pub uses: Vec<String>,
}

/// A checkout that could not be read
#[derive(Clone, Debug, serde::Serialize)]
pub struct RepoError {
    /// Its root
    pub root: PathBuf,
    /// What went wrong
    pub error: String,
}

/// Where a dependent's lock takes some of the library's crates from
#[derive(Clone, Debug, serde::Serialize)]
pub struct LibSource {
    /// The library's crates taken from it
    pub crates: Vec<String>,
    /// The source
    #[serde(flatten)]
    pub kind: SourceKind,
}

/// The kind of a [`LibSource`]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SourceKind {
    /// A git dependency, fixed at a commit by the lock
    Git {
        /// The repository URL
        url: String,
        /// The branch it follows, if it names one
        branch: Option<String>,
        /// The commit the lock holds
        commit: String,
        /// That commit against the library's checkout
        #[serde(flatten)]
        state: CommitState,
    },
    /// A path dependency: it builds against a checkout as it is
    Path {
        /// The path of one of the crates its members name, relative to the dependent's root
        path: Option<String>,
        /// Whether that path is inside the library's checkout this run measured
        this_checkout: bool,
    },
    /// A registry release
    Registry {
        /// The registry
        source: String,
        /// The released version the lock holds
        version: String,
    },
}

/// A locked commit against the library's checkout
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum CommitState {
    /// The lock holds the tip of the branch
    Current {
        /// The ref compared with
        tip: String,
    },
    /// The lock holds an older commit of the branch
    Behind {
        /// The ref compared with
        tip: String,
        /// Commits between them
        commits: u32,
    },
    /// The commit could not be placed
    Unknown {
        /// Why
        reason: String,
    },
}

/// What a checkout turned out to be
enum Outcome {
    Retest(Dependent),
    Unaffected(Unaffected),
    NotUser,
    Error(RepoError),
}

/// The dependents of `project` the table names, and the crates in each to retest
pub fn find(project: &Project, table: &Dependents, opts: &Options) -> Result<Report, GitError> {
    let base = opts
        .base
        .clone()
        .or_else(|| table.base.clone())
        .unwrap_or_else(|| git::default_branch(&project.root));
    let changed = if opts.all {
        changed::all(project)
    } else {
        changed::against(project, &base)?
    };
    let affected = changed::with_dependents(project, &changed.crates);
    let lib = Library {
        root: &project.root,
        members: project.packages.iter().map(|p| p.name.clone()).collect(),
        affected: &affected,
        base: &base,
        test: &table.test,
    };
    let mut report = Report {
        schema: SCHEMA,
        library: folder_name(&project.root),
        root: project.root.clone(),
        base: (!opts.all).then_some(base.clone()),
        changed: changed.crates.iter().cloned().collect(),
        workspace_wide: changed.workspace_wide,
        affected: affected.iter().cloned().collect(),
        retest: Vec::new(),
        unaffected: Vec::new(),
        errors: Vec::new(),
    };
    for (root, named) in checkouts(
        table,
        opts.home.as_deref(),
        &project.root,
        &mut report.errors,
    ) {
        match lib.examine(&root, named) {
            Outcome::Retest(d) => report.retest.push(d),
            Outcome::Unaffected(u) => report.unaffected.push(u),
            Outcome::NotUser => {}
            Outcome::Error(e) => report.errors.push(e),
        }
    }
    Ok(report)
}

/// The checkouts the table names: `(root, named one by one)`, the library's own root left out
fn checkouts(
    table: &Dependents,
    home: Option<&Path>,
    library: &Path,
    errors: &mut Vec<RepoError>,
) -> Vec<(PathBuf, bool)> {
    let own = std::fs::canonicalize(library).unwrap_or_else(|_| library.to_path_buf());
    let mut out = Vec::new();
    for s in &table.scan {
        let dir = table.resolve(s, home);
        let entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) => {
                errors.push(RepoError {
                    root: dir,
                    error: format!("cannot read the folder: {e}"),
                });
                continue;
            }
        };
        let mut found: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .map(|e| e.path())
            .filter(|p| p.join("Cargo.lock").is_file())
            .collect();
        found.sort();
        out.extend(found.into_iter().map(|p| (p, false)));
    }
    out.extend(table.repos.iter().map(|r| (table.resolve(r, home), true)));
    let mut seen = BTreeSet::new();
    out.into_iter()
        .map(|(p, named)| (std::fs::canonicalize(&p).unwrap_or(p), named))
        .filter(|(p, _)| *p != own && seen.insert(p.clone()))
        .collect()
}

fn folder_name(p: &Path) -> String {
    p.file_name().map_or_else(
        || p.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The library as the examination of a dependent needs it
struct Library<'a> {
    root: &'a Path,
    members: BTreeSet<String>,
    affected: &'a BTreeSet<String>,
    base: &'a str,
    test: &'a str,
}

impl Library<'_> {
    fn examine(&self, root: &Path, named: bool) -> Outcome {
        let err = |error: String| {
            Outcome::Error(RepoError {
                root: root.to_path_buf(),
                error,
            })
        };
        let text = match std::fs::read_to_string(root.join("Cargo.lock")) {
            Ok(t) => t,
            Err(e) if named => {
                return err(format!(
                    "cannot read Cargo.lock ({e}): `cargo generate-lockfile` writes one"
                ));
            }
            Err(_) => return Outcome::NotUser,
        };
        let lock = match Lock::parse(&text) {
            Ok(l) => l,
            Err(e) => return err(format!("Cargo.lock: {e}")),
        };
        if !lock.packages.iter().any(|p| self.members.contains(&p.name)) {
            return Outcome::NotUser;
        }
        let project = match Project::load(root) {
            Ok(p) => p,
            Err(e) => return err(e.to_string()),
        };
        let own: BTreeSet<&str> = project.packages.iter().map(|p| p.name.as_str()).collect();
        let is_lib = |i: usize| {
            let p = &lock.packages[i];
            self.members.contains(&p.name) && !(p.source.is_none() && own.contains(p.name.as_str()))
        };
        let lib: Vec<usize> = (0..lock.packages.len()).filter(|&i| is_lib(i)).collect();
        if lib.is_empty() {
            return Outcome::NotUser;
        }
        let hit = |i: usize| is_lib(i) && self.affected.contains(&lock.packages[i].name);
        let crates = retest_crates(&lock, &project, &is_lib, &hit);
        let name = folder_name(root);
        if crates.is_empty() {
            let uses = lib
                .iter()
                .map(|&i| lock.packages[i].name.clone())
                .collect::<BTreeSet<_>>();
            return Outcome::Unaffected(Unaffected {
                name,
                root: root.to_path_buf(),
                uses: uses.into_iter().collect(),
            });
        }
        let mut command = self.test.to_owned();
        for c in &crates {
            command.push_str(" -p ");
            command.push_str(&c.name);
        }
        Outcome::Retest(Dependent {
            name,
            root: root.to_path_buf(),
            crates,
            command,
            sources: self.sources(&lock, &lib, &project),
        })
    }

    /// Where the lock takes the library's crates from, one entry per source
    fn sources(&self, lock: &Lock, lib: &[usize], project: &Project) -> Vec<LibSource> {
        let mut out: Vec<(Option<String>, LibSource)> = Vec::new();
        for &i in lib {
            let p = &lock.packages[i];
            if let Some((_, s)) = out.iter_mut().find(|(src, _)| *src == p.source) {
                s.crates.push(p.name.clone());
                continue;
            }
            let kind = match &p.source {
                None => self.path_source(lock, lib, project),
                Some(s) => match parse_git(s) {
                    Some((url, branch, commit)) => {
                        let state = self.commit_state(&commit, branch.as_deref());
                        SourceKind::Git {
                            url,
                            branch,
                            commit,
                            state,
                        }
                    }
                    None => SourceKind::Registry {
                        source: s.clone(),
                        version: p.version.clone(),
                    },
                },
            };
            out.push((
                p.source.clone(),
                LibSource {
                    crates: vec![p.name.clone()],
                    kind,
                },
            ));
        }
        out.into_iter().map(|(_, s)| s).collect()
    }

    /// A path source: the first path a member of the dependent gives for a crate of the library
    fn path_source(&self, lock: &Lock, lib: &[usize], project: &Project) -> SourceKind {
        let names: BTreeSet<&str> = lib
            .iter()
            .filter(|&&i| lock.packages[i].source.is_none())
            .map(|&i| lock.packages[i].name.as_str())
            .collect();
        let path = project
            .packages
            .iter()
            .flat_map(|m| &m.deps)
            .find(|d| d.path.is_some() && names.contains(d.name.as_str()))
            .and_then(|d| d.path.clone());
        let own = std::fs::canonicalize(self.root).ok();
        let this_checkout = path.as_ref().is_some_and(|p| {
            std::fs::canonicalize(project.root.join(p))
                .ok()
                .zip(own.as_ref())
                .is_some_and(|(abs, own)| abs.starts_with(own))
        });
        SourceKind::Path {
            path,
            this_checkout,
        }
    }

    /// A locked commit against the tip of the branch it follows: the library's local branch, else its
    /// `origin/` one; the base when the source names no branch
    fn commit_state(&self, commit: &str, branch: Option<&str>) -> CommitState {
        let tip = match branch {
            None => self.base.to_owned(),
            Some(b) => match [b.to_owned(), format!("origin/{b}")]
                .into_iter()
                .find(|r| git::commit(self.root, r).is_ok())
            {
                Some(r) => r,
                None => {
                    return CommitState::Unknown {
                        reason: format!("no branch {b} in the library's checkout"),
                    };
                }
            },
        };
        let Ok(tip_commit) = git::commit(self.root, &tip) else {
            return CommitState::Unknown {
                reason: format!("no {tip} in the library's checkout"),
            };
        };
        if tip_commit.starts_with(commit) || commit.starts_with(&tip_commit) {
            CommitState::Current { tip }
        } else if git::is_ancestor(self.root, commit, &tip_commit) {
            CommitState::Behind {
                commits: git::count(self.root, commit, &tip_commit).unwrap_or(0),
                tip,
            }
        } else if git::commit(self.root, commit).is_ok() {
            CommitState::Unknown {
                reason: format!("the locked commit is not on {tip}"),
            }
        } else {
            CommitState::Unknown {
                reason: "the locked commit is not in the library's checkout: fetch it".to_owned(),
            }
        }
    }
}

/// The dependent's own crates whose lock graph reaches a package `hit` accepts; `is_lib` tells the library's
/// crates, where each crate's list of crates it uses stops
fn retest_crates(
    lock: &Lock,
    project: &Project,
    is_lib: &dyn Fn(usize) -> bool,
    hit: &dyn Fn(usize) -> bool,
) -> Vec<RetestCrate> {
    let leads = |i: usize| lock.reach(i).into_iter().any(hit);
    let mut out = Vec::new();
    for pkg in &project.packages {
        let Some(m) = lock
            .packages
            .iter()
            .position(|p| p.source.is_none() && p.name == pkg.name)
        else {
            continue;
        };
        let uses: BTreeSet<String> = lock
            .reach_until(m, is_lib)
            .into_iter()
            .filter(|&e| is_lib(e) && leads(e))
            .map(|e| lock.packages[e].name.clone())
            .collect();
        if uses.is_empty() {
            continue;
        }
        let deps = &lock.packages[m].deps;
        let direct = deps.iter().any(|&d| is_lib(d) && leads(d));
        let through: BTreeSet<String> = if direct {
            BTreeSet::new()
        } else {
            deps.iter()
                .filter(|&&d| leads(d))
                .map(|&d| lock.packages[d].name.clone())
                .collect()
        };
        out.push(RetestCrate {
            name: pkg.name.clone(),
            uses: uses.into_iter().collect(),
            through: through.into_iter().collect(),
        });
    }
    out
}

/// `(url, branch, commit)` of a git source (`git+URL?branch=B#COMMIT`)
fn parse_git(source: &str) -> Option<(String, Option<String>, String)> {
    let rest = source.strip_prefix("git+")?;
    let (url, commit) = rest.split_once('#')?;
    let (url, query) = url.split_once('?').unwrap_or((url, ""));
    let branch = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("branch="))
        .map(str::to_owned);
    Some((url.to_owned(), branch, commit.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_sources() {
        assert_eq!(
            parse_git("git+https://github.com/a/lib?branch=main#0123abc"),
            Some((
                "https://github.com/a/lib".to_owned(),
                Some("main".to_owned()),
                "0123abc".to_owned()
            ))
        );
        assert_eq!(
            parse_git("git+https://github.com/a/lib#0123abc"),
            Some((
                "https://github.com/a/lib".to_owned(),
                None,
                "0123abc".to_owned()
            ))
        );
        assert_eq!(
            parse_git("registry+https://github.com/rust-lang/crates.io-index"),
            None
        );
    }
}
