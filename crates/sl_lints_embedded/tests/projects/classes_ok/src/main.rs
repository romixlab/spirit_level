// The assertion policy done right: every task file classified, the service module denies the panicking lints,
// both handlers reach the safe state first, the boot counter is read before the first output
#![no_std]
#![no_main]

mod led;
mod power;

use embassy_gpio::{Level, Output, Speed};

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let crash = bedrock_crash::boot(reset_cause(), build_id());
    if crash.map_or(false, |c| c.boots_since < 3) {
        return;
    }
    let gate = Output::new(p.PA8, Level::Low, Speed::Low);
    spawner.must_spawn(power::control(gate, 1));
    spawner.must_spawn(led::blink(500));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    board::safe_state();
    bedrock_crash::capture_panic(info, 0);
    bedrock_crash::reset()
}

#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    let exc_return = bedrock_crash::exc_return!();
    board::safe_state();
    unsafe { bedrock_crash::capture_fault(ef, exc_return, 0) };
    bedrock_crash::reset()
}
