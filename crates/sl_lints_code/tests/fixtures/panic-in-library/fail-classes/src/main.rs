mod led;
mod power;
fn main() { power::set(Some(1)); led::blink(2); }
