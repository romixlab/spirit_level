# Lints

Every lint, grouped by category, with its level in each shipped tier (`off`: the tier does not enable it). A project changes any of them in its `spirit_level.toml` ([configuration](../configuration.md)).

## agents

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [agent-file-lines](agent-file-lines.md) | an agent file stays under its line limit | warn | warn | warn | warn | deny |
| [agent-file-links](agent-file-links.md) | links and @imports in agent files resolve | off | off | off | off | deny |
| [agent-rules-match-lints](agent-rules-match-lints.md) | limits an agent file states are the limits the lints check | off | off | off | warn | deny |
| [claude-md-duplicate](claude-md-duplicate.md) | CLAUDE.md points to AGENTS.md instead of copying it | off | off | off | off | deny |

## api

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [pub-surface](pub-surface.md) | a library's public surface stays small for its size | off | off | off | warn | off |

## audit

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [audit-unmaintained](audit-unmaintained.md) | no dependency is marked unmaintained or replaced in the RustSec database | off | off | off | off | off |
| [audit-unsound](audit-unsound.md) | no dependency is marked unsound in the RustSec database | off | off | off | off | off |
| [audit-vulnerabilities](audit-vulnerabilities.md) | no dependency has a published security vulnerability | off | off | off | off | off |
| [audit-yanked](audit-yanked.md) | Cargo.lock holds no yanked version | off | off | off | off | off |

## build

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [cargo-doc](cargo-doc.md) | cargo doc builds the workspace's documentation without a warning | warn | warn | deny | deny | off |
| [cargo-msrv](cargo-msrv.md) | every crate builds with the toolchain of its declared rust-version | off | off | warn | deny | off |
| [cargo-no-std](cargo-no-std.md) | every crate the map marks no_std builds for targets without std | off | off | off | deny | off |
| [cargo-wasm](cargo-wasm.md) | every crate the map marks wasm builds for wasm32-unknown-unknown | off | off | off | deny | off |

## clippy

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [clippy-build](clippy-build.md) | the workspace builds under clippy without compiler errors or warnings | off | off | off | off | off |
| [clippy-cargo](clippy-cargo.md) | no finding of the cargo-group clippy lints the project turned on | off | off | off | off | off |
| [clippy-complexity](clippy-complexity.md) | no finding of clippy's complexity group: simple things done simply | off | off | off | off | off |
| [clippy-correctness](clippy-correctness.md) | no finding of clippy's correctness group: code that is outright wrong | off | off | off | off | off |
| [clippy-nursery](clippy-nursery.md) | no finding of the nursery clippy lints the project turned on | off | off | off | off | off |
| [clippy-pedantic](clippy-pedantic.md) | no finding of the pedantic clippy lints the project turned on | off | off | off | off | off |
| [clippy-perf](clippy-perf.md) | no finding of clippy's perf group: no needlessly slow code | off | off | off | off | off |
| [clippy-restriction](clippy-restriction.md) | no finding of the restriction clippy lints the project turned on | off | off | off | off | off |
| [clippy-style](clippy-style.md) | no finding of clippy's style group: code written the idiomatic way | off | off | off | off | off |
| [clippy-suspicious](clippy-suspicious.md) | no finding of clippy's suspicious group: code that is most likely wrong | off | off | off | off | off |

## code

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [allow-without-reason](allow-without-reason.md) | every allow or expect of a lint says why | warn | warn | warn | deny | deny |
| [cli-subprocess](cli-subprocess.md) | workspace crates call each other as libraries, never by running the workspace's own binaries | off | off | deny | deny | off |
| [float-determinism](float-determinism.md) | deterministic crates compute float transcendentals through libm | off | off | off | deny | off |
| [panic-in-library](panic-in-library.md) | library code returns errors instead of panicking | off | off | warn | warn | off |
| [print-in-library](print-in-library.md) | library code does not print | warn | warn | warn | warn | off |
| [random-state-hash](random-state-hash.md) | deterministic crates use no randomly seeded hash maps | off | off | off | deny | off |
| [todo-macros](todo-macros.md) | no todo!() or unimplemented!() outside tests | warn | warn | warn | deny | deny |

## commits

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [commit-body-lines](commit-body-lines.md) | a commit body stays short | off | off | off | off | warn |
| [commit-prose](commit-prose.md) | commit messages use no filler, narration or emoji and do not describe the diff | off | off | off | off | warn |
| [commit-subject-length](commit-subject-length.md) | a commit subject stays under its length limit | off | off | warn | warn | deny |

## dead-code

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [dead-pub](dead-pub.md) | public items of workspace libraries are used outside their crate | off | off | off | warn | off |

## deny

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [deny-advisories](deny-advisories.md) | no dependency has a security advisory, is unmaintained, unsound or yanked | warn | warn | deny | deny | off |
| [deny-bans](deny-bans.md) | the dependency graph holds no banned crate and no duplicate deny.toml refuses | off | off | off | deny | off |
| [deny-licenses](deny-licenses.md) | every crate's licence is one deny.toml allows | off | off | warn | deny | off |
| [deny-sources](deny-sources.md) | every crate comes from a registry or git source deny.toml allows | off | off | off | deny | off |

## dependencies

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [default-features-off](default-features-off.md) | third-party dependencies are declared with default-features = false | warn | warn | warn | deny | off |
| [duplicate-versions](duplicate-versions.md) | no crate is in the dependency graph in two versions | off | warn | warn | deny | off |
| [git-dependency-pins](git-dependency-pins.md) | git dependencies follow a branch, with no rev or tag pin | warn | warn | deny | deny | off |
| [no-sys-crates](no-sys-crates.md) | the dependency graph has no -sys crates and no C build steps | deny | deny | deny | deny | off |
| [path-outside-workspace](path-outside-workspace.md) | path dependencies stay inside the workspace | deny | deny | deny | deny | off |

## errors

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [error-enum-growth](error-enum-growth.md) | a big error enum wraps the errors it comes from instead of listing every failure flat | off | off | warn | warn | off |

## geiger

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [geiger-unsafe-deps](geiger-unsafe-deps.md) | every dependency that uses unsafe code is one the project accepted | off | off | off | off | off |

## graph

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [core-crate-deps](core-crate-deps.md) | a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates | off | off | warn | deny | off |
| [core-crate-types](core-crate-types.md) | a _core crate holds types: no I/O, no threads, no async runtime | off | off | off | deny | off |
| [crate-cycles](crate-cycles.md) | workspace crates form no cycle, dev-dependencies included | off | warn | deny | deny | off |
| [crate-table](crate-table.md) | the crate table in the README is the one generated from the manifests and the map | off | off | off | deny | off |
| [external-form](external-form.md) | crates of other projects come by git as the map lists them, used only where their role allows | off | off | off | deny | off |
| [forbidden-closure](forbidden-closure.md) | a crate never reaches the crates the map forbids it, directly or through others | off | off | off | deny | off |
| [layer-edges](layer-edges.md) | every dependency edge between workspace crates is allowed by the map | off | off | off | deny | off |
| [map-complete](map-complete.md) | the crate map and the workspace list the same crates | off | off | off | deny | off |
| [third-party-placement](third-party-placement.md) | heavy third-party crates are used only by the crates the map names | off | off | off | deny | off |
| [unsafe-allow-list](unsafe-allow-list.md) | unsafe code is forbidden except in the crates the map marks unsafe | off | off | off | deny | off |

## hakari

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [hakari-generate](hakari-generate.md) | the workspace-hack package's dependencies are the ones hakari computes | off | off | off | warn | off |
| [hakari-manage-deps](hakari-manage-deps.md) | every member hakari manages depends on the workspace-hack package, and no other | off | off | off | warn | off |

## hygiene

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [agent-config-permissions](agent-config-permissions.md) | committed agent settings grant no blanket permissions | off | off | off | off | deny |
| [agent-config-secrets](agent-config-secrets.md) | committed agent configuration holds no secrets | off | off | off | off | deny |
| [agent-session-files](agent-session-files.md) | no agent transcripts or per-person agent files are committed | off | off | off | off | deny |
| [leftover-files](leftover-files.md) | no merge, patch, editor or scratch leftovers are committed | off | off | off | off | deny |

## machete

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [machete-unused-deps](machete-unused-deps.md) | no crate declares a dependency its sources never use | warn | warn | warn | deny | off |

## manifest

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [crate-docs](crate-docs.md) | every crate root opens with crate-level documentation | warn | warn | warn | deny | off |
| [edition-latest](edition-latest.md) | every crate is on the current edition | warn | warn | warn | warn | off |
| [feature-docs](feature-docs.md) | every cargo feature has a comment saying what it turns on | off | off | off | warn | off |
| [package-description](package-description.md) | every crate has a description in its Cargo.toml | warn | warn | warn | deny | off |
| [package-license](package-license.md) | every crate states its licence | warn | warn | warn | warn | off |
| [publish-false](publish-false.md) | workspace crates are publish = false unless listed as published | off | off | warn | warn | off |
| [readme-per-crate](readme-per-crate.md) | every crate has a README | warn | deny | deny | deny | off |
| [rust-version-set](rust-version-set.md) | every crate declares its minimum Rust version | off | off | warn | deny | off |

## prose

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [changelog-describes-diff](changelog-describes-diff.md) | a CHANGELOG entry says what changed for the user, not which code was edited | off | off | off | off | warn |
| [changelog-entry-length](changelog-entry-length.md) | a CHANGELOG entry is one short line | warn | warn | warn | warn | deny |
| [emoji](emoji.md) | docs and comments carry no emoji | off | off | off | off | warn |
| [filler-phrases](filler-phrases.md) | docs and comments use no filler phrases | off | off | off | off | deny |
| [narration](narration.md) | docs and comments state facts, not how they were written | off | off | off | off | deny |
| [paragraph-sentences](paragraph-sentences.md) | paragraphs in docs stay short | off | off | off | off | warn |
| [restating-comments](restating-comments.md) | a comment says what the code cannot | off | off | off | off | warn |
| [sentence-words](sentence-words.md) | sentences in docs stay short | off | off | off | off | warn |
| [thinking-comments](thinking-comments.md) | comments state conclusions, not a stream of thought | off | off | off | off | deny |

## repo

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [changelog-unreleased](changelog-unreleased.md) | CHANGELOG.md has an Unreleased section | warn | warn | warn | warn | off |
| [justfile-recipes](justfile-recipes.md) | the justfile has the standard recipes | warn | warn | warn | warn | off |
| [justfile-simple](justfile-simple.md) | justfile recipes only call programs: a few lines, no inline scripts | off | off | off | warn | off |
| [profile-defaults](profile-defaults.md) | the dev profile splits debug info and a deploy profile builds incrementally | warn | warn | warn | warn | off |
| [repo-files](repo-files.md) | the repo carries its rule files | warn | warn | warn | warn | deny |
| [toolchain-pinned](toolchain-pinned.md) | the repo pins its toolchain in rust-toolchain.toml | off | off | warn | deny | off |

## rustfmt

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [rustfmt-unformatted](rustfmt-unformatted.md) | every Rust file is formatted the way cargo fmt writes it | off | off | off | off | off |

## semver

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [semver-checks-breaking](semver-checks-breaking.md) | a stable crate changes its public API only with the version bump the change needs | off | off | off | deny | off |

## shear

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [shear-misplaced-deps](shear-misplaced-deps.md) | a dependency only tests use is a dev-dependency | off | off | off | off | off |
| [shear-unused-deps](shear-unused-deps.md) | no crate declares a dependency it never uses, and every workspace dependency is used | off | off | off | off | off |

## size

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [crate-lines](crate-lines.md) | a crate's src/ stays under its line limit | off | off | warn | warn | off |
| [function-complexity](function-complexity.md) | a function's cyclomatic complexity stays under its limit | warn | warn | warn | warn | off |
| [function-lines](function-lines.md) | a function stays under its line limit | warn | warn | warn | warn | off |
| [module-lines](module-lines.md) | a source file stays under its line limit | off | off | warn | warn | off |
| [test-lines](test-lines.md) | a crate's test code stays under its line limit | off | off | off | warn | off |

## state

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [lock-held-across](lock-held-across.md) | no lock guard is held across an .await or an I/O call | warn | warn | warn | deny | off |
| [lock-unwrap](lock-unwrap.md) | a crate keeps its lock().unwrap() calls under a limit | off | off | warn | warn | off |
| [nested-locks](nested-locks.md) | no second lock is taken while a guard is alive | off | warn | warn | warn | off |
| [static-interior-mutability](static-interior-mutability.md) | a static with interior mutability has one owner, or a reason why a global lock is right | warn | warn | warn | deny | off |
| [static-mut](static-mut.md) | no static mut | deny | deny | deny | deny | off |
| [thread-local-state](thread-local-state.md) | thread_local! state has one owner, or a reason | off | warn | warn | warn | off |

## stop-gap

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [commented-out-code](commented-out-code.md) | no code is left commented out | off | off | off | off | deny |
| [dead-features](dead-features.md) | every cargo feature enables something or is tested by a cfg | off | off | off | off | warn |
| [lint-table-allow-reason](lint-table-allow-reason.md) | a lint allowed in a Cargo.toml [lints] table says why in a comment | off | off | off | off | deny |
| [near-duplicate-fns](near-duplicate-fns.md) | no two functions are near copies of each other | off | off | off | off | warn |
| [near-duplicate-modules](near-duplicate-modules.md) | no two modules are near copies of each other | off | off | off | off | deny |
| [stop-gap-comments](stop-gap-comments.md) | comments carry no stop-gap markers: TODO, FIXME, HACK, for now, workaround | off | off | off | deny | deny |

## structure

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [module-graph-scc](module-graph-scc.md) | no module cycle holds most of a crate's code | off | off | warn | warn | off |

## taplo

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [taplo-fmt](taplo-fmt.md) | every TOML file parses and is formatted as taplo writes it | off | warn | warn | deny | off |

## typos

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [typos-misspelling](typos-misspelling.md) | no misspelled word in code, comments, docs or file names | warn | warn | warn | deny | off |

## udeps

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [udeps-unused-deps](udeps-unused-deps.md) | no crate declares a dependency the compiler never loads | off | off | off | off | off |

## waivers

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [waiver-invalid](waiver-invalid.md) | a waiver comment names known lints, gives a reason and has not expired | deny | deny | deny | deny | off |
| [waiver-unused](waiver-unused.md) | every waiver comment allows a finding | warn | warn | warn | deny | off |

## workspace

| Lint | Rule | small | small-workspace | medium | big | ai |
|---|---|---|---|---|---|---|
| [crate-dir-name](crate-dir-name.md) | a member's directory is named after its crate | off | warn | warn | warn | off |
| [macro-crate-suffix](macro-crate-suffix.md) | proc-macro crates and only they end in _macros or _derive | off | warn | warn | warn | off |
| [workspace-dependencies](workspace-dependencies.md) | members take every dependency from [workspace.dependencies] | off | warn | deny | deny | off |
| [workspace-inheritance](workspace-inheritance.md) | members inherit the shared package keys from [workspace.package] | off | deny | deny | deny | off |
| [workspace-lints](workspace-lints.md) | the root declares [workspace.lints] and every member inherits them | off | deny | deny | deny | off |
