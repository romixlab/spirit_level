# sl_lints_errors

The error-type lint of spirit_level, `error-enum-growth`: error enums with more variants than a limit and no structure (no variant wraps another error type or groups variants in an enum of their own).

Its documentation is in `docs/error-enum-growth.md`, its fixtures in `tests/fixtures/error-enum-growth/`. It depends on `sl_analysis`, `sl_core` and `sl_model` only.
