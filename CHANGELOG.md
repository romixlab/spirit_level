# Changelog

## [Unreleased]

### Added

- Tool adapters: clippy, deny, machete, shear, rustfmt, typos, taplo, semver-checks, audit, udeps, hakari, geiger (TOOL-1).
- Cargo checks: MSRV build, no_std and wasm builds of map crates, cargo doc warnings (TOOL-1).
- `spirit-level tools list|install`: pinned tool versions built into a cache keyed on rustc (TOOL-1).
- `spirit-level tools install --needed`: tools, toolchains and targets the enabled lints need (TOOL-1).
- Tool fixtures run offline against vendored crates; advisory-database cases run with `SPIRIT_LEVEL_NETWORK=1` (TOOL-1).
- Book page External tools: library or executable per tool, link cost, commands and lint mapping (TOOL-1).

## [0.1.2] - 2026-11-13

### Added

- `module-graph-scc`: share of a crate's code in its biggest module cycle, and the cheapest cut (CODE-2).
- `dead-pub`: pub items of workspace libraries no other crate names (CODE-2).
- `error-enum-growth`: big error enums that wrap no other error (CODE-2).
- `pub-surface`: public items per 1000 code lines of each library (CODE-2).

## [0.1.1] - 2026-11-13

### Added

- Licence files (MIT, Apache-2.0), `CONTRIBUTING.md` with dependency policy.
- Prose lints: CHANGELOG, commit and AGENTS.md limits, sentence and paragraph length (PROSE-1).
- Prose lints: filler, narration, emoji, restating comments; agent files link and match the lints (PROSE-2).
- Stop-gap and agent-hygiene lints: markers, commented-out code, near-duplicates, secrets, transcripts (AI-1).
- Tier `ai` beside the size tiers: `extends = ["medium", "ai"]` (TIER-3).

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
