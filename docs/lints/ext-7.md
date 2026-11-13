# ext-7

**Rule:** Register fields are range-checked before they are encoded.

| | |
|---|---|
| Category | [ext](index.md#ext) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | off |  |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | `warn` | bedrock warning: Register field encoded without a range check (`x - 1` underflows, masked to 0) |

## Why

A register field that encodes a user value as `value - 1` (or similar) underflows silently to a huge number, or truncates and masks to 0, when the caller passes an out-of-range value (0, or something bigger than the field's width). Nothing crashes; the chip is configured with a value nobody intended.

## Example

`bedrock_can`'s bit-timing types encode this correctly as the model: `mcan`'s `NominalBitTiming::new` / `DataBitTiming::new` are `const fn` that validate the RM0468/RM0444 ranges before ever touching a register — nominal prescaler 1..=512, seg1 2..=256, seg2 1..=128, SJW 1..=128 (data: 1..=32, 1..=32, 1..=16, 1..=16), plus SJW <= seg2 — and return a `BitTimingError` instead of silently writing a bad `value - 1` into NBTP/DBTP; the register encoding itself is a separately host-tested pure function (mcan T1).

## How to fix

Validate the caller's value against the field's real range before the `- 1` (or other) encoding, and fail with a typed error rather than letting the subtraction wrap.

## How it is checked

Src mid — a lint can flag an unchecked arithmetic encode feeding a register write, though proving the range is actually enforced needs the call graph back to the public API.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXT-7` (`book/checks.md`, detail in `book/checks/ext.md`). Found on: mcan T1.

## Configure

```toml
[lints.ext-7]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(ext-7) <reason>` ([waivers](../waivers.md)).
