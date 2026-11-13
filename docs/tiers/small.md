# small

One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers.

Use it with `extends = "small"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0002](../lints/C0002.md) | allow-without-reason | `warn` | an exception to a lint says why, or it outlives its reason | small |
| [C0006](../lints/C0006.md) | print-in-library | `warn` | printing is the binary's choice | small |
| [C0008](../lints/C0008.md) | todo-macros | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code | small |
| [C0012](../lints/C0012.md) | waiver-invalid | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [C0013](../lints/C0013.md) | waiver-unused | `warn` | a waiver that allows nothing is left over from changed code | small |
| [M0001](../lints/M0001.md) | default-features-off | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [M0003](../lints/M0003.md) | git-dependency-pins | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit | small |
| [M0004](../lints/M0004.md) | no-sys-crates | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [M0005](../lints/M0005.md) | path-outside-workspace | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [M0006](../lints/M0006.md) | crate-docs | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [M0007](../lints/M0007.md) | edition-latest | `warn` | new code starts on the current edition | small |
| [M0009](../lints/M0009.md) | package-description | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [M0010](../lints/M0010.md) | package-license | `warn` | a crate without a licence cannot be reused or published | small |
| [M0012](../lints/M0012.md) | readme-per-crate | `warn` | every crate says what it is and how to use it | small |
| [M0014](../lints/M0014.md) | changelog-unreleased | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [M0015](../lints/M0015.md) | justfile-recipes | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [M0017](../lints/M0017.md) | profile-defaults | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [M0018](../lints/M0018.md) | repo-files | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [P0001](../lints/P0001.md) | agent-file-lines | `warn` | agents read AGENTS.md whole every session; a long one is skimmed, not followed | small |
| [P0009](../lints/P0009.md) | changelog-entry-length | `warn` | a CHANGELOG is a list: one short line per change | small |
| [S0002](../lints/S0002.md) | function-complexity | `warn` | more paths than can be tested or held in one's head | small |
| [S0003](../lints/S0003.md) | function-lines | `warn` | a function longer than a screen is changed without being read whole | small |
| [T0001](../lints/T0001.md) | lock-held-across | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer | small |
| [T0004](../lints/T0004.md) | static-interior-mutability | `warn` | every piece of mutable state has one owner; a global lock says why | small |
| [T0005](../lints/T0005.md) | static-mut | `deny` | global mutable state with no synchronisation; there is always a better owner | small |
| [X0005](../lints/X0005.md) | cargo-doc | `warn` | broken intra-doc links and tags render wrong on docs.rs and in the IDE | small |
| [X0019](../lints/X0019.md) | deny-advisories | `warn` | a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml | small |
| [X0026](../lints/X0026.md) | machete-unused-deps | `warn` | an unused dependency costs build time, audit and lockfile churn for nothing | small |
| [X0032](../lints/X0032.md) | typos-misspelling | `warn` | a misspelled identifier is copied to every call site; typos has almost no false positives | small |

## The file

```toml
# Tier 1: one crate. The house basics every Rust repo meets from its first commit.
[tier]
description = "One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers"

[lints.waiver-invalid]
level = "deny"
reason = "a waiver without a reason, for an unknown lint, or expired hides a finding without saying why"

[lints.waiver-unused]
level = "warn"
reason = "a waiver that allows nothing is left over from changed code"

[lints.package-description]
level = "warn"
reason = "the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it"

[lints.package-license]
level = "warn"
reason = "a crate without a licence cannot be reused or published"

[lints.edition-latest]
level = "warn"
reason = "new code starts on the current edition"

[lints.readme-per-crate]
level = "warn"
reason = "every crate says what it is and how to use it"

[lints.crate-docs]
level = "warn"
reason = "rustdoc's front page of the crate is its crate-level docs"

[lints.default-features-off]
level = "warn"
reason = "dependencies are added with default-features = false and only the features in use"

[lints.no-sys-crates]
level = "deny"
reason = "pure-Rust crates only: no bundled C, no -sys bindings without a written reason"

[lints.git-dependency-pins]
level = "warn"
reason = "git dependencies follow a branch; Cargo.lock fixes the commit"

[lints.path-outside-workspace]
level = "deny"
reason = "a path into a sibling checkout builds differently on every PC"

[lints.profile-defaults]
level = "warn"
reason = "split debug info and an incremental deploy profile: measured large build-time wins"

[lints.repo-files]
level = "warn"
reason = "every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG"

[lints.changelog-unreleased]
level = "warn"
reason = "work lands under [Unreleased]; the release step turns it into a version section"

[lints.justfile-recipes]
level = "warn"
reason = "every repo answers to the same recipes: default, test, lint, install, deploy"

[lints.function-lines]
level = "warn"
reason = "a function longer than a screen is changed without being read whole"

[lints.function-complexity]
level = "warn"
reason = "more paths than can be tested or held in one's head"

[lints.static-mut]
level = "deny"
reason = "global mutable state with no synchronisation; there is always a better owner"

[lints.static-interior-mutability]
level = "warn"
reason = "every piece of mutable state has one owner; a global lock says why"

[lints.lock-held-across]
level = "warn"
reason = "a guard across .await or I/O stalls every other caller for as long as the slowest peer"

[lints.todo-macros]
level = "warn"
reason = "no stop-gaps: a todo!() is a panic waiting in shipped code"

[lints.allow-without-reason]
level = "warn"
reason = "an exception to a lint says why, or it outlives its reason"

[lints.print-in-library]
level = "warn"
reason = "printing is the binary's choice"

[lints.changelog-entry-length]
level = "warn"
reason = "a CHANGELOG is a list: one short line per change"

[lints.agent-file-lines]
level = "warn"
reason = "agents read AGENTS.md whole every session; a long one is skimmed, not followed"

[lints.typos-misspelling]
level = "warn"
reason = "a misspelled identifier is copied to every call site; typos has almost no false positives"

[lints.machete-unused-deps]
level = "warn"
reason = "an unused dependency costs build time, audit and lockfile churn for nothing"

[lints.deny-advisories]
level = "warn"
reason = "a dependency with a known vulnerability is a risk taken unseen; needs no deny.toml"

[lints.cargo-doc]
level = "warn"
reason = "broken intra-doc links and tags render wrong on docs.rs and in the IDE"
```
