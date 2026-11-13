# git-dependency-pins

**Rule:** Git dependencies follow a branch, with no rev or tag pin.

| | |
|---|---|
| Category | [dependencies](index.md#dependencies) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit |
| [small-workspace](../tiers/small-workspace.md) | `warn` | git dependencies follow a branch; Cargo.lock fixes the commit |
| [medium](../tiers/medium.md) | `deny` | git dependencies follow a branch; a pin hides a break until it is old |
| [big](../tiers/big.md) | `deny` | git dependencies follow a branch; a pin hides a break until it is old |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `forbid` | list of strings | `["rev", "tag"]` | the pins that are refused |

## Why

A git dependency follows a branch; `Cargo.lock` fixes the commit and `cargo update -p NAME` moves it on. A `rev` or `tag` pin in `Cargo.toml` hides a break in the other repo until the pin is old, and makes every update an edit in two places.

## How to fix

Drop `rev` / `tag` and give `branch` (or nothing, for the default branch). The pins refused are the parameter `forbid`.

## Configure

```toml
[lints.git-dependency-pins]
level = "deny"
forbid = ["rev", "tag"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(git-dependency-pins) <reason>` ([waivers](../waivers.md)).
