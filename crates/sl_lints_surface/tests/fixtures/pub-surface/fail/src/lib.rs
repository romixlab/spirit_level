//! Everything public: 8 items and 2 methods in 20 code lines.
pub mod shapes {
    pub struct Point(pub i32, pub i32);

    impl Point {
        pub fn new() -> Point {
            Point(0, 0)
        }

        pub fn x(&self) -> i32 {
            self.0
        }
    }

    pub const ORIGIN: Point = Point(0, 0);
}

pub use shapes::Point;

pub fn a() {}
pub fn b() {}
pub fn c() {}
pub type Num = i32;
pub static ZERO: i32 = 0;
pub trait Shape {}
