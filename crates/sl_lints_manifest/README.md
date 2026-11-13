# sl_lints_manifest

The manifest and repo-file lints of spirit_level: package metadata (description, licence, edition, rust-version, publish), workspace conformity (inheritance of `[workspace.package]` keys, `[lints] workspace = true`, dependencies from `[workspace.dependencies]`, `default-features = false`), the dependency graph's form (no `-sys` or C build steps, no duplicate versions, no git pins, no path dependencies outside the workspace), build profiles, the pinned toolchain, the repo's rule files and changelog, the justfile and documented features.

Each lint's documentation is in `docs/<id>.md`; its fixtures in `tests/fixtures/<id>/`. It depends on `sl_analysis`, `sl_core` and `sl_model` only.
