//! A description whose chip data or PCB is missing: the lints that need them report themselves as not run, with
//! the reason, never as a clean board.

use sl_analysis::Analysis;
use sl_config::{Config, Env};
use sl_core::Date;
use sl_engine::{RunOptions, all_lints};
use sl_model::Project;
use std::path::Path;

#[test]
fn missing_chip_data_and_pcb_are_not_run() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/projects/no_inputs");
    let today = Date::parse("2026-11-13").unwrap();
    let project = Project::load(&root).unwrap();
    let analyses: [&dyn Analysis; 1] = [&sl_lints_board::BOARD];
    let lints = all_lints(&analyses);
    let none = |_: &str| None;
    let env = Env {
        lints: &lints,
        tier: &none,
        today,
        version: None,
    };
    let file = std::env::temp_dir().join(format!(
        "sl_lints_board_not_run_{}.toml",
        std::process::id()
    ));
    let text = [
        "brd-1", "brd-2", "brd-3", "brd-4", "brd-5", "brd-6", "brd-7",
    ]
    .map(|id| ["[lints.", id, "]\nlevel = \"deny\"\nreason = \"test\"\n"].concat())
    .concat();
    std::fs::write(&file, text).unwrap();
    let config = Config::load(&root, Some(&file), &env).unwrap();
    std::fs::remove_file(&file).unwrap();
    let report = sl_engine::run(
        &project,
        &config,
        &analyses,
        &RunOptions {
            only: Vec::new(),
            today,
            path: None,
            elf: Vec::new(),
            landing: false,
        },
        &sl_engine::Silent,
    );
    let not_run: Vec<(&str, &str)> = report
        .not_run
        .iter()
        .map(|n| (n.lint.as_str(), n.reason.as_str()))
        .collect();
    let chip = "boards/t9/board.toml: chip data missing: boards/chips/STM32TEST.json is not there (stm32-data's data/chips/STM32TEST.json)";
    let pcb = "boards/t9/board.toml: PCB missing: boards/t9/t9.json is not there";
    assert_eq!(
        not_run,
        [
            ("brd-4", chip),
            ("brd-5", chip),
            ("brd-6", pcb),
            ("brd-7", pcb)
        ],
        "{not_run:?}"
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}
