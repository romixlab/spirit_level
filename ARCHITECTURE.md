# Architecture

spirit_level is a Cargo workspace of small crates in declared layers. This page explains how a check runs, which layer may depend on which, and what each crate holds.

## How a check runs

A run of `spl check` passes through four stages, each owned by one layer:

1. **Model.** `sl_model` reads the project through `cargo metadata`, the raw manifests where metadata lacks something, and the Rust sources parsed with `syn`. `sl_config` resolves `spirit_level.toml` and the tiers it extends into one level and one parameter set per lint.
2. **Analysis.** Each `sl_lints_*` crate implements the `Analysis` trait from `sl_analysis`. An analysis declares its lints with the `lint!` macro, including what it needs (a tool, a toolchain, the crate map, a linked firmware binary), and returns findings with a span, a stable item and a help line.
3. **Engine.** `sl_engine` picks the analyses the configuration enables, checks their needs, runs them, and applies levels, exceptions, waivers and exact fixes.
4. **Report.** `sl_report` writes coloured text for people or a stable JSON document for CI and agents.

The `spirit_level` crate holds the registry of every analysis and the shipped tier files. The `spl` binary in `spirit_level_cli` is a thin layer over it.

## Layers

The crate map in `spirit_level.toml` declares the layers, and `spl check` holds this repo to them at the `big` tier. A crate may depend only on the layers its own layer allows:

| Layer | Holds | May depend on |
|---|---|---|
| core | Types only: lint metadata, levels, parameters, diagnostics | nothing |
| model | Reading the project and the configuration | core, model |
| analysis | The analysis trait and helpers every lint crate uses | core, model |
| lints | One crate per analysis family; lint crates never see each other | core, model, analysis |
| engine | Running analyses, waivers, fixes, reports | core, model, analysis, engine |
| app | The registry, the book generator and the binary | every layer except tests |
| tests | Test-only crates; only dev-dependencies point here | core, model, analysis, engine, tests |

Because lint crates never see each other, a new analysis family is a new crate in the lints layer, listed in `spirit_level::analyses()` and in the map. Nothing else changes.

## Crates

The table below is generated from each crate's `description` and its layer in the map. `spl check --fix` rewrites it, and the [crate-table](docs/lints/G0004.md) lint fails while it is stale.

<!-- spirit-level:crates -->
| Crate | Layer | Description |
|---|---|---|
| [`sl_core`](crates/sl_core) | core | spirit_level's shared types: lint metadata, levels, parameters, diagnostics, spans, fixes, dates and globs |
| [`sl_board`](crates/sl_board) | model | spirit_level's model of board descriptions: board.toml through sl_board_model, stm32-data chip JSON, the MCU's nets from the PCB geometry |
| [`sl_board_model`](crates/sl_board_model) | model | The board description board.toml: schema, loading with source spans, revisions and reworks as patches |
| [`sl_config`](crates/sl_config) | model | spirit_level's configuration format: tiers, extends, lint levels, parameters, reasons, expiries, scoped exceptions and the crate map |
| [`sl_dependents`](crates/sl_dependents) | model | The repos that build on a library and the crates in them to retest when it changes (`spl dependents`) |
| [`sl_docs`](crates/sl_docs) | model | Pages as a reader meets them, from Markdown or a built site: their shape, reading metrics, and the Zensical book they belong to |
| [`sl_firmware`](crates/sl_firmware) | model | spirit_level's model of bare-metal firmware crates: cargo configuration, build script, memory.x, bedrock_fw.json, source facts, the linked ELF with frames and calls |
| [`sl_model`](crates/sl_model) | model | spirit_level's project model: cargo metadata, manifests with spans, Rust sources parsed with syn, line counts |
| [`sl_tools`](crates/sl_tools) | model | External tools for spirit_level's adapters: the tested versions, finding a tool on PATH and reading its version, running it, cargo's JSON messages |
| [`sl_analysis`](crates/sl_analysis) | analysis | The one trait every spirit_level analysis implements, its run context, and syntax helpers shared by the source analyses |
| [`sl_lints_ai`](crates/sl_lints_ai) | lints | spirit_level lints for code written with agents: stop-gap comments, commented-out code, near-duplicate functions, dead features, committed agent secrets and transcripts |
| [`sl_lints_board`](crates/sl_lints_board) | lints | spirit_level board lints: the board description board.toml against itself, the chip's data and the PCB's nets, for every revision and rework |
| [`sl_lints_cli`](crates/sl_lints_cli) | lints | spirit_level CLI lints: builds each command-line tool and checks its help, examples, completion, colour, version line and exit codes |
| [`sl_lints_code`](crates/sl_lints_code) | lints | spirit_level source lints: todo!() and unimplemented!(), panics and prints in library code, allow without reason, float and hashing determinism, calls of the workspace's own CLIs |
| [`sl_lints_dead`](crates/sl_lints_dead) | lints | spirit_level lint on dead code rustc cannot see: public items of workspace libraries no other crate names |
| [`sl_lints_docs`](crates/sl_lints_docs) | lints | spirit_level lints on docs as a reader meets them: prose runs, reading time, headings, code walls, image text, and Zensical book practices |
| [`sl_lints_embedded`](crates/sl_lints_embedded) | lints | spirit_level lints on bare-metal firmware: executors, interrupts, DMA, stack, memory, size, faults, boot, power, flash, pins, clocks, errata, USB, host input, logging, counters, build configuration (embedded_bedrock's bedrock_lint checks) |
| [`sl_lints_errors`](crates/sl_lints_errors) | lints | spirit_level lints on error types: flat error enums that grow a variant per failure instead of wrapping the errors they come from |
| [`sl_lints_graph`](crates/sl_lints_graph) | lints | spirit_level crate-graph lints against the crate map: layers edge by edge, forbidden closures, external git form, third-party placement, _core rules, cycles, the generated crate table, the unsafe allow-list |
| [`sl_lints_manifest`](crates/sl_lints_manifest) | lints | spirit_level lints on manifests and repo files: package metadata, workspace inheritance and lints, dependency form, -sys crates, duplicates, profiles, toolchain, repo rule files, justfile, feature docs, contract snapshots |
| [`sl_lints_modules`](crates/sl_lints_modules) | lints | spirit_level lints on a crate's module graph: the share of code caught in its biggest module cycle, and where to cut it |
| [`sl_lints_prose`](crates/sl_lints_prose) | lints | spirit_level lints on prose: CHANGELOG and commit limits, agent files, filler, narration, emoji, comments that restate the code |
| [`sl_lints_size`](crates/sl_lints_size) | lints | spirit_level size lints: god crates, modules and functions by code lines, test code size, cyclomatic complexity |
| [`sl_lints_state`](crates/sl_lints_state) | lints | spirit_level shared-state lints from the fewer-locks rule: statics with interior mutability, static mut, thread-local state, lock().unwrap() counts, guards held across await or I/O, nested locks |
| [`sl_lints_surface`](crates/sl_lints_surface) | lints | spirit_level lints on a library's public surface: public items reachable from the root per 1000 code lines |
| [`sl_lints_ui`](crates/sl_lints_ui) | lints | spirit_level UI lints: theme colours and sizes, disabled controls with a reason, confirmed destructive actions, no blocking or raw spawns in UI code, wording, egui ids and repaints |
| [`sl_lints_units`](crates/sl_lints_units) | lints | spirit_level lints that move plain numbers into exact_units types: quantities, positions and counts, bounds, and quantities inside function bodies |
| [`sl_tool_audit`](crates/sl_tool_audit) | lints | spirit_level adapter of cargo-audit: vulnerable, unmaintained, unsound and yanked dependencies from the RustSec database |
| [`sl_tool_cargo`](crates/sl_tool_cargo) | lints | spirit_level checks driven by cargo: the MSRV build, no_std and wasm builds of the map's crates, cargo doc without warnings |
| [`sl_tool_clippy`](crates/sl_tool_clippy) | lints | spirit_level adapter of clippy: runs cargo clippy, reads its JSON messages, one lint per clippy group |
| [`sl_tool_deny`](crates/sl_tool_deny) | lints | spirit_level adapter of cargo-deny: advisories, bans, licences and sources, one lint per check |
| [`sl_tool_geiger`](crates/sl_tool_geiger) | lints | spirit_level adapter of geiger, linked: dependencies that use unsafe code, counted in their sources |
| [`sl_tool_hakari`](crates/sl_tool_hakari) | lints | spirit_level adapter of hakari, linked: the workspace-hack package is up to date and every member depends on it |
| [`sl_tool_machete`](crates/sl_tool_machete) | lints | spirit_level adapter of cargo-machete: unused dependencies, found fast by searching the sources |
| [`sl_tool_rustfmt`](crates/sl_tool_rustfmt) | lints | spirit_level adapter of rustfmt: every Rust file is formatted the way cargo fmt writes it |
| [`sl_tool_semver`](crates/sl_tool_semver) | lints | spirit_level adapter of cargo-semver-checks: stable crates change their API only with the version bump it needs |
| [`sl_tool_shear`](crates/sl_tool_shear) | lints | spirit_level adapter of cargo-shear: unused and misplaced dependencies, also unused workspace entries |
| [`sl_tool_taplo`](crates/sl_tool_taplo) | lints | spirit_level adapter of taplo: every TOML file is formatted as taplo writes it, with the formatted file as a fix |
| [`sl_tool_typos`](crates/sl_tool_typos) | lints | spirit_level adapter of typos: misspelled words in code, comments and docs, with the correction as a fix |
| [`sl_tool_udeps`](crates/sl_tool_udeps) | lints | spirit_level adapter of cargo-udeps: dependencies the compiler never loaded, on nightly |
| [`sl_engine`](crates/sl_engine) | engine | spirit_level's engine: picks the analyses a configuration enables, checks their needs, runs them, applies levels, exceptions and waivers, and applies exact fixes |
| [`sl_report`](crates/sl_report) | engine | spirit_level's report output: coloured text for people, a stable JSON document for CI and agents |
| [`sl_book`](crates/sl_book) | app | Generates spirit_level's book pages (one per lint, one per tier, the indexes and the nav) from the lints' own metadata and the shipped tiers, and a book's footer build stamp |
| [`spirit_level`](crates/spirit_level) | app | spirit_level as a library: the registry of every analysis, the shipped tier files, and a one-call check |
| [`spirit_level_cli`](crates/spirit_level_cli) | app | The spl command: check a project, explain a lint, list lints and tiers, show the resolved configuration, write the book pages, list the dependents to retest, measure the docs pages, count its own usage |
| [`sl_testkit`](crates/sl_testkit) | tests | Golden fixture harness for spirit_level analyses: runs one lint on small projects and compares the findings with each project's expect.toml |
<!-- /spirit-level:crates -->
