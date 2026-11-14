//! Finding a project's files: a walk from the root that leaves out `target/`, hidden folders and nested cargo
//! projects that are not workspace members (test fixtures), and bounded reads of what it finds.

use sl_core::glob;
use std::io::Read;
use std::path::Path;

/// The most bytes a page may have; a bigger file is not a page a person reads, and is left unread
const PAGE_BYTES_MAX: u64 = 4 * 1024 * 1024;

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

/// A page's text: None when it cannot be read, is not UTF-8 or is over 4 MiB
pub fn read_page(root: &Path, rel: &str) -> Option<String> {
    let f = std::fs::File::open(root.join(rel)).ok()?;
    let mut text = String::new();
    f.take(PAGE_BYTES_MAX + 1).read_to_string(&mut text).ok()?;
    (text.len() as u64 <= PAGE_BYTES_MAX).then_some(text)
}
