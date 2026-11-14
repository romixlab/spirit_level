//! The firmware model: everything the embedded lints look at, read once per run for each firmware crate of a
//! project.
//!
//! - the crate's `Cargo.toml` and the workspace root's (from `sl_model`), the `.cargo/config.toml` files cargo
//!   would read (the crate directory and its parents up to the project root, nearest first), `build.rs` with the
//!   `cargo:` directives it prints, `memory.x`, `bedrock_fw.json` (chip, framework, template answers);
//! - the Rust source of the crate and of its path dependencies inside the project (`src/` only), and the facts drawn
//!   from it ([`Facts`]: tasks, executors, spawns, interrupt handlers and bindings, statics, priorities, a call graph
//!   by name);
//! - the `MEMORY` regions of `memory.x` ([`memory_x`]);
//! - the linked ELF, when one is given ([`elf`]): sections, LOAD segments, demangled symbols, and for Arm Thumb code
//!   each function's frame and calls ([`thumb`]).
//!
//! What could not be read is a [`Note`], not an error: the lints that need it find nothing there, and the
//! `firmware-readable` lint reports the note.
//!
//! ```no_run
//! let project = sl_model::Project::load(std::path::Path::new(".")).unwrap();
//! for fw in sl_firmware::Firmware::load_all(&project, &[]) {
//!     println!("{}: {} tasks", fw.name, fw.facts.tasks.len());
//! }
//! ```

pub mod build_script;
pub mod context;
pub mod elf;
pub mod memory_x;
pub mod panics;
pub mod scan;
mod source;
#[cfg(feature = "synth")]
pub mod synth;
pub mod thumb;
pub mod toml;

pub use build_script::{BuildScript, Directive};
pub use elf::Elf;
pub use memory_x::Region;
pub use source::{
    Binding, Call, Entry, ExecutorStart, Facts, Function, Handler, HandlerKind, Loc, PeripheralUse,
    PrioritySet, SourceFile, Spawn, Static, StaticUse, Task,
};
pub use source::{has_await, is_upper_name, tokens, type_name};
pub use toml::TomlExt;

use sl_model::toml_edit::Item;
use sl_model::{FileKind, Package, Project, TargetKind, TomlFile};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Something the loader could not read or understand
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The file, relative to the project root
    pub file: String,
    /// What is wrong with it
    pub message: String,
}

/// A text file of the firmware
#[derive(Clone, Debug)]
pub struct TextFile {
    /// Relative to the project root (`fw/memory.x`, `.cargo/config.toml`)
    pub rel: String,
    /// Absolute
    pub path: PathBuf,
    /// The text
    pub text: String,
}

impl TextFile {
    /// The 1-based line of the first line that `pred` accepts
    pub fn line_where(&self, pred: impl Fn(&str) -> bool) -> Option<u32> {
        self.text.lines().position(pred).map(|i| i as u32 + 1)
    }
}

/// `bedrock_fw.json`: what the `embedded_bedrock` template generated the firmware with
#[derive(Clone, Debug)]
pub struct BedrockFw {
    /// The file
    pub file: TextFile,
    /// `answers.chip` (`STM32G0B1RE`, `rp2350b`)
    pub chip: Option<Chip>,
    /// `answers.framework` (`embassy`, `hal`, `bare`)
    pub framework: Option<String>,
    /// `template.version`
    pub template_version: Option<String>,
    /// The whole document
    pub json: serde_json::Value,
}

impl BedrockFw {
    /// A template answer (`no_flip_link`, `log`)
    pub fn answer(&self, key: &str) -> Option<&serde_json::Value> {
        self.json.get("answers")?.get(key)
    }
}

/// The chip, as `bedrock_fw.json` or the HAL's chip feature names it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    /// As written (`STM32G0B1RE`)
    pub name: String,
    /// Lower-case family for chip tables (`stm32g0`, `stm32h7`, `rp2040`, `rp235x`, `nrf52`, `esp32c3`)
    pub family: String,
}

impl Chip {
    /// The chip of a name, with its family
    pub fn new(name: &str) -> Self {
        let l = name.to_ascii_lowercase();
        let family = if let Some(rest) = l.strip_prefix("stm32") {
            format!("stm32{}", rest.get(..2).unwrap_or(rest))
        } else if l.starts_with("rp235") {
            "rp235x".to_owned()
        } else if let Some(rest) = l.strip_prefix("nrf") {
            format!("nrf{}", rest.get(..2).unwrap_or(rest))
        } else {
            l.clone()
        };
        Self {
            name: name.to_owned(),
            family,
        }
    }
}

/// One firmware crate as the lints see it
#[derive(Debug)]
pub struct Firmware<'p> {
    /// The project it is part of
    pub project: &'p Project,
    /// Its package
    pub package: &'p Package,
    /// The crate directory, absolute
    pub dir: PathBuf,
    /// The project root, absolute: where discovery of configuration files stops
    pub root: PathBuf,
    /// The package name
    pub name: String,
    /// The crate's `Cargo.toml`
    pub manifest: &'p TomlFile,
    /// The workspace root's `Cargo.toml`, when the crate is a member below it
    pub workspace: Option<&'p TomlFile>,
    /// The `.cargo/config.toml` files cargo reads for a build in the crate directory, nearest first
    pub cargo_configs: Vec<TomlFile>,
    /// `build.rs` (or `package.build`)
    pub build_script: Option<BuildScript>,
    /// `memory.x` in the crate directory
    pub memory_x: Option<TextFile>,
    /// The `MEMORY` regions of memory.x
    pub memory: Vec<Region>,
    /// `bedrock_fw.json`, the crate's own or the nearest above it in the project
    pub bedrock_fw: Option<BedrockFw>,
    /// The parsed Rust files: the crate's `src/` and its path dependencies in the project
    pub sources: Vec<SourceFile>,
    /// What the sources say
    pub facts: Facts,
    /// The linked ELF, when given and readable
    pub elf: Option<Elf>,
    /// What could not be read
    pub notes: Vec<Note>,
    /// The task class of each module (`[embedded.classes]`), set by the caller; empty when none is declared
    pub classes: sl_core::Classes,
}

impl Firmware<'_> {
    /// The task class of a source file, by its path relative to the project root
    pub fn class_of(&self, rel: &str) -> Option<sl_core::TaskClass> {
        let f = self.sources.iter().find(|f| f.rel == rel)?;
        self.classes.class_of(&f.in_package)
    }
}

impl<'p> Firmware<'p> {
    /// Every firmware crate of the project, each with the ELF of `elfs` that is its own: the one named after one of
    /// its binaries, or the only ELF when the project has one firmware crate
    pub fn load_all(project: &'p Project, elfs: &[PathBuf]) -> Vec<Self> {
        let crates: Vec<&Package> = project
            .packages
            .iter()
            .filter(|p| is_firmware(project, p))
            .collect();
        crates
            .iter()
            .map(|pkg| {
                let named = elfs.iter().find(|e| {
                    let stem = e.file_stem().map(|s| s.to_string_lossy().to_string());
                    pkg.targets
                        .iter()
                        .any(|(k, n)| *k == TargetKind::Bin && stem.as_deref() == Some(n.as_str()))
                });
                let elf =
                    named.or_else(|| (crates.len() == 1 && elfs.len() == 1).then(|| &elfs[0]));
                Firmware::load(project, pkg, elf.map(PathBuf::as_path))
            })
            .collect()
    }

    /// Reads one firmware crate of the project
    pub fn load(project: &'p Project, package: &'p Package, elf: Option<&Path>) -> Self {
        let root = absolute(&project.root);
        let dir = root.join(&package.dir);
        let mut notes = Vec::new();
        let workspace = (!package.dir.is_empty()).then_some(&project.root_manifest);
        let cargo_configs = cargo_configs(&root, &dir, &mut notes);

        let build_script = {
            let file = package
                .manifest
                .get(&["package", "build"])
                .and_then(Item::as_str)
                .unwrap_or("build.rs");
            read_text(&root, &dir.join(file)).map(|f| BuildScript::parse(f, &mut notes))
        };
        let memory_x = read_text(&root, &dir.join("memory.x"));
        // the crate's own, else the nearest above it (the template keeps it at the top of its layout)
        let bedrock_fw = ancestors_within(&dir, &root)
            .find_map(|d| read_text(&root, &d.join("bedrock_fw.json")))
            .and_then(
                |file| match serde_json::from_str::<serde_json::Value>(&file.text) {
                    Ok(json) => {
                        let s = |a: &str, b: &str| json.get(a)?.get(b)?.as_str().map(str::to_owned);
                        Some(BedrockFw {
                            chip: s("answers", "chip").map(|c| Chip::new(&c)),
                            framework: s("answers", "framework"),
                            template_version: s("template", "version"),
                            file,
                            json,
                        })
                    }
                    Err(e) => {
                        notes.push(Note {
                            file: file.rel.clone(),
                            message: format!("not valid JSON: {e}"),
                        });
                        None
                    }
                },
            );

        let sources = sources(project, package, &root, &mut notes);
        let facts = Facts::extract(&sources);

        let memory = memory_x
            .as_ref()
            .map(|m| memory_x::regions(&m.text))
            .unwrap_or_default();
        if let Some(m) = &memory_x
            && memory.is_empty()
        {
            notes.push(Note {
                file: m.rel.clone(),
                message: "no MEMORY regions could be read".into(),
            });
        }

        let elf = elf.and_then(|p| match Elf::load(&absolute(p), &root) {
            Ok(e) => Some(e),
            Err(message) => {
                notes.push(Note {
                    file: p.display().to_string(),
                    message,
                });
                None
            }
        });

        Firmware {
            project,
            package,
            dir,
            root,
            name: package.name.clone(),
            manifest: &package.manifest,
            workspace,
            cargo_configs,
            build_script,
            memory_x,
            memory,
            bedrock_fw,
            sources,
            facts,
            elf,
            notes,
            classes: sl_core::Classes::default(),
        }
    }

    /// The chip: `bedrock_fw.json`'s, else the one the HAL dependency's chip feature names
    pub fn chip(&self) -> Option<Chip> {
        if let Some(c) = self.bedrock_fw.as_ref().and_then(|b| b.chip.clone()) {
            return Some(c);
        }
        self.dependencies().iter().find_map(|d| {
            let chip_feature = match d.package() {
                "embassy-stm32" | "stm32-metapac" => |f: &str| f.starts_with("stm32"),
                "embassy-rp" | "rp-pac" => {
                    |f: &str| f.starts_with("rp2040") || f.starts_with("rp235")
                }
                "embassy-nrf" | "nrf-pac" => {
                    |f: &str| f.starts_with("nrf5") || f.starts_with("nrf9")
                }
                _ => return None,
            };
            d.features()
                .into_iter()
                .find(|(f, _, _)| chip_feature(f))
                .map(|(f, _, _)| Chip::new(&f))
        })
    }

    /// A key of the cargo configuration as cargo resolves it: the nearest file that sets it, with the value
    pub fn cargo_config(&self, path: &[&str]) -> Option<(&TomlFile, &Item)> {
        self.cargo_configs
            .iter()
            .find_map(|c| c.lookup(path).map(|v| (c, v)))
    }

    /// The target triple of `[build] target` (the first one when it is a list)
    pub fn build_target(&self) -> Option<(&TomlFile, String)> {
        let (file, v) = self.cargo_config(&["build", "target"])?;
        let t = match v.as_value()? {
            sl_model::toml_edit::Value::String(s) => s.value().clone(),
            sl_model::toml_edit::Value::Array(a) => a.get(0)?.as_str()?.to_owned(),
            _ => return None,
        };
        Some((file, t))
    }

    /// The source file by its path relative to the project root
    pub fn source(&self, rel: &str) -> Option<&SourceFile> {
        self.sources.iter().find(|s| s.rel == rel)
    }

    /// A path in the crate directory, relative to the project root (`fw/memory.x`)
    pub fn path(&self, rel: &str) -> String {
        self.package.path(rel)
    }

    /// Every dependency table of the crate's manifest (`dependencies`, `target.<cfg>.dependencies`), with the
    /// workspace's `[workspace.dependencies]` entry folded in for `workspace = true`
    pub fn dependencies(&self) -> Vec<Dependency<'_>> {
        let m = self.manifest;
        let kinds = ["dependencies", "build-dependencies", "dev-dependencies"];
        let mut tables: Vec<(Vec<String>, &dyn sl_model::toml_edit::TableLike)> = Vec::new();
        for kind in kinds {
            if let Some(t) = m.root().get(kind).and_then(Item::as_table_like) {
                tables.push((vec![kind.to_owned()], t));
            }
        }
        if let Some(targets) = m.root().get("target").and_then(Item::as_table_like) {
            for (cfg, tv) in targets.iter() {
                for kind in kinds {
                    if let Some(t) = tv.get(kind).and_then(Item::as_table_like) {
                        tables.push((vec!["target".into(), cfg.to_owned(), kind.into()], t));
                    }
                }
            }
        }
        let ws = self.workspace.unwrap_or(self.manifest);
        let ws_deps = ws
            .get(&["workspace", "dependencies"])
            .and_then(Item::as_table_like);
        let mut out = Vec::new();
        for (table, t) in tables {
            for (name, value) in t.iter() {
                let inherited = value
                    .get("workspace")
                    .and_then(Item::as_bool)
                    .unwrap_or(false)
                    .then(|| ws_deps.and_then(|t| t.get(name)))
                    .flatten()
                    .map(|v| (ws, v));
                out.push(Dependency {
                    file: m,
                    table: table.clone(),
                    name: name.to_owned(),
                    value,
                    inherited,
                });
            }
        }
        out
    }
}

/// A dependency of the crate as written, with its workspace entry when it inherits one
#[derive(Clone, Debug)]
pub struct Dependency<'a> {
    /// The manifest it is declared in
    pub file: &'a TomlFile,
    /// `["dependencies"]`, `["target", "cfg(...)", "dependencies"]`
    pub table: Vec<String>,
    /// The key (`cortex-m-rt`)
    pub name: String,
    /// The value as written
    pub value: &'a Item,
    /// `[workspace.dependencies]` entry and its file, for `workspace = true`
    pub inherited: Option<(&'a TomlFile, &'a Item)>,
}

impl Dependency<'_> {
    /// The crate's real name (`package = "..."` renames it)
    pub fn package(&self) -> &str {
        self.value
            .get("package")
            .or_else(|| self.inherited.and_then(|(_, v)| v.get("package")))
            .and_then(Item::as_str)
            .unwrap_or(&self.name)
    }

    /// The features it turns on: its own and the workspace entry's, each with the file and line that names it
    pub fn features(&self) -> Vec<(String, &TomlFile, u32)> {
        let mut sources: Vec<(&TomlFile, Vec<String>, &Item)> =
            vec![(self.file, self.table.clone(), self.value)];
        if let Some((ws, v)) = self.inherited {
            sources.push((ws, vec!["workspace".into(), "dependencies".into()], v));
        }
        let mut out = Vec::new();
        for (file, base, v) in sources {
            let Some(a) = v.get("features").and_then(Item::as_array) else {
                continue;
            };
            let mut path: Vec<&str> = base.iter().map(String::as_str).collect();
            path.push(&self.name);
            let mut line = file.line_of(&path);
            if line == 0 {
                path.push("features");
                line = file.line_of(&path);
            }
            for f in a.iter().filter_map(|f| f.as_str()) {
                out.push((f.to_owned(), file, line));
            }
        }
        out
    }

    /// The line of its key in its manifest
    pub fn line(&self) -> u32 {
        let mut path: Vec<&str> = self.table.iter().map(String::as_str).collect();
        path.push(&self.name);
        self.file.line_of(&path)
    }
}

/// Whether a member is a firmware crate: what `[package.metadata.spirit-level] firmware` says when it is set, else
/// it builds a binary, and its binary's root is `#![no_std]`, or it has a `memory.x` or a `bedrock_fw.json`, or
/// cargo builds it for a bare-metal target (`-none-` in `[build] target` or a `[target.<triple>]` section)
pub(crate) fn is_firmware(project: &Project, pkg: &Package) -> bool {
    if let Some(marked) = pkg
        .manifest
        .get(&["package", "metadata", "spirit-level", "firmware"])
        .and_then(Item::as_bool)
    {
        return marked;
    }
    if !pkg.has_bin() {
        return false;
    }
    let root = absolute(&project.root);
    let dir = root.join(&pkg.dir);
    if dir.join("memory.x").is_file()
        || ancestors_within(&dir, &root).any(|d| d.join("bedrock_fw.json").is_file())
    {
        return true;
    }
    let mut notes = Vec::new();
    let bare_metal = cargo_configs(&root, &dir, &mut notes).iter().any(|c| {
        c.targets().iter().any(|(t, _)| t.contains("-none"))
            || c.lookup(&["build", "target"])
                .is_some_and(|t| match t.as_value() {
                    Some(sl_model::toml_edit::Value::String(s)) => s.value().contains("-none"),
                    Some(sl_model::toml_edit::Value::Array(a)) => a
                        .iter()
                        .any(|t| t.as_str().is_some_and(|t| t.contains("-none"))),
                    _ => false,
                })
    });
    bare_metal
        || project.sources(pkg).iter().any(|f| {
            f.kind == FileKind::Src
                && f.is_crate_root()
                && f.in_package != "src/lib.rs"
                && f.ast().is_ok_and(|ast| {
                    ast.attrs.iter().any(|a| {
                        a.path().is_ident("no_std") && matches!(a.style, syn::AttrStyle::Inner(_))
                    })
                })
        })
}

/// The `.cargo/config.toml` (or `.cargo/config`) files of `dir` and its parents up to `root`, nearest first
fn cargo_configs(root: &Path, dir: &Path, notes: &mut Vec<Note>) -> Vec<TomlFile> {
    let mut out = Vec::new();
    for d in ancestors_within(dir, root) {
        for name in ["config.toml", "config"] {
            let p = d.join(".cargo").join(name);
            if p.is_file() {
                match read_text(root, &p).map(|f| TomlFile::parse(f.rel, f.text)) {
                    Some(Ok(t)) => out.push(t),
                    Some(Err(message)) => notes.push(Note {
                        file: rel_path(root, &p),
                        message: format!("not valid TOML: {message}"),
                    }),
                    None => {}
                }
                break;
            }
        }
    }
    out
}

/// The `src/` files of the crate and of its path dependencies inside the project, parsed
fn sources(
    project: &Project,
    package: &Package,
    root: &Path,
    notes: &mut Vec<Note>,
) -> Vec<SourceFile> {
    let mut out = Vec::new();
    let mut add = |files: &[sl_model::SourceFile], notes: &mut Vec<Note>| {
        for f in files.iter().filter(|f| f.kind == FileKind::Src) {
            match f.ast() {
                Ok(ast) => out.push(SourceFile {
                    rel: f.rel.clone(),
                    in_package: f.in_package.clone(),
                    krate: f.package.replace('-', "_"),
                    module: f.module.clone(),
                    text: f.text.clone(),
                    ast: ast.clone(),
                }),
                Err(e) => notes.push(Note {
                    file: f.rel.clone(),
                    message: format!("Rust syntax error at {e}; its facts are missing"),
                }),
            }
        }
    };
    add(project.sources(package), notes);
    if !root.join(&package.dir).join("src").is_dir() {
        notes.push(Note {
            file: package.path("src"),
            message: "no src/ directory: no source facts".into(),
        });
    }
    let mut seen = vec![package.dir.clone()];
    for d in &package.deps {
        let Some(path) = &d.path else { continue };
        if path.starts_with("..") || seen.contains(path) {
            continue;
        }
        seen.push(path.clone());
        if let Some(member) = project.packages.iter().find(|p| p.dir == *path) {
            add(project.sources(member), notes);
        } else {
            // a path dependency in the project that is not a member: its src/ read here
            let mut files = Vec::new();
            rust_files(&root.join(path).join("src"), &mut files);
            files.sort();
            let parsed: Vec<sl_model::SourceFile> = files
                .iter()
                .filter_map(|p| {
                    let rel = rel_path(root, p);
                    let in_package = rel
                        .strip_prefix(&format!("{path}/"))
                        .unwrap_or(&rel)
                        .to_owned();
                    sl_model::SourceFile::read(root, &rel, &d.name, &in_package).ok()
                })
                .collect();
            add(&parsed, notes);
        }
    }
    out
}

/// `dir` and its parents up to and including `root` (only `dir` when it is not under `root`)
fn ancestors_within<'a>(dir: &'a Path, root: &'a Path) -> impl Iterator<Item = &'a Path> {
    let under = dir.starts_with(root);
    dir.ancestors()
        .take_while(move |d| under && d.starts_with(root))
        .chain((!under).then_some(dir))
}

fn absolute(p: &Path) -> PathBuf {
    let p = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    // fold `..` and `.` without touching symlinks
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// `p` relative to `base`, with `..` where it lies outside
pub fn rel_path(base: &Path, p: &Path) -> String {
    let p = absolute(p);
    let base: Vec<_> = base.components().collect();
    let target: Vec<_> = p.components().collect();
    let common = base.iter().zip(&target).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> =
        std::iter::repeat_n("..".to_owned(), base.len() - common).collect();
    parts.extend(
        target[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

fn read_text(root: &Path, p: &Path) -> Option<TextFile> {
    let text = fs::read_to_string(p).ok()?;
    Some(TextFile {
        rel: rel_path(root, p),
        path: p.to_path_buf(),
        text,
    })
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths() {
        let d = Path::new("/r/fw/app");
        assert_eq!(
            rel_path(d, Path::new("/r/fw/app/src/main.rs")),
            "src/main.rs"
        );
        assert_eq!(
            rel_path(d, Path::new("/r/.cargo/config.toml")),
            "../../.cargo/config.toml"
        );
    }

    #[test]
    fn chip_families() {
        assert_eq!(Chip::new("STM32G0B1RE").family, "stm32g0");
        assert_eq!(Chip::new("rp2350b").family, "rp235x");
        assert_eq!(Chip::new("rp2040").family, "rp2040");
        assert_eq!(Chip::new("nRF52840").family, "nrf52");
    }
}
