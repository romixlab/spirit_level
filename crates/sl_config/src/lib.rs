//! The configuration of a run: `spirit_level.toml` and the tiers it extends, resolved into one [`Config`].
//!
//! ```toml
//! extends = "big"                     # a shipped tier, a list of them, or a path to a shared file
//! map-file = "crates.toml"            # or an inline [map] table; a table when another tool owns part of it:
//!                                     # [map-file] path = "crates.toml", foreign = ["meta", "crates.*.area"]
//!
//! [lints.crate-lines]                 # an override: level and/or parameters, always with a reason
//! level = "deny"
//! warn-above = 15000
//! reason = "this repo splits early"
//! expires = "2027-06-30"              # optional: a date or a release (`v0.5.0`); expired fails the run
//!
//! [clippy.lints]                      # the Clippy set expected in [workspace.lints.clippy] (see [`clippy`])
//! unwrap_used = { level = "warn", reason = "errors are returned or expected with a message" }
//!
//! [[crate-tiers]]                     # tiers that hold some crates beside the project's own
//! tiers = ["embedded"]
//! crates = ["foo_fw"]
//! reason = "the firmware"
//!
//! [[exceptions]]                      # scoped to crates and/or paths
//! lint = "crate-lines"
//! crates = ["foo_sdk_types"]
//! deny-above = 120000
//! reason = "generated protocol types"
//! expires = "v0.4.0"
//!
//! [[exclude]]                         # crates and/or paths no lint reports about (see [`exclude`])
//! crates = ["old_ui"]
//! reason = "frozen; deleted with the rewrite"
//! expires = "2027-03-31"              # required
//!
//! [dependents]                        # the repos that build on this one (see [`dependents`])
//! scan = ["~/git"]
//!
//! [docs]                              # the pages: Markdown, or a built site (see [`docs`])
//! reference = ["docs/lints/**"]
//!
//! [cli]                               # how the CLI lints build and run the tools (see [`cli`])
//! timeout-secs = 20
//!
//! [board]                             # board descriptions and their chip data (see [`board`])
//! chip-data = "boards/chips"
//!
//! [embedded.classes]                  # task classes of firmware modules (see [`embedded`])
//! service = ["src/led.rs"]
//! ```

pub mod board;
pub mod cli;
pub mod clippy;
pub mod dependents;
pub mod docs;
pub mod embedded;
mod error;
pub mod exclude;
pub mod map;
mod reader;

pub use board::Board;
pub use cli::CliProbe;
pub use clippy::{ClippyConfig, ClippyLint, ClippySet};
pub use dependents::Dependents;
pub use docs::{Book, Docs, Generator, Site};
pub use embedded::Embedded;
pub use error::ConfigError;
pub use exclude::Exclude;
pub use map::{External, Layer, Map, MapCrate};

use reader::Reader;
use sl_core::{Date, Expiry, Level, LintMeta, ParamKind, ParamValue, Params, Version, glob};
use sl_model::TomlFile;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use toml_edit::Value;

/// The file looked for at the project root
pub const FILE: &str = "spirit_level.toml";

/// Where a setting was written
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Origin {
    /// `spirit_level.toml`, `tier:big`, or a shared file's path
    pub file: String,
    /// 1-based line
    pub line: u32,
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

/// A lint as configured
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct LintSetting {
    /// Its level
    pub level: Level,
    /// Every declared parameter, defaults filled in
    pub params: Params,
    /// Why it is at this level (the last layer's reason); empty for a lint no layer mentions
    pub reason: String,
    /// Where the level was set; None for a lint no layer mentions (allowed)
    pub origin: Option<Origin>,
    /// The expiry of the override that set it
    pub expires: Option<Expiry>,
}

/// A scoped exception: another level or other parameters for some crates or paths
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Exception {
    /// The lint
    pub lint: String,
    /// Crates it covers
    pub crates: Vec<String>,
    /// Path patterns it covers (relative to the project root; `*`, `**`, `?`)
    pub paths: Vec<String>,
    /// Why
    pub reason: String,
    /// When it stops holding
    pub expires: Option<Expiry>,
    /// The level inside its scope, when it changes it
    pub level: Option<Level>,
    /// Parameters it overrides inside its scope
    pub params: BTreeMap<String, ParamValue>,
    /// Where it is written
    pub origin: Origin,
}

impl Exception {
    /// Whether it covers a finding about `krate` at `path`
    pub fn covers(&self, krate: Option<&str>, path: Option<&str>) -> bool {
        krate.is_some_and(|k| self.crates.iter().any(|c| c == k))
            || path.is_some_and(|p| self.paths.iter().any(|g| glob::matches(g, p)))
    }
}

/// Crates held to tiers beside the project's own: their lints resolved as if the project extended those tiers too,
/// after its own and before its `[lints]`
#[derive(Clone, Debug, serde::Serialize)]
pub struct CrateTiers {
    /// The tiers, in the order they apply
    pub tiers: Vec<String>,
    /// The crates
    pub crates: Vec<String>,
    /// Every lint as configured for these crates
    #[serde(skip)]
    pub lints: BTreeMap<String, LintSetting>,
}

/// The resolved configuration
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Config {
    /// The project file, when there is one
    pub file: Option<String>,
    /// What it extends, in order
    pub extends: Vec<String>,
    /// Every registered lint
    pub lints: BTreeMap<String, LintSetting>,
    /// Scoped exceptions in file order
    pub exceptions: Vec<Exception>,
    /// Crates and paths left out of the check, in file order
    pub excludes: Vec<Exclude>,
    /// Crates held to more tiers than the project's own, one entry per distinct list of tiers
    pub crate_tiers: Vec<CrateTiers>,
    /// The crate map
    #[serde(skip)]
    pub map: Option<Map>,
    /// The Clippy set the project is expected to turn on
    pub clippy: ClippySet,
    /// Where the repos that build on this one are; None: `spl dependents` is off
    pub dependents: Option<Dependents>,
    /// The Markdown pages: the `[docs]` table, or its defaults
    pub docs: Docs,
    /// How the CLI lints build and run the tools
    pub cli: CliProbe,
    /// The board descriptions and their chip data: the `[board]` table, or its defaults
    pub board: Board,
    /// The `[embedded]` table: task classes of firmware modules
    pub embedded: Embedded,
}

/// What a resolution needs to know besides the files
pub struct Env<'a> {
    /// The registered lints
    pub lints: &'a [&'static LintMeta],
    /// The text of a shipped tier by name
    pub tier: &'a dyn Fn(&str) -> Option<&'static str>,
    /// Today, for expiries
    pub today: Date,
    /// The project's version, for release expiries
    pub version: Option<Version>,
}

impl Config {
    /// Reads `spirit_level.toml` at `root` (or `file` when given) and everything it extends
    pub fn load(root: &Path, file: Option<&Path>, env: &Env<'_>) -> Result<Self, Vec<ConfigError>> {
        Self::load_extending(root, file, None, env)
    }

    /// Reads `spirit_level.toml` as [`Config::load`] does, with its own `extends` replaced by `extends` when
    /// given: its overrides, exceptions and tables stay (`spl check --at big`)
    pub fn load_extending(
        root: &Path,
        file: Option<&Path>,
        extends: Option<&[String]>,
        env: &Env<'_>,
    ) -> Result<Self, Vec<ConfigError>> {
        let path = file.map_or_else(|| root.join(FILE), Path::to_path_buf);
        let rel = path
            .strip_prefix(root)
            .map_or_else(|_| path.display().to_string(), |p| p.display().to_string());
        let text = std::fs::read_to_string(&path).map_err(|e| {
            vec![ConfigError::new(
                &rel,
                0,
                format!("cannot read it ({e}); `spl init --tier small` writes one"),
            )]
        })?;
        let source = Source::File(path, rel.clone());
        let mut r = Resolver::new(env, root);
        r.project_extends = extends.map(<[String]>::to_vec);
        r.layer(&source, text.clone(), true);
        let entries = std::mem::take(&mut r.tier_entries);
        let mut c = r.finish(Some(rel))?;
        c.crate_tiers = crate_tiers(
            c.map.as_ref(),
            &entries,
            env,
            root,
            (&source, &text),
            extends,
        )?;
        Ok(c)
    }

    /// The configuration of shipped tiers alone, without a project file (`spl check --tier big`)
    pub fn from_tiers(tiers: &[&str], env: &Env<'_>) -> Result<Self, Vec<ConfigError>> {
        let mut r = Resolver::new(env, Path::new("."));
        r.record_at = 0;
        for t in tiers {
            r.extend(t, &ConfigError::new("--tier", 0, ""), Path::new("."));
        }
        r.finish(None)
    }

    /// The setting of a lint
    ///
    /// # Panics
    ///
    /// For an unregistered id: a bug in the analysis asking for it.
    // spirit-level: allow(panic-in-library) every registered lint has a setting; an analysis asking for another id is a bug its fixtures catch
    pub fn lint(&self, id: &str) -> &LintSetting {
        self.lints
            .get(id)
            .unwrap_or_else(|| panic!("lint {id} is not registered"))
    }

    /// The setting of a lint for one crate: the crate's tiers' when it has more than the project's own
    pub fn lint_for(&self, id: &str, krate: Option<&str>) -> &LintSetting {
        krate
            .and_then(|k| {
                self.crate_tiers
                    .iter()
                    .find(|t| t.crates.iter().any(|c| c == k))
            })
            .and_then(|t| t.lints.get(id))
            .unwrap_or_else(|| self.lint(id))
    }

    /// The highest level a lint has anywhere: the project's, a crate's tiers', an exception's
    pub fn highest_level(&self, id: &str) -> Level {
        let tiers = self
            .crate_tiers
            .iter()
            .filter_map(|t| t.lints.get(id).map(|l| l.level));
        let exceptions = self
            .exceptions
            .iter()
            .filter(|e| e.lint == id)
            .filter_map(|e| e.level);
        tiers
            .chain(exceptions)
            .fold(self.lint(id).level, Level::max)
    }

    /// Whether a lint runs at all: its level, a crate's tiers', or an exception's is above allow
    pub fn enabled(&self, id: &str) -> bool {
        self.highest_level(id) > Level::Allow
    }

    /// The level of a lint for a finding about `krate` at `path`, and the exception that set it
    pub fn level(
        &self,
        id: &str,
        krate: Option<&str>,
        path: Option<&str>,
    ) -> (Level, Option<&Exception>) {
        let base = self.lint_for(id, krate).level;
        self.exceptions
            .iter()
            .rev()
            .find(|e| e.lint == id && e.level.is_some() && e.covers(krate, path))
            .map_or((base, None), |e| (e.level.unwrap_or(base), Some(e)))
    }

    /// The parameters of a lint for one crate: the lint's (of the crate's tiers), with those of exceptions naming
    /// the crate on top
    pub fn params_for(&self, id: &str, krate: &str) -> Params {
        let mut p = self.lint_for(id, Some(krate)).params.clone();
        for e in &self.exceptions {
            if e.lint == id && e.crates.iter().any(|c| c == krate) {
                for (k, v) in &e.params {
                    p.0.insert(k.clone(), v.clone());
                }
            }
        }
        p
    }

    /// The exclusion that leaves out a finding about `krate` at `path`, if one does
    pub fn excluded(&self, krate: Option<&str>, path: Option<&str>) -> Option<&Exclude> {
        self.excludes.iter().find(|e| e.covers(krate, path))
    }

    /// Whether an exclusion leaves out the crate `name`
    pub fn excludes_crate(&self, name: &str) -> bool {
        self.excludes
            .iter()
            .any(|e| e.crates.iter().any(|c| c == name))
    }

    /// The exceptions of a lint that change its parameters
    pub fn param_exceptions(&self, id: &str) -> impl Iterator<Item = &Exception> {
        self.exceptions
            .iter()
            .filter(move |e| e.lint == id && !e.params.is_empty())
    }
}

enum Source {
    File(PathBuf, String),
    Tier(String),
}

/// A `[[crate-tiers]]` entry: its tiers (shipped names, or absolute paths) and crates, and where it is written
struct TierEntry {
    tiers: Vec<String>,
    crates: Vec<String>,
    at: ConfigError,
}

/// Whether an extended name is a path to a file rather than a shipped tier
fn is_path(name: &str) -> bool {
    name.contains('/')
        || Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
}

/// Resolves the project file once more for each distinct list of tiers some crates are held to
fn crate_tiers(
    map: Option<&Map>,
    entries: &[TierEntry],
    env: &Env<'_>,
    root: &Path,
    (source, text): (&Source, &str),
    extends: Option<&[String]>,
) -> Result<Vec<CrateTiers>, Vec<ConfigError>> {
    let mut errs = Vec::new();
    let mut per: BTreeMap<String, (Vec<String>, ConfigError)> = BTreeMap::new();
    let mut add = |krate: &str, tiers: &[String], at: &ConfigError| {
        let slot = per
            .entry(krate.to_owned())
            .or_insert_with(|| (Vec::new(), at.clone()));
        for t in tiers {
            if !slot.0.contains(t) {
                slot.0.push(t.clone());
            }
        }
    };
    if let Some(m) = map {
        for (name, c) in m.crates.iter().filter(|(_, c)| !c.tiers.is_empty()) {
            let at = ConfigError::new(&m.file, c.line, "");
            for t in c
                .tiers
                .iter()
                .filter(|t| !is_path(t) && (env.tier)(t).is_none())
            {
                errs.push(ConfigError::new(&at.file, at.line, unknown_tier(t)));
            }
            add(name, &c.tiers, &at);
        }
    }
    for e in entries {
        for k in &e.crates {
            add(k, &e.tiers, &e.at);
        }
    }
    let mut groups: BTreeMap<Vec<String>, (Vec<String>, ConfigError)> = BTreeMap::new();
    for (krate, (tiers, at)) in per {
        groups
            .entry(tiers)
            .or_insert_with(|| (Vec::new(), at))
            .0
            .push(krate);
    }
    let mut out = Vec::new();
    for (tiers, (crates, at)) in groups {
        let mut r = Resolver::new(env, root);
        r.extra = tiers.iter().map(|t| (t.clone(), at.clone())).collect();
        r.project_extends = extends.map(<[String]>::to_vec);
        r.layer(source, text.to_owned(), true);
        for e in r.errs.drain(..) {
            if !errs.contains(&e) {
                errs.push(e);
            }
        }
        out.push(CrateTiers {
            tiers,
            crates,
            lints: r.lints,
        });
    }
    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

fn unknown_tier(name: &str) -> String {
    format!(
        "extends {name:?}, which is not a shipped tier (`spl tiers` lists them) nor a path to a .toml file"
    )
}

struct Resolver<'a> {
    env: &'a Env<'a>,
    root: PathBuf,
    lints: BTreeMap<String, LintSetting>,
    exceptions: Vec<Exception>,
    excludes: Vec<Exclude>,
    tier_entries: Vec<TierEntry>,
    /// Tiers applied after the project file's own extends: a crate's tiers
    extra: Vec<(String, ConfigError)>,
    map: Option<Map>,
    clippy: ClippySet,
    dependents: Option<Dependents>,
    docs: Docs,
    cli: CliProbe,
    board: Board,
    embedded: Embedded,
    extends: Vec<String>,
    /// What the project file extends instead of its own `extends`, when set
    project_extends: Option<Vec<String>>,
    stack: Vec<String>,
    /// The depth at which an extended name is a top-level one (1 under a project file, 0 for bare tiers)
    record_at: usize,
    errs: Vec<ConfigError>,
}

impl<'a> Resolver<'a> {
    fn new(env: &'a Env<'a>, root: &Path) -> Self {
        let lints = env
            .lints
            .iter()
            .map(|m| {
                (
                    m.id.to_owned(),
                    LintSetting {
                        level: Level::Allow,
                        params: default_params(m),
                        reason: String::new(),
                        origin: None,
                        expires: None,
                    },
                )
            })
            .collect();
        Resolver {
            env,
            root: root.to_path_buf(),
            lints,
            exceptions: Vec::new(),
            excludes: Vec::new(),
            tier_entries: Vec::new(),
            extra: Vec::new(),
            map: None,
            clippy: ClippySet::default(),
            dependents: None,
            docs: Docs::default(),
            cli: CliProbe::default(),
            board: Board::default(),
            embedded: Embedded::default(),
            extends: Vec::new(),
            project_extends: None,
            stack: Vec::new(),
            record_at: 1,
            errs: Vec::new(),
        }
    }

    fn finish(self, file: Option<String>) -> Result<Config, Vec<ConfigError>> {
        if !self.errs.is_empty() {
            return Err(self.errs);
        }
        Ok(Config {
            file,
            extends: self.extends,
            lints: self.lints,
            exceptions: self.exceptions,
            excludes: self.excludes,
            crate_tiers: Vec::new(),
            map: self.map,
            clippy: self.clippy,
            dependents: self.dependents,
            docs: self.docs,
            cli: self.cli,
            board: self.board,
            embedded: self.embedded,
        })
    }

    /// The lint `name` names: its id or its code
    fn meta(&self, name: &str) -> Option<&'static LintMeta> {
        self.env.lints.iter().find(|m| m.is(name)).copied()
    }

    /// Applies one extended name: a shipped tier, or a path relative to the extending file's directory
    fn extend(&mut self, name: &str, at: &ConfigError, dir: &Path) {
        let is_path = is_path(name);
        let key = if is_path {
            dir.join(name).display().to_string()
        } else {
            format!("tier:{name}")
        };
        if self.stack.contains(&key) {
            self.errs.push(ConfigError::new(
                &at.file,
                at.line,
                format!("{name} extends itself through {}", self.stack.join(" -> ")),
            ));
            return;
        }
        if self.stack.len() == self.record_at {
            self.extends.push(name.to_owned());
        }
        if is_path {
            let path = dir.join(name);
            let rel = path
                .strip_prefix(&self.root)
                .map_or_else(|_| path.display().to_string(), |p| p.display().to_string());
            match std::fs::read_to_string(&path) {
                Ok(text) => self.layer(&Source::File(path, rel), text, false),
                Err(e) => self.errs.push(ConfigError::new(
                    &at.file,
                    at.line,
                    format!("extends {name}, which cannot be read: {e}"),
                )),
            }
        } else {
            match (self.env.tier)(name) {
                Some(text) => self.layer(&Source::Tier(name.to_owned()), text.to_owned(), false),
                None => self
                    .errs
                    .push(ConfigError::new(&at.file, at.line, unknown_tier(name))),
            }
        }
    }

    fn layer(&mut self, source: &Source, text: String, project: bool) {
        let (key, rel, dir, tier) = match source {
            Source::File(p, rel) => (
                p.display().to_string(),
                rel.clone(),
                p.parent()
                    .map_or_else(|| self.root.clone(), Path::to_path_buf),
                false,
            ),
            Source::Tier(n) => (
                format!("tier:{n}"),
                format!("tier:{n}"),
                self.root.clone(),
                true,
            ),
        };
        let file = match TomlFile::parse(rel.clone(), text) {
            Ok(f) => f,
            Err(e) => {
                self.errs.push(ConfigError::new(&rel, 0, e));
                return;
            }
        };
        self.stack.push(key);
        let mut errs = Vec::new();
        let mut r = Reader::new(&file, Vec::new(), file.root());

        let mut extends = r.opt_list("extends", true, &mut errs).unwrap_or_default();
        if project && let Some(e) = &self.project_extends {
            extends.clone_from(e);
        }
        let at = r.err("extends", "");
        for name in &extends {
            self.extend(name, &at, &dir);
        }
        if project {
            for (name, at) in std::mem::take(&mut self.extra) {
                self.extend(&name, &at, &dir);
            }
        }
        if tier && let Some(mut t) = r.table("tier", &mut errs) {
            t.opt_str("description", &mut errs);
            t.finish(&mut errs);
        }
        if let Some(lints) = r.table("lints", &mut errs) {
            self.lints_table(lints, project || !tier, &mut errs);
        }
        if let Some(c) = r.table("clippy", &mut errs) {
            self.clippy.read(c, &mut errs);
        }
        if !tier {
            self.project_tables(&mut r, &dir, &mut errs);
        }
        r.finish(&mut errs);
        self.errs.extend(errs);
        self.stack.pop();
    }

    /// The tables only a project file or a shared file sets, never a tier
    fn project_tables(&mut self, r: &mut Reader<'_>, dir: &Path, errs: &mut Vec<ConfigError>) {
        for t in tables(r, "exceptions", errs) {
            self.exception(t, errs);
        }
        for t in tables(r, "exclude", errs) {
            self.exclude(t, errs);
        }
        for t in tables(r, "crate-tiers", errs) {
            self.tier_entry(t, dir, errs);
        }
        if let Some(m) = r.table("map", errs) {
            self.map = Some(Map::read(m, errs));
        }
        if let Some(d) = r.table("dependents", errs) {
            self.dependents = Some(Dependents::read(d, dir, errs));
        }
        if let Some(d) = r.table("docs", errs) {
            self.docs = Docs::read(d, dir, errs);
        }
        if let Some(c) = r.table("cli", errs) {
            self.cli = CliProbe::read(c, errs);
        }
        if let Some(b) = r.table("board", errs) {
            self.board = Board::read(b, errs);
        }
        if let Some(e) = r.table("embedded", errs) {
            self.embedded = Embedded::read(e, errs);
        }
        if r.table.get("map-file").is_some() {
            self.map_file(r, dir, errs);
        }
    }

    /// `map-file = "crates.toml"`, or a table with the path and the keys the map's owner checks
    fn map_file(&mut self, r: &mut Reader<'_>, dir: &Path, errs: &mut Vec<ConfigError>) {
        let (mf, foreign, at) = if r.table.get("map-file").is_some_and(|i| i.is_str()) {
            (
                r.opt_str("map-file", errs),
                Vec::new(),
                r.err("map-file", ""),
            )
        } else if let Some(mut t) = r.table("map-file", errs) {
            let mf = t.str("path", errs);
            let foreign = t.opt_list("foreign", false, errs).unwrap_or_default();
            let at = t.err("foreign", "");
            t.finish(errs);
            (mf, foreign, at)
        } else {
            return;
        };
        let Some(mf) = mf else { return };
        if self.map.is_some() {
            errs.push(r.err(
                "map-file",
                "give the map either inline ([map]) or as map-file, not both",
            ));
            return;
        }
        let foreign = match reader::Foreign::new(&foreign) {
            Ok(f) => f,
            Err(e) => {
                errs.push(ConfigError::new(&at.file, at.line, e));
                return;
            }
        };
        let path = dir.join(&mf);
        let mrel = path
            .strip_prefix(&self.root)
            .map_or_else(|_| path.display().to_string(), |p| p.display().to_string());
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| TomlFile::parse(mrel, t))
        {
            Ok(mfile) => {
                let mr = Reader::new(&mfile, Vec::new(), mfile.root()).with_foreign(&foreign);
                self.map = Some(Map::read(mr, errs));
            }
            Err(e) => errs.push(r.err("map-file", format!("map-file {mf}: {e}"))),
        }
        for p in foreign.clashes() {
            errs.push(ConfigError::new(
                &at.file,
                at.line,
                format!(
                    "map-file.foreign lists {p}, which spl reads itself: remove it from the list"
                ),
            ));
        }
    }

    fn lints_table(&mut self, mut lints: Reader<'_>, expiring: bool, errs: &mut Vec<ConfigError>) {
        for key in lints.keys() {
            let Some(meta) = self.meta(&key) else {
                lints.touch(&key);
                errs.push(lints.err(&key, unknown_lint(&key, self.env.lints)));
                continue;
            };
            let id = meta.id;
            let Some(mut t) = lints.table(&key, errs) else {
                continue;
            };
            let line = lints.line(&key);
            let origin = Origin {
                file: t.file.rel.clone(),
                line,
            };
            let level = level_of(&mut t, errs);
            let reason = t.opt_str("reason", errs).unwrap_or_default();
            if reason.trim().is_empty() {
                errs.push(t.err(
                    "",
                    format!("lints.{id} needs a reason: say why it is set this way"),
                ));
            }
            let expires = self.expiry(&mut t, expiring, &format!("the override of {id}"), errs);
            let params = params_of(&mut t, meta, errs);
            t.finish(errs);
            let Some(s) = self.lints.get_mut(id) else {
                continue;
            };
            if level.is_none() && s.origin.is_none() {
                errs.push(ConfigError::new(
                    &origin.file,
                    origin.line,
                    format!("lints.{id} sets no level and no tier it extends enables {id}: add level = \"warn\" or \"deny\""),
                ));
            }
            if let Some(l) = level {
                s.level = l;
            }
            s.params.0.extend(params);
            s.reason = reason;
            s.origin = Some(origin);
            s.expires = expires;
        }
    }

    /// A `[[crate-tiers]]` entry: tiers, crates, reason
    fn tier_entry(&mut self, mut t: Reader<'_>, dir: &Path, errs: &mut Vec<ConfigError>) {
        let at = ConfigError::new(&t.file.rel, t.line("").max(t.line("tiers")), "");
        let tiers = t.opt_list("tiers", true, errs).unwrap_or_default();
        let crates = t.opt_list("crates", true, errs).unwrap_or_default();
        let reason = t.opt_str("reason", errs).unwrap_or_default();
        t.finish(errs);
        if tiers.is_empty() || crates.is_empty() {
            errs.push(ConfigError::new(
                &at.file,
                at.line,
                "a crate-tiers entry names its tiers and its crates",
            ));
        }
        if reason.trim().is_empty() {
            errs.push(ConfigError::new(
                &at.file,
                at.line,
                "a crate-tiers entry needs a reason: say why these crates are held to these tiers",
            ));
        }
        let mut resolved = Vec::new();
        for name in tiers {
            if is_path(&name) {
                let path = dir.join(&name);
                let rel = path.strip_prefix(&self.root).unwrap_or(&path);
                resolved.push(rel.display().to_string());
            } else if (self.env.tier)(&name).is_some() {
                resolved.push(name);
            } else {
                errs.push(ConfigError::new(&at.file, at.line, unknown_tier(&name)));
            }
        }
        self.tier_entries.push(TierEntry {
            tiers: resolved,
            crates,
            at,
        });
    }

    fn exception(&mut self, mut t: Reader<'_>, errs: &mut Vec<ConfigError>) {
        let line = t.line("").max(t.line("lint"));
        let origin = Origin {
            file: t.file.rel.clone(),
            line,
        };
        let Some(lint) = t.str("lint", errs) else {
            for k in t.keys() {
                t.touch(&k);
            }
            return;
        };
        let Some(meta) = self.meta(&lint) else {
            errs.push(t.err("lint", unknown_lint(&lint, self.env.lints)));
            for k in t.keys() {
                t.touch(&k);
            }
            return;
        };
        let lint = meta.id.to_owned();
        let crates = t.opt_list("crates", true, errs).unwrap_or_default();
        let paths = t.opt_list("paths", true, errs).unwrap_or_default();
        if crates.is_empty() && paths.is_empty() {
            errs.push(t.err(
                "lint",
                format!("the exception for {lint} names no crates and no paths"),
            ));
        }
        let level = level_of(&mut t, errs);
        let reason = t.opt_str("reason", errs).unwrap_or_default();
        if reason.trim().is_empty() {
            errs.push(t.err("lint", format!("the exception for {lint} needs a reason")));
        }
        let expires = self.expiry(&mut t, true, &format!("the exception for {lint}"), errs);
        let params = params_of(&mut t, meta, errs);
        if level.is_none() && params.is_empty() {
            errs.push(t.err(
                "lint",
                format!("the exception for {lint} changes neither the level nor a parameter"),
            ));
        }
        t.finish(errs);
        self.exceptions.push(Exception {
            lint,
            crates,
            paths,
            reason,
            expires,
            level,
            params,
            origin,
        });
    }

    /// An `[[exclude]]` entry: crates and/or paths, reason, expiry
    fn exclude(&mut self, mut t: Reader<'_>, errs: &mut Vec<ConfigError>) {
        let line = t.line("").max(t.line("crates")).max(t.line("paths"));
        let origin = Origin {
            file: t.file.rel.clone(),
            line,
        };
        let crates = t.opt_list("crates", true, errs).unwrap_or_default();
        let paths = t.opt_list("paths", true, errs).unwrap_or_default();
        let reason = t.opt_str("reason", errs).unwrap_or_default();
        let expires = self.expiry(&mut t, true, "the exclusion", errs);
        let has_expires = t.table.get("expires").is_some();
        t.finish(errs);
        let at = |msg: &str| ConfigError::new(&origin.file, origin.line, msg);
        if crates.is_empty() && paths.is_empty() {
            errs.push(at("an exclude entry names its crates and/or paths"));
        }
        if reason.trim().is_empty() {
            errs.push(at(
                "an exclude entry needs a reason: say why this code is not checked",
            ));
        }
        if !has_expires {
            errs.push(at(
                "an exclude entry needs expires: the date or release (\"2027-03-31\", \"v0.6.0\") by which the code is gone or checked again",
            ));
        }
        if let Some(expires) = expires {
            self.excludes.push(Exclude {
                crates,
                paths,
                reason,
                expires,
                origin,
            });
        }
    }

    fn expiry(
        &self,
        t: &mut Reader<'_>,
        check: bool,
        what: &str,
        errs: &mut Vec<ConfigError>,
    ) -> Option<Expiry> {
        let s = t.opt_str("expires", errs)?;
        let Some(e) = Expiry::parse(&s) else {
            errs.push(t.err(
                "expires",
                format!("expires = {s:?} is neither a date (2027-01-31) nor a release (v0.5.0)"),
            ));
            return None;
        };
        if check {
            if let Expiry::Version(_) = e
                && self.env.version.is_none()
            {
                errs.push(t.err(
                    "expires",
                    format!("{what} expires at a release, but the project has no version ([workspace.package] or [package] version)"),
                ));
            }
            if e.expired(self.env.today, self.env.version) {
                errs.push(t.err(
                    "expires",
                    format!("{what} expired ({e}): fix what it allowed and remove it, or renew it with a new reason and expiry"),
                ));
            }
        }
        Some(e)
    }
}

/// A reader for each table of the array of tables `key` (`[[exceptions]]`)
fn tables<'a>(r: &mut Reader<'a>, key: &str, errs: &mut Vec<ConfigError>) -> Vec<Reader<'a>> {
    let file = r.file;
    let Some(item) = r.item(key) else {
        return Vec::new();
    };
    if let Some(arr) = item.as_array_of_tables() {
        arr.iter()
            .enumerate()
            .map(|(i, t)| Reader::new(file, vec![format!("{key}[{i}]")], t))
            .collect()
    } else {
        errs.push(r.err(key, format!("{key} must be an array of tables ([[{key}]])")));
        Vec::new()
    }
}

fn level_of(t: &mut Reader<'_>, errs: &mut Vec<ConfigError>) -> Option<Level> {
    let s = t.opt_str("level", errs)?;
    if let Some(l) = Level::from_name(&s) {
        Some(l)
    } else {
        errs.push(t.err(
            "level",
            format!("level = {s:?}: use \"allow\", \"warn\" or \"deny\""),
        ));
        None
    }
}

fn params_of(
    t: &mut Reader<'_>,
    meta: &LintMeta,
    errs: &mut Vec<ConfigError>,
) -> BTreeMap<String, ParamValue> {
    let mut out = BTreeMap::new();
    for k in t.keys() {
        if sl_core::meta::RESERVED_KEYS.contains(&k.as_str()) {
            continue;
        }
        let Some(spec) = meta.param(&k) else {
            t.touch(&k);
            let known: Vec<&str> = meta.params.iter().map(|p| p.name).collect();
            errs.push(t.err(
                &k,
                if known.is_empty() {
                    format!("{} takes no parameters, so {k} is unknown", meta.id)
                } else {
                    format!(
                        "{} has no parameter {k} (it has: {})",
                        meta.id,
                        known.join(", ")
                    )
                },
            ));
            continue;
        };
        let item = t.item(&k);
        let v = item.and_then(|i| i.as_value()).and_then(value_of);
        match v {
            Some(v) if v.fits(spec.kind) => {
                let v = match (v, spec.kind) {
                    (ParamValue::Int(i), ParamKind::Float) => ParamValue::Float(i as f64),
                    (v, _) => v,
                };
                out.insert(k, v);
            }
            _ => errs.push(t.err(
                &k,
                format!("{}.{k} must be {}", meta.id, spec.kind.expected()),
            )),
        }
    }
    out
}

fn value_of(v: &Value) -> Option<ParamValue> {
    Some(match v {
        Value::Integer(i) => ParamValue::Int(*i.value()),
        Value::Float(f) => ParamValue::Float(*f.value()),
        Value::Boolean(b) => ParamValue::Bool(*b.value()),
        Value::String(s) => ParamValue::Str(s.value().clone()),
        Value::Array(a) => ParamValue::StrList(
            a.iter()
                .map(|x| x.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()?,
        ),
        _ => return None,
    })
}

/// The declared defaults of a lint's parameters
///
/// # Panics
///
/// On a default that is not a valid value of its kind: a declaration bug the registry test catches.
pub fn default_params(meta: &LintMeta) -> Params {
    let mut p = Params::default();
    for spec in meta.params {
        let doc = format!("v = {}", spec.default);
        let parsed = doc
            .parse::<toml_edit::DocumentMut>()
            .ok()
            .and_then(|d| d.get("v").and_then(|i| i.as_value()).and_then(value_of));
        match parsed {
            Some(v) if v.fits(spec.kind) => {
                let v = match (v, spec.kind) {
                    (ParamValue::Int(i), ParamKind::Float) => ParamValue::Float(i as f64),
                    (v, _) => v,
                };
                p.0.insert(spec.name.to_owned(), v);
            }
            // spirit-level: allow(panic-in-library) a default that does not fit its kind is a declaration bug; the registry test resolves every lint's defaults
            _ => panic!(
                "lint {}: the default of {} ({}) is not {}",
                meta.id,
                spec.name,
                spec.default,
                spec.kind.expected()
            ),
        }
    }
    p
}

fn unknown_lint(id: &str, lints: &[&'static LintMeta]) -> String {
    let mut near: Vec<&str> = lints
        .iter()
        .map(|m| m.id)
        .filter(|m| distance(m, id) <= 3 || m.contains(id) || id.contains(m))
        .collect();
    near.truncate(3);
    if near.is_empty() {
        format!("unknown lint {id} (`spl lints` lists them)")
    } else {
        format!("unknown lint {id}; did you mean {}?", near.join(", "))
    }
}

/// The Levenshtein edit distance between two strings, for a "did you mean" hint on an unknown key or lint id
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push(
                (prev[j] + usize::from(ca != *cb))
                    .min(prev[j + 1] + 1)
                    .min(cur[j] + 1),
            );
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests;
