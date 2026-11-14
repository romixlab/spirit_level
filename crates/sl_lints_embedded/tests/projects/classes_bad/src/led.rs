#![deny(clippy::unwrap_used, clippy::expect_used)]

#[embassy_executor::task]
pub async fn blink() {
    loop {
        embassy_time::Timer::after_millis(500).await;
    }
}
