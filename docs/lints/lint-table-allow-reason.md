# lint-table-allow-reason

**Rule:** A lint allowed in a Cargo.toml [lints] table says why in a comment.

| | |
|---|---|
| Category | [stop-gap](index.md#stop-gap) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | `deny` | a lint allowed for a whole crate in Cargo.toml says why |
| [embedded](../tiers/embedded.md) | off |  |

## Why

A lint turned off in `[lints]` or `[workspace.lints]` of a Cargo.toml applies to the whole crate or workspace. Without a reason nobody knows whether it can be turned back on. `#[allow]` attributes in code are covered by `allow-without-reason`; this rule covers the manifests.

An entry whose level is `allow` (`x = "allow"` or `x = { level = "allow", ... }`) needs a `#` comment on its line or the line above.

```toml
[workspace.lints.clippy]
# the builder API takes many arguments by design
too_many_arguments = "allow"
module_name_repetitions = "allow"      <- reported
```

## How to fix

Add the reason as a comment, or remove the entry.

## Configure

```toml
[lints.lint-table-allow-reason]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(lint-table-allow-reason) <reason>` ([waivers](../waivers.md)).
