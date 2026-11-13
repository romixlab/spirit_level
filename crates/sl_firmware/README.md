# sl_firmware

The model of bare-metal firmware crates that the embedded lints (`sl_lints_embedded`) read, ported from embedded_bedrock's bedrock_lint.

`Firmware::load_all(&project, &elfs)` finds the firmware crates of a project (a binary whose root is `#![no_std]`, or a `memory.x`, a `bedrock_fw.json` or a `-none-` target in `.cargo/config.toml`; `[package.metadata.spirit-level] firmware = true` or `false` in a crate's manifest decides it by hand) and reads each one: its `.cargo/config.toml` files, `build.rs` and its `cargo:` directives, `memory.x` and its `MEMORY` regions, `bedrock_fw.json`, the chip, facts from its sources (tasks, executors, interrupt handlers, statics, priorities, a call graph by name) and, when given, the linked ELF (sections, segments, symbols, Thumb frames and calls).

The `synth` feature writes small ELFs from an `elf.toml`, for fixtures.
