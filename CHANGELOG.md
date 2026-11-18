# Changelog

## [Unreleased]

### Added

- Book: a page per Clippy lint, group and setting of the tiers; search finds it first by name (DOCS-11).
- Book: a tool lint's page links the tool's own documentation of the check (DOCS-11).

## [0.5.1] - 2026-11-18

### Added

- `[docs.book]`: `page-links` resolves links of pages another repo's book takes from their place in that book (DOCS-10).

## [0.5.0] - 2026-11-18

### Added

- `cls-3` (F0111, deny in `embedded`): a `service` module holds no `unwrap`, `expect`, `panic!` or `assert!` (CODE-13).

### Changed

- **Breaking:** `panic-in-library` has no `classes` parameter: delete it from a config. The `embedded` tier no longer narrows the lint (CODE-13).

### Fixed

- Comment lints report the right line after a string continued with `\` at a line end (AI-1).

## [0.4.39] - 2026-11-17

### Added

- `spl book docs` writes coloured check output into marked blocks of hand-written pages (BOOK-6).

## [0.4.38] - 2026-11-17

### Changed

- `ci-workflow` also accepts steps that run each command of the `lint` or `test` recipe (MAN-10).

## [0.4.37] - 2026-11-17

### Added

- `ci-workflow`: the repo has a CI workflow that runs `just lint` and `just test`, or Clippy and tests (MAN-10).
- `manifest-homepage`: the manifest sets `repository`, and `homepage` is the book's `site_url`; `--fix` writes it (MAN-11).
- `readme-badges`: the README has badges for the book, CI and each published crate, linking to them (MAN-12).
- `book-deploy`: a book with a `site_url` is built strict and published by one workflow with `actions/deploy-pages` (DOCS-9).

## [0.4.36] - 2026-11-17

### Added

- The book is published at https://spl.vhrd.tech; CI runs `just lint` and `just test` (BOOK-5).

## [0.4.35] - 2026-11-17

### Added

- `spl book sample`: a project's coloured check report as an SVG picture for a README (BOOK-4).

## [0.4.34] - 2026-11-15

### Changed

- Book lints (nav-entries, orphan-pages, page-title, admonition-syntax, zensical-keys) also read `mkdocs.yml` (DOCS-2).

## [0.4.33] - 2026-11-15

### Added

- `[docs.site]`: `spl pages` and the reading lints read a built zola or Zensical site's HTML (DOCS-8).

## [0.4.32] - 2026-11-15

### Fixed

- `BLESS=1` rewrote every fixture's expect.toml and could corrupt `[params]` tables (TESTKIT-1).

## [0.4.31] - 2026-11-15

### Changed

- spl's commands register themselves per module, so `command-registry` passes on spl itself (CLI-7).

## [0.4.30] - 2026-11-15

### Added

- `ui-tooltip-repeats-label`: a tooltip that only repeats its control's label (UI-3).
- `ui-locale-strings`, opt-in: a literal string passed to a UI text call (UI-4).
- `ui-hover-text`, candidate: an icon-only or glyph button with no tooltip (UI-5).

## [0.4.29] - 2026-11-15

### Added

- `ui-unwrap-external-data` (U0014): `unwrap`/`expect` on a file, parse or connection in a UI crate (UI-2).
- `egui-panel-api` (U0015): egui panel and App API the version in use replaced (UI-6).
- `egui-checklist-menu` (U0016): a `menu_button` with several checkboxes closes on each click (UI-7).

## [0.4.28] - 2026-11-15

### Added

- `build-script-clock` (C0040): build.rs reads no clock and resolves git through the real git dir (CODE-5).

## [0.4.27] - 2026-11-15

### Added

- `fuzz-corpus-test` (M0031): every fuzz target has a corpus tracked in git and a test naming it (MAN-9).

## [0.4.26] - 2026-11-15

### Added

- `command-registry` (C0039): a CLI past `size` subcommands registers them per module with inventory or linkme, estimate (CODE-6).

## [0.4.25] - 2026-11-15

### Added

- `cli-no-abbreviations` (C0038): a CLI runs no subcommand from its unique prefix (CODE-12).

## [0.4.24] - 2026-11-15

### Added

- `spl book stamp`: a book's footer build stamp and reader-clock age script, for any project (BOOK-3).

## [0.4.23] - 2026-11-15

### Added

- `cli-secret-flags`: a flag whose long name looks like it takes a secret as its value, candidate confidence (CLI-6).

## [0.4.22] - 2026-11-15

### Added

- `url-tracking-params`: a link in docs or a comment carries no tracking query parameter (PROSE-5).

## [0.4.21] - 2026-11-15

### Added

- `tool-shadowed`: an out-of-range tool shadowing an in-range one on PATH is named, not told to reinstall (TOOL-5).

## [0.4.20] - 2026-11-15

### Added

- `large-blobs`: a git-tracked file over a size limit outside an allow list (AI-3).

## [0.4.19] - 2026-11-15

### Added

- `page-links`: a link between pages resolves to a relative `.md` path, never `.html` or absolute, book or not (DOCS-6).

## [0.4.18] - 2026-11-15

### Added

- `zensical-keys` (D0018): zensical.toml names only keys Zensical reads (DOCS-2).

## [0.4.17] - 2026-11-15

### Added

- `theme-feature-conflicts` (D0018): a Zensical book's theme features do not conflict, and `navigation.instant` has a `site_url` (DOCS-7).

## [0.4.16] - 2026-11-15

### Added

- `readme-index-clash` (D0018): a docs folder holding both `README.md` and `index.md` denies (DOCS-5).

## [0.4.15] - 2026-11-15

### Added

- `book-build-stamp`: a Zensical book's footer shows version, commit and build age (DOCS-4).

## [0.4.14] - 2026-11-15

### Added

- `abbreviation-first-use` (D0012), `term-before-definition` (D0013), `term-synonyms` (D0014): candidates on the words a page uses (DOCS-3).
- `passive-voice` (D0015) and `reading-ease` (D0016): a guide's passive share and Flesch score against a limit (DOCS-3).
- `spl pages` shows each page's passive share, unspelled abbreviations, early terms and synonyms (DOCS-3).

## [0.4.13] - 2026-11-15

### Added

- Pass fixtures for Bevy's dylib-feature pattern, so unused-deps and feature lints stay silent on it (TOOL-1).

## [0.4.12] - 2026-11-15

### Added

- Map key `licence` per layer or crate; `map-licence` checks it, `workspace-inheritance` accepts it (GRAPH-5).

## [0.4.11] - 2026-11-15

### Added

- Map key `workspace`: `map-complete` and the map lints also check nested workspaces the map names, like `tools/` (GRAPH-6).

## [0.4.10] - 2026-11-15

### Changed

- hakari and geiger are always linked into `spl`; the `hakari`/`geiger` cargo features are gone (TOOL-4).
- `rust-version` moves to 1.91: hakari and guppy need it.

## [0.4.9] - 2026-11-15

### Fixed

- `deny-licenses` without `deny.toml` reports one finding, not one per crate (TOOL-3).

## [0.4.8] - 2026-11-15

### Added

- `crate-folder-layer` (G0011): a crate's folder is its map layer, then its new `discipline` key; never deeper (GRAPH-4).
- `crate-folder-size` (S0006): a folder past its limit of crate folders is reported, flat or per layer (SIZE-3).

### Changed

- `crate-dir-name` documents and tests crates in layer and discipline folders (MAN-1).

## [0.4.7] - 2026-11-15

### Changed

- `core-crate-deps` lets `vocabulary` name workspace crates too, and exempts the workspace-hack package (GRAPH-1).

## [0.4.6] - 2026-11-15

### Changed

- ELF lints are skipped, not failed, in a workspace without a firmware binary they are enabled on (ENG-11).

## [0.4.5] - 2026-11-15

### Fixed

- `layer-edges` exempts edges to the workspace-hack package that `.config/hakari.toml` names (GRAPH-1).

## [0.4.4] - 2026-11-15

### Changed

- Usage records become opt-in, off by default: `SPL_USAGE_DIR=DIR` or `SPL_USAGE=on` enables them (CLI-5).

## [0.4.3] - 2026-11-15

### Added

- Usage records: every run counts its command and flags locally; `spl usage` shows them, `--unused` the never-used (CLI-2).

### Changed

- Building spl needs Rust 1.89 (`rust-version`), for the usage records' file lock (CLI-2).

## [0.4.2] - 2026-11-15

### Added

- `[map-file] foreign`: map keys another tool owns are left to it; unknown keys suggest the nearest spl key (CFG-6).
- `[[crate-tiers]]` and map key `tiers`: tiers such as embedded or egui on chosen crates only (CFG-7).
- Lint `contract-snapshot` (M0030): contract crates export VERSION and keep unchanged schema snapshots in git (MAN-4).

### Fixed

- `taplo-fmt` no longer reads "did not run" when taplo loses its log lines under load (TOOL-1).

## [0.4.1] - 2026-11-15

### Fixed

- `spl tools needed` exits 1 when a tool is outside its tested range, not only when one is missing (TOOL-2).

## [0.4.0] - 2026-11-15

### Added

- `spl tools needed`: the tools, toolchains and targets a project needs; table, `--spec`, `--components`, `--json`, `--missing`, `--all` (TOOL-2).
- `tool-version` (X0034, deny from small): a tool outside its tested version range runs, its results reported untested (TOOL-2).
- `spl check --allow-warn-not-run`: warn-level lints that cannot run are listed but do not fail the run (TOOL-2).

### Changed

- Tools are found on PATH only, their version read; a missing tool's reason names the `crate@version` to install (TOOL-2).
- **Breaking:** warn-level lints that cannot run now fail the run with exit 3, like deny-level ones (TOOL-2).

### Removed

- **Breaking:** `spl tools install` and the tool cache; install with `spl tools needed --spec | xargs cargo binstall -y` (TOOL-2).
- `spl tools list`: `spl tools needed --all` lists every tool (TOOL-2).

## [0.3.1] - 2026-11-14

### Added

- `exact-units-quantity` (C0033, warn from small): unit- and quantity-named plain numbers get their exact_units type (CODE-11).
- `exact-units-bounds` (C0035, warn from small): exact_units values without bounds whose range the crate states (CODE-11).
- `exact-units-plain-number` (C0034): indices, counts, sequence numbers and `Duration` as exact_units types; tier `units-strict` (CODE-11).
- `exact-units-in-bodies` (C0036): unit-named locals, literals, casts, float maths, comparisons, conversions; tier `units-paranoid` (CODE-11).

### Changed

- `unit-suffix` runs only where `exact-units-quantity` is turned off: the unit goes into the type (CODE-11).

## [0.3.0] - 2026-11-14

### Added

- `spl panics --elf`: each panic site of a release ELF with function, kind, place and class; JSON, `--ack` (TIER-6).
- `pan-1` (F0110, off): every panic site of the release ELF acknowledged with a reason in `panics.toml` (TIER-6).
- `compound-assert` (C0030): `assert!(a && b)` outside tests; warn in big and embedded (CODE-10).
- `assert-message` (C0031): `assert!` or `debug_assert!` without a message; warn in big and embedded (CODE-10).
- `float-partial-cmp-unwrap` (C0032): `partial_cmp(..).unwrap()` panics on NaN; warn from small (CODE-10).
- `cls-1` (F0106, warn) and `cls-2` (F0107, deny): task files have a class; `service` modules deny Clippy's panic lints (CODE-10).
- `flt-11` (F0108, warn): panic and HardFault handlers call `safe_state()` first (CODE-10).
- `flt-12` (F0109, warn): the crash record's `boots_since` is read before the first output is enabled (CODE-10).
- `panic-in-library` in the embedded tier: deny on `service` modules, `expect`, `unreachable!` and `assert!` included (CODE-10).
- Clippy sets: `debug_assert_with_mut_call` from small, `missing_panics_doc` warn from medium (CODE-10).
- `[embedded.classes]` table: task classes `control`, `service`, `protocol`, `boot` per module glob (CODE-10).

### Changed

- **Breaking:** `panic-in-library` counts bare `unwrap`, `panic!`, `todo!`, `unimplemented!`, message-less `unreachable!`; `expect` and `*_tests.rs` no longer (CODE-10).

## [0.2.8] - 2026-11-14

### Added

- Board lints `brd-1`..`brd-7` (F0099-F0105) check `board.toml` against chip data and PCB nets, in the embedded tier (TIER-5).
- `[board]` table: where descriptions and stm32-data chip JSON are (TIER-5).
- `sl_board_model`, the `board.toml` schema and loader, moved from embedded_bedrock; published (TIER-5).

## [0.2.7] - 2026-11-14

### Added

- Tiers `ui` and `egui`: 13 UI lints (U0001-U0013) and the "UI practices" catalogue page (UI-1).
- Crate map keys `ui` and `theme` mark UI crates and the theme crate (UI-1).

## [0.2.6] - 2026-11-14

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
