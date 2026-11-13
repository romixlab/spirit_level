//! A chain a -> b -> c with a re-export at the root, which modules use without making a cycle.
mod a;
mod b;
mod c;

pub use c::Thing;

pub fn run() -> u32 {
    a::start()
}
