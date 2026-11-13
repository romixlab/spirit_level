# sl_tool_taplo

The taplo adapter of spirit_level (layer lints): runs `taplo fmt --check` on the TOML files git knows and reports each file that is not formatted, or does not parse, under `taplo-fmt`; the fix is the file as `taplo fmt` writes it. taplo is run, not linked: the `taplo` library formats, but the configuration (`taplo.toml` with include, exclude and per-path rules) lives in `taplo-common`, which needs a tokio runtime and an async environment (158 crates).

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
