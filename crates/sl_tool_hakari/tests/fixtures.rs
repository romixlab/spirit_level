//! Every lint of the hakari adapter against its pass and fail projects (the dev-dependency on this crate turns
//! on `link`, so the linked analysis runs).

#[test]
fn tool_fixtures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    sl_testkit::check_tools(&sl_tool_hakari::HAKARI, &dir);
}
