# sl_tool_typos

The typos adapter of spirit_level (layer lints): runs `typos --format json` and reports each misspelling under `typos-misspelling`, with an exact fix when typos knows exactly one correction. typos is run, not linked: the `typos` library has the checker but not the configuration (`typos.toml`), the file walk or the built-in dictionary policy, which live in the binary crate `typos-cli`.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
