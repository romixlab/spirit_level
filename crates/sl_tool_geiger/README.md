# sl_tool_geiger

The geiger adapter of spirit_level (layer lints). The `geiger` library (the scanner of cargo-geiger, without cargo) is linked behind the `link` feature (spirit_level's `geiger` feature): `geiger-unsafe-deps` counts the `unsafe` functions, expressions, impls, traits and methods in every dependency's sources. Without the feature the lint reports that it did not run.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
