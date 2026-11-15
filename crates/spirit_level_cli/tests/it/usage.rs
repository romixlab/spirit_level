//! `spl usage` and the records every run leaves (CLI-2): counted per command and flag, from a script, on the
//! injected day, in a scratch store. Made opt-in, off by default, in `vc_usage` (CLI-5); these tests still opt
//! in explicitly, so they pass under the locked `vc_usage` either way.
#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

/// A scratch usage store, removed on drop
struct Store(PathBuf);

impl Store {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sl_usage_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn spl(&self, args: &[&str]) -> Output {
        spl_in(&self.0, args)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `spl` with its records in `dir`, from a script: no terminal, no agent session around it
fn spl_in(dir: &Path, args: &[&str]) -> Output {
    crate::spl()
        .args(args)
        .env("SPL_USAGE_DIR", dir)
        .env_remove("CLAUDECODE")
        .env_remove("VC_USAGE_SOURCE")
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .expect("spl starts")
}

fn signal<'a>(doc: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    doc["signals"]
        .as_array()
        .expect("a list of signals")
        .iter()
        .find(|s| s["signal"] == name)
        .unwrap_or_else(|| panic!("no signal {name} in {doc}"))
}

#[test]
fn runs_are_counted_per_command_and_flag() {
    let store = Store::new("counted");
    for args in [
        &["tiers"][..],
        &["tiers"],
        &["--color", "never", "lints", "--format", "json"],
        &["explain", "crate-lines"],
    ] {
        assert!(store.spl(args).status.success(), "spl {args:?}");
    }
    let out = store.spl(&["usage", "--format", "json"]);
    assert!(out.status.success());
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON");
    assert_eq!(doc["schema"], "spirit_level.usage/1");
    let tiers = signal(&doc, "spl tiers");
    assert_eq!(
        (
            &tiers["runs"],
            &tiers["script"],
            &tiers["terminal"],
            &tiers["agent"]
        ),
        (&2.into(), &2.into(), &0.into(), &0.into())
    );
    assert_eq!(tiers["last"], "2026-11-13");
    assert_eq!(signal(&doc, "spl --color=never")["runs"], 1);
    assert_eq!(signal(&doc, "spl lints --format=json")["runs"], 1);
    assert_eq!(signal(&doc, "spl explain")["runs"], 1);
    // a default is not a use; the lint named is not recorded
    assert_eq!(signal(&doc, "spl lints --format=text")["runs"], 0);
    assert_eq!(signal(&doc, "spl check --fix")["runs"], 0);
    let log = std::fs::read_to_string(store.0.join("log.tsv")).expect("the log");
    assert!(!log.contains("crate-lines"), "{log}");
}

#[test]
fn unused_lists_only_what_never_ran() {
    let store = Store::new("unused");
    assert!(store.spl(&["tiers"]).status.success());
    let out = store.spl(&["usage", "--unused"]);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("never       spl check --fix\n"), "{text}");
    assert!(!text.contains("spl tiers"), "{text}");
    assert!(text.starts_with("records in "), "{text}");
}

#[test]
fn records_off_writes_nothing_and_usage_says_so() {
    let out = crate::spl()
        .arg("usage")
        .env("SPL_USAGE_DIR", "off")
        .output()
        .expect("spl starts");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage records are off"));
}

#[test]
fn a_store_that_cannot_be_written_never_fails_the_run() {
    let store = Store::new("unwritable");
    std::fs::create_dir_all(store.0.parent().expect("a parent")).expect("the temp dir");
    // a file where the store's directory would be
    std::fs::write(&store.0, "not a directory").expect("write the blocker");
    let out = store.spl(&["tiers"]);
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_file(&store.0);
}
