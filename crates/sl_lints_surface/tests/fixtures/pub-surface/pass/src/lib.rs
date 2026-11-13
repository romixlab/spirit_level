//! One public function; the rest stays inside.
mod shapes {
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

pub fn origin_x() -> i32 {
    shapes::ORIGIN.x() + shapes::Point::new().x()
}

fn a() {}
fn b() {}
fn c() {}

#[cfg(test)]
mod tests {
    pub fn helper() {
        super::a();
        super::b();
        super::c();
    }
}
