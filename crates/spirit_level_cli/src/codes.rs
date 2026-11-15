//! `spl codes`: the registry of lint codes, listed or held to the lints

use crate::TOOL;
use crate::registry::{Registered, Run};
use vc_ui::{Exit, Paint};

/// List every lint code ever given, with its lint id, group and status; `--check` holds the registry to the
/// lints (for `spirit_level`'s own `just lint`)
#[derive(clap::Parser)]
#[command(
    name = "codes",
    display_order = 2,
    after_help = "Examples:\n  spl codes\n  spl codes --check --base main\n\nA code is never reused or changed; a retired lint keeps its code reserved."
)]
pub(crate) struct Codes {
    /// Check the registry instead of listing it: every lint has its code, no code or id twice
    #[arg(long)]
    check: bool,
    /// With --check: the git revision whose registry every row must still match (rows only go from active to
    /// retired)
    #[arg(long, requires = "check", value_name = "REV")]
    base: Option<String>,
}

inventory::submit! { Registered::of::<Codes>() }

impl Run for Codes {
    fn run(self) -> Result<Exit, String> {
        codes(self.check, self.base.as_deref())
    }
}

fn codes(check: bool, base: Option<&str>) -> Result<Exit, String> {
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
