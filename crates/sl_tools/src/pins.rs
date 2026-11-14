//! The tools of `tools.toml`: how each is installed, the versions it is tested at, how its version is read.

use semver::{Version, VersionReq};
use std::sync::OnceLock;

/// How a tool is installed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinKind {
    /// A crates.io crate
    Crate {
        /// The crate (`typos-cli`)
        krate: String,
        /// The exact version to install, inside the tested range
        version: Version,
    },
    /// A rustup component, versioned with the toolchain (`clippy`)
    Component(String),
}

/// One tool of `tools.toml`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    /// The tool's name (`cargo-deny`, `typos`), what `Need::Tool` names
    pub name: String,
    /// How it is installed
    pub kind: PinKind,
    /// The versions whose output is tested; None for a component that takes any version of its toolchain
    pub tested: Option<VersionReq>,
    /// The executable the adapters start
    pub exe: String,
    /// The arguments that print its version
    pub version_args: Vec<String>,
    /// Runs on a nightly toolchain
    pub nightly: bool,
}

impl Pin {
    /// The version to install, `toolchain` for a component
    pub fn version(&self) -> String {
        match &self.kind {
            PinKind::Crate { version, .. } => version.to_string(),
            PinKind::Component(_) => "toolchain".to_owned(),
        }
    }

    /// The tested range as written (`>=0.20, <0.21`), `toolchain` for a component without one
    pub fn tested_text(&self) -> String {
        self.tested
            .as_ref()
            .map_or_else(|| "toolchain".to_owned(), ToString::to_string)
    }

    /// `crate@version` of a crate, the install spec of `cargo binstall` and `cargo install`; None for a component
    pub fn spec(&self) -> Option<String> {
        match &self.kind {
            PinKind::Crate { krate, version } => Some(format!("{krate}@{version}")),
            PinKind::Component(_) => None,
        }
    }

    /// How to install it, for a help line
    pub fn install_help(&self) -> String {
        match &self.kind {
            PinKind::Crate { krate, version } => format!(
                "install {krate}@{version}: `cargo binstall {krate}@{version}` or `cargo install --locked {krate}@{version}`"
            ),
            PinKind::Component(c) => format!("`rustup component add {c}`"),
        }
    }

    /// Whether `version` is in the tested range (always for a component without one)
    pub fn tested_at(&self, version: &Version) -> bool {
        self.tested.as_ref().is_none_or(|r| r.matches(version))
    }
}

/// The version in a tool's `--version` output: the first word shaped like `1.2.3` (or `1.2`), its pre-release
/// and build parts dropped (`1.10.0-stable` is 1.10.0), so the tested range compares releases
pub(crate) fn parse_version(output: &str) -> Option<Version> {
    output.split_whitespace().find_map(|w| {
        let w = w.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        let w = w.strip_prefix('v').unwrap_or(w);
        let core = w.split(['-', '+']).next()?;
        let parts: Vec<&str> = core.split('.').collect();
        if !(2..=3).contains(&parts.len())
            || parts
                .iter()
                .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let n = |i: usize| parts.get(i).map_or(Some(0), |p| p.parse::<u64>().ok());
        Some(Version::new(n(0)?, n(1)?, n(2)?))
    })
}

const TOOLS: &str = include_str!("../tools.toml");

/// Every tool, in the order of `tools.toml`
///
/// # Panics
///
/// When the compiled-in `tools.toml` does not parse; a unit test parses it.
pub fn pins() -> &'static [Pin] {
    static PINS: OnceLock<Vec<Pin>> = OnceLock::new();
    // spirit-level: allow(panic-in-library) tools.toml is compiled in, and a unit test parses it
    PINS.get_or_init(|| parse(TOOLS).unwrap_or_else(|e| panic!("tools.toml: {e}")))
}

/// The tool called `name`
pub fn pin(name: &str) -> Option<&'static Pin> {
    pins().iter().find(|p| p.name == name)
}

const KEYS: &[&str] = &[
    "crate",
    "version",
    "component",
    "tested",
    "exe",
    "version-args",
    "nightly",
];

fn parse(text: &str) -> Result<Vec<Pin>, String> {
    let doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let mut out = Vec::new();
    for (name, item) in doc.iter() {
        let t = item
            .as_table()
            .ok_or_else(|| format!("{name} is not a table"))?;
        if let Some((k, _)) = t.iter().find(|(k, _)| !KEYS.contains(k)) {
            return Err(format!("{name}: unknown key {k}"));
        }
        let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_owned);
        let tested = s("tested")
            .map(|r| VersionReq::parse(&r).map_err(|e| format!("{name}: tested {r:?}: {e}")))
            .transpose()?;
        let kind = match (s("crate"), s("version"), s("component")) {
            (Some(krate), Some(v), None) => PinKind::Crate {
                krate,
                version: Version::parse(&v).map_err(|e| format!("{name}: version {v:?}: {e}"))?,
            },
            (None, None, Some(c)) => PinKind::Component(c),
            _ => return Err(format!("{name}: give crate and version, or component")),
        };
        if let PinKind::Crate { version, .. } = &kind {
            match &tested {
                None => return Err(format!("{name}: a crate needs a tested range")),
                Some(r) if !r.matches(version) => {
                    return Err(format!("{name}: version {version} is outside tested {r}"));
                }
                Some(_) => {}
            }
        }
        let version_args = match t.get("version-args") {
            None => vec!["--version".to_owned()],
            Some(v) => v
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| format!("{name}: version-args is not a list of strings"))?,
        };
        out.push(Pin {
            name: name.to_owned(),
            kind,
            tested,
            exe: s("exe").ok_or_else(|| format!("{name}: no exe"))?,
            version_args,
            nightly: t.get("nightly").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_table_parses() {
        let names: Vec<&str> = pins().iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "clippy",
                "rustfmt",
                "cargo-deny",
                "cargo-machete",
                "cargo-shear",
                "typos",
                "taplo",
                "cargo-semver-checks",
                "cargo-udeps",
                "cargo-audit"
            ]
        );
        let deny = pin("cargo-deny").unwrap();
        assert_eq!(deny.exe, "cargo-deny");
        assert_eq!(deny.spec().as_deref(), Some("cargo-deny@0.20.2"));
        assert_eq!(deny.tested_text(), ">=0.20, <0.21");
        assert_eq!(deny.version_args, ["--version"]);
        let clippy = pin("clippy").unwrap();
        assert_eq!(clippy.kind, PinKind::Component("clippy".into()));
        assert_eq!(clippy.spec(), None);
        assert_eq!(clippy.tested_text(), "toolchain");
        assert_eq!(clippy.install_help(), "`rustup component add clippy`");
        let udeps = pin("cargo-udeps").unwrap();
        assert!(udeps.nightly);
        assert_eq!(udeps.version_args, ["udeps", "--version"]);
        assert_eq!(
            pin("typos").unwrap().spec().as_deref(),
            Some("typos-cli@1.52.2")
        );
    }

    #[test]
    fn bad_tables_fail() {
        let crate_tool = "[x]\ncrate = \"x\"\nversion = \"1.0.0\"\nexe = \"x\"\n";
        assert!(parse(crate_tool).unwrap_err().contains("tested range"));
        assert!(
            parse(&format!("{crate_tool}tested = \">=2\"\n"))
                .unwrap_err()
                .contains("outside")
        );
        assert!(parse(&format!("{crate_tool}tested = \"nonsense\"\n")).is_err());
        assert!(parse("[x]\ncrate = \"x\"\nexe = \"x\"\n").is_err());
        assert!(parse("[x]\ncomponent = \"x\"\nexe = \"x\"\ncolour = 1\n").is_err());
        assert!(parse("[x]\ncomponent = \"x\"\nexe = \"x\"\nversion-args = [1]\n").is_err());
    }

    #[test]
    fn range_check() {
        let deny = pin("cargo-deny").unwrap();
        assert!(deny.tested_at(&Version::new(0, 20, 0)));
        assert!(deny.tested_at(&Version::new(0, 20, 9)));
        assert!(!deny.tested_at(&Version::new(0, 19, 4)));
        assert!(!deny.tested_at(&Version::new(0, 21, 0)));
        assert!(pin("rustfmt").unwrap().tested_at(&Version::new(0, 0, 1)));
    }

    /// The `--version` output of each tool, as each prints it
    #[test]
    fn versions_of_each_tool() {
        let cases = [
            ("clippy 0.1.99 (b940084d7e 2026-09-28)\n", (0, 1, 99)),
            (
                "rustfmt 1.10.0-stable (b940084d7e 2026-09-28)\n",
                (1, 10, 0),
            ),
            ("cargo-deny 0.20.2\n", (0, 20, 2)),
            ("0.9.2\n", (0, 9, 2)),
            ("Version: 1.14.1\n\n", (1, 14, 1)),
            ("typos-cli 1.52.2\n", (1, 52, 2)),
            ("taplo 0.10.0\n", (0, 10, 0)),
            ("cargo-semver-checks 0.51.0\n", (0, 51, 0)),
            ("cargo-udeps 0.1.61\n", (0, 1, 61)),
            ("cargo-audit-audit 0.23.0\n", (0, 23, 0)),
            ("tool v2.3\n", (2, 3, 0)),
        ];
        for (out, (a, b, c)) in cases {
            assert_eq!(parse_version(out), Some(Version::new(a, b, c)), "{out}");
        }
        assert_eq!(
            parse_version("error: unexpected argument '--version' found"),
            None
        );
        assert_eq!(parse_version("built 2026-09-28"), None);
        assert_eq!(parse_version(""), None);
    }
}
