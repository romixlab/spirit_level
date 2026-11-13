# crate-cycles

**Rule:** Workspace crates form no cycle, dev-dependencies included.

| | |
|---|---|
| Category | [graph](index.md#graph) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | crates in a cycle cannot be built or tested apart |
| [medium](../tiers/medium.md) | `deny` | crates in a cycle cannot be built or tested apart |
| [big](../tiers/big.md) | `deny` | crates in a cycle cannot be built or tested apart |

## Why

Cargo refuses a cycle of normal dependencies, but allows one through dev-dependencies: `a` depends on `b`, and `b`'s tests depend on `a`. Then neither can be built, tested or moved apart, and `-p` test runs rebuild both. The lint reports every strongly connected component of the member graph, dev edges included.

## How to fix

Move the shared test helpers into a test-only crate both use as a dev-dependency.

## Configure

```toml
[lints.crate-cycles]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(crate-cycles) <reason>` ([waivers](../waivers.md)).
