//! x
pub struct Plugin(pub &'static str);
inventory::collect!(Plugin);
inventory::submit! { Plugin("hello") }
