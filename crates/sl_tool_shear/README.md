# sl_tool_shear

The cargo-shear adapter of spirit_level (layer lints), the alternative to cargo-machete: runs `cargo shear --format json` and reports unused dependencies (also unused `[workspace.dependencies]` entries) under `shear-unused-deps` and dependencies in the wrong table under `shear-misplaced-deps`. It is run, not linked: its library is the internals of its binary and does not build outside its own `Cargo.lock`.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
