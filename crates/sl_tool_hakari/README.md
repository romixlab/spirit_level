# sl_tool_hakari

The hakari adapter of spirit_level (layer lints). hakari is a library meant for use (cargo-hakari is a thin CLI over it), so it is linked behind the `link` feature (spirit_level's `hakari` feature): `hakari-generate` is `cargo hakari generate --diff` with an exact fix, `hakari-manage-deps` is `cargo hakari manage-deps --dry-run`. Without the feature the lints report that they did not run.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
