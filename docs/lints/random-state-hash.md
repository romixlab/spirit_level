# random-state-hash

**Rule:** Deterministic crates use no randomly seeded hash maps.

| | |
|---|---|
| Category | [code](index.md#code) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | crates the map marks deterministic iterate in the same order every run |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `also` | list of strings | `[]` | crates held to it besides those the map marks deterministic |

## Why

`HashMap` and `HashSet` with the default hasher are seeded randomly per process, so iteration order changes from run to run. In a crate that promises deterministic output, any loop over such a map that reaches an output (a file, a log, a hash) breaks it.

The lint applies to crates the map marks `deterministic = true` and to those listed in `also`, and reports `HashMap<K, V>` / `HashSet<T>` written without an explicit hasher type.

## How to fix

Use `BTreeMap` / `BTreeSet`, or a map with a fixed-seed hasher (`HashMap<K, V, FixedState>`), where the order can reach an output.

## Configure

```toml
[lints.random-state-hash]
level = "deny"
also = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(random-state-hash) <reason>` ([waivers](../waivers.md)).
