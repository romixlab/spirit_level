//! Bounded values, and values the crate says nothing about.

pub struct Fixed<U, P, T, B>(pub U, pub P, pub T, pub B);
pub struct Millis<T>(pub T);
pub struct Range<const MIN: i64, const MAX: i64>;
pub struct Ratio;
pub struct Centi;

pub struct State {
    pub level: Fixed<Ratio, Centi, u8, Range<0, 100>>,
    /// Version 1.2 of the protocol.
    pub uptime: Millis<u32>,
}

pub fn level(raw: u8) -> u8 {
    let level = raw.clamp(0, 100);
    level
}
