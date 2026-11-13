use crate::Thing;

pub fn middle() -> u32 {
    Thing::new().0 + super::c::deep()
}
