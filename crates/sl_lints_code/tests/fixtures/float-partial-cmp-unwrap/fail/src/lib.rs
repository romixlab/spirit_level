//! x
pub fn sort(xs: &mut [f32]) {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
pub fn max(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).expect("not NaN")
}
