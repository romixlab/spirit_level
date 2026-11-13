//! x
#[allow(dead_code)]
fn a() {}
#[cfg(test)]
mod tests {
    #[expect(unused)]
    fn t() {}
}
