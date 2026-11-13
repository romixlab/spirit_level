# Lints

Every lint, grouped by category, with its level in each shipped tier (`off`: the tier does not enable it). A project changes any of them in its `spirit_level.toml` ([configuration](../configuration.md)).

## dependencies

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [default-features-off](default-features-off.md) | third-party dependencies are declared with default-features = false | warn | warn | warn | deny |
| [duplicate-versions](duplicate-versions.md) | no crate is in the dependency graph in two versions | off | warn | warn | deny |
| [git-dependency-pins](git-dependency-pins.md) | git dependencies follow a branch, with no rev or tag pin | warn | warn | deny | deny |
| [no-sys-crates](no-sys-crates.md) | the dependency graph has no -sys crates and no C build steps | deny | deny | deny | deny |
| [path-outside-workspace](path-outside-workspace.md) | path dependencies stay inside the workspace | deny | deny | deny | deny |

## manifest

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [crate-docs](crate-docs.md) | every crate root opens with crate-level documentation | warn | warn | warn | deny |
| [edition-latest](edition-latest.md) | every crate is on the current edition | warn | warn | warn | warn |
| [feature-docs](feature-docs.md) | every cargo feature has a comment saying what it turns on | off | off | off | warn |
| [package-description](package-description.md) | every crate has a description in its Cargo.toml | warn | warn | warn | deny |
| [package-license](package-license.md) | every crate states its licence | warn | warn | warn | warn |
| [publish-false](publish-false.md) | workspace crates are publish = false unless listed as published | off | off | warn | warn |
| [readme-per-crate](readme-per-crate.md) | every crate has a README | warn | deny | deny | deny |
| [rust-version-set](rust-version-set.md) | every crate declares its minimum Rust version | off | off | warn | deny |

## repo

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [changelog-unreleased](changelog-unreleased.md) | CHANGELOG.md has an Unreleased section | warn | warn | warn | warn |
| [justfile-recipes](justfile-recipes.md) | the justfile has the standard recipes | warn | warn | warn | warn |
| [justfile-simple](justfile-simple.md) | justfile recipes only call programs: a few lines, no inline scripts | off | off | off | warn |
| [profile-defaults](profile-defaults.md) | the dev profile splits debug info and a deploy profile builds incrementally | warn | warn | warn | warn |
| [repo-files](repo-files.md) | the repo carries its rule files | warn | warn | warn | warn |
| [toolchain-pinned](toolchain-pinned.md) | the repo pins its toolchain in rust-toolchain.toml | off | off | warn | deny |

## size

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [crate-lines](crate-lines.md) | a crate's src/ stays under its line limit | off | off | warn | warn |
| [function-complexity](function-complexity.md) | a function's cyclomatic complexity stays under its limit | warn | warn | warn | warn |
| [function-lines](function-lines.md) | a function stays under its line limit | warn | warn | warn | warn |
| [module-lines](module-lines.md) | a source file stays under its line limit | off | off | warn | warn |
| [test-lines](test-lines.md) | a crate's test code stays under its line limit | off | off | off | warn |

## state

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [lock-held-across](lock-held-across.md) | no lock guard is held across an .await or an I/O call | warn | warn | warn | deny |
| [lock-unwrap](lock-unwrap.md) | a crate keeps its lock().unwrap() calls under a limit | off | off | warn | warn |
| [nested-locks](nested-locks.md) | no second lock is taken while a guard is alive | off | warn | warn | warn |
| [static-interior-mutability](static-interior-mutability.md) | a static with interior mutability has one owner, or a reason why a global lock is right | warn | warn | warn | deny |
| [static-mut](static-mut.md) | no static mut | deny | deny | deny | deny |
| [thread-local-state](thread-local-state.md) | thread_local! state has one owner, or a reason | off | warn | warn | warn |

## waivers

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [waiver-invalid](waiver-invalid.md) | a waiver comment names known lints, gives a reason and has not expired | deny | deny | deny | deny |
| [waiver-unused](waiver-unused.md) | every waiver comment allows a finding | warn | warn | warn | deny |

## workspace

| Lint | Rule | small | small-workspace | medium | big |
|---|---|---|---|---|---|
| [crate-dir-name](crate-dir-name.md) | a member's directory is named after its crate | off | warn | warn | warn |
| [macro-crate-suffix](macro-crate-suffix.md) | proc-macro crates and only they end in _macros or _derive | off | warn | warn | warn |
| [workspace-dependencies](workspace-dependencies.md) | members take every dependency from [workspace.dependencies] | off | warn | deny | deny |
| [workspace-inheritance](workspace-inheritance.md) | members inherit the shared package keys from [workspace.package] | off | deny | deny | deny |
| [workspace-lints](workspace-lints.md) | the root declares [workspace.lints] and every member inherits them | off | deny | deny | deny |
