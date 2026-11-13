# siz-4

**Rule:** The release profile sets codegen-units = 1, lto, panic = "abort" and a measured opt-level.

| | |
|---|---|
| Category | [siz](index.md#siz) |
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
| [embedded](../tiers/embedded.md) | `warn` | bedrock suggestion, reported without failing the run: Release profile without `codegen-units = 1`, `lto`, `panic = "abort"`, and an `opt-level` chosen by measurement (z vs 3 is the largest lever: Donguru 501.7 → 321.8 KiB, 10-25 % slower USB) |

## Why

A release profile missing `codegen-units = 1`, `lto` (thin or fat), `panic = "abort"`, or an `opt-level` chosen without measuring it against the alternative (`z` vs `3` is the largest lever of the group).

These are the standard, well-understood size/speed trade-offs for embedded Rust; skipping them isn't wrong so much as leaving an easy, already-known-safe win on the table, and picking `opt-level` by habit instead of measurement can go either way depending on the actual code (size vs speed is not monotonic).

## Example

Typec_dongle FW-57 and the Donguru measurement wire_weaver's AGENTS.md cites: `opt-level = "z"` brought one firmware from 501.7 to 321.8 KiB, at a real cost of 10-25 % more USB latency — a trade worth making deliberately, not by default.

## How to fix

Set `codegen-units = 1`, `lto = "fat"` (or `"thin"` if build time matters more), `panic = "abort"`; measure `opt-level = "z"` against `"3"` on the actual firmware before picking one.

## How it is checked

`cfg easy`: the four settings' presence in `Cargo.toml`'s release profile; whether `opt-level` was *measured* isn't itself automatable, hence the suggestion severity.

## Source

Ported from embedded_bedrock's firmware check catalogue, `SIZ-4` (`book/checks.md`, detail in `book/checks/siz.md`). Found on: typec_dongle FW-57; wire_weaver AGENTS.md.

## Configure

```toml
[lints.siz-4]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(siz-4) <reason>` ([waivers](../waivers.md)).
