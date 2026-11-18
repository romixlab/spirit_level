#![deny(clippy::unwrap_used, clippy::expect_used)]

#[embassy_executor::task]
pub async fn blink(period_ms: u64) {
    assert!(period_ms > 0, "LED period");
    let half_ms = period_ms.checked_div(2).expect("two is not zero");
    loop {
        embassy_time::Timer::after_millis(half_ms).await;
    }
}
