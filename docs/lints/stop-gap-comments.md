# stop-gap-comments

**Rule:** Comments carry no stop-gap markers: TODO, FIXME, HACK, for now, workaround.

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
| [big](../tiers/big.md) | `deny` | no stop-gaps: a TODO or a 'for now' names its tracker id or goes |
| [ai](../tiers/ai.md) | `deny` | a TODO or a 'for now' marks a path the final design throws away; tracked work names its id |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `markers` | list of strings | `["TODO", "FIXME", "XXX", "HACK"]` | markers matched as whole words, case-sensitive |
| `phrases` | list of strings | `["for now", "hack", "workaround", "shim", "quick fix", "temporary fix", "temporary solution", "temporary workaround", "temporary hack", "temporary code", "temporary implementation", "temporary patch", "is temporary", "temporarily disabled", "fix later", "refactor later", "clean up later"]` | phrases matched as whole words, case-insensitive |
| `tracked-ok` | boolean | `true` | a comment with a tracker id (`CFG-3`), an issue number (`#12`) or a URL is tracked work and passes |
| `docs` | boolean | `true` | also check doc comments |

## Why

The top rule of agent-written code: no stop-gaps. A `TODO`, a `HACK` or "for now" marks a second path that the final design throws away, and agents leave them by the dozen. A marker that names planned work (`TODO(CFG-3)`, an issue link) is tracked and passes.

The rule checks every comment of the workspace members, doc comments included (`docs`):

- `markers` match as whole words, case-sensitive (`TODO`, not "a todo list");
- `phrases` match as whole words, case-insensitive.

"temporary" alone is no stop-gap: it describes the domain as often as the code ("a temporary buffer for the 2 ms burst"). The default list only names phrases about the code itself: "temporary fix", "is temporary", "temporarily disabled".

```rust
// TODO: handle errors properly           <- reported
// TODO(CFG-3): include keys              <- tracked, passes
// hold the line low for a temporary 2 ms reset pulse   <- passes
```

## How to fix

Build the final version now. Or name the planned work: a FEATURES.md id or an issue link in the comment.

## Configure

```toml
[lints.stop-gap-comments]
level = "deny"
markers = ["TODO", "FIXME", "XXX", "HACK"]
phrases = ["for now", "hack", "workaround", "shim", "quick fix", "temporary fix", "temporary solution", "temporary workaround", "temporary hack", "temporary code", "temporary implementation", "temporary patch", "is temporary", "temporarily disabled", "fix later", "refactor later", "clean up later"]
tracked-ok = true
docs = true
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(stop-gap-comments) <reason>` ([waivers](../waivers.md)).
