//! The built book's own search finds a lint's page first for its code, its id and a Clippy name of the tiers.
//!
//! The book is built with the repo's Zensical (`uv run zensical build --strict`) from a scratch copy of `docs/`, and
//! `search.mjs` runs the site's search worker on its index with node, as the browser does.

#![expect(
    clippy::let_underscore_must_use,
    reason = "the scratch directory is removed as far as it can be"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// A scratch directory, removed when dropped
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory of the scratch copy");
    for e in std::fs::read_dir(from).expect("read a directory of the book") {
        let e = e.expect("read a directory entry of the book");
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).expect("copy a file of the book");
        }
    }
}

fn require(tool: &str) {
    let found = Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(
        found,
        "skipped: {tool} missing (the book search test builds the site with uv and searches it with node)"
    );
}

/// The first hit of each query: (location, text)
fn first_hits(site: &Path, queries: &[String]) -> Vec<(String, String)> {
    let out = Command::new("node")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it/search.mjs"))
        .arg(site)
        .args(queries)
        .output()
        .expect("run node");
    assert!(
        out.status.success(),
        "search.mjs failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("search.mjs prints UTF-8")
        .lines()
        .map(|l| {
            let v: serde_json::Value =
                serde_json::from_str(l).expect("search.mjs prints JSON lines");
            let s = |k: &str| v[k].as_str().unwrap_or_default().to_owned();
            (s("location"), s("text"))
        })
        .collect()
}

#[test]
fn a_code_an_id_and_a_clippy_name_find_their_page_first() {
    require("uv");
    require("node");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scratch =
        Scratch(std::env::temp_dir().join(format!("sl_book_search_{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&scratch.0);
    let dir = &scratch.0;
    copy_dir(&repo.join("docs"), &dir.join("docs"));
    copy_dir(&repo.join("docs-overrides"), &dir.join("docs-overrides"));
    copy_dir(&repo.join("assets/logo"), &dir.join("docs/assets/logo"));
    std::fs::copy(repo.join("zensical.toml"), dir.join("zensical.toml")).unwrap();
    let build = Command::new("uv")
        .args(["run", "--frozen", "--project"])
        .arg(&repo)
        .args(["zensical", "build", "--strict"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "zensical build failed: {}{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let tiers = sl_book::tier_levels().unwrap();
    let rule = "Rule: The project's [lints.clippy]";
    let use_self = "Clippy lint: Self follows a rename.";
    let mut cases: Vec<(String, String, &str)> = vec![
        ("M0025".into(), "lints/M0025/".into(), rule),
        ("tier-clippy-set".into(), "lints/M0025/".into(), rule),
        ("use_self".into(), "clippy/use_self/".into(), use_self),
        ("use-self".into(), "clippy/use_self/".into(), use_self),
    ];
    for name in sl_book::clippy::entries(&tiers).keys() {
        cases.push((name.clone(), format!("clippy/{name}/"), ""));
    }
    let queries: Vec<String> = cases.iter().map(|(q, _, _)| q.clone()).collect();
    let hits = first_hits(&dir.join("site"), &queries);
    assert_eq!(hits.len(), cases.len(), "one answer per query");
    let wrong: Vec<String> = cases
        .iter()
        .zip(&hits)
        .filter(|((_, want, excerpt), (got, text))| got != want || !text.contains(excerpt))
        .map(|((q, want, excerpt), (got, text))| {
            format!("{q}: first hit {got} (want {want}), excerpt {text:?} (want {excerpt:?})")
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
