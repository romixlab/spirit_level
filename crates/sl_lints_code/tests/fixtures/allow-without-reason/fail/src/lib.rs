//! x
#[allow(dead_code)]
fn a() {}
#[cfg(test)]
mod tests {
    #[expect(unused)]
    fn t() {}
}
#[cfg_attr(not(test), allow(dead_code))]
fn b() {}
#[cfg_attr(feature = "x", derive(Debug), expect(unused))]
struct S;
