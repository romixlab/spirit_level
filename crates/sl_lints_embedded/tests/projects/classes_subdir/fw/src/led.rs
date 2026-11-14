#[embassy_executor::task]
pub async fn blink() {
    loop {
        embassy_time::Timer::after_millis(500).await;
    }
}
