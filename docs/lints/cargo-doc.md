# cargo-doc

**Rule:** Cargo doc builds the workspace's documentation without a warning.

| | |
|---|---|
| Category | [build](index.md#build) |
| Source | cargo (run as an executable) |
| Fix | by hand |
| Needs | `cargo` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | broken intra-doc links and tags render wrong on docs.rs and in the IDE |
| [small-workspace](../tiers/small-workspace.md) | `warn` | broken intra-doc links and tags render wrong on docs.rs and in the IDE |
| [medium](../tiers/medium.md) | `deny` | the docs of internal crates are the map of the code; they build without a warning |
| [big](../tiers/big.md) | `deny` | the docs of internal crates are the map of the code; they build without a warning |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `args` | list of strings | `["--workspace", "--no-deps"]` | arguments to `cargo doc` |

## Why

A broken intra-doc link, an unclosed HTML tag or a bare URL in a doc comment renders wrong on docs.rs and in the IDE. `cargo doc` warns about them; this lint reports every warning, as `RUSTDOCFLAGS="-D warnings"` in CI would.

## How to fix

Fix the doc comment the warning points at: link with the path rustdoc can resolve (`` [`crate::Foo`] ``), escape brackets that are not links.

## Configure

```toml
[lints.cargo-doc]
level = "deny"
args = ["--workspace", "--no-deps"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(cargo-doc) <reason>` ([waivers](../waivers.md)).
