# stk-2

**Rule:** Build.rs prints only directives Cargo knows.

| | |
|---|---|
| Category | [stk](index.md#stk) |
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
| [embedded](../tiers/embedded.md) | `deny` | bedrock error: `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs: not a build-script directive, Cargo ignores it silently |

## Why

A build script that prints `cargo:rustc-linker=...` (or any other `cargo:` key Cargo doesn't recognise) expecting it to take effect.

Cargo silently ignores a build-script directive it doesn't recognise — there is no warning, no error, nothing in the build log. A build.rs that means to set the linker this way builds and links fine, just with whatever linker `.cargo/config.toml` actually specifies, which may not be flip-link (`stk-1`).

## Example

Embedded_bedrock ROB-12: a build.rs attempt to set the linker via a `cargo:` print did nothing, and flip-link's absence was only found later by its real symptom.

## How to fix

Set the linker in `.cargo/config.toml` (`stk-1`), not from build.rs; use build.rs only for directives Cargo documents.

## How it is checked

`cfg easy`: scan `build.rs` for `cargo:` prints against the set of keys Cargo actually recognises, and flag anything outside it.

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-2` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: embedded_bedrock ROB-12.

## Configure

```toml
[lints.stk-2]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(stk-2) <reason>` ([waivers](../waivers.md)).
