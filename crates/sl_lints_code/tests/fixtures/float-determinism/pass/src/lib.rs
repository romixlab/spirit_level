//! x
pub fn f(x: f64) -> f64 { libm::sin(x) + x.abs() + x * 2.0 }
