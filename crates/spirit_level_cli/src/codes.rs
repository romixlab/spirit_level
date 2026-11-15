//! `spl codes`: the registry of lint codes, listed or held to the lints

use crate::TOOL;
use vc_ui::{Exit, Paint};

pub(crate) fn codes(check: bool, base: Option<&str>) -> Result<Exit, String> {
    use spirit_level::codes;
    if !check {
        for r in codes::rows() {
            println!("{} {:28} {:9} {}", r.code, r.id, r.group, r.status.name());
        }
        return Ok(Exit::SUCCESS);
    }
    let base_text = match &base {
        None => None,
        Some(rev) => {
            let out = std::process::Command::new("git")
                .args(["show", &format!("{rev}:crates/spirit_level/codes.toml")])
                .output()
                .map_err(|e| format!("--base {rev}: cannot run git: {e}"))?;
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).into_owned())
            } else {
                eprintln!("codes.toml is not in {rev} yet: nothing to compare with");
                None
            }
        }
    };
    let problems = codes::check(
        &spirit_level::lints(),
        codes::REGISTRY,
        base_text.as_deref(),
    );
    for p in &problems {
        let e = Exit::FAILURE.error(format!("codes.toml: {p}"));
        eprintln!("{}", vc_ui::error_line(TOOL, &e, Paint::stderr()));
    }
    Ok(if problems.is_empty() {
        Exit::SUCCESS
    } else {
        Exit::FAILURE
    })
}
