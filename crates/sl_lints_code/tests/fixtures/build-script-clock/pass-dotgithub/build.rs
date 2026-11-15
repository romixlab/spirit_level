fn main() {
    println!("cargo:rerun-if-changed=.github/workflows/ci.yml");
    println!("cargo:rerun-if-changed=.gitignore");
}
