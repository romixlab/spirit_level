# small

One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers.

Use it with `extends = "small"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Lint | Level | Reason | Set in |
|---|---|---|---|
| [changelog-unreleased](../lints/changelog-unreleased.md) | `warn` | work lands under [Unreleased]; the release step turns it into a version section | small |
| [crate-docs](../lints/crate-docs.md) | `warn` | rustdoc's front page of the crate is its crate-level docs | small |
| [default-features-off](../lints/default-features-off.md) | `warn` | dependencies are added with default-features = false and only the features in use | small |
| [edition-latest](../lints/edition-latest.md) | `warn` | new code starts on the current edition | small |
| [function-complexity](../lints/function-complexity.md) | `warn` | more paths than can be tested or held in one's head | small |
| [function-lines](../lints/function-lines.md) | `warn` | a function longer than a screen is changed without being read whole | small |
| [git-dependency-pins](../lints/git-dependency-pins.md) | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit | small |
| [justfile-recipes](../lints/justfile-recipes.md) | `warn` | every repo answers to the same recipes: default, test, lint, install, deploy | small |
| [no-sys-crates](../lints/no-sys-crates.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason | small |
| [package-description](../lints/package-description.md) | `warn` | the crate's one-line description is written once, in Cargo.toml; tables and READMEs are made from it | small |
| [package-license](../lints/package-license.md) | `warn` | a crate without a licence cannot be reused or published | small |
| [path-outside-workspace](../lints/path-outside-workspace.md) | `deny` | a path into a sibling checkout builds differently on every PC | small |
| [profile-defaults](../lints/profile-defaults.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins | small |
| [readme-per-crate](../lints/readme-per-crate.md) | `warn` | every crate says what it is and how to use it | small |
| [repo-files](../lints/repo-files.md) | `warn` | every work repo carries README, AGENTS, CLAUDE, FEATURES and CHANGELOG | small |
| [waiver-invalid](../lints/waiver-invalid.md) | `deny` | a waiver without a reason, for an unknown lint, or expired hides a finding without saying why | small |
| [waiver-unused](../lints/waiver-unused.md) | `warn` | a waiver that allows nothing is left over from changed code | small |

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
```
