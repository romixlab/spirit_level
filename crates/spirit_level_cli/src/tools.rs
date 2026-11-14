//! `spl tools`: where each pinned tool is found, and installing the pinned versions

use clap::Subcommand;
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum ToolsCmd {
    /// Every pinned tool: its version and where it is found (the tool cache, PATH, or missing)
    #[command(after_help = "Examples:\n  spl tools list")]
    List,
    /// Install the pinned version of a tool, or of every tool: crates are built into the tool cache, rustup
    /// components are added to the project's toolchain
    #[command(
        after_help = "Examples:\n  spl tools install\n  spl tools install cargo-deny\n  spl tools install --needed\n\nExit codes: 0 installed, 1 an install failed, 2 usage error"
    )]
    Install {
        /// The tool (`spl tools list` names them); none: every tool
        #[arg(add = ArgValueCandidates::new(tool_candidates), conflicts_with = "needed")]
        tool: Option<String>,
        /// The project whose toolchain gets rustup components
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Only what the project's enabled lints need: their tools, the rust-version toolchains of `cargo-msrv`,
        /// the targets of `cargo-no-std` and `cargo-wasm`
        #[arg(long)]
        needed: bool,
    },
}

fn tool_candidates() -> Vec<CompletionCandidate> {
    sl_tools::pins()
        .iter()
        .map(|p| CompletionCandidate::new(&p.name).help(Some(p.version().to_owned().into())))
        .collect()
}

pub fn run(cmd: ToolsCmd) -> Result<vc_ui::Exit, String> {
    match cmd {
        ToolsCmd::List => Ok(list()),
        ToolsCmd::Install {
            tool,
            path,
            needed: false,
        } => install(tool.as_deref(), &path),
        ToolsCmd::Install {
            path, needed: true, ..
        } => install_needed(&path),
    }
}

fn list() -> vc_ui::Exit {
    let cache = sl_tools::Cache::from_env();
    println!("cache: {}", cache.root.display());
    for p in sl_tools::pins() {
        let at = match (cache.find(p), sl_tools::on_path(&p.exe, None)) {
            (Some(c), _) => format!("cache {}", c.display()),
            (None, Some(path)) => format!("PATH {}", path.display()),
            (None, None) => "missing".to_owned(),
        };
        let nightly = if p.nightly { " (nightly)" } else { "" };
        println!("{:20} {:10} {at}{nightly}", p.name, p.version());
    }
    vc_ui::Exit::SUCCESS
}

fn install(tool: Option<&str>, path: &Path) -> Result<vc_ui::Exit, String> {
    let pins: Vec<&sl_tools::Pin> = match tool {
        Some(t) => vec![
            sl_tools::pin(t).ok_or_else(|| format!("no tool {t} (`spl tools list` names them)"))?,
        ],
        None => sl_tools::pins().iter().collect(),
    };
    Ok(failed_if(install_pins(&pins, path) > 0))
}

fn install_needed(path: &Path) -> Result<vc_ui::Exit, String> {
    let project = sl_model::Project::load(path).map_err(|e| e.to_string())?;
    let opts = spirit_level::Options::new(sl_core::Date::today_utc());
    let config = spirit_level::config(&project, &opts).map_err(|e| e.to_string())?;
    let needs = spirit_level::tool_needs(&project, &config);
    let pins: Vec<&sl_tools::Pin> = needs
        .tools
        .iter()
        .filter_map(|t| sl_tools::pin(t))
        .collect();
    let mut failed = install_pins(&pins, path);
    let toolchains = needs.toolchains.iter().map(|v| {
        let r = sl_tools::rustup::install_toolchain(v, path);
        (format!("toolchain {v}"), r)
    });
    let targets = needs.targets.iter().map(|t| {
        let r = sl_tools::rustup::add_target(t, path);
        (format!("target {t}"), r)
    });
    for (what, r) in toolchains.chain(targets) {
        match r {
            Ok(()) => println!("{what}: installed"),
            Err(e) => {
                failed += 1;
                eprintln!("{what}: {e}");
            }
        }
    }
    Ok(failed_if(failed > 0))
}

fn failed_if(failed: bool) -> vc_ui::Exit {
    if failed {
        vc_ui::Exit::FAILURE
    } else {
        vc_ui::Exit::SUCCESS
    }
}

/// Installs each pin; the number that failed
fn install_pins(pins: &[&sl_tools::Pin], path: &Path) -> usize {
    let cache = sl_tools::Cache::from_env();
    let mut failed = 0;
    for p in pins {
        eprintln!("{} {}", p.name, p.version());
        match cache.install(p, path) {
            Ok((exe, true)) => println!("{}: cached {}", p.name, exe.display()),
            Ok((exe, false)) => println!("{}: installed {}", p.name, exe.display()),
            Err(e) => {
                failed += 1;
                eprintln!("{}: {e}", p.name);
            }
        }
    }
    failed
}
