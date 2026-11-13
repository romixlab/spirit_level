//! A library whose public items are all named by its user, some only inside macros or signatures.
mod inner;

pub use inner::Reexported;

pub struct Ret;

pub fn used() -> Ret {
    Ret
}

pub fn in_macro() -> u32 {
    1
}

pub mod open {
    pub struct Opened;

    pub fn also_used() -> Opened {
        Opened
    }
}

pub(crate) fn private() {}
