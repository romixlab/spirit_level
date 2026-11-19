## Why

A pinned `rust-toolchain.toml` is bypassed by every command that chooses its own toolchain: `cargo +nightly fuzz run` in a justfile, `toolchain: nightly` or `dtolnay/rust-toolchain@stable` in a CI workflow. Those run with whatever nightly or stable the PC or the runner has that day.

The lint reads every justfile and every workflow under `.github/workflows` and checks each toolchain they choose against the pair of [toolchain-pinned](M0019.md) (its `stable` and `nightly` parameters):

- `cargo +X`, `rustc +X`, `rustdoc +X`, `rustup run X`, `--toolchain X`;
- in a workflow, `toolchain: X` and `dtolnay/rust-toolchain@X`;
- in a justfile, `+{{ nightly }}` is checked at the variable's assignment.

## How to fix

Keep the nightly in one variable at the top of the justfile:

```just
# The pinned nightly of the fuzz and miri recipes; rust-toolchain.toml pins the stable one
nightly := "nightly-2026-11-19"

fuzz target:
    cargo +{{ nightly }} fuzz run {{ target }}
```

In a workflow, name the dated nightly or the pinned release (`dtolnay/rust-toolchain@1.100.0`). `spl check --fix` writes the expected toolchain in place of each one found.
