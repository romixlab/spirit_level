# Features

Status: ✅ done · 🚧 in progress · 📋 planned · 🐛 bug. Ids are stable; never renumber or reuse.

## CFG - configuration format

- ✅ `CFG-1` Tiers as TOML files that extend each other; a project extends one or several, or a shared file by path (`sl_config`).
- ✅ `CFG-2` Overrides per lint: level, parameters, required reason, optional expiry (date or release); expired fails.
- ✅ `CFG-3` Unknown keys, lint ids, parameters and wrong types fail loudly with file and line.
- ✅ `CFG-4` Scoped exceptions by crate or path glob: level and/or parameters, reason, expiry.
- ✅ `CFG-5` The crate map: layers with allow lists, crates with layer and flags, forbid, external, third-party.
- ✅ `CFG-6` `foreign-map-keys`: `[map-file] foreign` names the key paths the map's owner checks; spl leaves them unread, and a typo in an spl key stays an error.
- ✅ `CFG-7` `per-crate-tiers`: `[[crate-tiers]]` and the map key `tiers` hold chosen crates to more tiers; their levels and parameters resolve per crate.

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
- ✅ `ENG-11` `skip-without-firmware`: lints that read the linked ELF are skipped, never failed, when no firmware binary they are enabled on exists ([output](docs/output.md#skipped-lints)).

## CLI - the command

- ✅ `CLI-1` check, explain, lints, tiers, config, init, book, dependents; dynamic completion; version with SHA.
- ✅ `CLI-2` Usage records: every run counts its command and flags in `~/.local/state/spl/usage` (vc_usage); `spl usage` lists them, `--unused` ([usage records](docs/usage.md)).
- ✅ `CLI-3` The binary is `spl`; crate and repo stay spirit_level.
- ✅ `CLI-4` The generic CLI parts from vhrd_cli: `--version` (vc_build), `--color`, error line, exit codes (vc_ui), completion (vc_complete).
- ✅ `CLI-5` Usage records are opt-in, off by default (`vc_usage` USAGE-2): `SPL_USAGE_DIR=DIR` or `SPL_USAGE=on` enables them; `spl usage` says so when off ([usage records](docs/usage.md)).
- ✅ `CLI-6` `cli-secret-flags`: an option whose long name looks like it holds a secret (token, password, secret, api-key) and takes a value is a candidate finding, unless it points at the secret (`--*-file`, `--*-env`).
- ✅ `CLI-7` spl's commands register themselves per module (inventory); the root command, completion and usage signals come from the registry.

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
- ✅ `MAN-4` `contract-snapshot` (M0030): a crate the map marks contract exports `VERSION` and keeps `schema/v<N>.json` in git; a snapshot the base has never changes.
- 📋 `MAN-5` `git-dependency-pins` also holds git `[patch]` overrides to the pin policy.
- 📋 `MAN-6` `default-members`: a workspace past a size parameter names its main binary in `default-members`.
- 📋 `MAN-7` `example-metadata`: every example carries name, description and category in the manifest; the examples index is generated.
- 📋 `MAN-8` Markdown check of READMEs and docs as a tool adapter.
- ✅ `MAN-9` `fuzz-corpus-test` (M0031): every `fuzz/fuzz_targets/*.rs` has a corpus tracked in git and a `#[test]` naming it.
- ✅ `MAN-10` `ci-workflow` (M0032): `.github/workflows/ci.yml` runs `just lint` and `just test`, or without those recipes Clippy and the tests.
- ✅ `MAN-11` `manifest-homepage` (M0033): `[workspace.package]` (or `[package]`) sets `repository`, and `homepage` equals the book's `site_url`, with an exact fix.
- ✅ `MAN-12` `readme-badges` (M0034): the README carries a badge for the book (`site_url`), the CI workflow (its `badge.svg`), and crates.io and docs.rs per published crate; `--fix` inserts them.

## GRAPH - crate graph and map

- ✅ `GRAPH-1` Map completeness, layer edges, forbidden closures, external form, third-party placement, `_core` rules, crate cycles, crate table, unsafe allow-list (`sl_lints_graph`).
- 📋 `GRAPH-3` `clippy-policy-bans`: clippy.toml bans unbounded channels and wall-clock timers in tests. Raw spawns, blocking calls and colours in UI crates are `UI-1`'s source lints.
- ✅ `GRAPH-4` `crate-folder-layer`: crates sit in `crates/<layer>/<crate>`, a large layer by discipline (map key `discipline`), at most two folders deep.
- ✅ `GRAPH-6` `nested-workspaces`: the map key `workspace` puts a crate in a nested workspace; map-complete and the map lints read every workspace the map names.
- ✅ `GRAPH-2` linkme elements and unsafe attributes count as unsafe; crates marked unsafe copy the workspace lints.
- ✅ `GRAPH-5` `licence-per-layer`: the map sets a licence per layer or crate; `map-licence` checks each manifest, `workspace-inheritance` accepts it.

## SIZE - size and complexity

- ✅ `SIZE-1` Crate, test, module and function lines; cyclomatic complexity.
- ✅ `SIZE-2` `function-lines` ratchet: `target-lines` below the limit reports at most a warning (`Diagnostic::at_most`).
- ✅ `SIZE-3` `crate-folder-size`: crate folders per folder, `max-flat` for a flat `crates/`, `max-crates` for a layer or discipline folder.

## CODE - source lints

- ✅ `CODE-1` todo!/unimplemented!, panics and prints in libraries, allows without reason, float and hash determinism, CLI subprocesses (`sl_lints_code`); the unsafe allow-list is GRAPH-1's.
- ✅ `CODE-3` `allow-without-reason` looks inside `cfg_attr(predicate, allow(...))`.
- ✅ `CODE-2` Module-graph SCC share, dead pub items, error enum growth, pub surface (`sl_lints_modules`, `sl_lints_dead`, `sl_lints_errors`, `sl_lints_surface`).
- ✅ `CODE-4` `build-info-env` and `test-support-gated`: build info and test fakes stay out of library builds.
- ✅ `CODE-7` `orphan-source-file`: files under `src/` that no `mod` reaches, test modules and `#[path]` included.
- ✅ `CODE-8` Limits and units: `sized-ints-in-contracts` (map key `contract`), `unit-suffix`, `unbounded-read`.
- ✅ `CODE-10` Assertion policy: `panic-in-library` counts bare `unwrap` and message-less panics; `compound-assert`, `assert-message`, `float-partial-cmp-unwrap`; Clippy `missing_panics_doc`, `debug_assert_with_mut_call`; task classes `[embedded.classes]` with `cls-1`, `cls-2`, `flt-11`, `flt-12`.
- ✅ `CODE-11` exact_units lints (`sl_lints_units`): `exact-units-quantity` and `exact-units-bounds` from small, `exact-units-plain-number` in `units-strict`, `exact-units-in-bodies` in `units-paranoid`; `unit-suffix` only where the quantity lint is off.
- ✅ `CODE-12` `cli-no-abbreviations`: a CLI's unique-prefix abbreviation of a subcommand (clap's `infer_subcommands`) runs that subcommand.
- ✅ `CODE-13` `cls-3`: a `service` module holds no call that panics in release, `assert!` included; `panic-in-library` checks library code in every tier, firmware too.
- ✅ `CODE-6` `command-registry`: a CLI past a size parameter registers commands per module, not in one central Subcommand enum (estimate).
- ✅ `CODE-5` `build-script-clock`: build.rs reads no clock and finds git through the real git dir, so worktree builds stay clean.
- ✅ `CODE-9` CLI lints (`sl_lints_cli`): spl builds each CLI and checks its help, examples, completion, colour, version line and exit codes; the map key `cli` and the `[cli]` table.

## STATE - shared state (fewer-locks)

- ✅ `STATE-1` Statics with interior mutability, static mut, thread_local state, lock().unwrap() counts, guards across await or I/O, nested locks.

## PROSE - prose and agent text

- ✅ `PROSE-1` Terse-output limits: CHANGELOG entries, commit subjects and bodies, AGENTS.md length, sentences, paragraphs (`sl_lints_prose`).
- ✅ `PROSE-2` AI-isms and agent files: filler, narration, stream-of-thought, emoji, restating comments, diff descriptions; CLAUDE.md pointer, links, stated limits match the lints.
- ✅ `PROSE-3` Fix commits: `fixes-tag` checks `Fixes:` hash and subject, `fix-without-test` wants a test in a fix.
- ✅ `PROSE-4` Commit hygiene: `commit-tested-line` wants a `Tested:` trailer, `commit-mixed-move` a move without edits.
- ✅ `PROSE-5` url-tracking-params: a link in docs or a comment carries no tracking query parameter (`utm_`, `?si=`, and similar).

## DOCS - docs as the reader meets them

- ✅ `DOCS-1` Reading metrics per page (`spl pages`, text and JSON) and lints: prose runs, reading time, heading length, sections, code walls, image text, code languages; the `[docs]` table (`sl_docs`, `sl_lints_docs`).
- ✅ `DOCS-2` Zensical book lints: navigation entries that exist, orphan pages, one title per page, admonition syntax, the configuration naming only keys Zensical reads; `zensical.toml` or, the same way, `mkdocs.yml`.
- ✅ `DOCS-3` Readable words: abbreviations spelled out at first use, terms defined before use, one name per concept (a project word list), passive share and Flesch reading ease per page, in `spl pages` and as lints.
- ✅ `DOCS-4` book-build-stamp: a Zensical book's footer carries a build stamp (version, commit, UTC time) and a script showing its age from the reader's own clock.
- ✅ `DOCS-5` readme-index-clash: no docs folder holds both `README.md` and `index.md`, since Zensical does not say which one it builds.
- ✅ `DOCS-7` theme-feature-conflicts: theme features Zensical does not support together, and features that need `project.site_url` set.
- ✅ `DOCS-6` page-links: a link between pages is a relative path to a Markdown file that exists, never `.html` or an absolute site path; checked on every page `[docs]` selects, book or not.
- ✅ `DOCS-8` built-site-pages: `[docs.site]` reads a built site's HTML (zola, Zensical) in place of its Markdown sources for `spl pages` and the reading and word lints; content element, skipped classes, calls to action and terminal captures as keys.
- ✅ `DOCS-9` `book-deploy` (D0022): a book with a `site_url` has one workflow that builds it strict (`zensical build --strict`, `mkdocs build --strict`, or a just recipe running it) and publishes it with `actions/deploy-pages`.
- ✅ `DOCS-10` `[docs.book]`: pages another repo's book takes; `page-links` resolves their links from their place in that book and looks its other pages up in its checkout.
- ✅ `DOCS-11` book-lint-search: the book's search finds a lint by code, id or Clippy name first; each Clippy lint, group and `clippy.toml` setting of the tiers has a generated page (level per tier, reason, Clippy's docs), and tool lints link the tool's docs of the check.

## AI - code written with agents

- ✅ `AI-1` Stop-gap comments, commented-out code, lint-table allows without reason, near-duplicate fns and modules, dead features, leftover files, agent-config secrets, permissions and transcripts (`sl_lints_ai`).
- ✅ `AI-2` `fixme-before-land`: a FIXME is a candidate on the branch and a finding in a landing run; `stop-gap-comments` leaves FIXME to it.
- ✅ `AI-3` `large-blobs`: a git-tracked file over `max-bytes` that matches no glob in `allow`.

## UI - user interface lints

- ✅ `UI-1` `ui-practice-lints`: UI practices as lints: `ui` tier (toolkit-neutral, calls as parameters), `egui` tier (egui calls, ids, repaints, theme, glyphs); map keys `ui` and `theme` ([UI practices](docs/ui.md)).
- ✅ `UI-2` `ui-unwrap-external-data`: `unwrap`/`expect` on a file, parse or connection call in a UI crate; `panic-in-library` covers only library crates.
- ✅ `UI-6` `egui-panel-api`: `SidePanel`/`TopBottomPanel`, a panel's `.show` on a `Context`, and `App::update`, named as parameters so they follow egui releases.
- ✅ `UI-7` `egui-checklist-menu`: a `menu_button` closure with several checkboxes closes on each click; the shared checklist widget keeps it open.
- ✅ `UI-3` `ui-tooltip-repeats-label`: a tooltip whose text exactly repeats the label of the control it is attached to.
- ✅ `UI-4` `ui-locale-strings`, opt-in: a string literal with a letter, passed to a text call, needs a translation catalogue entry.
- ✅ `UI-5` `ui-hover-text`, candidate: an icon-only or glyph button with no tooltip; glyph detection reuses `egui-status-glyph`'s `glyphs`.

## TOOL - external tools

- ✅ `TOOL-1` Adapters for clippy, cargo-deny, machete, shear, typos, taplo, semver-checks, hakari and geiger (linked), udeps, rustfmt, cargo-audit; cargo checks (MSRV, no_std, wasm, doc) ([tools](docs/tools.md)).
- ✅ `TOOL-2` `tools-needed`: tools found on PATH with their version checked against a tested range (`tool-version`); `spl tools needed` lists what a project needs, as a table, `crate@version` specs or JSON; any enabled lint that cannot run fails the run (`--allow-warn-not-run` lets warn-level ones through) ([tools](docs/tools.md#installing-the-tools)).
- ✅ `TOOL-3` `deny-licenses` without `deny.toml`: one finding asking for the file, not one per crate (cargo-deny's own default policy allows no licence).
- ✅ `TOOL-4` full-by-default: hakari and geiger are always linked into `spl`, no cargo feature to build them in ([tools](docs/tools.md)).
- ✅ `TOOL-5` `tool-shadowed`: an out-of-range tool with an in-range one further down PATH is named by both path and version, in the table's `shadowed by` column, the JSON `shadow` field and the lint's help; `--spec`/`--components` skip it ([tools](docs/tools.md#a-tool-shadowed-on-path)).

## BOOK - the book

- ✅ `BOOK-1` Lint and tier pages generated from metadata; `book --check` fails when stale.
- ✅ `BOOK-2` Logo and brand assets from a Typst source (`assets/`); book favicon, header logo, README header.
- ✅ `BOOK-3` `spl book stamp`: a book's footer build stamp (crate version, git commit, build time) and the reader-clock age script, serving `book-build-stamp` (D0017); `sl_book::stamp`.
- ✅ `BOOK-4` `spl book sample`: a sample project's coloured `spl check` report as an SVG picture for the README; `book sample --check` fails when stale.
- ✅ `BOOK-5` The book is published at spl.vhrd.tech from GitHub Actions; CI runs `just lint` and `just test`.
- ✅ `BOOK-6` Coloured check output in the book: `<!-- spirit-level:check PATH ... -->` blocks of hand-written pages, written by `spl book docs` as HTML from the CLI's own colours; `book docs --check` fails when stale.

## TESTKIT - the golden fixture harness

- ✅ `TESTKIT-1` `BLESS=1` rewrites only the fixture cases whose findings changed; an untouched case, including a `[params]` table, round-trips byte for byte (`sl_testkit::bless`).
