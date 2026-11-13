# sl_tool_rustfmt

The rustfmt adapter of spirit_level (layer lints): runs `cargo fmt --all --check --message-format short` and reports each file that is not formatted under `rustfmt-unformatted`. rustfmt is run, not linked: its library needs `rustc_private` on nightly.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
