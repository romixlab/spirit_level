fn main() {
    let now = chrono::Utc::now();
    println!("cargo:rustc-env=FX_BUILD_TIME={now}");
    println!("cargo:rerun-if-changed=.git/HEAD");
}
