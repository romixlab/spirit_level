#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

#[embassy_executor::task]
pub async fn blink(period_ms: u64) {
    debug_assert!(period_ms > 0, "LED period");
    let half_ms = half(period_ms);
    loop {
        embassy_time::Timer::after_millis(half_ms).await;
    }
}

fn half(period_ms: u64) -> u64 {
    (period_ms / 2).max(1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn half_of_zero_still_blinks() {
        assert_eq!(super::half(0), 1);
        assert_eq!("500".parse::<u64>().map(super::half).unwrap(), 250);
    }
}
