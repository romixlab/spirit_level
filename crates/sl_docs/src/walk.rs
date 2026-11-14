//! Finding a project's files: a walk from the root that leaves out `target/`, hidden folders and nested cargo
//! projects that are not workspace members (test fixtures).

use sl_core::glob;
use std::path::Path;

/// Every file under `root` matching one of `include` and none of `skip`, relative to the root and sorted; `members`
/// are the directories of the workspace members, relative to the root
pub fn files(root: &Path, members: &[&str], include: &[String], skip: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    walk(root, "", members, &mut out);
    out.retain(|f| {
        include.iter().any(|g| glob::matches(g, f)) && !skip.iter().any(|g| glob::matches(g, f))
    });
    out.sort();
    out
}

fn walk(root: &Path, rel: &str, members: &[&str], out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(root.join(rel)) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if e.path().join("Cargo.toml").exists() && !members.contains(&path.as_str()) {
                continue;
            }
            walk(root, &path, members, out);
        } else if ft.is_file() {
            out.push(path);
        }
    }
}
