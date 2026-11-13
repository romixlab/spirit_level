//! Toolchains and targets through rustup, for the checks that build with another compiler or for another target

use crate::InstallError;
use std::path::Path;

/// Installs the toolchain `version` with the minimal profile, offline when it is installed already
pub fn install_toolchain(version: &str, project: &Path) -> Result<(), InstallError> {
    let installed = sl_model::command("rustc", project)
        .arg("-V")
        .env("RUSTUP_TOOLCHAIN", version)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .output()
        .is_ok_and(|o| o.status.success());
    if installed {
        return Ok(());
    }
    rustup(
        &["toolchain", "install", version, "--profile", "minimal"],
        project,
    )
}

/// Adds `target` to the project's toolchain (its `rust-toolchain.toml`, or the default)
pub fn add_target(target: &str, project: &Path) -> Result<(), InstallError> {
    rustup(&["target", "add", target], project)
}

fn rustup(args: &[&str], project: &Path) -> Result<(), InstallError> {
    let st = sl_model::command("rustup", project)
        .args(args)
        .status()
        .map_err(InstallError::Io)?;
    if st.success() {
        Ok(())
    } else {
        Err(InstallError::Failed(format!(
            "`rustup {}` failed ({st})",
            args.join(" ")
        )))
    }
}
