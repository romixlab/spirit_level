# sl_tool_semver

The cargo-semver-checks adapter of spirit_level (layer lints): runs `cargo semver-checks -p <crate>` for each stable crate (the crate map's `stable = true`, else the published ones) and reports each failed check under `semver-checks-breaking`. It is run, not linked: the library costs 308 crates and C (ring), and its report gives the required bump but not the failed checks, which only its text output lists.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
