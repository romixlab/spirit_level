# rustfmt-unformatted

**Rule:** Every Rust file is formatted the way cargo fmt writes it.

| | |
|---|---|
| Category | [rustfmt](index.md#rustfmt) |
| Source | rustfmt (run as an executable) |
| Fix | by hand |
| Needs | `rustfmt` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Why

One formatting for the whole workspace keeps diffs about the change, not about the layout. rustfmt formats by the project's `rustfmt.toml` (or its defaults); this lint reports files it would change.

## How to fix

Run `cargo fmt --all`. Code that must keep its layout (a table of numbers) gets `#[rustfmt::skip]`.

## Configure

```toml
[lints.rustfmt-unformatted]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(rustfmt-unformatted) <reason>` ([waivers](../waivers.md)).
