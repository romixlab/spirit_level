//! `spirit-level tools`: where each pinned tool is found, and installing the pinned versions

use anstream::{eprint, println};
use clap::Subcommand;
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ToolsCmd {
    /// Every pinned tool: its version and where it is found (the tool cache, PATH, or missing)
    List,
    /// Install the pinned version of a tool, or of every tool: crates are built into the tool cache, rustup
    /// components are added to the project's toolchain
    #[command(after_help = "Exit codes: 0 installed, 1 an install failed, 2 usage error")]
    Install {
        /// The tool (`spirit-level tools list` names them); none: every tool
        #[arg(add = ArgValueCandidates::new(tool_candidates))]
        tool: Option<String>,
        /// The project whose toolchain gets rustup components
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn tool_candidates() -> Vec<CompletionCandidate> {
    sl_tools::pins()
        .iter()
        .map(|p| CompletionCandidate::new(&p.name).help(Some(p.version().to_string().into())))
        .collect()
}

pub fn run(cmd: ToolsCmd) -> Result<u8, String> {
    match cmd {
        ToolsCmd::List => list(),
        ToolsCmd::Install { tool, path } => install(tool, path),
    }
}

fn list() -> Result<u8, String> {
    let cache = sl_tools::Cache::from_env();
    println!("cache: {}", cache.root.display());
    for p in sl_tools::pins() {
        let at = match (cache.find(p), sl_tools::on_path(&p.exe, None)) {
            (Some(c), _) => format!("cache {}", c.display()),
            (None, Some(path)) => format!("PATH {}", path.display()),
            (None, None) => "missing".to_string(),
        };
        let nightly = if p.nightly { " (nightly)" } else { "" };
        println!("{:20} {:10} {at}{nightly}", p.name, p.version());
    }
    Ok(0)
}

fn install(tool: Option<String>, path: PathBuf) -> Result<u8, String> {
    let pins: Vec<&sl_tools::Pin> = match &tool {
        Some(t) => vec![sl_tools::pin(t)
            .ok_or_else(|| format!("no tool {t} (`spirit-level tools list` names them)"))?],
        None => sl_tools::pins().iter().collect(),
    };
    let cache = sl_tools::Cache::from_env();
    let mut failed = 0;
    for p in pins {
        eprint!("{} {}\n", p.name, p.version());
        match cache.install(p, &path) {
            Ok((exe, true)) => println!("{}: cached {}", p.name, exe.display()),
            Ok((exe, false)) => println!("{}: installed {}", p.name, exe.display()),
            Err(e) => {
                failed += 1;
                eprint!("{}: {e}\n", p.name);
            }
        }
    }
    Ok(u8::from(failed > 0))
}
