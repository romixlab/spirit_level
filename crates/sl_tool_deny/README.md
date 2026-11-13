# sl_tool_deny

The cargo-deny adapter of spirit_level (layer lints): runs `cargo deny --format json check <check>` for each enabled check and reports its diagnostics under `deny-advisories`, `deny-bans`, `deny-licenses` and `deny-sources`. cargo-deny is run, not linked: its library is the internals of its binary, and linking it costs 143 crates and C (ring, zstd-sys).

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
