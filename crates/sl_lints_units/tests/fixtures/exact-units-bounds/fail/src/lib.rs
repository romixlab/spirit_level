//! exact_units values whose range the crate knows, without bounds.

pub struct Fixed<U, P, T>(pub U, pub P, pub T);
pub struct Index<Of, T>(pub Of, pub T);
pub struct Percent<T>(pub T);
pub struct Kelvin;
pub struct Centi;
pub struct Slot;

pub const DIE_MAX: i16 = 12_500;

pub struct State {
    pub level: Percent<u8>,
    /// The slot, `0..8`.
    pub slot: Index<Slot, u8>,
    pub die: Fixed<Kelvin, Centi, i16>,
}

pub fn level(raw: u8) -> u8 {
    let level = raw.clamp(0, 100);
    level
}

pub fn check(die: i16) -> Result<(), ()> {
    if die > DIE_MAX {
        return Err(());
    }
    if die < -4_000 {
        return Err(());
    }
    Ok(())
}
