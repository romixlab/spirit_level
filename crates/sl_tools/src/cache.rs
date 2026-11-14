//! The tool cache: `cargo install --locked` builds of the pinned versions.
//!
//! A build lives in `<root>/<tool>/<version>-<key>/bin/`, where the key is the SHA-256 (first 16 hex digits) of
//! the crate, its version, its features and `rustc -vV`. A crates.io version never changes, so crate and version
//! stand for its sources and the `Cargo.lock` packaged with it; a new compiler gives a new key and a new build.
//! The root is `$SPIRIT_LEVEL_TOOLS`, else `$XDG_CACHE_HOME/spirit_level/tools`, else
//! `~/.cache/spirit_level/tools`.

use crate::pins::{Pin, PinKind};
use sha2::{Digest, Sha256};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The tool cache
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cache {
    /// Where builds are kept
    pub root: PathBuf,
}

/// Why an install failed
#[derive(Debug)]
pub enum InstallError {
    /// No home or cache directory to keep builds in
    NoRoot,
    /// `rustc -vV` failed: no Rust toolchain
    NoRustc,
    /// The installer could not be started or the cache not written
    Io(std::io::Error),
    /// The installer failed
    Failed(String),
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRoot => {
                write!(f, "no cache directory: set SPIRIT_LEVEL_TOOLS or HOME")
            }
            Self::NoRustc => write!(f, "`rustc -vV` failed: no Rust toolchain on PATH"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Failed(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for InstallError {}

impl Cache {
    /// The cache of this environment
    pub fn from_env() -> Self {
        let root = std::env::var_os("SPIRIT_LEVEL_TOOLS")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_CACHE_HOME")
                    .map(|d| PathBuf::from(d).join("spirit_level/tools"))
            })
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/spirit_level/tools"))
            })
            .unwrap_or_default();
        Self { root }
    }

    /// The cache key of a crate tool built with the compiler `rustc_vv` describes; None for a component
    pub fn key(pin: &Pin, rustc_vv: &str) -> Option<String> {
        let PinKind::Crate {
            krate,
            version,
            default_features,
            features,
        } = &pin.kind
        else {
            return None;
        };
        let mut h = Sha256::new();
        h.update(format!(
            "spirit_level tool cache 1\n{krate} {version} --locked\ndefault-features={default_features}\nfeatures={}\n{rustc_vv}",
            features.join(",")
        ));
        let d = h.finalize();
        const HEX: &[u8; 16] = b"0123456789abcdef";
        Some(
            d.iter()
                .take(8)
                .flat_map(|b| [HEX[usize::from(b >> 4)], HEX[usize::from(b & 15)]])
                .map(char::from)
                .collect(),
        )
    }

    /// The directory of a build (`--root` of its `cargo install`)
    pub fn dir(&self, pin: &Pin, key: &str) -> PathBuf {
        self.root
            .join(&pin.name)
            .join(format!("{}-{key}", pin.version()))
    }

    /// The executable of a cached build of the pinned version: the build for the current compiler, else any
    /// build of that version (a working binary does not depend on the compiler it was built with)
    pub fn find(&self, pin: &Pin) -> Option<PathBuf> {
        if self.root.as_os_str().is_empty() || matches!(pin.kind, PinKind::Component(_)) {
            return None;
        }
        if let Some(vv) = crate::rustc_vv()
            && let Some(key) = Self::key(pin, vv)
        {
            let exe = self.dir(pin, &key).join("bin").join(&pin.exe);
            if exe.is_file() {
                return Some(exe);
            }
        }
        let prefix = format!("{}-", pin.version());
        let mut builds: Vec<PathBuf> = std::fs::read_dir(self.root.join(&pin.name))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .map(|e| e.path().join("bin").join(&pin.exe))
            .filter(|p| p.is_file())
            .collect();
        builds.sort();
        builds.pop()
    }

    /// Installs a pinned tool: a crate is built into the cache unless its build for this compiler is there
    /// already; a component is added with rustup to the toolchain of `project`. Returns the executable, and
    /// whether it was there already. The installer's output goes to this process's standard error
    pub fn install(&self, pin: &Pin, project: &Path) -> Result<(PathBuf, bool), InstallError> {
        match &pin.kind {
            PinKind::Component(c) => {
                let st = sl_model::command("rustup", project)
                    .args(["component", "add", c])
                    .status()
                    .map_err(InstallError::Io)?;
                if !st.success() {
                    return Err(InstallError::Failed(format!(
                        "`rustup component add {c}` failed ({st})"
                    )));
                }
                crate::on_path(&pin.exe, None)
                    .map(|p| (p, false))
                    .ok_or_else(|| InstallError::Failed(format!("{} is not on PATH", pin.exe)))
            }
            PinKind::Crate {
                krate,
                version,
                default_features,
                features,
            } => {
                if self.root.as_os_str().is_empty() {
                    return Err(InstallError::NoRoot);
                }
                let vv = crate::rustc_vv().ok_or(InstallError::NoRustc)?;
                let key = Self::key(pin, vv).ok_or(InstallError::NoRustc)?;
                let dir = self.dir(pin, &key);
                let exe = dir.join("bin").join(&pin.exe);
                if exe.is_file() {
                    return Ok((exe, true));
                }
                let parent = self.root.join(&pin.name);
                std::fs::create_dir_all(&parent).map_err(InstallError::Io)?;
                let tmp = parent.join(format!(".build-{}", std::process::id()));
                #[expect(
                    clippy::let_underscore_must_use,
                    reason = "a leftover build directory may not be there"
                )]
                let _ = std::fs::remove_dir_all(&tmp);
                let mut cmd: Command = sl_model::command("cargo", &parent);
                cmd.arg("install")
                    .arg("--locked")
                    .args(["--version", version])
                    .arg("--root")
                    .arg(&tmp);
                if !default_features {
                    cmd.arg("--no-default-features");
                }
                if !features.is_empty() {
                    cmd.args(["--features", &features.join(",")]);
                }
                cmd.arg(krate);
                let st = cmd.status().map_err(InstallError::Io)?;
                if !st.success() {
                    #[expect(
                        clippy::let_underscore_must_use,
                        reason = "the install already failed; its error is the one reported"
                    )]
                    let _ = std::fs::remove_dir_all(&tmp);
                    return Err(InstallError::Failed(format!(
                        "`cargo install --locked --version {version} {krate}` failed ({st})"
                    )));
                }
                std::fs::rename(&tmp, &dir).map_err(InstallError::Io)?;
                Ok((exe, false))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_follows_version_features_and_compiler() {
        let deny = crate::pin("cargo-deny").unwrap().clone();
        let a = Cache::key(&deny, "rustc 1.99.0").unwrap();
        assert_eq!(a.len(), 16);
        assert_eq!(a, Cache::key(&deny, "rustc 1.99.0").unwrap());
        assert_ne!(a, Cache::key(&deny, "rustc 1.99.1").unwrap());
        let mut other = deny.clone();
        if let PinKind::Crate { version, .. } = &mut other.kind {
            *version = "0.0.1".into();
        }
        assert_ne!(a, Cache::key(&other, "rustc 1.99.0").unwrap());
        assert_eq!(Cache::key(crate::pin("clippy").unwrap(), "x"), None);
    }

    #[test]
    fn finds_a_build_of_the_pinned_version() {
        let root = std::env::temp_dir().join(format!("sl_tools_cache_{}", std::process::id()));
        let cache = Cache { root: root.clone() };
        let pin = crate::pin("cargo-machete").unwrap();
        assert_eq!(cache.find(pin), None);
        // a build made by another compiler still counts
        let bin = cache.dir(pin, "0000000000000000").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join(&pin.exe), "").unwrap();
        assert_eq!(cache.find(pin), Some(bin.join(&pin.exe)));
        // another version does not
        let old = root.join(&pin.name).join("0.0.1-0000000000000000/bin");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::remove_file(bin.join(&pin.exe)).unwrap();
        std::fs::write(old.join(&pin.exe), "").unwrap();
        assert_eq!(cache.find(pin), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
