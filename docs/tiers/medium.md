# medium

A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict.

Extends [small-workspace](small-workspace.md): everything below is on top of it.

Use it with `extends = "medium"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Lint | Level | Reason | Set in |
|---|---|---|---|
| [changelog-unreleased](../lints/changelog-unreleased.md) | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [crate-dir-name](../lints/crate-dir-name.md) | `warn` | flat crates/<name>: the folder is the crate | small-workspace |
| [crate-docs](../lints/crate-docs.md) | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [crate-lines](../lints/crate-lines.md) | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step | medium |
| [default-features-off](../lints/default-features-off.md) | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [duplicate-versions](../lints/duplicate-versions.md) | `warn` | two versions of a crate double its build time and split its types | small-workspace |
| [edition-latest](../lints/edition-latest.md) | `warn` | new code starts on the current edition | small |
| [function-complexity](../lints/function-complexity.md) | `warn` | more paths than can be tested or held in one's head | small |
| [function-lines](../lints/function-lines.md) | `warn` | a function longer than a screen is changed without being read whole | small |
| [git-dependency-pins](../lints/git-dependency-pins.md) | `deny` | git dependencies follow a branch; a pin hides a break until it is old | medium |
| [justfile-recipes](../lints/justfile-recipes.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [lock-held-across](../lints/lock-held-across.md) | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [lock-unwrap](../lints/lock-unwrap.md) | `warn` | the count of lock().unwrap() per crate shows where shared state concentrates | medium |
| [macro-crate-suffix](../lints/macro-crate-suffix.md) | `warn` | _macros and _derive mean proc macros and nothing else | small-workspace |
| [module-lines](../lints/module-lines.md) | `warn` | a file of thousands of lines holds several subjects and collects every edit | medium |
| [nested-locks](../lints/nested-locks.md) | `warn` | two locks at once need one lock order everywhere | small-workspace |
| [no-sys-crates](../lints/no-sys-crates.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [package-description](../lints/package-description.md) | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [package-license](../lints/package-license.md) | `warn` | a crate without a licence cannot be reused or published | small |
| [path-outside-workspace](../lints/path-outside-workspace.md) | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [profile-defaults](../lints/profile-defaults.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [publish-false](../lints/publish-false.md) | `warn` | internal crates are a build unit, not an API; only listed crates are published | medium |
| [readme-per-crate](../lints/readme-per-crate.md) | `deny` | every crate of a workspace says what it is and what it may depend on | small-workspace |
| [repo-files](../lints/repo-files.md) | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [rust-version-set](../lints/rust-version-set.md) | `warn` | the MSRV is declared so it can be checked | medium |
| [static-interior-mutability](../lints/static-interior-mutability.md) | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [static-mut](../lints/static-mut.md) | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [thread-local-state](../lints/thread-local-state.md) | `warn` | thread-local state is a hidden side channel between functions | small-workspace |
| [toolchain-pinned](../lints/toolchain-pinned.md) | `warn` | every PC and the gate build with the same compiler | medium |
| [waiver-invalid](../lints/waiver-invalid.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [waiver-unused](../lints/waiver-unused.md) | `warn` | a waiver that allows nothing is left over from changed code | small |
| [workspace-dependencies](../lints/workspace-dependencies.md) | `deny` | one version per dependency per workspace, written once in [workspace.dependencies] | medium |
| [workspace-inheritance](../lints/workspace-inheritance.md) | `deny` | edition, rust-version, licence and repository are declared once in [workspace.package] | small-workspace |
| [workspace-lints](../lints/workspace-lints.md) | `deny` | every crate inherits [workspace.lints], so one table is the policy | small-workspace |

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
```
