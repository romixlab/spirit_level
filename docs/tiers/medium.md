# medium

A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict.

Extends [small-workspace](small-workspace.md): everything below is on top of it.

Use it with `extends = "medium"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0002](../lints/C0002.md) | allow-without-reason | `warn` | an exception to a lint says why, or it outlives its reason | small |
| [C0003](../lints/C0003.md) | cli-subprocess | `deny` | components call each other as libraries, not through their CLIs | medium |
| [C0005](../lints/C0005.md) | panic-in-library | `warn` | a library returns errors; its caller decides what is fatal | medium |
| [C0006](../lints/C0006.md) | print-in-library | `warn` | printing is the binary's choice | small |
| [C0008](../lints/C0008.md) | todo-macros | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [C0010](../lints/C0010.md) | error-enum-growth | `warn` | a flat error enum grows a variant per failure and loses the source errors; wrap them by layer | medium |
| [C0011](../lints/C0011.md) | module-graph-scc | `warn` | a module cycle holding most of a crate can't be split into crates; cut it while it is small | medium |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [C0013](../lints/C0013.md) | waiver-unused | `warn` | a waiver that allows nothing is left over from changed code | small |
| [G0001](../lints/G0001.md) | core-crate-deps | `warn` | a _core crate holds shared types and drags nothing in | medium |
| [G0003](../lints/G0003.md) | crate-cycles | `deny` | crates in a cycle cannot be built or tested apart | medium |
| [M0001](../lints/M0001.md) | default-features-off | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [M0002](../lints/M0002.md) | duplicate-versions | `warn` | two versions of a crate double its build time and split its types | small-workspace |
| [M0003](../lints/M0003.md) | git-dependency-pins | `deny` | git dependencies follow a branch; a pin hides a break until it is old | medium |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0009](../lints/M0009.md) | package-description | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0011](../lints/M0011.md) | publish-false | `warn` | internal crates are a build unit, not an API; only listed crates are published | medium |
| [M0012](../lints/M0012.md) | readme-per-crate | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [M0013](../lints/M0013.md) | rust-version-set | `warn` | the MSRV is declared so it can be checked | medium |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [M0019](../lints/M0019.md) | toolchain-pinned | `warn` | every PC and the gate build with the same compiler | medium |
| [M0020](../lints/M0020.md) | crate-dir-name | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [M0021](../lints/M0021.md) | macro-crate-suffix | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [M0022](../lints/M0022.md) | workspace-dependencies | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] | medium |
| [M0023](../lints/M0023.md) | workspace-inheritance | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [M0024](../lints/M0024.md) | workspace-lints | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0007](../lints/P0007.md) | commit-subject-length | `warn` | the subject is what one-line logs and reviews show | medium |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [S0001](../lints/S0001.md) | crate-lines | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step | medium |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [S0004](../lints/S0004.md) | module-lines | `warn` | a file of thousands of lines holds several subjects and collects every edit | medium |
| [T0001](../lints/T0001.md) | lock-held-across | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [T0002](../lints/T0002.md) | lock-unwrap | `warn` | the count of lock().unwrap() per crate shows where shared state concentrates | medium |
| [T0003](../lints/T0003.md) | nested-locks | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [T0006](../lints/T0006.md) | thread-local-state | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [X0005](../lints/X0005.md) | cargo-doc | `deny` | the docs of internal crates are the map of the code; they build without a warning | medium |
| [X0006](../lints/X0006.md) | cargo-msrv | `warn` | the declared rust-version is built, so it stays true | medium |
| [X0019](../lints/X0019.md) | deny-advisories | `deny` | a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml | medium |
| [X0021](../lints/X0021.md) | deny-licenses | `warn` | every linked licence is one the project accepts, listed once in deny.toml | medium |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` | an unused dependency costs build time, audit and lockfile churn for nothing | small |
| [X0031](../lints/X0031.md) | taplo-fmt | `warn` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change | small-workspace |
| [X0032](../lints/X0032.md) | typos-misspelling | `warn` | a misspelled identifier is copied to every call site; typos has almost no false positives | small |

## The file

```toml
# Tier 3: a medium workspace (wire_weaver's size): internal crates, MSRV, pinned toolchain.
extends = "small-workspace"

[tier]
description = "A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict"

[lints.rust-version-set]
level = "warn"
reason = "the MSRV is declared so it can be checked"

[lints.publish-false]
level = "warn"
reason = "internal crates are a build unit, not an API; only listed crates are published"

[lints.toolchain-pinned]
level = "warn"
reason = "every PC and the gate build with the same compiler"

[lints.workspace-dependencies]
level = "deny"
reason = "one version per dependency per workspace, written once in [workspace.dependencies]"

[lints.git-dependency-pins]
level = "deny"
reason = "git dependencies follow a branch; a pin hides a break until it is old"

[lints.module-lines]
level = "warn"
reason = "a file of thousands of lines holds several subjects and collects every edit"

[lints.crate-lines]
level = "warn"
reason = "split a crate before it passes ~20,000 lines: one crate is one serial compile step"

[lints.lock-unwrap]
level = "warn"
reason = "the count of lock().unwrap() per crate shows where shared state concentrates"

[lints.crate-cycles]
level = "deny"
reason = "crates in a cycle cannot be built or tested apart"

[lints.core-crate-deps]
level = "warn"
reason = "a _core crate holds shared types and drags nothing in"

[lints.panic-in-library]
level = "warn"
reason = "a library returns errors; its caller decides what is fatal"

[lints.cli-subprocess]
level = "deny"
reason = "components call each other as libraries, not through their CLIs"

[lints.commit-subject-length]
level = "warn"
reason = "the subject is what one-line logs and reviews show"

[lints.module-graph-scc]
level = "warn"
reason = "a module cycle holding most of a crate can't be split into crates; cut it while it is small"


[lints.error-enum-growth]
level = "warn"
reason = "a flat error enum grows a variant per failure and loses the source errors; wrap them by layer"

[lints.deny-advisories]
level = "deny"
reason = "a vulnerable dependency is updated, or its advisory ignored with a reason in deny.toml"

[lints.deny-licenses]
level = "warn"
reason = "every linked licence is one the project accepts, listed once in deny.toml"

[lints.cargo-msrv]
level = "warn"
reason = "the declared rust-version is built, so it stays true"

[lints.cargo-doc]
level = "deny"
reason = "the docs of internal crates are the map of the code; they build without a warning"
```
