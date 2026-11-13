# big

A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity.

Extends [medium](medium.md): everything below is on top of it.

Use it with `extends = "big"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Lint | Level | Reason | Set in |
|---|---|---|---|
| [allow-without-reason](../lints/allow-without-reason.md) | `deny` | an exception to a lint says why, or it outlives its reason | big |
| [changelog-unreleased](../lints/changelog-unreleased.md) | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [cli-subprocess](../lints/cli-subprocess.md) | `deny` | components call each other as libraries, not through their CLIs | medium |
| [core-crate-deps](../lints/core-crate-deps.md) | `deny` | a _core crate holds shared types and drags nothing in | big |
| [core-crate-types](../lints/core-crate-types.md) | `deny` | _core crates hold only types: no I/O, threads or runtimes | big |
| [crate-cycles](../lints/crate-cycles.md) | `deny` | crates in a cycle cannot be built or tested apart | medium |
| [crate-dir-name](../lints/crate-dir-name.md) | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [crate-docs](../lints/crate-docs.md) | `deny` | rustdoc's front page of the crate is its crate-level docs | big |
| [crate-lines](../lints/crate-lines.md) | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step | medium |
| [crate-table](../lints/crate-table.md) | `deny` | the crate table is generated from the manifests and the map, never written by hand | big |
| [default-features-off](../lints/default-features-off.md) | `deny` | feature creep in a big graph costs every build | big |
| [duplicate-versions](../lints/duplicate-versions.md) | `deny` | two versions of a crate double its build time and split its types | big |
| [edition-latest](../lints/edition-latest.md) | `warn` | new code starts on the current edition | small |
| [external-form](../lints/external-form.md) | `deny` | outside projects come by git on a branch, listed once, used where their role allows | big |
| [feature-docs](../lints/feature-docs.md) | `warn` | a feature without a comment cannot be chosen by a reader | big |
| [float-determinism](../lints/float-determinism.md) | `deny` | crates the map marks deterministic compute the same bits on every platform | big |
| [forbidden-closure](../lints/forbidden-closure.md) | `deny` | some crates must never end up in a binary, however far away they arrive | big |
| [function-complexity](../lints/function-complexity.md) | `warn` | more paths than can be tested or held in one's head | small |
| [function-lines](../lints/function-lines.md) | `warn` | a function longer than a screen is changed without being read whole | small |
| [git-dependency-pins](../lints/git-dependency-pins.md) | `deny` | git dependencies follow a branch; a pin hides a break until it is old | medium |
| [justfile-recipes](../lints/justfile-recipes.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [justfile-simple](../lints/justfile-simple.md) | `warn` | recipes call programs; logic lives in Rust where it is tested | big |
| [layer-edges](../lints/layer-edges.md) | `deny` | layers are declared and checked edge by edge; move a crate, never reorder layers | big |
| [lock-held-across](../lints/lock-held-across.md) | `deny` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | big |
| [lock-unwrap](../lints/lock-unwrap.md) | `warn` | the count of lock().unwrap() per crate shows where shared state concentrates | medium |
| [macro-crate-suffix](../lints/macro-crate-suffix.md) | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [map-complete](../lints/map-complete.md) | `deny` | the crate map is the source of truth: every crate in it, nothing gone | big |
| [module-lines](../lints/module-lines.md) | `warn` | a file of thousands of lines holds several subjects and collects every edit | medium |
| [nested-locks](../lints/nested-locks.md) | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [no-sys-crates](../lints/no-sys-crates.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [package-description](../lints/package-description.md) | `deny` | the generated crate table and READMEs are made from it | big |
| [package-license](../lints/package-license.md) | `warn` | a crate without a licence cannot be reused or published | small |
| [panic-in-library](../lints/panic-in-library.md) | `warn` | a library returns errors; its caller decides what is fatal | medium |
| [path-outside-workspace](../lints/path-outside-workspace.md) | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [print-in-library](../lints/print-in-library.md) | `warn` | printing is the binary's choice | small |
| [profile-defaults](../lints/profile-defaults.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [publish-false](../lints/publish-false.md) | `warn` | internal crates are a build unit, not an API; only listed crates are published | medium |
| [random-state-hash](../lints/random-state-hash.md) | `deny` | crates the map marks deterministic iterate in the same order every run | big |
| [readme-per-crate](../lints/readme-per-crate.md) | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [repo-files](../lints/repo-files.md) | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [rust-version-set](../lints/rust-version-set.md) | `deny` | the MSRV is declared so it can be checked | big |
| [static-interior-mutability](../lints/static-interior-mutability.md) | `deny` | every piece of mutable state has one owner; a global lock says why on the item | big |
| [static-mut](../lints/static-mut.md) | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [test-lines](../lints/test-lines.md) | `warn` | test code builds and links too | big |
| [third-party-placement](../lints/third-party-placement.md) | `deny` | heavy dependencies sit in one crate each | big |
| [thread-local-state](../lints/thread-local-state.md) | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [todo-macros](../lints/todo-macros.md) | `deny` | no stop-gaps: a todo!() is a panic waiting in shipped code | big |
| [toolchain-pinned](../lints/toolchain-pinned.md) | `deny` | every PC and the gate build with the same compiler | big |
| [unsafe-allow-list](../lints/unsafe-allow-list.md) | `deny` | unsafe is forbidden except in the crates the map lists | big |
| [waiver-invalid](../lints/waiver-invalid.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [waiver-unused](../lints/waiver-unused.md) | `deny` | every waiver allows something, or it goes | big |
| [workspace-dependencies](../lints/workspace-dependencies.md) | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] | medium |
| [workspace-inheritance](../lints/workspace-inheritance.md) | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [workspace-lints](../lints/workspace-lints.md) | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |

## The file

```toml
# Tier 4: a big workspace (rerun, zed, bevy, intent): layers declared and checked, sizes watched, everything strict.
extends = "medium"

[tier]
description = "A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity"

[lints.waiver-unused]
level = "deny"
reason = "every waiver allows something, or it goes"

[lints.package-description]
level = "deny"
reason = "the generated crate table and READMEs are made from it"

[lints.crate-docs]
level = "deny"
reason = "rustdoc's front page of the crate is its crate-level docs"

[lints.default-features-off]
level = "deny"
reason = "feature creep in a big graph costs every build"

[lints.duplicate-versions]
level = "deny"
reason = "two versions of a crate double its build time and split its types"

[lints.rust-version-set]
level = "deny"
reason = "the MSRV is declared so it can be checked"

[lints.toolchain-pinned]
level = "deny"
reason = "every PC and the gate build with the same compiler"

[lints.feature-docs]
level = "warn"
reason = "a feature without a comment cannot be chosen by a reader"

[lints.justfile-simple]
level = "warn"
reason = "recipes call programs; logic lives in Rust where it is tested"

[lints.test-lines]
level = "warn"
reason = "test code builds and links too"

[lints.static-interior-mutability]
level = "deny"
reason = "every piece of mutable state has one owner; a global lock says why on the item"

[lints.lock-held-across]
level = "deny"
reason = "a guard across .await or I/O stalls every other caller for as long as the slowest peer"

[lints.map-complete]
level = "deny"
reason = "the crate map is the source of truth: every crate in it, nothing gone"

[lints.layer-edges]
level = "deny"
reason = "layers are declared and checked edge by edge; move a crate, never reorder layers"

[lints.forbidden-closure]
level = "deny"
reason = "some crates must never end up in a binary, however far away they arrive"

[lints.external-form]
level = "deny"
reason = "outside projects come by git on a branch, listed once, used where their role allows"

[lints.third-party-placement]
level = "deny"
reason = "heavy dependencies sit in one crate each"

[lints.core-crate-deps]
level = "deny"
reason = "a _core crate holds shared types and drags nothing in"

[lints.core-crate-types]
level = "deny"
reason = "_core crates hold only types: no I/O, threads or runtimes"

[lints.crate-table]
level = "deny"
reason = "the crate table is generated from the manifests and the map, never written by hand"

[lints.unsafe-allow-list]
level = "deny"
reason = "unsafe is forbidden except in the crates the map lists"

[lints.todo-macros]
level = "deny"
reason = "no stop-gaps: a todo!() is a panic waiting in shipped code"

[lints.allow-without-reason]
level = "deny"
reason = "an exception to a lint says why, or it outlives its reason"

[lints.float-determinism]
level = "deny"
reason = "crates the map marks deterministic compute the same bits on every platform"

[lints.random-state-hash]
level = "deny"
reason = "crates the map marks deterministic iterate in the same order every run"
```
