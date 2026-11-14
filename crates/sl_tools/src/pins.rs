//! The pinned tools of `tools.toml`.

use std::sync::OnceLock;

/// How a tool is installed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinKind {
    /// A crates.io crate, built with `cargo install --locked`
    Crate {
        /// The crate (`typos-cli`)
        krate: String,
        /// The exact version
        version: String,
        /// Whether its default features are built
        default_features: bool,
        /// Features turned on
        features: Vec<String>,
    },
    /// A rustup component, versioned with the toolchain (`clippy`)
    Component(String),
}

/// One tool of `tools.toml`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    /// The tool's name (`cargo-deny`, `typos`), what `Need::Tool` and `tools install` name
    pub name: String,
    /// How it is installed
    pub kind: PinKind,
    /// The executable the adapters start
    pub exe: String,
    /// Runs on a nightly toolchain
    pub nightly: bool,
}

impl Pin {
    /// The pinned version, `toolchain` for a component
    pub fn version(&self) -> &str {
        match &self.kind {
            PinKind::Crate { version, .. } => version,
            PinKind::Component(_) => "toolchain",
        }
    }
}

const TOOLS: &str = include_str!("../tools.toml");

/// Every pinned tool, in the order of `tools.toml`
///
/// # Panics
///
/// When the compiled-in `tools.toml` does not parse; a unit test parses it.
pub fn pins() -> &'static [Pin] {
    static PINS: OnceLock<Vec<Pin>> = OnceLock::new();
    // spirit-level: allow(panic-in-library) tools.toml is compiled in, and a unit test parses it
    PINS.get_or_init(|| parse(TOOLS).unwrap_or_else(|e| panic!("tools.toml: {e}")))
}

/// The pinned tool called `name`
pub fn pin(name: &str) -> Option<&'static Pin> {
    pins().iter().find(|p| p.name == name)
}

fn parse(text: &str) -> Result<Vec<Pin>, String> {
    let doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let mut out = Vec::new();
    for (name, item) in doc.iter() {
        let t = item
            .as_table()
            .ok_or_else(|| format!("{name} is not a table"))?;
        let s = |k: &str| t.get(k).and_then(|v| v.as_str()).map(str::to_owned);
        for (k, _) in t {
            if ![
                "crate",
                "version",
                "features",
                "default-features",
                "component",
                "exe",
                "nightly",
            ]
            .contains(&k)
            {
                return Err(format!("{name}: unknown key {k}"));
            }
        }
        let kind = match (s("crate"), s("version"), s("component")) {
            (Some(krate), Some(version), None) => PinKind::Crate {
                krate,
                version,
                default_features: t
                    .get("default-features")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                features: t
                    .get("features")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            (None, None, Some(c)) => PinKind::Component(c),
            _ => return Err(format!("{name}: give crate and version, or component")),
        };
        out.push(Pin {
            name: name.to_owned(),
            kind,
            exe: s("exe").ok_or_else(|| format!("{name}: no exe"))?,
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
        assert!(matches!(deny.kind, PinKind::Crate { .. }));
        assert_eq!(
            pin("clippy").unwrap().kind,
            PinKind::Component("clippy".into())
        );
        assert!(pin("cargo-udeps").unwrap().nightly);
    }

    #[test]
    fn bad_tables_fail() {
        assert!(parse("[x]\ncrate = \"x\"\nexe = \"x\"\n").is_err());
        assert!(parse("[x]\ncomponent = \"x\"\nexe = \"x\"\ncolour = 1\n").is_err());
    }
}
