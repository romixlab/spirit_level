//! A library with public items nobody else names.
mod inner;

pub use inner::Reexported;

pub struct Ret;

pub fn used() -> Ret {
    Ret
}

pub fn dead() {}

pub mod open {
    pub fn also_dead() {}
}

pub(crate) fn private() {}

pub fn used_in_test_only() {
    private();
}
