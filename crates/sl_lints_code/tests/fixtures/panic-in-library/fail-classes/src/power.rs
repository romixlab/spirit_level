pub fn set(x: Option<u8>) -> u8 {
    assert!(x.is_some(), "a level is given");
    x.expect("checked above")
}
