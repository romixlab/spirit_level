## Why

A crate the map marks `no_std = true` promises to build without the standard library, for firmware and other bare-metal targets. One `use std::` or a dependency with its std feature on breaks that, and a host build never shows it. This lint builds each such crate for targets without std.

```toml
[map.crates]
wire_core = { layer = "core", no_std = true }

[lints.cargo-no-std]
level = "deny"
reason = "the firmware links wire_core"
targets = ["thumbv7em-none-eabihf", "riscv32imac-unknown-none-elf"]
```

## How to fix

Use `core` and `alloc` instead of `std`, and turn off the std feature of dependencies (`default-features = false`). Each target must be installed (`rustup target add thumbv7em-none-eabihf`).
