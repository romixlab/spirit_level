//! The library's one user.
pub fn go() {
    let _ = fx_a::used();
    let _ = fx_a::Reexported;
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        fx_a::used_in_test_only();
    }
}
