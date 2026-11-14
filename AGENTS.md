# Working on spirit_level

Guidance for AI agents and contributors. Read it before changing code.

spirit_level is a linter for Rust projects of every size. Lints are declared in Rust with their metadata, configured in TOML (tiers that extend each other, a project's `spirit_level.toml` on top), and documented in a book generated from that metadata.

## 1. The top rule: no stop-gap solutions

- Build the final architecture from the start. No shims, no "refactor later", no second path the final design throws away.
- A `TODO` that defers the right design is a defect; a `TODO(ID)` names a FEATURES.md item that is planned work.
- The lint-definition format (`LintMeta`, the TOML of tiers and `spirit_level.toml`) is meant not to change. Extend it only by adding: a new `Need`, a new `ParamKind`, a new optional key.

## 2. Terse output

Result first, then only what the reader must decide or do. Bullets for lists, tables for options, one fact per line. No recap, no narration. Limits: CHANGELOG entry 1 line and 20 words, commit subject 60 characters, this file 150 lines. spirit_level's own `prose` lints check these.

## 3. Layout

| Crate | Layer | What |
|---|---|---|
| `sl_core` | core | types: `LintMeta`, `Level`, params, `Diagnostic`, dates, globs |
| `sl_model` | model | the project: cargo metadata, manifests (toml_edit), sources (syn), line counts |
| `sl_config` | model | `spirit_level.toml` and tiers: extends, overrides, exceptions, expiries, the crate map |
| `sl_firmware` | model | firmware crates: cargo config, build.rs, memory.x, bedrock_fw.json, source facts, the linked ELF |
| `sl_analysis` | analysis | the `Analysis` trait, `Context`, the `lint!` macro, syntax helpers |
| `sl_lints_*` | lints | one crate per analysis family; each lint's docs in `docs/<id>.md`, fixtures in `tests/fixtures/<id>/` |
| `sl_engine` | engine | runs analyses, needs, levels, exceptions, waivers, fixes; its own lints `waiver-*` |
| `sl_report` | engine | text and JSON output |
| `spirit_level` | app | the registry of analyses and the shipped tiers (`tiers/*.toml` at the root) |
| `sl_book` | app | generated book pages |
| `spirit_level_cli` | app | the `spl` binary |
| `sl_testkit` | tests | the golden fixture harness |

The map in `spirit_level.toml` is the source of truth for layers; `spl check` enforces it on this repo at the `big` tier.

## 4. Adding a lint

1. Declare it with `lint!(STATIC, "CODE", "id", "category", "rule as a statement", params: [...], needs: [...], fix: None|Exact)` in its analysis crate, add it to the crate's `LINTS`. CODE is the group's next free number (`spl codes`); add its row to `crates/spirit_level/codes.toml`. A code never changes; a removed lint's row stays, `status = "retired"`.
2. Write `docs/<id>.md`: `## Why`, an example where it helps, `## How to fix`.
3. Add `tests/fixtures/<id>/pass/` and `fail/` (a cargo project with `[workspace]` in its `Cargo.toml`, and `expect.toml`; `project = "..."` runs one shared under `tests/projects/`). Text-pattern lints add false-positive cases to `pass/`. `BLESS=1 cargo test -p <crate> --test fixtures` writes `expect.toml`; read it before committing.
4. Enable it in a tier file with a level and a reason.
5. `just book` regenerates the pages; `just lint` fails while they are stale.

A new analysis family is a new crate `sl_lints_<family>` with a `static` implementing `Analysis`, listed in `spirit_level::analyses()` and in the map.

Lint ids are kebab-case and stable. Tool adapters prefix the tool's own ids (`clippy-`, `deny-`).

## 5. Rules for analyses

- Read cargo metadata (`sl_model`), never parse `Cargo.toml` for what metadata gives; read the raw manifest only for what it lacks (inheritance, `[lints]`, comments, spans).
- Source analyses use `syn` through `Project::sources`; skip test code with `syntax::without_tests`.
- Every finding has a primary span, a stable `item` (fingerprint), the crate when it is about one, and a `help`.
- A measured lint attaches a `metric` and takes its limit from a parameter.
- A heuristic says so: `Confidence::Estimate` (the message states the assumption) or `Candidate` (never fails a run).
- An exact fix only when applying it is the whole fix; anything that needs judgement gets a `help` instead.
- No numbers in code that a project might want different: they are parameters with a default and a doc line.

## 6. Dependencies

- All in `[workspace.dependencies]`, `default-features = false`, only the features in use.
- No bundled C, no `-sys` crates with a native build. Check `cargo tree -e normal,build` when adding one.
- External tools: a tool with a usable library on stable Rust is linked behind a cargo feature (`hakari`, `geiger`); the others run as executables, pinned in `crates/sl_tools/tools.toml` ([docs/tools.md](docs/tools.md)).
- `rayon-core` (with `hakari`) sets `links` only to stay unique in a graph; it builds no C.

## 7. FEATURES.md and CHANGELOG.md

- FEATURES.md is the only status tracker, ids per area (`CFG-3`). Name ids with a slug when talking to people: `CFG-3 expiring-overrides`. New ids: `tpm work claim AREA`.
- CHANGELOG.md: entries under `## [Unreleased]` (`### Added`, `### Changed`, `### Fixed`, `### Removed`), one line each, with the id. A branch never touches the version; landing does.
- Design notes stay out of this repo: the book explains things as they are.

## 8. Commands

```sh
just            # the recipes
just test       # nextest; `just test --changed` tests only what changed against main
just lint       # fmt, clippy -D warnings, spl on itself at the big tier, book freshness
just book       # regenerate the book's generated pages
just tools      # install the pinned external tools into the tool cache
just docs       # serve the book (uv, Zensical)
just install    # build with the deploy profile, put spl on PATH
```

## 9. Tests

- Golden fixtures per lint (pass and fail) through `sl_testkit`; unit tests beside the code.
- Tests are deterministic: today is injected (`2026-11-13` in fixtures, `--today` / `SPIRIT_LEVEL_TODAY` in the CLI); nothing reads the host's state.
- Cargo is started with the outer cargo's `CARGO_*` variables removed (`sl_model`), so tests see projects as a person would.
