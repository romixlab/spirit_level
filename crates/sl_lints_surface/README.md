# sl_lints_surface

The public-surface lint of spirit_level, `pub-surface`: per library crate, the public items reachable from its root (through `pub mod` and `pub use`, with the public methods of public types) per 1000 code lines.

Its documentation is in `docs/pub-surface.md`, its fixtures in `tests/fixtures/pub-surface/`. It depends on `sl_analysis` and `sl_core` (and syn) only.
