# spirit_level

A linter for Rust projects of every size, from one crate to a workspace of hundreds. Tiers are TOML files of lints; a project picks one and adds its own spin.

Working name. Private for now.

## Crates

<!-- spirit-level:crates -->
| Crate | Layer | Description |
|---|---|---|
| [`sl_core`](crates/sl_core) | core | spirit_level's shared types: lint metadata, levels, parameters, diagnostics, spans, fixes, dates and globs |
| [`sl_config`](crates/sl_config) | model | spirit_level's configuration format: tiers, extends, lint levels, parameters, reasons, expiries, scoped exceptions and the crate map |
| [`sl_model`](crates/sl_model) | model | spirit_level's project model: cargo metadata, manifests with spans, Rust sources parsed with syn, line counts |
| [`sl_analysis`](crates/sl_analysis) | analysis | The one trait every spirit_level analysis implements, its run context, and syntax helpers shared by the source analyses |
| [`sl_lints_ai`](crates/sl_lints_ai) | lints | spirit_level lints for code written with agents: stop-gap comments, commented-out code, near-duplicate functions, dead features, committed agent secrets and transcripts |
| [`sl_lints_code`](crates/sl_lints_code) | lints | spirit_level source lints: todo!() and unimplemented!(), panics and prints in library code, allow without reason, float and hashing determinism, calls of the workspace's own CLIs |
| [`sl_lints_graph`](crates/sl_lints_graph) | lints | spirit_level crate-graph lints against the crate map: layers edge by edge, forbidden closures, external git form, third-party placement, _core rules, cycles, the generated crate table, the unsafe allow-list |
| [`sl_lints_manifest`](crates/sl_lints_manifest) | lints | spirit_level lints on manifests and repo files: package metadata, workspace inheritance and lints, dependency form, -sys crates, duplicates, profiles, toolchain, repo rule files, justfile, feature docs |
| [`sl_lints_prose`](crates/sl_lints_prose) | lints | spirit_level lints on prose: CHANGELOG and commit limits, agent files, filler, narration, emoji, comments that restate the code |
| [`sl_lints_size`](crates/sl_lints_size) | lints | spirit_level size lints: god crates, modules and functions by code lines, test code size, cyclomatic complexity |
| [`sl_lints_state`](crates/sl_lints_state) | lints | spirit_level shared-state lints from the fewer-locks rule: statics with interior mutability, static mut, thread-local state, lock().unwrap() counts, guards held across await or I/O, nested locks |
| [`sl_engine`](crates/sl_engine) | engine | spirit_level's engine: picks the analyses a configuration enables, checks their needs, runs them, applies levels, exceptions and waivers, and applies exact fixes |
| [`sl_report`](crates/sl_report) | engine | spirit_level's report output: coloured text for people, a stable JSON document for CI and agents |
| [`sl_book`](crates/sl_book) | app | Generates spirit_level's book pages (one per lint, one per tier, the indexes and the nav) from the lints' own metadata and the shipped tiers |
| [`spirit_level`](crates/spirit_level) | app | spirit_level as a library: the registry of every analysis, the shipped tier files, and a one-call check |
| [`spirit_level_cli`](crates/spirit_level_cli) | app | The spirit-level command: check a project, explain a lint, list lints and tiers, show the resolved configuration, write the book pages |
| [`sl_testkit`](crates/sl_testkit) | tests | Golden fixture harness for spirit_level analyses: runs one lint on small projects and compares the findings with each project's expect.toml |
<!-- /spirit-level:crates -->

## Licence

MIT OR Apache-2.0. See `CONTRIBUTING.md`.
