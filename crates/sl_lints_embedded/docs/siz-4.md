## Why

A release profile missing `codegen-units = 1`, `lto` (thin or fat), `panic = "abort"`, or an `opt-level` chosen without measuring it against the alternative (`z` vs `3` is the largest lever of the group).

These are the standard, well-understood size/speed trade-offs for embedded Rust; skipping them isn't wrong so much as leaving an easy, already-known-safe win on the table, and picking `opt-level` by habit instead of measurement can go either way depending on the actual code (size vs speed is not monotonic).

## Example

Typec_dongle FW-57 and the Portwright measurement wire_weaver's AGENTS.md cites: `opt-level = "z"` brought one firmware from 501.7 to 321.8 KiB, at a real cost of 10-25 % more USB latency — a trade worth making deliberately, not by default.

## How to fix

Set `codegen-units = 1`, `lto = "fat"` (or `"thin"` if build time matters more), `panic = "abort"`; measure `opt-level = "z"` against `"3"` on the actual firmware before picking one.

## How it is checked

`cfg easy`: the four settings' presence in `Cargo.toml`'s release profile; whether `opt-level` was *measured* isn't itself automatable, hence the suggestion severity.

## Source

Ported from embedded_bedrock's firmware check catalogue, `SIZ-4` (`book/checks.md`, detail in `book/checks/siz.md`). Found on: portwright FW-57; wire_weaver AGENTS.md.
