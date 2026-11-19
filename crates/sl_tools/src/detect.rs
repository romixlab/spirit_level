//! Finding a tool on PATH and reading its version against the tested range.

use crate::pins::{Pin, parse_version};
use semver::Version;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A tool as found on PATH
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detection {
    /// The tool
    pub pin: &'static Pin,
    /// Its executable; None when it is not on PATH
    pub exe: Option<PathBuf>,
    /// Its version, or why it could not be read (the command and what it printed)
    pub version: Result<Version, String>,
    /// A later entry on PATH whose version is in the tested range, while `exe`'s is not; None when `exe` is in
    /// range, missing, or no later entry on PATH has an in-range version
    pub shadowed: Option<Shadowed>,
}

/// A tool of the same name, found further down PATH than the one `detect` picked, whose version is in range
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shadowed {
    /// Its executable
    pub exe: PathBuf,
    /// Its version
    pub version: Version,
}

/// How a found tool compares with its tested range
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Not on PATH
    Missing,
    /// Its version is in the tested range
    InRange,
    /// Its version is outside the tested range: it runs, its output is untested
    OutOfRange,
    /// It printed no version: it runs, its output is untested
    Unreadable,
}

impl State {
    /// `missing`, `in range`, `out of range`, `unreadable`
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::InRange => "in range",
            Self::OutOfRange => "out of range",
            Self::Unreadable => "unreadable",
        }
    }
}

impl Detection {
    /// How it compares with its tested range
    pub fn state(&self) -> State {
        match (&self.exe, &self.version) {
            (None, _) => State::Missing,
            (Some(_), Ok(v)) if self.pin.tested_at(v) => State::InRange,
            (Some(_), Ok(_)) => State::OutOfRange,
            (Some(_), Err(_)) => State::Unreadable,
        }
    }

    /// Why a lint that needs it cannot run: None unless it is missing
    pub fn missing_reason(&self) -> Option<String> {
        self.exe
            .is_none()
            .then(|| crate::missing_reason(&self.pin.name))
    }

    /// Why its output is untested: None when it is missing or in range
    pub fn untested_reason(&self) -> Option<String> {
        let exe = self.exe.as_ref()?;
        let name = &self.pin.name;
        let base = match (&self.version, self.state()) {
            (Ok(v), State::OutOfRange) => format!(
                "{name} {v} ({}) is outside the tested range {}",
                exe.display(),
                self.pin.tested_text()
            ),
            (Err(why), _) => format!("{name} ({}) printed no version: {why}", exe.display()),
            _ => return None,
        };
        Some(match &self.shadowed {
            None => base,
            Some(s) => format!(
                "{base}; {name} {} ({}) later on PATH is in range: the earlier one shadows it",
                s.version,
                s.exe.display()
            ),
        })
    }

    /// How to fix it being untested: install the tested version, or, when a later entry on PATH already has it,
    /// remove the earlier one or move its directory after the later one's
    pub fn fix_help(&self) -> String {
        match (&self.exe, &self.shadowed) {
            (Some(exe), Some(s)) => format!(
                "remove {} from PATH, or move its directory after {}'s: {} {} there is already in range",
                exe.display(),
                s.exe.display(),
                self.pin.name,
                s.version
            ),
            _ => self.pin.install_help(),
        }
    }
}

/// Finds the tool `pin` on PATH (or on `path`) and reads its version with its `version-args`, run in `dir` (a
/// rustup proxy picks the toolchain of the directory)
pub fn detect(pin: &'static Pin, path: Option<&OsStr>, dir: &Path) -> Detection {
    let exe = crate::on_path(&pin.exe, path);
    let version = match &exe {
        None => Err("not installed".to_owned()),
        Some(exe) => read_version(pin, exe, dir),
    };
    let in_range = matches!(&version, Ok(v) if pin.tested_at(v));
    let shadowed = match &exe {
        Some(found) if !in_range => shadow(pin, found, path, dir),
        _ => None,
    };
    Detection {
        pin,
        exe,
        version,
        shadowed,
    }
}

/// The first entry past `found` on PATH (or `path`) whose version of `pin` is in its tested range
fn shadow(pin: &'static Pin, found: &Path, path: Option<&OsStr>, dir: &Path) -> Option<Shadowed> {
    let path_var = path
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var_os("PATH"))?;
    let mut past_found = false;
    for candidate in std::env::split_paths(&path_var).map(|d| d.join(&pin.exe)) {
        if !past_found {
            past_found = candidate == found;
            continue;
        }
        if candidate.is_file()
            && let Ok(v) = read_version(pin, &candidate, dir)
            && pin.tested_at(&v)
        {
            return Some(Shadowed {
                exe: candidate,
                version: v,
            });
        }
    }
    None
}

fn read_version(pin: &Pin, exe: &Path, dir: &Path) -> Result<Version, String> {
    let args: Vec<&str> = pin.version_args.iter().map(String::as_str).collect();
    let command = format!("`{} {}`", pin.exe, args.join(" "));
    let out = crate::run(exe, &args, dir, &[]).map_err(|e| format!("{command}: {e}"))?;
    parse_version(&out.stdout)
        .filter(|_| out.code == Some(0))
        .ok_or_else(|| {
            let said = format!("{}{}", out.stdout, out.stderr);
            let first = said.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            format!("{command} gave {:?}", first.trim())
        })
}

#[cfg(all(test, unix))]
#[expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// A scratch PATH directory of fake tools, each a shell script printing a fixed `--version` line
    struct FakeBin(PathBuf);

    impl FakeBin {
        fn new(name: &str, tools: &[(&str, &str)]) -> Self {
            let dir = std::env::temp_dir().join(format!("sl_tools_{name}_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create the fake PATH directory");
            for (exe, script) in tools {
                let f = dir.join(exe);
                std::fs::write(&f, format!("#!/bin/sh\n{script}\n")).expect("write a fake tool");
                std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755))
                    .expect("make a fake tool executable");
            }
            Self(dir)
        }
    }

    impl Drop for FakeBin {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn states_from_a_fake_path() {
        let bin = FakeBin::new(
            "states",
            &[
                ("cargo-deny", "echo 'cargo-deny 0.20.5'"),
                ("typos", "echo 'typos-cli 1.50.3'"),
                ("taplo", "echo 'no version here'"),
                (
                    "cargo-udeps",
                    "[ \"$1\" = udeps ] && echo 'cargo-udeps 0.1.61' || exit 2",
                ),
            ],
        );
        let path = Some(bin.0.as_os_str());
        let d = |n: &str| detect(crate::pin(n).unwrap(), path, &bin.0);

        let deny = d("cargo-deny");
        assert_eq!(deny.state(), State::InRange);
        assert_eq!(deny.version, Ok(Version::new(0, 20, 5)));
        assert_eq!(deny.exe, Some(bin.0.join("cargo-deny")));
        assert_eq!(deny.missing_reason(), None);
        assert_eq!(deny.untested_reason(), None);

        let typos = d("typos");
        assert_eq!(typos.state(), State::OutOfRange);
        let why = typos.untested_reason().unwrap();
        assert!(why.starts_with("typos 1.50.3 ("), "{why}");
        assert!(
            why.ends_with("is outside the tested range >=1.52, <1.53"),
            "{why}"
        );

        let taplo = d("taplo");
        assert_eq!(taplo.state(), State::Unreadable);
        assert!(
            taplo
                .untested_reason()
                .unwrap()
                .ends_with("printed no version: `taplo --version` gave \"no version here\""),
            "{:?}",
            taplo.untested_reason()
        );

        assert_eq!(
            d("cargo-udeps").state(),
            State::InRange,
            "version-args reach the tool"
        );

        let machete = d("cargo-machete");
        assert_eq!(machete.state(), State::Missing);
        assert_eq!(
            machete.missing_reason().unwrap(),
            "cargo-machete is not installed: install cargo-machete@0.10.1: `cargo binstall cargo-machete@0.10.1` or `cargo install --locked cargo-machete@0.10.1`"
        );
        assert_eq!(
            d("clippy").missing_reason().unwrap(),
            "clippy is not installed: `rustup component add clippy`"
        );
    }

    #[test]
    fn a_shadowed_in_range_tool_is_named() {
        let front = FakeBin::new("shadow_front", &[("typos", "echo 'typos-cli 1.50.3'")]);
        let back = FakeBin::new("shadow_back", &[("typos", "echo 'typos-cli 1.52.2'")]);
        let path = std::env::join_paths([&front.0, &back.0]).expect("join PATH");

        let d = detect(
            crate::pin("typos").unwrap(),
            Some(path.as_os_str()),
            &front.0,
        );
        assert_eq!(d.state(), State::OutOfRange);
        let s = d.shadowed.clone().expect("a later entry is in range");
        assert_eq!(s.exe, back.0.join("typos"));
        assert_eq!(s.version, Version::new(1, 52, 2));

        let why = d.untested_reason().unwrap();
        assert!(why.starts_with("typos 1.50.3 ("), "{why}");
        assert!(
            why.ends_with(&format!(
                "typos 1.52.2 ({}) later on PATH is in range: the earlier one shadows it",
                back.0.join("typos").display()
            )),
            "{why}"
        );

        let fix = d.fix_help();
        assert_eq!(
            fix,
            format!(
                "remove {} from PATH, or move its directory after {}'s: typos 1.52.2 there is already in range",
                front.0.join("typos").display(),
                back.0.join("typos").display()
            )
        );
    }

    #[test]
    fn no_shadow_when_nothing_later_on_path_is_in_range() {
        let front = FakeBin::new("noshadow_front", &[("typos", "echo 'typos-cli 1.50.3'")]);
        let back = FakeBin::new("noshadow_back", &[("typos", "echo 'typos-cli 1.40.0'")]);
        let path = std::env::join_paths([&front.0, &back.0]).expect("join PATH");

        let d = detect(
            crate::pin("typos").unwrap(),
            Some(path.as_os_str()),
            &front.0,
        );
        assert_eq!(d.shadowed, None);
        assert_eq!(d.fix_help(), d.pin.install_help());
    }
}
