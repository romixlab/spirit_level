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

## CLI - the command

- ✅ `CLI-1` check, explain, lints, tiers, config, init, book; dynamic completion; version with SHA.
- 📋 `CLI-2` Usage records per subcommand.

## TIER - shipped tiers

- ✅ `TIER-1` small, small-workspace, medium, big.
- 📋 `TIER-2` embedded: every bedrock_lint check (port of embedded_bedrock's bedrock_lint).
- ✅ `TIER-3` ai: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene; stands beside the size tiers.

## MAN - manifest and repo lints

- ✅ `MAN-1` 24 lints: metadata, inheritance, workspace lints and deps, default features, -sys, duplicates, pins, paths, profiles, toolchain, repo files, changelog, justfile, feature docs, naming.

## GRAPH - crate graph and map

- ✅ `GRAPH-1` Map completeness, layer edges, forbidden closures, external form, third-party placement, `_core` rules, crate cycles, crate table, unsafe allow-list (`sl_lints_graph`).

## SIZE - size and complexity

- ✅ `SIZE-1` Crate, test, module and function lines; cyclomatic complexity.

## CODE - source lints

- ✅ `CODE-1` todo!/unimplemented!, panics and prints in libraries, allows without reason, float and hash determinism, CLI subprocesses (`sl_lints_code`); the unsafe allow-list is GRAPH-1's.
- ✅ `CODE-2` Module-graph SCC share, dead pub items, error enum growth, pub surface (`sl_lints_modules`, `sl_lints_dead`, `sl_lints_errors`, `sl_lints_surface`).

## STATE - shared state (fewer-locks)

- ✅ `STATE-1` Statics with interior mutability, static mut, thread_local state, lock().unwrap() counts, guards across await or I/O, nested locks.

## PROSE - prose and agent text

- ✅ `PROSE-1` Terse-output limits: CHANGELOG entries, commit subjects and bodies, AGENTS.md length, sentences, paragraphs (`sl_lints_prose`).
- ✅ `PROSE-2` AI-isms and agent files: filler, narration, stream-of-thought, emoji, restating comments, diff descriptions; CLAUDE.md pointer, links, stated limits match the lints.

## AI - code written with agents

- ✅ `AI-1` Stop-gap comments, commented-out code, lint-table allows without reason, near-duplicate fns and modules, dead features, leftover files, agent-config secrets, permissions and transcripts (`sl_lints_ai`).

## TOOL - external tools

- 📋 `TOOL-1` Adapters for clippy, cargo-deny, machete, typos, taplo, semver-checks, hakari, geiger, udeps, rustfmt, cargo-audit; tool cache keyed on sources, Cargo.lock and rustc.

## BOOK - the book

- ✅ `BOOK-1` Lint and tier pages generated from metadata; `book --check` fails when stale.
