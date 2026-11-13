//! Every lint of the cargo-deny adapter against its pass and fail projects; skipped visibly when cargo-deny is missing.

#[test]
fn tool_fixtures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    sl_testkit::check_tools(&sl_tool_deny::DENY, &dir);
}
