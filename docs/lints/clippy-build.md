# clippy-build

**Rule:** The workspace builds under clippy without compiler errors or warnings.

| | |
|---|---|
| Category | [clippy](index.md#clippy) |
| Source | clippy (run as an executable) |
| Fix | by hand |
| Needs | `clippy` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `args` | list of strings | `["--workspace", "--all-targets"]` | arguments to `cargo clippy` (packages, targets, features); every clippy lint reads them |

## Why

Clippy compiles the workspace; a compiler error stops it before it reaches most lints, and a compiler warning left standing hides the next one. This lint carries the compiler's own errors and warnings seen during the clippy run, and its `args` say how clippy is run for every `clippy-*` lint.

```toml
[lints.clippy-build]
level = "deny"
reason = "warnings are fixed, not collected"
args = ["--workspace", "--all-targets", "--all-features"]
```

## How to fix

Fix the code the compiler points at. A warning that must stay gets `#[expect(lint, reason = "...")]` at the place, so it is written down where it is.

## Configure

```toml
[lints.clippy-build]
level = "deny"
args = ["--workspace", "--all-targets"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(clippy-build) <reason>` ([waivers](../waivers.md)).
