//! `spl commit-msg`: the message-only lints on one file, no cargo project needed.
#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

/// A scratch directory, removed on drop
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let dir =
            std::env::temp_dir().join(format!("sl_cli_commit_msg_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (p, t) in files {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().expect("a file path has a parent"))
                .expect("create the scratch directory");
            std::fs::write(f, t).expect("write a scratch file");
        }
        Self(dir)
    }

    fn write(&self, p: &str, t: &str) {
        std::fs::write(self.0.join(p), t).expect("write a scratch file");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str]) -> Output {
    crate::spl()
        .args(args)
        .current_dir(dir)
        .env("SPIRIT_LEVEL_TODAY", "2026-11-13")
        .env("NO_COLOR", "1")
        .output()
        .expect("spl starts")
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// CLI-11: a 73-char subject is denied with the limit; a 72-char one passes. No `Cargo.toml` anywhere:
/// `commit-msg` reads only `spirit_level.toml`, never cargo metadata.
#[test]
fn a_long_subject_is_denied_a_short_one_passes() {
    let p = Scratch::new("ai", &[("spirit_level.toml", "extends = \"ai\"\n")]);
    p.write(
        "msg",
        &format!("{}\n\nTested: not needed\n", "a".repeat(73)),
    );
    let o = run(&p.0, &["commit-msg", "msg"]);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    let t = text(&o);
    assert!(t.contains("commit-subject-length"), "{t}");
    assert!(t.contains("limit 72"), "{t}");

    p.write("msg", &format!("{}\n", "a".repeat(72)));
    let o = run(&p.0, &["commit-msg", "msg"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
}

/// A repo with no `spirit_level.toml` above the working directory: nothing to check, never an error.
#[test]
fn no_spirit_level_toml_passes() {
    let p = Scratch::new("none", &[("msg", "a subject\n")]);
    let o = run(&p.0, &["commit-msg", "msg"]);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
}

/// git's own comment lines (`# ...`), left by an editor template, are not part of the message.
#[test]
fn comment_lines_are_stripped_before_the_subject_is_measured() {
    let p = Scratch::new("comments", &[("spirit_level.toml", "extends = \"ai\"\n")]);
    p.write(
        "msg",
        &format!(
            "# {}\n{}\n# Please enter the commit message\n",
            "x".repeat(73),
            "a".repeat(73)
        ),
    );
    let o = run(&p.0, &["commit-msg", "msg"]);
    let t = text(&o);
    assert_eq!(o.status.code(), Some(1), "{t}");
    // the comment's own 75-char line never becomes the measured subject
    assert!(t.contains("commit subject of 73 characters"), "{t}");
}
