//! A test

#[test]
fn formats() {
    assert_eq!(itoa::Buffer::new().format(1), "1");
}
