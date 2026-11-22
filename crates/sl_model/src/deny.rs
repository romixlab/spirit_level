//! What a project's `deny.toml` says about duplicate versions: the targets of `[graph]`, whether
//! `multiple-versions-include-dev` is on, and the `skip` and `skip-tree` lists of `[bans]`.
//!
//! cargo-deny and `spirit_level`'s `duplicate-versions` judge the same graph by the same lists, so a duplicate the
//! project accepts is named once, in deny.toml. cargo-deny's own default drops a crate reached only through
//! dev-dependency edges from that graph before it ever gets to `skip`/`skip-tree`.

use crate::graph::Resolve;
use crate::project::DepKind;
use cargo_metadata::semver::{Version, VersionReq};
use std::collections::BTreeSet;
use toml_edit::{DocumentMut, Item, Value};

/// A crate a `skip` or `skip-tree` entry names: `name`, `name@version` or `name:version` (a bare version is exact,
/// anything else a requirement), or a table with `crate` (or the older `name` and `version`)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageSpec {
    /// The crate's name
    pub name: String,
    /// The versions it covers; None covers every version
    pub version: Option<VersionReq>,
}

impl PackageSpec {
    /// Parses `name`, `name@version` or `name:version`
    ///
    /// # Errors
    /// A version that is neither a version nor a requirement
    pub fn parse(spec: &str) -> Result<Self, String> {
        let Some((name, v)) = spec.split_once(['@', ':']) else {
            return Ok(Self {
                name: spec.to_owned(),
                version: None,
            });
        };
        let req = if let Ok(exact) = Version::parse(v) {
            VersionReq::parse(&format!("={exact}"))
        } else {
            VersionReq::parse(v)
        }
        .map_err(|e| format!("`{spec}`: the version is not a version requirement: {e}"))?;
        Ok(Self {
            name: name.to_owned(),
            version: Some(req),
        })
    }

    /// Whether it covers the crate `name` in version `version` (a version that does not parse is covered only by
    /// an entry without one)
    #[must_use]
    pub fn matches(&self, name: &str, version: &str) -> bool {
        if self.name != name {
            return false;
        }
        match &self.version {
            None => true,
            Some(req) => Version::parse(version).is_ok_and(|v| req.matches(&v)),
        }
    }
}

/// The graph and the duplicate lists of deny.toml
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenyConfig {
    /// `[graph] targets`: the target triples the graph is built for; empty means every target
    pub targets: Vec<String>,
    /// `[bans] multiple-versions-include-dev`: whether a crate reached only through dev-dependency edges counts
    /// in the duplicate check. cargo-deny's own default is `false`
    pub multiple_versions_include_dev: bool,
    /// `skip`: versions left out of the duplicate check
    pub skip: Vec<PackageSpec>,
    /// `skip-tree`: crates left out with their dependencies, down to `depth` levels (None: all of them)
    pub skip_tree: Vec<(PackageSpec, Option<u32>)>,
}

impl DenyConfig {
    /// Reads deny.toml's text
    ///
    /// # Errors
    /// The text is not TOML, or an entry is not one cargo-deny takes
    pub fn parse(text: &str) -> Result<Self, String> {
        let doc: DocumentMut = text
            .parse()
            .map_err(|e| format!("deny.toml is not TOML: {e}"))?;
        let mut out = Self {
            targets: targets(doc.get("graph"))?,
            ..Self::default()
        };
        let Some(bans) = doc.get("bans") else {
            return Ok(out);
        };
        if let Some(v) = bans.get("multiple-versions-include-dev") {
            out.multiple_versions_include_dev = v.as_bool().ok_or_else(|| {
                "deny.toml: [bans] multiple-versions-include-dev is not a bool".to_owned()
            })?;
        }
        for item in entries(bans, "skip")? {
            out.skip.push(entry(item, "skip")?.0);
        }
        for item in entries(bans, "skip-tree")? {
            out.skip_tree.push(entry(item, "skip-tree")?);
        }
        Ok(out)
    }

    /// The packages of `res` the lists leave out of the duplicate check: each `skip` match, and each `skip-tree`
    /// match with the packages it reaches through normal and build edges down to its depth
    #[must_use]
    pub fn skipped(&self, res: &Resolve) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        for (i, rp) in res.packages.iter().enumerate() {
            if self.skip.iter().any(|s| s.matches(&rp.name, &rp.version)) {
                out.insert(i);
            }
            for (spec, depth) in &self.skip_tree {
                if spec.matches(&rp.name, &rp.version) {
                    tree(res, i, *depth, &mut out);
                }
            }
        }
        out
    }
}

/// Inserts `start` and what it reaches through normal and build edges, at most `depth` edges away
fn tree(res: &Resolve, start: usize, depth: Option<u32>, out: &mut BTreeSet<usize>) {
    let mut level = vec![start];
    let mut seen = BTreeSet::from([start]);
    let mut d = 0_u32;
    while !level.is_empty() && depth.is_none_or(|max| d <= max) {
        let mut next = Vec::new();
        for i in level {
            out.insert(i);
            for (j, kinds) in &res.packages[i].deps {
                if kinds.iter().any(|k| *k != DepKind::Dev) && seen.insert(*j) {
                    next.push(*j);
                }
            }
        }
        level = next;
        d += 1;
    }
}

/// The triples of `[graph] targets`: a string, or a table with `triple`
fn targets(graph: Option<&Item>) -> Result<Vec<String>, String> {
    let Some(t) = graph.and_then(|g| g.get("targets")) else {
        return Ok(Vec::new());
    };
    let bad = || "deny.toml: [graph] targets is not a list of triples".to_owned();
    t.as_array()
        .ok_or_else(bad)?
        .iter()
        .map(|v| {
            v.as_str()
                .or_else(|| v.as_inline_table()?.get("triple")?.as_str())
                .map(str::to_owned)
                .ok_or_else(bad)
        })
        .collect()
}

/// The array `key` of `[bans]`, empty when absent
fn entries<'a>(bans: &'a Item, key: &str) -> Result<Vec<&'a Value>, String> {
    match bans.get(key) {
        None => Ok(Vec::new()),
        Some(item) => item
            .as_array()
            .map(|a| a.iter().collect())
            .ok_or_else(|| format!("deny.toml: [bans] {key} is not an array")),
    }
}

/// One entry: a spec string, or a table with `crate` (or `name` and `version`) and, in `skip-tree`, `depth`
fn entry(v: &Value, key: &str) -> Result<(PackageSpec, Option<u32>), String> {
    if let Some(s) = v.as_str() {
        return Ok((PackageSpec::parse(s)?, None));
    }
    let bad = || format!("deny.toml: an entry of [bans] {key} names no crate: {v}");
    let t = v.as_inline_table().ok_or_else(bad)?;
    let str_of = |k: &str| t.get(k).and_then(Value::as_str);
    let spec = if let Some(c) = str_of("crate") {
        PackageSpec::parse(c)?
    } else {
        let name = str_of("name").ok_or_else(bad)?;
        let version = str_of("version")
            .map(VersionReq::parse)
            .transpose()
            .map_err(|e| format!("deny.toml: [bans] {key} {name}: {e}"))?;
        PackageSpec {
            name: name.to_owned(),
            version,
        }
    };
    let depth = match t.get("depth") {
        None => None,
        Some(d) => Some(
            d.as_integer()
                .and_then(|d| u32::try_from(d).ok())
                .ok_or_else(|| {
                    format!(
                        "deny.toml: [bans] {key} {}: depth is not a count",
                        spec.name
                    )
                })?,
        ),
    };
    Ok((spec, depth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ResolvedPackage;

    fn pkg(name: &str, version: &str, deps: &[usize]) -> ResolvedPackage {
        ResolvedPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            source: None,
            member: false,
            links: None,
            manifest_path: std::path::PathBuf::new(),
            deps: deps.iter().map(|&j| (j, vec![DepKind::Normal])).collect(),
            features: std::collections::BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    #[test]
    fn specs() {
        let exact = PackageSpec::parse("aes@0.8.4").unwrap();
        assert!(exact.matches("aes", "0.8.4"));
        assert!(!exact.matches("aes", "0.8.5"));
        let req = PackageSpec::parse("syn:<2").unwrap();
        assert!(req.matches("syn", "1.0.109"));
        assert!(!req.matches("syn", "2.0.1"));
        let any = PackageSpec::parse("windows-sys").unwrap();
        assert!(any.matches("windows-sys", "0.52.0"));
        assert!(PackageSpec::parse("x@not a version").is_err());
    }

    #[test]
    fn lists() {
        let b = DenyConfig::parse(
            r#"
[bans]
skip = ["itoa@1.0.1", { crate = "aes@0.8.4", reason = "r" }, { name = "syn", version = "<2" }]
skip-tree = [{ crate = "hakari", reason = "r" }, { crate = "geiger", depth = 1 }]
"#,
        )
        .unwrap();
        assert_eq!(b.skip.len(), 3);
        assert_eq!(b.skip[2].name, "syn");
        assert_eq!(b.skip_tree[1].1, Some(1));
        assert!(!b.multiple_versions_include_dev);
        assert_eq!(DenyConfig::parse("").unwrap(), DenyConfig::default());
        let dev = DenyConfig::parse("[bans]\nmultiple-versions-include-dev = true").unwrap();
        assert!(dev.multiple_versions_include_dev);
        assert!(DenyConfig::parse("[bans]\nmultiple-versions-include-dev = \"yes\"").is_err());
        let g = DenyConfig::parse(
            "[graph]\ntargets = [\"x86_64-unknown-linux-gnu\", { triple = \"wasm32-unknown-unknown\" }]",
        )
        .unwrap();
        assert_eq!(
            g.targets,
            ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
        );
        assert!(DenyConfig::parse("[graph]\ntargets = [1]").is_err());
        assert!(DenyConfig::parse("[bans]\nskip = 1").is_err());
        assert!(DenyConfig::parse("[bans]\nskip = [{ reason = \"r\" }]").is_err());
    }

    #[test]
    fn trees_to_their_depth() {
        // 0 geiger -> 1 syn 2 -> 2 unicode-ident; 3 hakari -> 4 itoa
        let res = Resolve {
            packages: vec![
                pkg("geiger", "1.0.0", &[1]),
                pkg("syn", "2.0.0", &[2]),
                pkg("unicode-ident", "1.0.0", &[]),
                pkg("hakari", "0.17.0", &[4]),
                pkg("itoa", "1.0.0", &[]),
            ],
        };
        let all = DenyConfig::parse("[bans]\nskip-tree = [\"geiger\"]").unwrap();
        assert_eq!(all.skipped(&res), BTreeSet::from([0, 1, 2]));
        let one =
            DenyConfig::parse("[bans]\nskip-tree = [{ crate = \"geiger\", depth = 1 }]").unwrap();
        assert_eq!(one.skipped(&res), BTreeSet::from([0, 1]));
        let skip = DenyConfig::parse("[bans]\nskip = [\"itoa@1.0.0\"]").unwrap();
        assert_eq!(skip.skipped(&res), BTreeSet::from([4]));
    }
}
