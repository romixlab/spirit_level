//! x
pub fn sort(xs: &mut [f32]) {
    xs.sort_by(|a, b| a.total_cmp(b));
}
pub fn cmp(a: f64, b: f64) -> Option<std::cmp::Ordering> {
    a.partial_cmp(&b)
}
pub fn or_equal(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}
#[cfg(test)]
mod tests { #[test] fn t() { assert!(1.0f32.partial_cmp(&2.0).unwrap().is_lt()); } }
