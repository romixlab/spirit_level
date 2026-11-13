# external-form

**Rule:** Crates of other projects come by git as the map lists them, used only where their role allows.

| | |
|---|---|
| Category | [graph](index.md#graph) |
| Source | spirit_level |
| Fix | by hand |
| Needs | the crate map ([map] or map-file) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | outside projects come by git on a branch, listed once, used where their role allows |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

Crates of other projects come in by git, following a branch; Cargo.lock fixes the commit and a nightly update finds breaks early. A `rev` or `tag` pin hides a break; a different URL brings a second copy; a crate used from a layer that may not see its role leaks a dependency upward. Every git dependency is listed in `[map.external]`, so the set of outside projects is visible in one place.

## How to fix

Take it by git with the URL and branch the map lists, no `rev` or `tag`, from a crate whose layer may see the crate's `role`. A new outside crate gets an entry first.

## Configure

```toml
[lints.external-form]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(external-form) <reason>` ([waivers](../waivers.md)).
