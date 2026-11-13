# sl_lints_modules

The module-graph lint of spirit_level, `module-graph-scc`: per crate, the graph of its modules from `crate::`, `super::` and `self::` paths and `use` items, the share of code lines in its biggest strongly connected component, and the references that would cut that component cheapest.

Its documentation is in `docs/module-graph-scc.md`, its fixtures in `tests/fixtures/module-graph-scc/`. It depends on `sl_analysis` and `sl_core` (and syn) only.
