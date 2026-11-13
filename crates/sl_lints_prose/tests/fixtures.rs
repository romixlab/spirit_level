//! Every lint of the prose analysis against its pass and fail projects.

#[test]
fn fixtures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    sl_testkit::check(&sl_lints_prose::PROSE, &dir);
}
