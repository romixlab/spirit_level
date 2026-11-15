// Mentioning SystemTime::now() and .git/HEAD in a comment is not a call or a path literal.
fn main() {
    println!("cargo:rustc-env=FX_REV=unknown");
    println!("cargo:rerun-if-changed=build.rs");
}
