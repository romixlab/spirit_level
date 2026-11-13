# sl_tool_cargo

Checks that run cargo itself (layer lints): `cargo-msrv` builds each crate with the toolchain of its `rust-version`, `cargo-no-std` and `cargo-wasm` build the crates the map marks `no_std` or `wasm` for their targets, `cargo-doc` builds the docs and reports every rustdoc warning. Compiler messages are read from `--message-format json`.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
