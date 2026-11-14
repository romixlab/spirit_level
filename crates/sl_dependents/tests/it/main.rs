//! `sl_dependents::find` on the fixture library and its dependents, in a scratch copy made a git repository.
#![expect(
    clippy::let_underscore_must_use,
    reason = "a test removes its scratch directory as far as it can"
)]

use sl_config::{Dependents, Origin};
use sl_dependents::{CommitState, Options, Report, SourceKind, find};
use sl_model::Project;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The fixtures copied to a scratch folder, removed on drop
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sl_dependents_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        copy(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
            &dir,
        );
        Self(dir)
    }

    fn lib(&self) -> PathBuf {
        self.0.join("lib")
    }

    fn write(&self, rel: &str, text: &str) {
        std::fs::write(self.0.join(rel), text).unwrap();
    }

    /// Makes the library a git repository on `main` with one commit; returns it
    fn git_init(&self) -> String {
        git(&self.lib(), &["init", "-q", "-b", "main"]);
        self.commit("first")
    }

    fn commit(&self, msg: &str) -> String {
        git(&self.lib(), &["add", "-A"]);
        git(&self.lib(), &["commit", "-q", "-m", msg]);
        git(&self.lib(), &["rev-parse", "HEAD"]).trim().to_owned()
    }

    /// Puts `commit` into app_a's lock as the library commit
    fn lock_app_a_at(&self, commit: &str) {
        let p = self.0.join("app_a/Cargo.lock");
        let t = std::fs::read_to_string(&p).unwrap();
        std::fs::write(p, t.replace("LIB_COMMIT", commit)).unwrap();
    }

    fn find(&self, opts: &Options) -> Report {
        let project = Project::load(&self.lib()).unwrap();
        let table = Dependents {
            scan: vec!["..".to_owned()],
            repos: vec![],
            base: None,
            test: "cargo nextest run".to_owned(),
            dir: self.lib(),
            origin: Origin {
                file: "spirit_level.toml".to_owned(),
                line: 3,
            },
        };
        find(&project, &table, opts).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn names<'a>(it: impl IntoIterator<Item = &'a String>) -> Vec<&'a str> {
    it.into_iter().map(String::as_str).collect()
}

#[test]
fn an_uncommitted_change_lists_its_dependents_and_an_old_lock() {
    let s = Scratch::new("uncommitted");
    let first = s.git_init();
    s.write("lib/crates/lib_extra/src/lib.rs", "//! lib_extra, second\n");
    s.commit("second");
    s.lock_app_a_at(&first);
    s.write("lib/crates/lib_io/src/lib.rs", "//! lib_io, changed\n");

    let r = s.find(&Options::default());
    assert_eq!(r.base.as_deref(), Some("main"));
    assert_eq!(names(&r.changed), ["lib_io"]);
    assert_eq!(names(&r.affected), ["lib_cli", "lib_io"]);
    assert_eq!(r.retest.len(), 1, "{r:#?}");
    let a = &r.retest[0];
    assert_eq!(a.name, "app_a");
    let crates: Vec<(&str, Vec<&str>, Vec<&str>)> = a
        .crates
        .iter()
        .map(|c| (c.name.as_str(), names(&c.uses), names(&c.through)))
        .collect();
    assert_eq!(
        crates,
        [
            ("a_app", vec!["lib_io"], vec!["a_core"]),
            ("a_core", vec!["lib_io"], vec![])
        ]
    );
    assert_eq!(a.command, "cargo nextest run -p a_app -p a_core");
    assert_eq!(a.sources.len(), 1);
    assert_eq!(
        names(&a.sources[0].crates),
        ["lib_core", "lib_extra", "lib_io"]
    );
    match &a.sources[0].kind {
        SourceKind::Git { branch, state, .. } => {
            assert_eq!(branch.as_deref(), Some("main"));
            assert_eq!(
                *state,
                CommitState::Behind {
                    tip: "main".to_owned(),
                    commits: 1
                }
            );
        }
        k => panic!("{k:?}"),
    }
    assert_eq!(r.unaffected.len(), 1);
    assert_eq!(r.unaffected[0].name, "app_b");
    assert_eq!(names(&r.unaffected[0].uses), ["lib_core"]);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
}

#[test]
fn a_base_counts_committed_changes_and_dev_edges_do_not_spread() {
    let s = Scratch::new("base");
    let first = s.git_init();
    s.write("lib/crates/lib_extra/src/lib.rs", "//! lib_extra, second\n");
    let second = s.commit("second");
    s.lock_app_a_at(&second);

    let r = s.find(&Options {
        base: Some(first),
        ..Options::default()
    });
    assert_eq!(names(&r.changed), ["lib_extra"]);
    assert_eq!(
        names(&r.affected),
        ["lib_extra"],
        "lib_cli takes lib_extra as a dev-dependency only"
    );
    let a = &r.retest[0];
    assert_eq!(a.command, "cargo nextest run -p a_tool");
    match &a.sources[0].kind {
        SourceKind::Git { state, .. } => {
            assert_eq!(
                *state,
                CommitState::Current {
                    tip: "main".to_owned()
                }
            );
        }
        k => panic!("{k:?}"),
    }
}

#[test]
fn the_root_manifest_changes_every_crate() {
    let s = Scratch::new("root");
    s.git_init();
    s.write("lib/Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n\n[workspace.package]\nversion = \"0.2.1\"\nedition = \"2024\"\n");
    let r = s.find(&Options::default());
    assert_eq!(r.workspace_wide.as_deref(), Some("Cargo.toml"));
    assert_eq!(r.changed.len(), 4);
    assert_eq!(names(r.retest.iter().map(|d| &d.name)), ["app_a", "app_b"]);
}

#[test]
fn all_needs_no_git_and_a_path_dependency_says_so() {
    let s = Scratch::new("all");
    let r = s.find(&Options {
        all: true,
        ..Options::default()
    });
    assert_eq!(r.base, None);
    assert_eq!(names(r.retest.iter().map(|d| &d.name)), ["app_a", "app_b"]);
    let b = &r.retest[1];
    assert_eq!(b.command, "cargo nextest run -p b");
    match &b.sources[0].kind {
        SourceKind::Path {
            path,
            this_checkout,
        } => {
            assert_eq!(path.as_deref(), Some("../lib/crates/lib_core"));
            assert!(this_checkout);
        }
        k => panic!("{k:?}"),
    }
    match &r.retest[0].sources[0].kind {
        SourceKind::Git {
            state: CommitState::Unknown { reason },
            ..
        } => {
            assert!(reason.contains("no branch main"), "{reason}");
        }
        k => panic!("{k:?}"),
    }
}

#[test]
fn a_library_outside_git_fails_without_all() {
    let s = Scratch::new("nogit");
    let project = Project::load(&s.lib()).unwrap();
    let table = Dependents {
        scan: vec![],
        repos: vec!["../app_b".to_owned()],
        base: None,
        test: "cargo test".to_owned(),
        dir: s.lib(),
        origin: Origin {
            file: "spirit_level.toml".to_owned(),
            line: 3,
        },
    };
    let e = find(&project, &table, &Options::default()).unwrap_err();
    assert!(e.to_string().contains("merge-base"), "{e}");
}
