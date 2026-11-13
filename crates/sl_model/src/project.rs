//! The workspace as cargo sees it, read through `cargo metadata`, plus the manifests' text.

use crate::source::SourceFile;
use crate::toml_file::TomlFile;
use cargo_metadata::{DependencyKind, Metadata, TargetKind as CmTargetKind};
use sl_core::Version;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// Why the project could not be read
#[derive(Debug)]
pub enum LoadError {
    /// `cargo metadata` failed (a manifest does not parse, a member is missing)
    Metadata(String),
    /// A manifest could not be read or parsed as TOML
    Manifest(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Metadata(e) => write!(f, "cargo metadata failed: {e}"),
            LoadError::Manifest(e) => write!(f, "manifest: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Normal, dev or build dependency
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DepKind {
    /// `[dependencies]`
    Normal,
    /// `[dev-dependencies]`
    Dev,
    /// `[build-dependencies]`
    Build,
}

impl DepKind {
    /// The manifest table it is declared in
    pub fn table(self) -> &'static str {
        match self {
            DepKind::Normal => "dependencies",
            DepKind::Dev => "dev-dependencies",
            DepKind::Build => "build-dependencies",
        }
    }
}

/// A dependency as declared by a member
#[derive(Clone, Debug)]
pub struct Dep {
    /// The package name
    pub name: String,
    /// The name it is used under, when renamed
    pub rename: Option<String>,
    /// Normal, dev or build
    pub kind: DepKind,
    /// `cfg(...)` or a triple, when target-specific
    pub target: Option<String>,
    /// The source as cargo writes it (`registry+https://...`, `git+https://...?tag=v1`); None for a path dependency
    pub source: Option<String>,
    /// For a path dependency, the path relative to the project root (may start with `..`)
    pub path: Option<String>,
    /// `optional = true`
    pub optional: bool,
    /// Default features on
    pub default_features: bool,
    /// Whether it is a member of the workspace
    pub member: bool,
}

impl Dep {
    /// The key it is declared under in the manifest
    pub fn key(&self) -> &str {
        self.rename.as_deref().unwrap_or(&self.name)
    }

    /// The manifest path of its declaration: `[dependencies] foo` or `[target.'cfg(..)'.dependencies] foo`
    pub fn manifest_path(&self) -> Vec<&str> {
        match &self.target {
            Some(t) => vec!["target", t.as_str(), self.kind.table(), self.key()],
            None => vec![self.kind.table(), self.key()],
        }
    }

    /// The git URL when it comes from git (without `git+` and the query)
    pub fn git_url(&self) -> Option<&str> {
        let s = self.source.as_deref()?.strip_prefix("git+")?;
        Some(s.split(['?', '#']).next().unwrap_or(s))
    }

    /// The git reference pins of a git dependency: `("rev", "abc")`, `("tag", "v1")`, `("branch", "main")`
    pub fn git_pins(&self) -> Vec<(String, String)> {
        let Some(s) = self.source.as_deref().and_then(|s| s.strip_prefix("git+")) else {
            return Vec::new();
        };
        let Some((_, q)) = s.split('#').next().unwrap_or(s).split_once('?') else {
            return Vec::new();
        };
        q.split('&')
            .filter_map(|kv| kv.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Whether it comes from a registry (crates.io or another)
    pub fn is_registry(&self) -> bool {
        self.source.as_deref().is_some_and(|s| s.starts_with("registry+") || s.starts_with("sparse+"))
    }
}

/// The kind of a build target
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// A library (lib, rlib, cdylib, ...)
    Lib,
    /// A procedural macro library
    ProcMacro,
    /// A binary
    Bin,
    /// An integration test, bench, example or build script
    Other,
}

/// A workspace member
#[derive(Debug)]
pub struct Package {
    /// The package name
    pub name: String,
    /// Its version
    pub version: String,
    /// Its directory relative to the project root (`crates/foo`, or `` for a root package)
    pub dir: String,
    /// The manifest
    pub manifest: TomlFile,
    /// `description`
    pub description: Option<String>,
    /// `license`
    pub license: Option<String>,
    /// `edition`
    pub edition: String,
    /// `rust-version`
    pub rust_version: Option<String>,
    /// `publish = false` (or an empty registry list)
    pub publish: bool,
    /// Declared dependencies
    pub deps: Vec<Dep>,
    /// Its targets' kinds and names
    pub targets: Vec<(TargetKind, String)>,
    /// Its features and what each turns on
    pub features: BTreeMap<String, Vec<String>>,
    /// `links`
    pub links: Option<String>,
    sources: OnceLock<Vec<SourceFile>>,
}

impl Package {
    /// Whether it has a library target (a proc-macro counts)
    pub fn has_lib(&self) -> bool {
        self.targets
            .iter()
            .any(|(k, _)| matches!(k, TargetKind::Lib | TargetKind::ProcMacro))
    }

    /// Whether its library is a proc-macro
    pub fn is_proc_macro(&self) -> bool {
        self.targets.iter().any(|(k, _)| *k == TargetKind::ProcMacro)
    }

    /// Whether it builds a binary
    pub fn has_bin(&self) -> bool {
        self.targets.iter().any(|(k, _)| *k == TargetKind::Bin)
    }

    /// A path in its directory, relative to the project root
    pub fn path(&self, rel: &str) -> String {
        if self.dir.is_empty() {
            rel.to_string()
        } else {
            format!("{}/{rel}", self.dir)
        }
    }
}

/// A package of the resolved graph
#[derive(Clone, Debug)]
pub struct ResolvedPackage {
    /// Name
    pub name: String,
    /// Version
    pub version: String,
    /// Source (None for path packages)
    pub source: Option<String>,
    /// Whether it is a workspace member
    pub member: bool,
    /// `links`
    pub links: Option<String>,
    /// Its normal, dev and build edges: (index into `packages`, kinds)
    pub deps: Vec<(usize, Vec<DepKind>)>,
}

/// The resolved dependency graph (full `cargo metadata`)
#[derive(Clone, Debug, Default)]
pub struct Resolve {
    /// Every package in the graph
    pub packages: Vec<ResolvedPackage>,
}

impl Resolve {
    /// Indexes of the packages a member reaches through normal and build edges (dev edges only from the start
    /// packages themselves when `dev` is set), the start included
    pub fn closure(&self, start: usize, dev: bool) -> Vec<usize> {
        let mut seen = vec![false; self.packages.len()];
        let mut stack = vec![(start, true)];
        let mut out = Vec::new();
        while let Some((i, first)) = stack.pop() {
            if std::mem::replace(&mut seen[i], true) {
                continue;
            }
            out.push(i);
            for (j, kinds) in &self.packages[i].deps {
                let ok = kinds.iter().any(|k| *k != DepKind::Dev) || (dev && first);
                if ok {
                    stack.push((*j, false));
                }
            }
        }
        out
    }

    /// The index of a package by name (the first one when there are several versions)
    pub fn find(&self, name: &str) -> Option<usize> {
        self.packages.iter().position(|p| p.name == name)
    }
}

/// The project: a workspace (or one crate) as cargo sees it
#[derive(Debug)]
pub struct Project {
    /// The workspace root, absolute
    pub root: PathBuf,
    /// The root manifest
    pub root_manifest: TomlFile,
    /// Workspace members, ordered by directory
    pub packages: Vec<Package>,
    /// The project's version: `[workspace.package] version`, else the root package's
    pub version: Option<Version>,
    resolve: OnceLock<Result<Resolve, String>>,
}

impl Project {
    /// Reads the project whose manifest is in `dir` or above (cargo's own search)
    pub fn load(dir: &Path) -> Result<Project, LoadError> {
        let out = cargo(dir)
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .output()
            .map_err(|e| LoadError::Metadata(format!("cannot run cargo: {e}")))?;
        if !out.status.success() {
            return Err(LoadError::Metadata(
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ));
        }
        let meta = cargo_metadata::MetadataCommand::parse(String::from_utf8_lossy(&out.stdout))
            .map_err(|e| LoadError::Metadata(e.to_string()))?;
        Project::from_metadata(meta)
    }

    fn from_metadata(meta: Metadata) -> Result<Project, LoadError> {
        let root: PathBuf = meta.workspace_root.clone().into();
        let read = |abs: &Path| -> Result<TomlFile, LoadError> {
            let rel = rel_path(&root, abs);
            let text = std::fs::read_to_string(abs)
                .map_err(|e| LoadError::Manifest(format!("{rel}: {e}")))?;
            TomlFile::parse(rel, text).map_err(LoadError::Manifest)
        };
        let root_manifest = read(&root.join("Cargo.toml"))?;
        let member_names: Vec<String> = meta
            .workspace_members
            .iter()
            .filter_map(|id| meta.packages.iter().find(|p| &p.id == id))
            .map(|p| p.name.to_string())
            .collect();
        let mut packages = Vec::new();
        for id in &meta.workspace_members {
            let Some(p) = meta.packages.iter().find(|p| &p.id == id) else {
                continue;
            };
            let manifest_path: PathBuf = p.manifest_path.clone().into();
            let dir = rel_path(&root, manifest_path.parent().unwrap_or(&root));
            let manifest = read(&manifest_path)?;
            let deps = p
                .dependencies
                .iter()
                .map(|d| Dep {
                    name: d.name.clone(),
                    rename: d.rename.clone(),
                    kind: match d.kind {
                        DependencyKind::Development => DepKind::Dev,
                        DependencyKind::Build => DepKind::Build,
                        _ => DepKind::Normal,
                    },
                    target: d.target.as_ref().map(|t| t.to_string()),
                    source: d.source.as_ref().map(|s| s.to_string()),
                    path: d
                        .path
                        .as_ref()
                        .map(|pth| rel_path(&root, pth.as_std_path())),
                    optional: d.optional,
                    default_features: d.uses_default_features,
                    member: d.path.is_some() && member_names.contains(&d.name),
                })
                .collect();
            let targets = p
                .targets
                .iter()
                .map(|t| {
                    let k = if t.kind.contains(&CmTargetKind::ProcMacro) {
                        TargetKind::ProcMacro
                    } else if t.kind.iter().any(|k| {
                        matches!(
                            k,
                            CmTargetKind::Lib
                                | CmTargetKind::RLib
                                | CmTargetKind::DyLib
                                | CmTargetKind::CDyLib
                                | CmTargetKind::StaticLib
                        )
                    }) {
                        TargetKind::Lib
                    } else if t.kind.contains(&CmTargetKind::Bin) {
                        TargetKind::Bin
                    } else {
                        TargetKind::Other
                    };
                    (k, t.name.clone())
                })
                .collect();
            packages.push(Package {
                name: p.name.to_string(),
                version: p.version.to_string(),
                dir,
                manifest,
                description: p.description.clone(),
                license: p.license.clone(),
                edition: p.edition.to_string(),
                rust_version: p.rust_version.as_ref().map(|v| v.to_string()),
                publish: p.publish.as_ref().is_none_or(|r| !r.is_empty()),
                deps,
                targets,
                features: p.features.clone(),
                links: p.links.clone(),
                sources: OnceLock::new(),
            });
        }
        packages.sort_by(|a, b| a.dir.cmp(&b.dir));
        let version = root_manifest
            .get(&["workspace", "package", "version"])
            .or_else(|| root_manifest.get(&["package", "version"]))
            .and_then(|v| v.as_str())
            .and_then(Version::parse);
        Ok(Project {
            root,
            root_manifest,
            packages,
            version,
            resolve: OnceLock::new(),
        })
    }

    /// Whether this is a workspace of several crates or a virtual root, rather than one crate
    pub fn is_workspace(&self) -> bool {
        self.packages.len() > 1 || self.root_manifest.get(&["package"]).is_none()
    }

    /// The member called `name`
    pub fn package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// The Rust sources of a member, read and parsed on first use. Every `.rs` file under the package's
    /// directory, skipping `target/`, hidden directories and nested packages (a directory with its own
    /// `Cargo.toml`, like a test fixture)
    pub fn sources<'a>(&'a self, pkg: &'a Package) -> &'a [SourceFile] {
        pkg.sources.get_or_init(|| {
            let dir = self.root.join(&pkg.dir);
            let mut files = Vec::new();
            walk(&dir, &dir, &mut files);
            files.sort();
            files
                .into_iter()
                .filter_map(|in_pkg| {
                    let rel = pkg.path(&in_pkg);
                    SourceFile::read(&self.root, &rel, &pkg.name, &in_pkg).ok()
                })
                .collect()
        })
    }

    /// Every source file of every member
    pub fn all_sources(&self) -> impl Iterator<Item = &SourceFile> {
        self.packages.iter().flat_map(|p| self.sources(p))
    }

    /// The resolved dependency graph, from a full `cargo metadata` on first use (it may need the registry index;
    /// with a Cargo.lock it runs `--locked`, so the lock is never rewritten)
    pub fn resolve(&self) -> Result<&Resolve, &str> {
        self.resolve
            .get_or_init(|| self.load_resolve())
            .as_ref()
            .map_err(String::as_str)
    }

    fn load_resolve(&self) -> Result<Resolve, String> {
        let mut cmd = cargo(&self.root);
        cmd.args(["metadata", "--format-version", "1"]);
        if self.root.join("Cargo.lock").exists() {
            cmd.arg("--locked");
        }
        let out = cmd.output().map_err(|e| format!("cannot run cargo: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        let meta = cargo_metadata::MetadataCommand::parse(String::from_utf8_lossy(&out.stdout))
            .map_err(|e| e.to_string())?;
        let resolve = meta.resolve.ok_or("cargo metadata gave no resolve")?;
        let index: BTreeMap<_, usize> = meta
            .packages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id.clone(), i))
            .collect();
        let mut packages: Vec<ResolvedPackage> = meta
            .packages
            .iter()
            .map(|p| ResolvedPackage {
                name: p.name.to_string(),
                version: p.version.to_string(),
                source: p.source.as_ref().map(|s| s.to_string()),
                member: meta.workspace_members.contains(&p.id),
                links: p.links.clone(),
                deps: Vec::new(),
            })
            .collect();
        for node in &resolve.nodes {
            let Some(&i) = index.get(&node.id) else { continue };
            for d in &node.deps {
                let Some(&j) = index.get(&d.pkg) else { continue };
                let mut kinds: Vec<DepKind> = d
                    .dep_kinds
                    .iter()
                    .map(|k| match k.kind {
                        DependencyKind::Development => DepKind::Dev,
                        DependencyKind::Build => DepKind::Build,
                        _ => DepKind::Normal,
                    })
                    .collect();
                kinds.sort();
                kinds.dedup();
                packages[i].deps.push((j, kinds));
            }
        }
        Ok(Resolve { packages })
    }

    /// A file's text relative to the root, if it exists
    pub fn read(&self, rel: &str) -> Option<String> {
        std::fs::read_to_string(self.root.join(rel)).ok()
    }

    /// Whether a file exists relative to the root
    pub fn exists(&self, rel: &str) -> bool {
        self.root.join(rel).exists()
    }
}

/// `cargo` with the environment of an outer cargo removed, so a run under `cargo test` or a build script sees
/// the project as a person would (CARGO_HOME and registry settings stay)
fn cargo(dir: &Path) -> Command {
    let mut cmd = Command::new(std::env::var_os("SPIRIT_LEVEL_CARGO").unwrap_or_else(|| "cargo".into()));
    cmd.current_dir(dir);
    for (k, _) in std::env::vars_os() {
        let Some(k) = k.to_str() else { continue };
        let keep = k == "CARGO_HOME"
            || k.starts_with("CARGO_REGISTRIES_")
            || k.starts_with("CARGO_NET_")
            || k.starts_with("CARGO_HTTP_");
        if (k.starts_with("CARGO_") && !keep) || k.starts_with("RUSTC") || k == "RUSTFLAGS" {
            cmd.env_remove(k);
        }
    }
    cmd
}

fn rel_path(root: &Path, p: &Path) -> String {
    match p.strip_prefix(root) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => {
            // outside the root: express it with `..` where the two share a prefix
            let (mut a, mut b) = (root.components().peekable(), p.components().peekable());
            while let (Some(x), Some(y)) = (a.peek(), b.peek()) {
                if x != y {
                    break;
                }
                a.next();
                b.next();
            }
            let mut parts: Vec<String> = a.map(|_| "..".to_string()).collect();
            parts.extend(b.map(|c| c.as_os_str().to_string_lossy().into_owned()));
            parts.join("/")
        }
    }
}

fn walk(pkg_dir: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if name.starts_with('.') || name == "target" || p.join("Cargo.toml").exists() {
                continue;
            }
            walk(pkg_dir, &p, out);
        } else if name.ends_with(".rs")
            && let Ok(r) = p.strip_prefix(pkg_dir)
        {
            out.push(r.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rel_paths() {
        assert_eq!(rel_path(Path::new("/a/b"), Path::new("/a/b/c/d")), "c/d");
        assert_eq!(rel_path(Path::new("/a/b"), Path::new("/a/x")), "../x");
    }

    #[test]
    fn git_pins() {
        let d = Dep {
            name: "x".into(),
            rename: None,
            kind: DepKind::Normal,
            target: None,
            source: Some("git+https://github.com/a/b?tag=v1".into()),
            path: None,
            optional: false,
            default_features: true,
            member: false,
        };
        assert_eq!(d.git_url(), Some("https://github.com/a/b"));
        assert_eq!(d.git_pins(), vec![("tag".to_string(), "v1".to_string())]);
    }
}
