# Firmware

The `embedded` tier checks bare-metal firmware: its build configuration, its sources and, when you pass it, the linked ELF. This page explains which crates of a workspace count as firmware and which lints run on each of them.

## Firmware crates

A firmware crate is a workspace member that has one of these:

- a binary whose root is `#![no_std]`;
- a `memory.x`;
- a `bedrock_fw.json`;
- a `-none-` target in its `.cargo/config.toml`, as `[build] target` or a `[target.<triple>]` section.

When the guess is wrong, set it by hand in the crate's `Cargo.toml`:

```toml
[package.metadata.spirit-level]
firmware = true
```

A firmware binary is a firmware crate that builds a binary: the crates an ELF comes from. A host workspace with `no_std` libraries has firmware crates but no firmware binary.

## The linked ELF

The lints that read the ELF need it on the command line. Pass `--elf` once per firmware binary:

```console
$ cargo build --release
$ spl check --elf target/thumbv7em-none-eabihf/release/board
```

An ELF goes with the crate that has a binary of its file name, or with the only firmware crate. Without `--elf`, these lints report "not run" and the run exits with code 3. A workspace without a firmware binary has them [skipped](output.md#skipped-lints): the run says so and passes.

## What each lint needs

Apart from the ELF, each lint needs something of the crate it checks: a `memory.x`, a build script, a known chip, or Thumb code in the ELF. A crate without it gives no findings for that lint. The chip comes from `bedrock_fw.json`, or else from the HAL's chip feature (`embassy-stm32`, `embassy-rp`, `embassy-nrf`).

Findings name places relative to the workspace root (`fw/src/main.rs`), not to the firmware crate.

## Waivers and exceptions

A [waiver](waivers.md) works in Rust sources, `build.rs` included, and in `Cargo.toml`. A finding in `.cargo/config.toml`, `memory.x` or `bedrock_fw.json` has no place for a comment, so allow it with an exception by path in `spirit_level.toml` ([configuration](configuration.md)).
