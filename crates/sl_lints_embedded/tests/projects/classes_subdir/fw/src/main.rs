// A firmware below the workspace root: the class globs match paths relative to fw/
#![no_std]
#![no_main]

mod led;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    spawner.must_spawn(led::blink());
}
