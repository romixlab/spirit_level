//! The workspace as cargo sees it, read through `cargo metadata`, plus the manifests' text.

use crate::graph::Resolve;
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
    /// A nested workspace's directory is not the root of a workspace of its own
    NotOwnWorkspace {
        /// The directory, relative to the outer project's root
        dir: String,
        /// The workspace cargo found it in instead, relative to the outer project's root
        found: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metadata(e) => write!(f, "cargo metadata failed: {e}"),
            Self::Manifest(e) => write!(f, "manifest: {e}"),
            Self::NotOwnWorkspace { dir, found } if found.is_empty() => write!(
                f,
                "{dir} is not a workspace root: cargo reads it as part of the workspace at the project root"
            ),
            Self::NotOwnWorkspace { dir, found } => write!(
                f,
                "{dir} is not a workspace root: cargo reads it as part of the workspace in {found}/"
            ),
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
            Self::Normal => "dependencies",
            Self::Dev => "dev-dependencies",
            Self::Build => "build-dependencies",
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

/// A git URL as `host/path`, lower case: `https://`, `ssh://`, `git@host:`, a trailing `/` and `.git` dropped, so
/// two spellings of one repo compare equal
pub fn git_url_key(url: &str) -> String {
    let u = url.trim_end_matches('/').trim_end_matches(".git");
    let u = u.strip_prefix("ssh://").unwrap_or(u);
    let u = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))
        .unwrap_or(u);
    let u = u.strip_prefix("git@").unwrap_or(u);
    u.replacen(':', "/", 1).to_ascii_lowercase()
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
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    /// Whether its git URL is under `prefix` (`github.com/org`, a host), compared as [`git_url_key`]s
    pub fn git_under(&self, prefix: &str) -> bool {
        let (Some(url), prefix) = (self.git_url(), git_url_key(prefix)) else {
            return false;
        };
        git_url_key(url)
            .strip_prefix(&prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/') || prefix.ends_with('/'))
    }

    /// Whether it comes from a registry (crates.io or another)
    pub fn is_registry(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|s| s.starts_with("registry+") || s.starts_with("sparse+"))
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
    /// An integration test: one test binary
    Test,
    /// A bench, example or build script
    Other,
}

/// A workspace member
#[derive(Debug)]
pub struct Package {
    /// The package name
    pub name: String,
    /// Its version
    pub version: String,
    /// Its directory relative to the project root (`crates/foo`, or empty for a root package)
    pub dir: String,
    /// The manifest
    pub manifest: TomlFile,
    /// `description`
    pub description: Option<String>,
    /// `license`
    pub license: Option<String>,
    /// `repository`
    pub repository: Option<String>,
    /// `homepage`
    pub homepage: Option<String>,
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
    /// Its targets' root files, relative to its directory (`src/lib.rs`, `examples/demo.rs`)
    pub target_roots: Vec<String>,
    /// Its features and what each turns on
    pub features: BTreeMap<String, Vec<String>>,
    /// `links`
    pub links: Option<String>,
    sources: OnceLock<Vec<SourceFile>>,
    test_modules: OnceLock<Vec<String>>,
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
        self.targets
            .iter()
            .any(|(k, _)| *k == TargetKind::ProcMacro)
    }

    /// Whether it builds a binary
    pub fn has_bin(&self) -> bool {
        self.targets.iter().any(|(k, _)| *k == TargetKind::Bin)
    }

    /// A path in its directory, relative to the project root
    pub fn path(&self, rel: &str) -> String {
        if self.dir.is_empty() {
            rel.to_owned()
        } else {
            format!("{}/{rel}", self.dir)
        }
    }
}

/// Where cargo-hakari looks for its configuration, in order
pub const HAKARI_CONFIG_PATHS: &[&str] = &[".config/hakari.toml", ".guppy/hakari.toml"];

/// The project: a workspace (or one crate) as cargo sees it
///
/// Every relative path in it (a package's `dir`, a manifest's `rel`, a source file's path) is relative to `root`.
/// A nested workspace ([`Project::nested`]) keeps the outer project's root, so its paths name the same files a
/// finding of the outer project would.
#[derive(Debug)]
pub struct Project {
    /// The root every relative path starts from: the checked workspace's root, absolute
    pub root: PathBuf,
    /// The workspace's own directory relative to `root`: empty for the checked workspace, `tools` for a nested
    /// workspace in `tools/`
    pub workspace: String,
    /// The root manifest
    pub root_manifest: TomlFile,
    /// Workspace members, ordered by directory
    pub packages: Vec<Package>,
    /// The project's version: `[workspace.package] version`, else the root package's
    pub version: Option<Version>,
    resolve: OnceLock<Result<Resolve, String>>,
    tracked: OnceLock<Result<Vec<String>, String>>,
    commits: crate::git::Cache,
}

impl Project {
    /// Reads the project whose manifest is in `dir` or above (cargo's own search)
    pub fn load(dir: &Path) -> Result<Self, LoadError> {
        let meta = metadata(dir)?;
        let root: PathBuf = meta.workspace_root.clone().into();
        Self::from_metadata(&meta, root)
    }

    /// Reads the workspace whose root is `dir`, relative to this project's root (a tools workspace beside the
    /// main one); its paths stay relative to this project's root
    pub fn nested(&self, dir: &str) -> Result<Self, LoadError> {
        let abs = self.root.join(dir);
        if !abs.join("Cargo.toml").is_file() {
            return Err(LoadError::Manifest(format!(
                "{dir}/Cargo.toml: no such file"
            )));
        }
        let meta = metadata(&abs)?;
        let found: PathBuf = meta.workspace_root.clone().into();
        let same = match (abs.canonicalize(), found.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        };
        if !same {
            return Err(LoadError::NotOwnWorkspace {
                dir: dir.to_owned(),
                found: rel_path(&self.root, &found),
            });
        }
        Self::from_metadata(&meta, self.root.clone())
    }

    fn from_metadata(meta: &Metadata, root: PathBuf) -> Result<Self, LoadError> {
        let ws: PathBuf = meta.workspace_root.clone().into();
        let workspace = rel_path(&root, &ws);
        let read = |abs: &Path| -> Result<TomlFile, LoadError> {
            let rel = rel_path(&root, abs);
            let text = std::fs::read_to_string(abs)
                .map_err(|e| LoadError::Manifest(format!("{rel}: {e}")))?;
            TomlFile::parse(rel, text).map_err(LoadError::Manifest)
        };
        let root_manifest = read(&ws.join("Cargo.toml"))?;
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
                    } else if t.kind.contains(&CmTargetKind::Test) {
                        TargetKind::Test
                    } else {
                        TargetKind::Other
                    };
                    (k, t.name.clone())
                })
                .collect();
            let pkg_dir = root.join(&dir);
            let target_roots = p
                .targets
                .iter()
                .map(|t| rel_path(&pkg_dir, t.src_path.as_std_path()))
                .collect();
            packages.push(Package {
                name: p.name.to_string(),
                version: p.version.to_string(),
                dir,
                manifest,
                description: p.description.clone(),
                license: p.license.clone(),
                repository: p.repository.clone(),
                homepage: p.homepage.clone(),
                edition: p.edition.to_string(),
                rust_version: p.rust_version.as_ref().map(|v| v.to_string()),
                publish: p.publish.as_ref().is_none_or(|r| !r.is_empty()),
                deps,
                targets,
                target_roots,
                features: p.features.clone(),
                links: p.links.clone(),
                sources: OnceLock::new(),
                test_modules: OnceLock::new(),
            });
        }
        packages.sort_by(|a, b| a.dir.cmp(&b.dir));
        let version = root_manifest
            .get(&["workspace", "package", "version"])
            .or_else(|| root_manifest.get(&["package", "version"]))
            .and_then(|v| v.as_str())
            .and_then(Version::parse);
        Ok(Self {
            root,
            workspace,
            root_manifest,
            packages,
            version,
            resolve: OnceLock::new(),
            tracked: OnceLock::new(),
            commits: crate::git::Cache::default(),
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

    /// Whether a file is test code: under tests/, benches/ or examples/, or a module declared with
    /// `#[cfg(test)] mod name;` (and everything below it)
    pub fn is_test_file(&self, pkg: &Package, f: &SourceFile) -> bool {
        if f.kind.is_test_like() {
            return true;
        }
        let tm = pkg.test_modules.get_or_init(|| {
            let mut out = Vec::new();
            for f in self.sources(pkg) {
                if let Ok(ast) = f.ast() {
                    let dir = f.in_package.rsplit_once('/').map_or("", |(d, _)| d);
                    crate::source::cfg_test_mods(&ast.items, &f.module, dir, &mut out);
                }
            }
            out
        });
        tm.iter().any(|m| match m.strip_prefix("file:") {
            Some(path) => f.in_package == path,
            None => f.module == *m || f.module.starts_with(&format!("{m}::")),
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
            .get_or_init(|| self.load_resolve(&[]))
            .as_ref()
            .map_err(String::as_str)
    }

    /// The resolved graph of the dependencies built for the target triples `targets` (`--filter-platform`); every
    /// call runs cargo, so a caller keeps the result. No targets is the whole graph, as [`Self::resolve`] has it
    ///
    /// # Errors
    /// cargo cannot run or resolve the workspace, or knows no such target
    pub fn resolve_for(&self, targets: &[String]) -> Result<Resolve, String> {
        self.load_resolve(targets)
    }

    fn load_resolve(&self, targets: &[String]) -> Result<Resolve, String> {
        let ws = self.root.join(&self.workspace);
        let mut cmd = cargo(&ws);
        cmd.args(["metadata", "--format-version", "1"]);
        for t in targets {
            cmd.args(["--filter-platform", t]);
        }
        if ws.join("Cargo.lock").exists() {
            cmd.arg("--locked");
        }
        let out = cmd.output().map_err(|e| format!("cannot run cargo: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
        }
        let meta = cargo_metadata::MetadataCommand::parse(String::from_utf8_lossy(&out.stdout))
            .map_err(|e| e.to_string())?;
        let platforms = targets
            .iter()
            .map(|t| Ok((t.clone(), target_cfg(&ws, t)?)))
            .collect::<Result<Vec<_>, String>>()?;
        Resolve::from_metadata(&meta, &platforms)
    }

    /// A file's text relative to the root, if it exists
    pub fn read(&self, rel: &str) -> Option<String> {
        std::fs::read_to_string(self.root.join(rel)).ok()
    }

    /// Whether a file exists relative to the root
    pub fn exists(&self, rel: &str) -> bool {
        self.root.join(rel).exists()
    }

    /// The graph and duplicate lists of `deny.toml`; empty without the file
    ///
    /// # Errors
    /// deny.toml is not TOML, or an entry is not one cargo-deny takes
    pub fn deny_config(&self) -> Result<crate::DenyConfig, String> {
        self.read("deny.toml").map_or_else(
            || Ok(crate::DenyConfig::default()),
            |t| crate::DenyConfig::parse(&t),
        )
    }

    /// The workspace-hack package hakari's configuration names (`hakari-package` in the first of
    /// [`HAKARI_CONFIG_PATHS`] that exists); None without one, or when it does not parse
    pub fn workspace_hack(&self) -> Option<String> {
        let ws = self.root.join(&self.workspace);
        let path = HAKARI_CONFIG_PATHS.iter().find(|c| ws.join(c).exists())?;
        let doc: toml_edit::DocumentMut =
            std::fs::read_to_string(ws.join(path)).ok()?.parse().ok()?;
        doc.get("hakari-package")?.as_str().map(str::to_owned)
    }

    /// The files git tracks under the root, relative to it (`git ls-files`), read once; an error when the
    /// project is not in a git checkout or git cannot run
    pub fn tracked(&self) -> Result<&[String], &str> {
        self.tracked
            .get_or_init(|| {
                let out = command("git", &self.root)
                    .args(["ls-files", "-z"])
                    .output()
                    .map_err(|e| format!("cannot run git: {e}"))?;
                if !out.status.success() {
                    return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
                }
                Ok(String::from_utf8_lossy(&out.stdout)
                    .split('\0')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect())
            })
            .as_deref()
            .map_err(String::as_str)
    }

    /// The commits in `base..HEAD` (merges left out, oldest first), read once per base; an error when `base` is
    /// not a commit or git cannot run
    pub fn commits(&self, base: &str) -> Result<std::sync::Arc<[crate::git::Commit]>, String> {
        self.commits.get(&self.root, base)
    }

    /// Every commit list this run has read so far, for the waivers in their messages
    pub fn commits_read(&self) -> Vec<std::sync::Arc<[crate::git::Commit]>> {
        self.commits.read_so_far()
    }
}

/// `cargo metadata --no-deps` for the workspace whose manifest is in `dir` or above
fn metadata(dir: &Path) -> Result<Metadata, LoadError> {
    let out = cargo(dir)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| LoadError::Metadata(format!("cannot run cargo: {e}")))?;
    if !out.status.success() {
        return Err(LoadError::Metadata(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    cargo_metadata::MetadataCommand::parse(String::from_utf8_lossy(&out.stdout))
        .map_err(|e| LoadError::Metadata(e.to_string()))
}

/// `cargo` with the environment of an outer cargo removed, so a run under `cargo test` or a build script sees
/// the project as a person would (`CARGO_HOME` and registry settings stay)
fn cargo(dir: &Path) -> Command {
    command(
        std::env::var_os("SPIRIT_LEVEL_CARGO").unwrap_or_else(|| "cargo".into()),
        dir,
    )
}

/// The cfg values of the target triple `target` (`rustc --print cfg`), run in `dir` so its toolchain answers
fn target_cfg(
    dir: &Path,
    target: &str,
) -> Result<Vec<cargo_metadata::cargo_platform::Cfg>, String> {
    let out = command("rustc", dir)
        .args(["--print", "cfg", "--target", target])
        .output()
        .map_err(|e| format!("cannot run rustc: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            l.parse()
                .map_err(|e| format!("rustc --print cfg --target {target}: {e}"))
        })
        .collect()
}

/// A program started in `dir` with the environment of an outer cargo removed.
///
/// Removed are `CARGO_*` but `CARGO_HOME` and the registry and network settings, `RUSTC*`, `RUSTFLAGS`, and the
/// `RUSTUP_TOOLCHAIN` rustup sets for the outer cargo. Every tool `spirit_level` runs sees the project, and the
/// toolchain it pins, as a person would, also under `cargo test` or `cargo run`.
pub fn command(program: impl AsRef<std::ffi::OsStr>, dir: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir);
    for (k, _) in std::env::vars_os() {
        let Some(k) = k.to_str() else { continue };
        let keep = k == "CARGO_HOME"
            || k.starts_with("CARGO_REGISTRIES_")
            || k.starts_with("CARGO_NET_")
            || k.starts_with("CARGO_HTTP_");
        if (k.starts_with("CARGO_") && !keep)
            || k == "CARGO"
            || k.starts_with("RUSTC")
            || k == "RUSTFLAGS"
            || k == "RUSTUP_TOOLCHAIN"
        {
            cmd.env_remove(k);
        }
    }
    cmd
}

fn rel_path(root: &Path, p: &Path) -> String {
    if let Ok(r) = p.strip_prefix(root) {
        r.to_string_lossy().replace('\\', "/")
    } else {
        // outside the root: express it with `..` where the two share a prefix
        let (mut a, mut b) = (root.components().peekable(), p.components().peekable());
        while let (Some(x), Some(y)) = (a.peek(), b.peek()) {
            if x != y {
                break;
            }
            a.next();
            b.next();
        }
        let mut parts: Vec<String> = a.map(|_| "..".to_owned()).collect();
        parts.extend(b.map(|c| c.as_os_str().to_string_lossy().into_owned()));
        parts.join("/")
    }
}

fn walk(pkg_dir: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
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
        assert_eq!(d.git_pins(), vec![("tag".to_owned(), "v1".to_owned())]);
        assert!(d.git_under("github.com/a"));
        assert!(d.git_under("git@github.com:A/"));
        assert!(!d.git_under("github.com/ab"));
        assert_eq!(git_url_key("git@github.com:a/b.git"), "github.com/a/b");
    }

    #[test]
    #[expect(
        clippy::let_underscore_must_use,
        reason = "a test removes its scratch directory as far as it can"
    )]
    fn nested_workspaces() {
        let dir = std::env::temp_dir().join(format!("sl_model_nested_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let krate = |d: &Path, name: &str| {
            std::fs::create_dir_all(d.join("src")).unwrap();
            std::fs::write(d.join("src/lib.rs"), "").unwrap();
            std::fs::write(
                d.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
            )
            .unwrap();
        };
        krate(&dir.join("a"), "a");
        krate(&dir.join("tools/t"), "t");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace]\nmembers = [\"a\"]\nexclude = [\"tools\"]\nresolver = \"3\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("tools/Cargo.toml"),
            "[workspace]\nmembers = [\"t\"]\nresolver = \"3\"\n",
        )
        .unwrap();
        let p = Project::load(&dir).unwrap();
        assert_eq!(p.workspace, "");
        let t = p.nested("tools").unwrap();
        assert_eq!(t.workspace, "tools");
        assert_eq!(t.root, p.root);
        assert_eq!(t.packages[0].dir, "tools/t");
        assert_eq!(t.packages[0].manifest.rel, "tools/t/Cargo.toml");
        assert_eq!(t.root_manifest.rel, "tools/Cargo.toml");
        let e = p.nested("a").unwrap_err().to_string();
        assert_eq!(
            e,
            "a is not a workspace root: cargo reads it as part of the workspace at the project root"
        );
        let e = p.nested("gone").unwrap_err().to_string();
        assert_eq!(e, "manifest: gone/Cargo.toml: no such file");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
