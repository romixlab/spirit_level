//! The crate map of a workspace: layers and what each may see, the layer and properties of every crate, closures
//! that must never be reached, external git dependencies and where heavy third-party crates may be used.
//!
//! ```toml
//! [map.layers.core]
//! desc = "Types only"
//! allow = ["core"]
//!
//! [map.layers.app]
//! allow = ["core", "app"]
//!
//! [map.crates]
//! foo_core = { layer = "core" }
//! foo_cli = { layer = "app", deps = ["foo_core"] }
//!
//! [map.forbid]
//! foo_cli = ["foo_gpu"]
//!
//! [map.external]
//! exact_units = { git = "https://github.com/romixlab/exact_units", branch = "main", role = "core" }
//!
//! [map.third-party]
//! wgpu = ["foo_gpu"]
//! ```

use crate::error::ConfigError;
use crate::reader::Reader;
use std::collections::BTreeMap;

/// A layer
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    /// Its name
    pub name: String,
    /// What lives in it
    pub desc: Option<String>,
    /// The layers its crates may depend on (its own only when listed)
    pub allow: Vec<String>,
}

/// One crate's entry
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapCrate {
    /// Its layer
    pub layer: String,
    /// Its description, when the map is where it is written
    pub desc: Option<String>,
    /// The workspace crates it may depend on; None: any its layer allows
    pub deps: Option<Vec<String>>,
    /// May allow `unsafe` code
    pub r#unsafe: bool,
    /// Generated code, checked in: exempt from size limits
    pub generated: bool,
    /// Published with a stable API: semver-checked
    pub stable: bool,
    /// Builds without std
    pub no_std: bool,
    /// Builds for wasm32-unknown-unknown
    pub wasm: bool,
    /// Promises bit-identical results on every platform: floats only through libm, no random-state hashing
    pub deterministic: bool,
    /// The line of its entry
    pub line: u32,
}

/// An external project's crate taken by git
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct External {
    /// The git URL
    pub git: String,
    /// The branch it follows (no rev, no tag)
    pub branch: Option<String>,
    /// The layer whose crates may use it (and the layers above that allow that layer)
    pub role: Option<String>,
    /// The line of its entry
    pub line: u32,
}

/// The crate map
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    /// The file it was read from
    pub file: String,
    /// The line of `[map]` (or 1 for a map file)
    pub line: u32,
    /// Layers in the order written (bottom up by convention)
    pub layers: Vec<Layer>,
    /// Crates by name
    pub crates: BTreeMap<String, MapCrate>,
    /// For a crate (a binary usually), the crates its closure must never reach
    pub forbid: BTreeMap<String, Vec<String>>,
    /// External git crates by package name
    pub external: BTreeMap<String, External>,
    /// Third-party crates that only the listed workspace crates may depend on directly
    pub third_party: BTreeMap<String, Vec<String>>,
}

impl Map {
    /// The layer called `name`
    pub fn layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// The layer of a crate
    pub fn layer_of(&self, krate: &str) -> Option<&Layer> {
        self.crates.get(krate).and_then(|c| self.layer(&c.layer))
    }

    /// Reads a map table
    pub(crate) fn read(mut r: Reader<'_>, errs: &mut Vec<ConfigError>) -> Map {
        let mut map = Map {
            file: r.file.rel.clone(),
            line: r.line("").max(1),
            ..Map::default()
        };
        if let Some(mut layers) = r.table("layers", errs) {
            for name in layers.keys() {
                let Some(mut l) = layers.table(&name, errs) else {
                    continue;
                };
                let desc = l.opt_str("desc", errs);
                let allow = l.opt_list("allow", false, errs).unwrap_or_default();
                l.finish(errs);
                map.layers.push(Layer { name, desc, allow });
            }
            layers.finish(errs);
        }
        if let Some(mut crates) = r.table("crates", errs) {
            for name in crates.keys() {
                let line = crates.line(&name);
                let Some(mut c) = crates.table(&name, errs) else {
                    continue;
                };
                let entry = MapCrate {
                    layer: c.str("layer", errs).unwrap_or_default(),
                    desc: c.opt_str("desc", errs),
                    deps: c.opt_list("deps", false, errs),
                    r#unsafe: c.opt_bool("unsafe", errs).unwrap_or(false),
                    generated: c.opt_bool("generated", errs).unwrap_or(false),
                    stable: c.opt_bool("stable", errs).unwrap_or(false),
                    no_std: c.opt_bool("no_std", errs).unwrap_or(false),
                    wasm: c.opt_bool("wasm", errs).unwrap_or(false),
                    deterministic: c.opt_bool("deterministic", errs).unwrap_or(false),
                    line,
                };
                c.finish(errs);
                map.crates.insert(name, entry);
            }
            crates.finish(errs);
        }
        for (key, field) in [("forbid", 0), ("third-party", 1)] {
            if let Some(mut t) = r.table(key, errs) {
                for name in t.keys() {
                    let v = t.opt_list(&name, false, errs).unwrap_or_default();
                    if field == 0 {
                        map.forbid.insert(name, v);
                    } else {
                        map.third_party.insert(name, v);
                    }
                }
                t.finish(errs);
            }
        }
        if let Some(mut ext) = r.table("external", errs) {
            for name in ext.keys() {
                let line = ext.line(&name);
                let Some(mut e) = ext.table(&name, errs) else {
                    continue;
                };
                let entry = External {
                    git: e.str("git", errs).unwrap_or_default(),
                    branch: e.opt_str("branch", errs),
                    role: e.opt_str("role", errs),
                    line,
                };
                e.finish(errs);
                map.external.insert(name, entry);
            }
            ext.finish(errs);
        }
        let file = r.file;
        r.finish(errs);
        map.check(file, errs);
        map
    }

    fn check(&self, file: &sl_model::TomlFile, errs: &mut Vec<ConfigError>) {
        let base: Vec<&str> = if file.get(&["map"]).is_some() {
            vec!["map"]
        } else {
            vec![]
        };
        let line = |path: &[&str]| {
            let mut p = base.clone();
            p.extend_from_slice(path);
            file.line(&p)
        };
        for l in &self.layers {
            for a in &l.allow {
                if self.layer(a).is_none() {
                    errs.push(ConfigError::new(
                        &self.file,
                        line(&["layers", &l.name, "allow"]),
                        format!(
                            "layer {} allows {a}, which is not a layer of the map",
                            l.name
                        ),
                    ));
                }
            }
        }
        for (name, c) in &self.crates {
            if !c.layer.is_empty() && self.layer(&c.layer).is_none() {
                errs.push(ConfigError::new(
                    &self.file,
                    c.line,
                    format!(
                        "crate {name} is in layer {}, which is not a layer of the map",
                        c.layer
                    ),
                ));
            }
        }
        for (name, e) in &self.external {
            if let Some(r) = &e.role
                && self.layer(r).is_none()
            {
                errs.push(ConfigError::new(
                    &self.file,
                    e.line,
                    format!("external crate {name} has role {r}, which is not a layer of the map"),
                ));
            }
        }
    }
}
