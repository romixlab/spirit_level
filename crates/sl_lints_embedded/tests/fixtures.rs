//! Every lint of the embedded analysis against its pass and fail cases. The cases run the firmware projects under
//! `tests/projects/` (embedded_bedrock's bedrock_lint fixtures); a project's `elf.toml` becomes its linked ELF.

use std::path::Path;

fn elf(project: &Path) -> Option<Vec<u8>> {
    let text = std::fs::read_to_string(project.join("elf.toml")).ok()?;
    let spec = sl_firmware::synth::Spec::parse(&text)
        .unwrap_or_else(|e| panic!("{}: {e}", project.display()));
    Some(sl_firmware::synth::write(&spec).unwrap_or_else(|e| panic!("{}: {e}", project.display())))
}

#[test]
fn fixtures() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    sl_testkit::check_with(
        &sl_lints_embedded::EMBEDDED,
        &dir,
        &sl_testkit::Hooks { elf: Some(&elf) },
    );
}
