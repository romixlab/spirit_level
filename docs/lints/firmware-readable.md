# firmware-readable

**Rule:** Every file of a firmware crate that the embedded lints read can be read.

| | |
|---|---|
| Category | [embedded](index.md#embedded) |
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
| [embedded](../tiers/embedded.md) | `deny` | a firmware file the embedded lints cannot read hides every finding in it |

## Why

The embedded lints read a firmware crate's `memory.x`, `bedrock_fw.json`, `.cargo/config.toml`, `build.rs`, its Rust sources and the ELF given with `--elf`. A file they cannot read (a syntax error, JSON that does not parse, a `memory.x` without a `MEMORY` block, a file that is not a 32-bit ELF) gives them nothing to look at: every lint that needs it finds nothing in that crate, and the run would look clean.

## Example

A `memory.x` whose `MEMORY` block lost its closing brace: `siz-1`, `stk-3` and `mem-1` have no regions to measure and say nothing; this lint names the file instead.

## How to fix

Fix the file the finding names, or pass the right ELF with `--elf target/<triple>/release/<name>`.

## Source

bedrock_lint printed these as loader notes beside its report; spirit_level reports them as findings so a deny level can fail the run.

## Configure

```toml
[lints.firmware-readable]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(firmware-readable) <reason>` ([waivers](../waivers.md)).
