## Why

C libraries bring memory bugs Rust cannot see and builds that need a C toolchain, system libraries and cross-compilers (static musl, Windows, embedded). The house rule is pure-Rust crates: rustls not OpenSSL, gix not git2.

The lint walks what each checked crate builds, features resolved for it alone ([what each crate builds](../configuration.md#what-each-crate-builds)), with normal, build and its own dev edges, and reports crates that are a C build step (`cc`, `bindgen`, `cmake`, `pkg-config`, `vcpkg`, `autotools`) or link a native library (`links` in their manifest). Pure-Rust binding crates such as `windows-sys` declare neither and are not reported.

## How to fix

Find who pulls it with `cargo tree -i NAME -e normal,build`, and pick a pure-Rust crate or turn off the feature that pulls it. When it must stay, list it in the parameter `allow` and write the reason in AGENTS.md.
