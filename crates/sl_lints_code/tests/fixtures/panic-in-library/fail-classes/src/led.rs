pub fn blink(period: u32) -> u32 {
    assert!(period > 0, "LED period");
    Some(period).expect("set")
}
