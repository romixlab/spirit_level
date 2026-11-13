# crate-lines

**Rule:** A crate's src/ stays under its line limit.

| | |
|---|---|
| Category | [size](index.md#size) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step |
| [big](../tiers/big.md) | `warn` | split a crate before it passes ~20,000 lines: one crate is one serial compile step |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `max-lines` | integer | `20000` | code lines in src/ above which a crate is reported |

## Why

rustc's front end runs one crate on one thread, so a big crate is one long serial step of every build, and every change in it rebuilds all of it. Past about 20,000 lines a crate also stops having one subject: tpm_cli's root crate reached 316,000 lines with 85% of them in one tangle. Split before it grows on.

Code lines are lines that are not blank and not only a comment, in `src/` (tests apart, see [test-lines](test-lines.md)). Crates the [crate map](../map.md) marks `generated` are not counted.

## Exceptions with expiry

A crate that must stay big for a while gets a limit of its own, a reason and an expiry, so every exception is visible and temporary:

```toml
[[exceptions]]
lint = "crate-lines"
crates = ["foo_editor"]
max-lines = 45000
level = "deny"
reason = "split into foo_editor_core and foo_editor_ui is FEATURES SPLIT-3"
expires = "2027-03-31"
```

`spirit-level config` lists every exception with its reason and expiry.

## How to fix

Split along the seams: types into a `_core` crate, a heavy dependency into a crate of its own, parts that change at their own pace apart.

## Configure

```toml
[lints.crate-lines]
level = "deny"
max-lines = 20000
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(crate-lines) <reason>` ([waivers](../waivers.md)).
