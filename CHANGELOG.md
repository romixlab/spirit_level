# Changelog

## [Unreleased]

## [0.1.0] - 2026-11-13

### Added

- Lint format: `LintMeta`, tiers that extend, overrides with reason and expiry, scoped exceptions (CFG-1..4).
- Engine: needs, levels, waivers, exact fixes, text and JSON reports, exit codes (ENG-1..5).
- `spirit-level` CLI: check, explain, lints, tiers, config, init, book (CLI-1).
- Manifest lints: 24 lints on metadata, workspace conformity, dependencies, profiles, repo files (MAN-1).
- Size lints: crate, test, module and function lines; cyclomatic complexity (SIZE-1).
- Shared-state lints: interior-mutable statics, static mut, thread-locals, lock().unwrap(), guards across await or I/O (STATE-1).
- Graph lints against the crate map: layers, forbid, external form, _core, cycles, crate table, unsafe list (GRAPH-1).
- Source lints: todo macros, panics and prints in libraries, allow reasons, determinism, CLI subprocesses (CODE-1).
- Tiers `small`, `small-workspace`, `medium`, `big` (TIER-1).
- Book generated from lint metadata (BOOK-1).
