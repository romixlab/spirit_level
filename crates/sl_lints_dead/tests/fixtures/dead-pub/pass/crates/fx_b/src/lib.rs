//! The library's one user.
use fx_a::open::also_used;

pub fn go() {
    let _ = fx_a::used();
    let _ = fx_a::Reexported;
    let _ = also_used();
    println!("{}", fx_a::in_macro());
}
