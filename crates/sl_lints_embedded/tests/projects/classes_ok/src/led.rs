#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

#[embassy_executor::task]
pub async fn blink() {
    loop {
        embassy_time::Timer::after_millis(500).await;
    }
}
