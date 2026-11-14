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

/// Whether a nightly toolchain is installed
pub fn nightly_installed(dir: &Path) -> bool {
    crate::run(
        Path::new("rustup"),
        &["run", "nightly", "rustc", "-V"],
        dir,
        &[],
    )
    .is_ok_and(|o| o.code == Some(0))
}
