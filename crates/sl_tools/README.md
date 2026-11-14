# sl_tools

What spirit_level's tool adapters share (layer model):

- `tools.toml`: every external tool: the crate or rustup component, the exact version to install, the tested version range, how to read its version.
- `find`: a tool's executable on PATH; `detect` also reads its version and compares it with the tested range.
- `rustup`: whether a toolchain or a target is installed.
- `run`: a tool started in the project with the outer cargo's environment removed.
- `cargo_json`: the compiler messages of `cargo ... --message-format json` as spans and codes.

It may depend on `sl_core` and `sl_model`.
