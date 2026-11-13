## Why

A `[profile.release.package.<name>]` override of `opt-level` in a workspace that also sets `lto = "fat"` on the top-level profile.

Fat LTO re-optimises the whole program as one unit after per-crate compilation, discarding the per-package `opt-level` the override asked for; the override costs nothing but also does nothing, which is the quiet failure mode here — not a bug, just wasted Cargo.toml and a false sense of control over size.

## Example

Typec_dongle FW-57: a per-package override that looked deliberate turned out to have no effect once fat LTO was already on.

## How to fix

Drop the override (fat LTO already re-optimises everything), or switch to `lto = "thin"`/no LTO if the per-package control is actually needed.

## How it is checked

`cfg easy`: a per-package `opt-level` override present alongside `lto = "fat"` in the same profile.

## Source

Ported from embedded_bedrock's firmware check catalogue, `SIZ-3` (`book/checks.md`, detail in `book/checks/siz.md`). Found on: typec_dongle FW-57.
