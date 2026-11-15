//! Stands for the inventory crate: `submit!` takes an entry and expands to nothing

#[macro_export]
macro_rules! submit {
    ($($entry:tt)*) => {};
}
