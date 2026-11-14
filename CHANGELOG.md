# Changelog

## [Unreleased]

### Added

- CLI lints C0020-C0029: spl builds each CLI and checks help, examples, completion, colour, version and exit codes (CODE-9).
- Crate map key `cli` and the `[cli]` table: which crates are CLIs and how they are probed (CODE-9).

### Fixed

- `spl --help` opens with spl's own description, not vc_ui ColorFlag's rustdoc (CLI-4).

## [0.2.5] - 2026-11-14

### Added

- Reading lints D0001-D0007: prose runs, reading time, heading length, sections, code walls, image text, code languages (DOCS-1).
- `[docs]` table: which Markdown files are pages, and which are reference pages (DOCS-1).
- `spl pages`: words, reading time, longest prose run, headings, figures, links per page; JSON to diff runs (DOCS-1).
- Zensical book lints D0008-D0011: navigation entries, orphan pages, page titles, admonition syntax (DOCS-2).

## [0.2.4] - 2026-11-14

### Added

- `spl check --landing` (`SPIRIT_LEVEL_LANDING`) and `Spirit-Level: allow(...)` commit trailers as waivers (ENG-10).
- fixme-before-land (A0011): FIXME is a candidate on the branch, a finding when the branch lands (AI-2).
- fixes-tag (P0017): a `Fixes:` trailer names a resolving hash of 12+ digits and the culprit's subject (PROSE-3).
- fix-without-test (P0018): a commit that fixes code adds a test, or says why not (PROSE-3).
- sized-ints-in-contracts (C0017): serde types of a crate the map marks `contract` use sized integers (CODE-8).
- unit-suffix (C0018): numeric fields and constants named for a time or size carry a unit (CODE-8).
- unbounded-read (C0019): `read_to_end` and `read_to_string` are bounded with `take()`; candidates (CODE-8).
- commit-tested-line (P0019): each branch commit carries a `Tested:` trailer (PROSE-4).
- commit-mixed-move (P0020): a commit that moves a Rust file edits only its paths (PROSE-4).
- orphan-source-file (C0016): every `.rs` file under `src/` is reached by a `mod` declaration (CODE-7).

### Changed

- The small tier's Clippy set adds `ignore_without_reason`: an `#[ignore]` says why (CODE-8).
- function-lines (S0003) takes `target-lines`: functions between target and limit are warnings, so the limit can tighten (SIZE-2).
- stop-gap-comments (A0010) no longer reports FIXME; fixme-before-land does (AI-2).

### Fixed

- Module trees resolve `#[path]` in a non-mod.rs file from that file's directory, as rustc does (CODE-7).

## [0.2.3] - 2026-11-14

### Added

- `spl dependents`: dependent repos and the crates in them to retest; `[dependents]` turns it on (DEP-1).

### Changed

- cognitive-complexity-threshold 25 -> 20; module-lines (S0004) 2000 -> 1000.

## [0.2.2] - 2026-11-14

### Added

- Choice parameters: a value outside the listed strings is a configuration error (MAN-2).
- profile-no-lto (M0026): dev, test and deploy profiles build without LTO, inherited settings included (MAN-3).
- patch-outside-workspace (M0027): no committed `[patch]` or `[replace]` takes a path outside the workspace (MAN-3).
- test-binaries (M0028): one integration-test binary per crate, `tests/it/main.rs` (MAN-3).
- macro-compile-fail (M0029): every proc-macro crate has a trybuild or ui_test crate (MAN-3).
- build-info-env (C0014): library code reads git SHA and build time at run time, not through `env!` (CODE-4).
- test-support-gated (C0015): fakes and testing modules compile only for tests or behind `test-support` (CODE-4).

### Changed

- git-dependency-pins (M0003) checks a per-project pin policy, separately for own repos (MAN-2).
- external-form (G0005) leaves rev and tag to git-dependency-pins (MAN-2).
- The medium tier's Clippy set sets `max-fn-params-bools = 1` (MAN-3).
- spirit_level_cli's integration tests run as one binary, `tests/it` (MAN-3).
- workspace-lints (M0024) accepts a copied lint table in crates the map marks unsafe (GRAPH-2).

### Fixed

- unsafe-allow-list (G0010) sees linkme elements and no_mangle, export_name, link_section (GRAPH-2).

## [0.2.1] - 2026-11-14

### Added

- Each tier declares its Clippy set; `tier-clippy-set` (M0025) checks `[lints.clippy]` and `clippy.toml` (TIER-4).

### Changed

- spirit_level runs the big tier's Clippy set itself (TIER-4).
- Multi-line help is indented in text output (TIER-4).
- function-complexity (S0002) limit 25 -> 15.
- Theme overrides folder renamed `docs-overrides/`.
- Tier TOMLs moved to `tiers/` at the repo root.

### Fixed

- Hakari fixtures resolve again (workspace-hack 0.1).
- Tool fixtures of two checkouts no longer share a temp copy.

## [0.2.0] - 2026-11-14

### Added

- `--color auto|always|never`; colour, error line, exit codes and completion from vhrd_cli (CLI-4).
- Stable lint codes (`S0001`) beside ids, in output, JSON, config, waivers and `explain` (ENG-9).
- `spl codes`: every code ever given; `--check` guards the registry `codes.toml` (ENG-9).
- Source excerpts per group with the range marked; `--excerpts N` (ENG-8).
- `--all` lists every place; `--group-by crate|file` (ENG-7).

### Changed

- **Breaking:** the binary is `spl`; `spirit-level` is gone (CLI-3).
- Book lint pages are named after codes (`lints/S0001.md`), with the id as anchor (ENG-9).
- Spans carry an end column; source and manifest lints report exact ranges (ENG-8).
- Text output groups findings by lint: deny first, by count, 5 places each, summary last (ENG-7).

### Fixed

- `allow-without-reason` finds `allow` and `expect` inside `cfg_attr` (CODE-3).

## [0.1.5] - 2026-11-13

### Added

- Logo, icons and social card from `assets/logo/logo.typ`; in the book and README (BOOK-2).

## [0.1.4] - 2026-11-13

### Added

- Tier `embedded`: bedrock_lint's 97 firmware checks, `--elf`, migration page (TIER-2).

### Fixed

- `stop-gap-comments` takes `markers` and `phrases` from crate-scoped exceptions (AI-1).

## [0.1.3] - 2026-11-13

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
