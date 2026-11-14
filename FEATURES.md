# Features

Status: ✅ done · 🚧 in progress · 📋 planned · 🐛 bug. Ids are stable; never renumber or reuse.

## CFG - configuration format

- ✅ `CFG-1` Tiers as TOML files that extend each other; a project extends one or several, or a shared file by path (`sl_config`).
- ✅ `CFG-2` Overrides per lint: level, parameters, required reason, optional expiry (date or release); expired fails.
- ✅ `CFG-3` Unknown keys, lint ids, parameters and wrong types fail loudly with file and line.
- ✅ `CFG-4` Scoped exceptions by crate or path glob: level and/or parameters, reason, expiry.
- ✅ `CFG-5` The crate map: layers with allow lists, crates with layer and flags, forbid, external, third-party.

## ENG - engine and output

- ✅ `ENG-1` Analyses behind one trait; needs (resolve, map, tool, nightly) reported as not run.
- ✅ `ENG-2` Source and TOML waivers with reason and optional expiry; `waiver-invalid`, `waiver-unused`.
- ✅ `ENG-3` Text and JSON reports; exit codes 0, 1, 2, 3.
- ✅ `ENG-4` Exact fixes applied with `--fix`.
- 📋 `ENG-5` SARIF output.
- 📋 `ENG-6` Baseline file: compare findings with a committed baseline.
- ✅ `ENG-7` Grouped text output: by lint (an adapter rule is its own group), crate or file; deny first, by count; 5 places, `--all` for every one; JSON stays complete and ungrouped (`sl_report::text`).
- ✅ `ENG-8` Exact spans (`end_column`) and a source excerpt per group drawn with annotate-snippets, `--excerpts N` (`sl_report::excerpt`).
- ✅ `ENG-9` Stable lint codes (group letter and four digits) in `LintMeta`, the registry `crates/spirit_level/codes.toml`, `spl codes --check`; codes work wherever ids do; adapter findings carry the tool's rule (`X-clippy:needless_return`).
- ✅ `ENG-10` Commit checks: the branch's commits read once per base (`sl_model::git`), `Spirit-Level: allow(...)` trailers as commit waivers, `--landing` runs ([commits](docs/commits.md)).

## CLI - the command

- ✅ `CLI-1` check, explain, lints, tiers, config, init, book, dependents; dynamic completion; version with SHA.
- 📋 `CLI-2` Usage records per subcommand.
- ✅ `CLI-3` The binary is `spl`; crate and repo stay spirit_level.
- ✅ `CLI-4` The generic CLI parts from vhrd_cli: `--version` (vc_build), `--color`, error line, exit codes (vc_ui), completion (vc_complete).

## DEP - dependents (crater-lite)

- ✅ `DEP-1` `spl dependents`: the repos that build on a library and the crates in them to retest when it changes, from their `Cargo.lock`; `[dependents]` in `spirit_level.toml` turns it on (`sl_dependents`).
- 📋 `DEP-2` `spl dependents --run`: run the listed tests against the changed library through a `[patch]` in a scratch copy of each dependent, never in its checkout; a failure fails the run.

## TIER - shipped tiers

- ✅ `TIER-1` small, small-workspace, medium, big.
- ✅ `TIER-2` embedded: every bedrock_lint check (port of embedded_bedrock's bedrock_lint): `sl_firmware`, `sl_lints_embedded`, `--elf`, migration page.
- ✅ `TIER-3` ai: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene; stands beside the size tiers.
- ✅ `TIER-4` Clippy set per tier (`[clippy]` in tier files, modelled on rerun's), checked by `tier-clippy-set` (M0025) in the project's `[lints.clippy]` and `clippy.toml`.
- ✅ `TIER-6` `elf-panic-inventory`: `spl panics --elf` lists every panic site of a release ELF (DWARF); `pan-1` holds each to `panics.toml` with a reason ([pan-1](docs/lints/F0110.md)).
- 📋 `TIER-7` `board-safe-state`: `board.toml` `safe` entries per output and `safe_state = "none"`; `flt-11` reads them instead of the function name alone.
- ✅ `TIER-5` `board-lints`: board descriptions (`board.toml`) in the embedded tier: `brd-1`..`brd-7` against the schema, chip data and PCB nets; `[board]` table; the schema and loader `sl_board_model`, moved from embedded_bedrock, published ([Board descriptions](docs/boards.md)).

## MAN - manifest and repo lints

- ✅ `MAN-1` 25 lints: metadata, inheritance, workspace lints and deps, default features, -sys, duplicates, pins, paths, profiles, toolchain, repo files, changelog, justfile, feature docs, naming.
- ✅ `MAN-2` Pin policy per project: `git-dependency-pins` checks branch, default-branch, dev-branch, tag, release or any, separately for own repos.
- ✅ `MAN-3` Survey gaps: `profile-no-lto`, `patch-outside-workspace`, `test-binaries`, `macro-compile-fail`; medium's Clippy set caps bool parameters at one.
- 📋 `MAN-4` `contract-snapshot`: a crate the map marks contract carries a VERSION and a schema snapshot per version; a changed snapshot needs a bump against the base.
- 📋 `MAN-5` `git-dependency-pins` also holds git `[patch]` overrides to the pin policy.
- 📋 `MAN-6` `default-members`: a workspace past a size parameter names its main binary in `default-members`.
- 📋 `MAN-7` `example-metadata`: every example carries name, description and category in the manifest; the examples index is generated.
- 📋 `MAN-8` Markdown check of READMEs and docs as a tool adapter.

## GRAPH - crate graph and map

- ✅ `GRAPH-1` Map completeness, layer edges, forbidden closures, external form, third-party placement, `_core` rules, crate cycles, crate table, unsafe allow-list (`sl_lints_graph`).
- 📋 `GRAPH-3` `clippy-policy-bans`: clippy.toml bans unbounded channels and wall-clock timers in tests. Raw spawns, blocking calls and colours in UI crates are `UI-1`'s source lints.
- ✅ `GRAPH-2` linkme elements and unsafe attributes count as unsafe; crates marked unsafe copy the workspace lints.

## SIZE - size and complexity

- ✅ `SIZE-1` Crate, test, module and function lines; cyclomatic complexity.
- ✅ `SIZE-2` `function-lines` ratchet: `target-lines` below the limit reports at most a warning (`Diagnostic::at_most`).

## CODE - source lints

- ✅ `CODE-1` todo!/unimplemented!, panics and prints in libraries, allows without reason, float and hash determinism, CLI subprocesses (`sl_lints_code`); the unsafe allow-list is GRAPH-1's.
- ✅ `CODE-3` `allow-without-reason` looks inside `cfg_attr(predicate, allow(...))`.
- ✅ `CODE-2` Module-graph SCC share, dead pub items, error enum growth, pub surface (`sl_lints_modules`, `sl_lints_dead`, `sl_lints_errors`, `sl_lints_surface`).
- ✅ `CODE-4` `build-info-env` and `test-support-gated`: build info and test fakes stay out of library builds.
- ✅ `CODE-7` `orphan-source-file`: files under `src/` that no `mod` reaches, test modules and `#[path]` included.
- ✅ `CODE-8` Limits and units: `sized-ints-in-contracts` (map key `contract`), `unit-suffix`, `unbounded-read`.
- ✅ `CODE-10` Assertion policy: `panic-in-library` counts bare `unwrap` and message-less panics; `compound-assert`, `assert-message`, `float-partial-cmp-unwrap`; Clippy `missing_panics_doc`, `debug_assert_with_mut_call`; task classes `[embedded.classes]` with `cls-1`, `cls-2`, `flt-11`, `flt-12`.
- ✅ `CODE-11` exact_units lints (`sl_lints_units`): `exact-units-quantity` and `exact-units-bounds` from small, `exact-units-plain-number` in `units-strict`, `exact-units-in-bodies` in `units-paranoid`; `unit-suffix` only where the quantity lint is off.
- 📋 `CODE-5` `build-script-clock`: build.rs reads no clock and finds git through the real git dir, so worktree builds stay clean.
- 📋 `CODE-6` `command-registry`: a CLI past a size parameter registers commands per module, not in one central Subcommand enum (estimate).
- ✅ `CODE-9` CLI lints (`sl_lints_cli`): spl builds each CLI and checks its help, examples, completion, colour, version line and exit codes; the map key `cli` and the `[cli]` table.

## STATE - shared state (fewer-locks)

- ✅ `STATE-1` Statics with interior mutability, static mut, thread_local state, lock().unwrap() counts, guards across await or I/O, nested locks.

## PROSE - prose and agent text

- ✅ `PROSE-1` Terse-output limits: CHANGELOG entries, commit subjects and bodies, AGENTS.md length, sentences, paragraphs (`sl_lints_prose`).
- ✅ `PROSE-2` AI-isms and agent files: filler, narration, stream-of-thought, emoji, restating comments, diff descriptions; CLAUDE.md pointer, links, stated limits match the lints.
- ✅ `PROSE-3` Fix commits: `fixes-tag` checks `Fixes:` hash and subject, `fix-without-test` wants a test in a fix.
- ✅ `PROSE-4` Commit hygiene: `commit-tested-line` wants a `Tested:` trailer, `commit-mixed-move` a move without edits.

## DOCS - docs as the reader meets them

- ✅ `DOCS-1` Reading metrics per page (`spl pages`, text and JSON) and lints: prose runs, reading time, heading length, sections, code walls, image text, code languages; the `[docs]` table (`sl_docs`, `sl_lints_docs`).
- ✅ `DOCS-2` Zensical book lints: navigation entries that exist, orphan pages, one title per page, admonition syntax.

## AI - code written with agents

- ✅ `AI-1` Stop-gap comments, commented-out code, lint-table allows without reason, near-duplicate fns and modules, dead features, leftover files, agent-config secrets, permissions and transcripts (`sl_lints_ai`).
- ✅ `AI-2` `fixme-before-land`: a FIXME is a candidate on the branch and a finding in a landing run; `stop-gap-comments` leaves FIXME to it.

## UI - user interface lints

- ✅ `UI-1` `ui-practice-lints`: UI practices as lints: `ui` tier (toolkit-neutral, calls as parameters), `egui` tier (egui calls, ids, repaints, theme, glyphs); map keys `ui` and `theme` ([UI practices](docs/ui.md)).

## TOOL - external tools

- ✅ `TOOL-1` Adapters for clippy, cargo-deny, machete, shear, typos, taplo, semver-checks, hakari and geiger (linked), udeps, rustfmt, cargo-audit; cargo checks (MSRV, no_std, wasm, doc); tool cache keyed on crate, version and rustc ([tools](docs/tools.md)).

## BOOK - the book

- ✅ `BOOK-1` Lint and tier pages generated from metadata; `book --check` fails when stale.
- ✅ `BOOK-2` Logo and brand assets from a Typst source (`assets/`); book favicon, header logo, README header.
