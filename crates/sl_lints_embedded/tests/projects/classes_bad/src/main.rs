// The assertion policy not followed: a task file without a class, a service module without the deny set, the
// handlers capture before the safe state, an output enabled before the boot counter is read
#![no_std]
#![no_main]

mod led;
mod power;
mod usb;

use embassy_gpio::{Level, Output, Speed};

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let gate = Output::new(p.PA8, Level::Low, Speed::Low);
    let crash = bedrock_crash::boot(reset_cause(), build_id());
    if crash.map_or(false, |c| c.boots_since < 3) {
        return;
    }
    spawner.must_spawn(power::control(gate));
    spawner.must_spawn(led::blink());
    spawner.must_spawn(usb::serve());
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    bedrock_crash::capture_panic(info, 0);
    board::safe_state();
    bedrock_crash::reset()
}

#[exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    let exc_return = bedrock_crash::exc_return!();
    unsafe { bedrock_crash::capture_fault(ef, exc_return, 0) };
    bedrock_crash::reset()
}
