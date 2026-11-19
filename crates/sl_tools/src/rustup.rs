//! Toolchains and targets through rustup, for the checks that build with another compiler or for another target

use std::path::Path;

/// Why the toolchain `version` cannot be selected in `dir`, or None when it can.
///
/// It cannot when it is not installed, or without rustup: a distribution's rustc ignores `RUSTUP_TOOLCHAIN`, so a
/// build would run on the wrong compiler. rustup is told not to install a missing toolchain by itself. None too
/// when rustc does not start.
pub fn toolchain_missing(version: &str, dir: &Path) -> Option<String> {
    let env = [("RUSTUP_TOOLCHAIN", version), ("RUSTUP_AUTO_INSTALL", "0")];
    let out = crate::run(Path::new("rustc"), &["-V"], dir, &env).ok()?;
    if is_missing_toolchain(&out.stderr) {
        return Some(format!(
            "the toolchain {version} is not installed: `rustup toolchain install {version} --profile minimal`"
        ));
    }
    let got = out.stdout.trim();
    (!got.starts_with(&format!("rustc {version}"))).then(|| {
        format!("asked for rust {version}, got `{got}`: the MSRV build needs rustup to select the toolchain")
    })
}

/// Whether rustup's error output says a toolchain is not installed
pub fn is_missing_toolchain(stderr: &str) -> bool {
    stderr.lines().any(|l| {
        let l = l.to_lowercase();
        (l.contains("toolchain") && l.contains("is not installed"))
            || l.contains("no such toolchain")
    })
}

/// The targets installed for the toolchain of `dir`; None without rustup (a distribution's toolchain, where cargo
/// says itself when a target is missing)
pub fn installed_targets(dir: &Path) -> Option<Vec<String>> {
    let out = crate::run(
        Path::new("rustup"),
        &["target", "list", "--installed"],
        dir,
        &[],
    )
    .ok()?;
    (out.code == Some(0)).then(|| out.stdout.lines().map(|l| l.trim().to_owned()).collect())
}

/// The nightly channel the project pins (`rust-toolchain-nightly.toml`'s `[toolchain] channel`, a dated nightly so
/// every box builds the same one), the plain `nightly` channel where it has none
pub fn nightly_channel(root: &Path) -> String {
    std::fs::read_to_string(root.join("rust-toolchain-nightly.toml"))
        .ok()
        .and_then(|s| s.parse::<toml_edit::DocumentMut>().ok())
        .and_then(|doc| {
            doc.get("toolchain")?
                .get("channel")?
                .as_str()
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "nightly".to_owned())
}

/// Whether the project's nightly channel ([`nightly_channel`]) is installed
pub fn nightly_installed(dir: &Path) -> bool {
    crate::run(
        Path::new("rustup"),
        &["run", &nightly_channel(dir), "rustc", "-V"],
        dir,
        &[],
    )
    .is_ok_and(|o| o.code == Some(0))
}

#[cfg(test)]
#[expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A scratch project root, removed on drop
    struct ScratchRoot(PathBuf);

    impl ScratchRoot {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("sl_tools_rustup_{name}_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create the scratch root");
            Self(dir)
        }
    }

    impl Drop for ScratchRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn nightly_channel_defaults_without_a_pin() {
        let root = ScratchRoot::new("default");
        assert_eq!(nightly_channel(&root.0), "nightly");
    }

    #[test]
    fn nightly_channel_reads_the_pin() {
        let root = ScratchRoot::new("pinned");
        std::fs::write(
            root.0.join("rust-toolchain-nightly.toml"),
            "[toolchain]\nchannel = \"nightly-2026-11-19\"\n",
        )
        .expect("write the pin file");
        assert_eq!(nightly_channel(&root.0), "nightly-2026-11-19");
    }
}
