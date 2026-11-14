//! A stand-in for exact_units: the fixture needs the dependency, not the crate.

pub struct Millis(u32);

impl Millis {
    pub const fn get(self) -> u32 {
        self.0
    }
}
