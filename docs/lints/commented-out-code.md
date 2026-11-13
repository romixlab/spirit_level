# commented-out-code

**Rule:** No code is left commented out.

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
| [ai](../tiers/ai.md) | `deny` | git keeps the old code; commented-out code is a decision not taken |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `min-lines` | integer | `1` | lines a run of comments needs to be judged |

## Why

Commented-out code is a decision not taken: it rots, confuses search and review, and git already keeps the old version. Agents leave it when they rewrite a function and keep the old one "just in case".

A run of consecutive `//` comments (not doc comments, not after code on the line), or a block comment, is code when both hold:

- its text holds a mark of code: `;`, `{`, `}`, `=`, or a call (`(` with the text ending in `)`, `;` or `,`);
- it parses as Rust statements or items (syn).

Prose rarely parses, and the parse is what keeps "returns x (the default)" from being reported.

```rust
// let old = parse_v1(input)?;     <- reported
// u16: the frame length field is two bytes
```

## How to fix

Delete it. A variant that is still needed becomes a feature, a test or a branch.

## Configure

```toml
[lints.commented-out-code]
level = "deny"
min-lines = 1
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(commented-out-code) <reason>` ([waivers](../waivers.md)).
