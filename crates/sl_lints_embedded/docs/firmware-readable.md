## Why

The embedded lints read a firmware crate's `memory.x`, `bedrock_fw.json`, `.cargo/config.toml`, `build.rs`, its Rust sources and the ELF given with `--elf`. A file they cannot read (a syntax error, JSON that does not parse, a `memory.x` without a `MEMORY` block, a file that is not a 32-bit ELF) gives them nothing to look at: every lint that needs it finds nothing in that crate, and the run would look clean.

## Example

A `memory.x` whose `MEMORY` block lost its closing brace: `siz-1`, `stk-3` and `mem-1` have no regions to measure and say nothing; this lint names the file instead.

## How to fix

Fix the file the finding names, or pass the right ELF with `--elf target/<triple>/release/<name>`.

## Source

bedrock_lint printed these as loader notes beside its report; spirit_level reports them as findings so a deny level can fail the run.
