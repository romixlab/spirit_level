## Why

Every `unsafe` block in a dependency is code the compiler does not check for memory safety, linked into the project. Most are fine and some are needed (a hash map, an allocator), but which dependencies use it, and how much, is worth knowing and choosing. geiger counts `unsafe` functions, expressions, impls, traits and methods in each dependency's sources.

The count covers every `.rs` file of the package, used in this build or not, so it is an estimate; cargo-geiger counts only the files the build compiled. The packages are those the checked crates build, features resolved for each alone ([what each crate builds](../configuration.md#what-each-crate-builds)), without dev-dependencies.

```toml
[lints.geiger-unsafe-deps]
level = "warn"
reason = "unsafe dependencies are chosen"
allow = ["hashbrown", "memchr", "libc"]
```

## How to fix

Replace the dependency with one that forbids unsafe code, or accept it in `allow` with the reason in the review that chose it.
