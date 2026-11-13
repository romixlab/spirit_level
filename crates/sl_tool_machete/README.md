# sl_tool_machete

The cargo-machete adapter of spirit_level (layer lints): runs `cargo machete --skip-target-dir` and reports each unused dependency under `machete-unused-deps`. cargo-machete has no library target, so it is run; its output is plain text in a fixed shape (`<crate> -- <manifest>:` followed by tab-indented dependency names).

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
