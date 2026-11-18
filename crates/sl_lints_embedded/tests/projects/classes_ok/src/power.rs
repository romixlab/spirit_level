#[embassy_executor::task]
pub async fn control(mut gate: embassy_gpio::Output<'static>, period_ms: u64) {
    // a control module may panic: the handler turns the outputs off first
    assert!(period_ms > 0, "control period");
    loop {
        gate.set_high();
        embassy_time::Timer::after_millis(period_ms).await;
    }
}
