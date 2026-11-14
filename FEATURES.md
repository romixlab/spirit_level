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

## CLI - the command

- ✅ `CLI-1` check, explain, lints, tiers, config, init, book; dynamic completion; version with SHA.
- 📋 `CLI-2` Usage records per subcommand.
- ✅ `CLI-3` The binary is `spl`; crate and repo stay spirit_level.
- ✅ `CLI-4` The generic CLI parts from vhrd_cli: `--version` (vc_build), `--color`, error line, exit codes (vc_ui), completion (vc_complete).

## TIER - shipped tiers

- ✅ `TIER-1` small, small-workspace, medium, big.
- ✅ `TIER-2` embedded: every bedrock_lint check (port of embedded_bedrock's bedrock_lint): `sl_firmware`, `sl_lints_embedded`, `--elf`, migration page.
- ✅ `TIER-3` ai: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene; stands beside the size tiers.
- ✅ `TIER-4` Clippy set per tier (`[clippy]` in tier files, modelled on rerun's), checked by `tier-clippy-set` (M0025) in the project's `[lints.clippy]` and `clippy.toml`.

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
- 📋 `GRAPH-3` `clippy-policy-bans`: map roles `ui` and `theme`; clippy.toml bans raw spawns and blocking Command in UI crates, colours outside the theme, unbounded channels, wall-clock timers in tests.
- ✅ `GRAPH-2` linkme elements and unsafe attributes count as unsafe; crates marked unsafe copy the workspace lints.

## SIZE - size and complexity

- ✅ `SIZE-1` Crate, test, module and function lines; cyclomatic complexity.

## CODE - source lints

- ✅ `CODE-1` todo!/unimplemented!, panics and prints in libraries, allows without reason, float and hash determinism, CLI subprocesses (`sl_lints_code`); the unsafe allow-list is GRAPH-1's.
- ✅ `CODE-3` `allow-without-reason` looks inside `cfg_attr(predicate, allow(...))`.
- ✅ `CODE-2` Module-graph SCC share, dead pub items, error enum growth, pub surface (`sl_lints_modules`, `sl_lints_dead`, `sl_lints_errors`, `sl_lints_surface`).
- ✅ `CODE-4` `build-info-env` and `test-support-gated`: build info and test fakes stay out of library builds.
- 📋 `CODE-5` `build-script-clock`: build.rs reads no clock and finds git through the real git dir, so worktree builds stay clean.
- 📋 `CODE-6` `command-registry`: a CLI past a size parameter registers commands per module, not in one central Subcommand enum (estimate).

## STATE - shared state (fewer-locks)

- ✅ `STATE-1` Statics with interior mutability, static mut, thread_local state, lock().unwrap() counts, guards across await or I/O, nested locks.

## PROSE - prose and agent text

- ✅ `PROSE-1` Terse-output limits: CHANGELOG entries, commit subjects and bodies, AGENTS.md length, sentences, paragraphs (`sl_lints_prose`).
- ✅ `PROSE-2` AI-isms and agent files: filler, narration, stream-of-thought, emoji, restating comments, diff descriptions; CLAUDE.md pointer, links, stated limits match the lints.

## AI - code written with agents

- ✅ `AI-1` Stop-gap comments, commented-out code, lint-table allows without reason, near-duplicate fns and modules, dead features, leftover files, agent-config secrets, permissions and transcripts (`sl_lints_ai`).

## TOOL - external tools

- ✅ `TOOL-1` Adapters for clippy, cargo-deny, machete, shear, typos, taplo, semver-checks, hakari and geiger (linked), udeps, rustfmt, cargo-audit; cargo checks (MSRV, no_std, wasm, doc); tool cache keyed on crate, version and rustc ([tools](docs/tools.md)).

## BOOK - the book

- ✅ `BOOK-1` Lint and tier pages generated from metadata; `book --check` fails when stale.
- ✅ `BOOK-2` Logo and brand assets from a Typst source (`assets/`); book favicon, header logo, README header.
