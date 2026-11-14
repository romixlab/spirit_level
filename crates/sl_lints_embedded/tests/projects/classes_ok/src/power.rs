#[embassy_executor::task]
pub async fn control(mut gate: embassy_gpio::Output<'static>) {
    loop {
        gate.set_high();
        embassy_time::Timer::after_millis(1).await;
    }
}
