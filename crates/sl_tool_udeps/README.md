# sl_tool_udeps

The cargo-udeps adapter of spirit_level (layer lints): runs `cargo +nightly udeps --output json` and reports each dependency the compiler never used under `udeps-unused-deps`. It is run, not linked: it reads nightly-only compiler output and builds on the `cargo` crate (libgit2, curl).

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
