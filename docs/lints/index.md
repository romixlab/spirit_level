# Lints

Every lint, grouped by category, with its level in each shipped tier (`off`: the tier does not enable it). A project changes any of them in its `spirit_level.toml` ([configuration](../configuration.md)).

## agents

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [agent-file-lines](agent-file-lines.md) | an agent file stays under its line limit | warn | warn | warn | warn | deny | off |
| [agent-file-links](agent-file-links.md) | links and @imports in agent files resolve | off | off | off | off | deny | off |
| [agent-rules-match-lints](agent-rules-match-lints.md) | limits an agent file states are the limits the lints check | off | off | off | warn | deny | off |
| [claude-md-duplicate](claude-md-duplicate.md) | CLAUDE.md points to AGENTS.md instead of copying it | off | off | off | off | deny | off |

## api

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [pub-surface](pub-surface.md) | a library's public surface stays small for its size | off | off | off | warn | off | off |

## audit

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [audit-unmaintained](audit-unmaintained.md) | no dependency is marked unmaintained or replaced in the RustSec database | off | off | off | off | off | off |
| [audit-unsound](audit-unsound.md) | no dependency is marked unsound in the RustSec database | off | off | off | off | off | off |
| [audit-vulnerabilities](audit-vulnerabilities.md) | no dependency has a published security vulnerability | off | off | off | off | off | off |
| [audit-yanked](audit-yanked.md) | Cargo.lock holds no yanked version | off | off | off | off | off | off |

## boot

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [boot-1](boot-1.md) | a firmware entered by a jump detects it and resets once | off | off | off | off | off | deny |
| [boot-10](boot-10.md) | backup-domain helpers restore what they found | off | off | off | off | off | warn |
| [boot-2](boot-2.md) | the H7 supply configuration is read back, never written blind | off | off | off | off | off | deny |

## build

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [cargo-doc](cargo-doc.md) | cargo doc builds the workspace's documentation without a warning | warn | warn | deny | deny | off | off |
| [cargo-msrv](cargo-msrv.md) | every crate builds with the toolchain of its declared rust-version | off | off | warn | deny | off | off |
| [cargo-no-std](cargo-no-std.md) | every crate the map marks no_std builds for targets without std | off | off | off | deny | off | off |
| [cargo-wasm](cargo-wasm.md) | every crate the map marks wasm builds for wasm32-unknown-unknown | off | off | off | deny | off | off |

## cfg

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [cfg-1](cfg-1.md) | every probe-rs runner and recipe names its probe with --probe VID:PID | off | off | off | off | off | deny |
| [cfg-2](cfg-2.md) | F0/G0/L0 firmware is flashed by a runner that halts the core first | off | off | off | off | off | warn |
| [cfg-4](cfg-4.md) | every crate is in the tree from one source | off | off | off | off | off | deny |
| [cfg-5](cfg-5.md) | the HAL's chip feature names the part on the board | off | off | off | off | off | deny |
| [cfg-6](cfg-6.md) | thumbv6m firmware that needs compare-and-swap uses portable-atomic with critical-section | off | off | off | off | off | warn |
| [cfg-7](cfg-7.md) | the build info carries the git SHA and a dirty flag | off | off | off | off | off | warn |

## clippy

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [clippy-build](clippy-build.md) | the workspace builds under clippy without compiler errors or warnings | off | off | off | off | off | off |
| [clippy-cargo](clippy-cargo.md) | no finding of the cargo-group clippy lints the project turned on | off | off | off | off | off | off |
| [clippy-complexity](clippy-complexity.md) | no finding of clippy's complexity group: simple things done simply | off | off | off | off | off | off |
| [clippy-correctness](clippy-correctness.md) | no finding of clippy's correctness group: code that is outright wrong | off | off | off | off | off | off |
| [clippy-nursery](clippy-nursery.md) | no finding of the nursery clippy lints the project turned on | off | off | off | off | off | off |
| [clippy-pedantic](clippy-pedantic.md) | no finding of the pedantic clippy lints the project turned on | off | off | off | off | off | off |
| [clippy-perf](clippy-perf.md) | no finding of clippy's perf group: no needlessly slow code | off | off | off | off | off | off |
| [clippy-restriction](clippy-restriction.md) | no finding of the restriction clippy lints the project turned on | off | off | off | off | off | off |
| [clippy-style](clippy-style.md) | no finding of clippy's style group: code written the idiomatic way | off | off | off | off | off | off |
| [clippy-suspicious](clippy-suspicious.md) | no finding of clippy's suspicious group: code that is most likely wrong | off | off | off | off | off | off |

## clk

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [clk-2](clk-2.md) | the embassy time driver names a timer that nothing else uses | off | off | off | off | off | deny |
| [clk-3](clk-3.md) | the clock security system watches HSE and LSE, with a handler | off | off | off | off | off | warn |
| [clk-5](clk-5.md) | USB has an exact 48 MHz clock | off | off | off | off | off | deny |

## cnt

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [cnt-1](cnt-1.md) | every error branch counts what happened with cnt! | off | off | off | off | off | warn |
| [cnt-2](cnt-2.md) | counter buffers an interrupt updates are never borrowed mutably or read unaligned | off | off | off | off | off | warn |
| [cnt-3](cnt-3.md) | a proc-macro crate and its runtime crate are pinned to the same exact version | off | off | off | off | off | warn |

## code

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [allow-without-reason](allow-without-reason.md) | every allow or expect of a lint says why | warn | warn | warn | deny | deny | off |
| [cli-subprocess](cli-subprocess.md) | workspace crates call each other as libraries, never by running the workspace's own binaries | off | off | deny | deny | off | off |
| [float-determinism](float-determinism.md) | deterministic crates compute float transcendentals through libm | off | off | off | deny | off | off |
| [panic-in-library](panic-in-library.md) | library code returns errors instead of panicking | off | off | warn | warn | off | off |
| [print-in-library](print-in-library.md) | library code does not print | warn | warn | warn | warn | off | off |
| [random-state-hash](random-state-hash.md) | deterministic crates use no randomly seeded hash maps | off | off | off | deny | off | off |
| [todo-macros](todo-macros.md) | no todo!() or unimplemented!() outside tests | warn | warn | warn | deny | deny | off |

## commits

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [commit-body-lines](commit-body-lines.md) | a commit body stays short | off | off | off | off | warn | off |
| [commit-prose](commit-prose.md) | commit messages use no filler, narration or emoji and do not describe the diff | off | off | off | off | warn | off |
| [commit-subject-length](commit-subject-length.md) | a commit subject stays under its length limit | off | off | warn | warn | deny | off |

## dead-code

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [dead-pub](dead-pub.md) | public items of workspace libraries are used outside their crate | off | off | off | warn | off | off |

## deny

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [deny-advisories](deny-advisories.md) | no dependency has a security advisory, is unmaintained, unsound or yanked | warn | warn | deny | deny | off | off |
| [deny-bans](deny-bans.md) | the dependency graph holds no banned crate and no duplicate deny.toml refuses | off | off | off | deny | off | off |
| [deny-licenses](deny-licenses.md) | every crate's licence is one deny.toml allows | off | off | warn | deny | off | off |
| [deny-sources](deny-sources.md) | every crate comes from a registry or git source deny.toml allows | off | off | off | deny | off | off |

## dependencies

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [default-features-off](default-features-off.md) | third-party dependencies are declared with default-features = false | warn | warn | warn | deny | off | off |
| [duplicate-versions](duplicate-versions.md) | no crate is in the dependency graph in two versions | off | warn | warn | deny | off | off |
| [git-dependency-pins](git-dependency-pins.md) | git dependencies follow a branch, with no rev or tag pin | warn | warn | deny | deny | off | off |
| [no-sys-crates](no-sys-crates.md) | the dependency graph has no -sys crates and no C build steps | deny | deny | deny | deny | off | off |
| [path-outside-workspace](path-outside-workspace.md) | path dependencies stay inside the workspace | deny | deny | deny | deny | off | off |

## dma

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [dma-1](dma-1.md) | DMA buffers sit in memory the DMA master can reach | off | off | off | off | off | deny |
| [dma-2](dma-2.md) | DMA buffers in cacheable RAM are cleaned and invalidated, or not cacheable | off | off | off | off | off | deny |
| [dma-4](dma-4.md) | every DMA ring consumer counts its overruns | off | off | off | off | off | warn |
| [dma-6](dma-6.md) | where an erratum asks for DMA, the code uses DMA instead of polling | off | off | off | off | off | warn |

## embedded

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [firmware-readable](firmware-readable.md) | every file of a firmware crate that the embedded lints read can be read | off | off | off | off | off | deny |

## err

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [err-11](err-11.md) | the silicon revision is read from a source that tells every revision apart | off | off | off | off | off | warn |
| [err-2](err-2.md) | every register poke for an erratum quotes the erratum next to it | off | off | off | off | off | deny |
| [err-3](err-3.md) | flash prefetch is off where code crosses into bank 2 on G0B0/G0B1/G0C1 | off | off | off | off | off | deny |
| [err-4](err-4.md) | FDCAN edge filtering and dedicated Tx buffers stay off | off | off | off | off | off | deny |
| [err-5](err-5.md) | FDCAN one-shot mode retransmits after a lost arbitration | off | off | off | off | off | warn |
| [err-6](err-6.md) | RP2350 A2 pads with a pull-down enable their input only for the read | off | off | off | off | off | warn |
| [err-7](err-7.md) | RP2350 ROM reboots use a delay and only the allowed FRCE_OFF bits | off | off | off | off | off | warn |
| [err-8](err-8.md) | G0 ADC calibration waits two ADC clocks after the disable | off | off | off | off | off | warn |

## errors

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [error-enum-growth](error-enum-growth.md) | a big error enum wraps the errors it comes from instead of listing every failure flat | off | off | warn | warn | off | off |

## exe

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [exe-1](exe-1.md) | a task with a hard answer deadline runs on its own executor, not beside bulk work on the thread executor | off | off | off | off | off | deny |
| [exe-10](exe-10.md) | one crate provides the executor and its pender | off | off | off | off | off | deny |
| [exe-12](exe-12.md) | every async fn awaits something | off | off | off | off | off | warn |
| [exe-13](exe-13.md) | a ring-buffered reader in a loop batches its reads | off | off | off | off | off | warn |
| [exe-2](exe-2.md) | tasks on an interrupt executor and interrupt handlers never block | off | off | off | off | off | deny |
| [exe-3](exe-3.md) | an interrupt executor's priority is set before it is started | off | off | off | off | off | deny |
| [exe-4](exe-4.md) | an interrupt executor runs below the interrupts that wake its tasks | off | off | off | off | off | warn |
| [exe-5](exe-5.md) | an interrupt executor uses a vector no driver binds and no used peripheral raises | off | off | off | off | off | deny |
| [exe-6](exe-6.md) | state shared between an interrupt executor and thread mode sits behind a critical-section mutex | off | off | off | off | off | deny |
| [exe-7](exe-7.md) | a loop that drains a queue in a shared task is bounded and yields | off | off | off | off | off | warn |
| [exe-8](exe-8.md) | a request handler answers without waiting on a bus or a slow peripheral | off | off | off | off | off | warn |
| [exe-9](exe-9.md) | no future that reaches embassy-time is polled with a no-op waker | off | off | off | off | off | deny |

## ext

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [ext-12](ext-12.md) | an RCC enable is read back before the peripheral's first access | off | off | off | off | off | warn |
| [ext-4](ext-4.md) | external chip registers are written from shadow copies, not read-modify-write | off | off | off | off | off | warn |
| [ext-7](ext-7.md) | register fields are range-checked before they are encoded | off | off | off | off | off | warn |

## fls

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [fls-1](fls-1.md) | flash a power cut can tear is read through ECC-aware access on parts with flash ECC | off | off | off | off | off | deny |
| [fls-2](fls-2.md) | the flash peripheral has one owner | off | off | off | off | off | deny |
| [fls-3](fls-3.md) | RP flash is erased and written from core 0 with nothing else reading flash | off | off | off | off | off | deny |
| [fls-4](fls-4.md) | the image in flash is checked by a periodic CRC, and ECC is on where the part has it | off | off | off | off | off | warn |

## flt

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [flt-1](flt-1.md) | the HardFault handler records the crash and resets instead of spinning | off | off | off | off | off | deny |
| [flt-2](flt-2.md) | HardFault and DefaultHandler do nothing that can panic or log | off | off | off | off | off | deny |
| [flt-3](flt-3.md) | bedrock_crash::exc_return!() is the first statement of HardFault | off | off | off | off | off | warn |
| [flt-4](flt-4.md) | UsageFault, MemManage and BusFault have handlers and are enabled where the core has them | off | off | off | off | off | warn |
| [flt-5](flt-5.md) | the reset cause is read and reported at boot | off | off | off | off | off | warn |
| [flt-7](flt-7.md) | a watchdog runs, fed from a bounded loop of the main work | off | off | off | off | off | deny |
| [flt-8](flt-8.md) | a boot-loop rescue falls back to the ROM bootloader after repeated unhealthy boots | off | off | off | off | off | warn |
| [flt-9](flt-9.md) | the healthy mark is set on the transport in use | off | off | off | off | off | warn |

## geiger

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [geiger-unsafe-deps](geiger-unsafe-deps.md) | every dependency that uses unsafe code is one the project accepted | off | off | off | off | off | off |

## graph

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [core-crate-deps](core-crate-deps.md) | a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates | off | off | warn | deny | off | off |
| [core-crate-types](core-crate-types.md) | a _core crate holds types: no I/O, no threads, no async runtime | off | off | off | deny | off | off |
| [crate-cycles](crate-cycles.md) | workspace crates form no cycle, dev-dependencies included | off | warn | deny | deny | off | off |
| [crate-table](crate-table.md) | the crate table in the README is the one generated from the manifests and the map | off | off | off | deny | off | off |
| [external-form](external-form.md) | crates of other projects come by git as the map lists them, used only where their role allows | off | off | off | deny | off | off |
| [forbidden-closure](forbidden-closure.md) | a crate never reaches the crates the map forbids it, directly or through others | off | off | off | deny | off | off |
| [layer-edges](layer-edges.md) | every dependency edge between workspace crates is allowed by the map | off | off | off | deny | off | off |
| [map-complete](map-complete.md) | the crate map and the workspace list the same crates | off | off | off | deny | off | off |
| [third-party-placement](third-party-placement.md) | heavy third-party crates are used only by the crates the map names | off | off | off | deny | off | off |
| [unsafe-allow-list](unsafe-allow-list.md) | unsafe code is forbidden except in the crates the map marks unsafe | off | off | off | deny | off | off |

## hakari

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [hakari-generate](hakari-generate.md) | the workspace-hack package's dependencies are the ones hakari computes | off | off | off | warn | off | off |
| [hakari-manage-deps](hakari-manage-deps.md) | every member hakari manages depends on the workspace-hack package, and no other | off | off | off | warn | off | off |

## host

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [host-1](host-1.md) | nothing reachable from host input can panic | off | off | off | off | off | deny |
| [host-3](host-3.md) | unknown enum values from the host are refused, not mapped to a default | off | off | off | off | off | warn |
| [host-5](host-5.md) | truncated and oversized requests are counted and dropped | off | off | off | off | off | warn |

## hygiene

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [agent-config-permissions](agent-config-permissions.md) | committed agent settings grant no blanket permissions | off | off | off | off | deny | off |
| [agent-config-secrets](agent-config-secrets.md) | committed agent configuration holds no secrets | off | off | off | off | deny | off |
| [agent-session-files](agent-session-files.md) | no agent transcripts or per-person agent files are committed | off | off | off | off | deny | off |
| [leftover-files](leftover-files.md) | no merge, patch, editor or scratch leftovers are committed | off | off | off | off | deny | off |

## irq

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [irq-1](irq-1.md) | interrupt handlers clear exactly the flags they handled | off | off | off | off | off | deny |
| [irq-2](irq-2.md) | write-1-to-clear flags are cleared with a write, not modify() | off | off | off | off | off | deny |
| [irq-4](irq-4.md) | no driver disables a shared interrupt vector that other drivers use | off | off | off | off | off | deny |
| [irq-5](irq-5.md) | every routed interrupt line is enabled and has a handler bound | off | off | off | off | off | warn |
| [irq-6](irq-6.md) | interrupts are disabled through critical sections, never nested by hand | off | off | off | off | off | warn |
| [irq-8](irq-8.md) | the default interrupt handler records which IRQ fired | off | off | off | off | off | warn |

## log

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [log-1](log-1.md) | RTT never blocks | off | off | off | off | off | deny |
| [log-2](log-2.md) | DEFMT_RTT_BUFFER_SIZE and DEFMT_LOG are set in .cargo/config.toml | off | off | off | off | off | warn |

## machete

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [machete-unused-deps](machete-unused-deps.md) | no crate declares a dependency its sources never use | warn | warn | warn | deny | off | off |

## manifest

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [crate-docs](crate-docs.md) | every crate root opens with crate-level documentation | warn | warn | warn | deny | off | off |
| [edition-latest](edition-latest.md) | every crate is on the current edition | warn | warn | warn | warn | off | off |
| [feature-docs](feature-docs.md) | every cargo feature has a comment saying what it turns on | off | off | off | warn | off | off |
| [package-description](package-description.md) | every crate has a description in its Cargo.toml | warn | warn | warn | deny | off | off |
| [package-license](package-license.md) | every crate states its licence | warn | warn | warn | warn | off | off |
| [publish-false](publish-false.md) | workspace crates are publish = false unless listed as published | off | off | warn | warn | off | off |
| [readme-per-crate](readme-per-crate.md) | every crate has a README | warn | deny | deny | deny | off | off |
| [rust-version-set](rust-version-set.md) | every crate declares its minimum Rust version | off | off | warn | deny | off | off |

## mem

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [mem-1](mem-1.md) | the FLASH region of memory.x leaves out the regions kept for other uses | off | off | off | off | off | deny |
| [mem-10](mem-10.md) | no LOAD segment is writable and executable, and .rodata holds nothing writable | off | off | off | off | off | warn |
| [mem-12](mem-12.md) | a firmware with defmt links with -Tdefmt.x | off | off | off | off | off | deny |
| [mem-2](mem-2.md) | extra RAM sections end without a tail ALIGN or symbol | off | off | off | off | off | warn |
| [mem-3](mem-3.md) | the RAM init and zeroing ranges stay clear of .uninit | off | off | off | off | off | deny |
| [mem-6](mem-6.md) | .cnt_bkp_buffer sits at the backup-register address | off | off | off | off | off | warn |
| [mem-7](mem-7.md) | embassy-boot partition symbols are offsets from the flash origin | off | off | off | off | off | warn |

## perf

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [perf-2](perf-2.md) | hot paths on cores without 64-bit maths avoid it, and float formatting | off | off | off | off | off | warn |
| [perf-3](perf-3.md) | RAM functions are #[inline(never)] | off | off | off | off | off | warn |

## pin

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [pin-1](pin-1.md) | pin drivers live as long as the pin is used | off | off | off | off | off | deny |
| [pin-16](pin-16.md) | pins held by a dead-battery pull-down are read only after it is released | off | off | off | off | off | warn |
| [pin-2](pin-2.md) | the board is put in its safe state first thing after the HAL init | off | off | off | off | off | deny |
| [pin-4](pin-4.md) | peripheral pins stay on their function while the peripheral is enabled | off | off | off | off | off | deny |
| [pin-6](pin-6.md) | every pin has one owner | off | off | off | off | off | deny |
| [pin-8](pin-8.md) | unused pins are analog or pulled, never floating inputs | off | off | off | off | off | warn |
| [pin-9](pin-9.md) | the pin map comes from the board at run time, not from a cargo feature | off | off | off | off | off | deny |

## prose

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [changelog-describes-diff](changelog-describes-diff.md) | a CHANGELOG entry says what changed for the user, not which code was edited | off | off | off | off | warn | off |
| [changelog-entry-length](changelog-entry-length.md) | a CHANGELOG entry is one short line | warn | warn | warn | warn | deny | off |
| [emoji](emoji.md) | docs and comments carry no emoji | off | off | off | off | warn | off |
| [filler-phrases](filler-phrases.md) | docs and comments use no filler phrases | off | off | off | off | deny | off |
| [narration](narration.md) | docs and comments state facts, not how they were written | off | off | off | off | deny | off |
| [paragraph-sentences](paragraph-sentences.md) | paragraphs in docs stay short | off | off | off | off | warn | off |
| [restating-comments](restating-comments.md) | a comment says what the code cannot | off | off | off | off | warn | off |
| [sentence-words](sentence-words.md) | sentences in docs stay short | off | off | off | off | warn | off |
| [thinking-comments](thinking-comments.md) | comments state conclusions, not a stream of thought | off | off | off | off | deny | off |

## pwr

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [pwr-1](pwr-1.md) | a brown-out reset level or voltage detector fits the board's supply | off | off | off | off | off | warn |
| [pwr-3](pwr-3.md) | the backup domain is reset at boot when nothing keeps state there | off | off | off | off | off | warn |
| [pwr-4](pwr-4.md) | a firmware that keeps state in the backup domain never changes the RTC clock source | off | off | off | off | off | warn |

## repo

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [changelog-unreleased](changelog-unreleased.md) | CHANGELOG.md has an Unreleased section | warn | warn | warn | warn | off | off |
| [justfile-recipes](justfile-recipes.md) | the justfile has the standard recipes | warn | warn | warn | warn | off | off |
| [justfile-simple](justfile-simple.md) | justfile recipes only call programs: a few lines, no inline scripts | off | off | off | warn | off | off |
| [profile-defaults](profile-defaults.md) | the dev profile splits debug info and a deploy profile builds incrementally | warn | warn | warn | warn | off | off |
| [repo-files](repo-files.md) | the repo carries its rule files | warn | warn | warn | warn | deny | off |
| [toolchain-pinned](toolchain-pinned.md) | the repo pins its toolchain in rust-toolchain.toml | off | off | warn | deny | off | off |

## rustfmt

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [rustfmt-unformatted](rustfmt-unformatted.md) | every Rust file is formatted the way cargo fmt writes it | off | off | off | off | off | off |

## semver

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [semver-checks-breaking](semver-checks-breaking.md) | a stable crate changes its public API only with the version bump the change needs | off | off | off | deny | off | off |

## shear

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [shear-misplaced-deps](shear-misplaced-deps.md) | a dependency only tests use is a dev-dependency | off | off | off | off | off | off |
| [shear-unused-deps](shear-unused-deps.md) | no crate declares a dependency it never uses, and every workspace dependency is used | off | off | off | off | off | off |

## siz

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [siz-1](siz-1.md) | flash and RAM regions stay under a fill limit | off | off | off | off | off | warn |
| [siz-3](siz-3.md) | no per-package opt-level beside lto = "fat" | off | off | off | off | off | warn |
| [siz-4](siz-4.md) | the release profile sets codegen-units = 1, lto, panic = "abort" and a measured opt-level | off | off | off | off | off | warn |
| [siz-5](siz-5.md) | core::fmt stays out of the firmware | off | off | off | off | off | warn |
| [siz-6](siz-6.md) | no large .data initializer | off | off | off | off | off | warn |

## size

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [crate-lines](crate-lines.md) | a crate's src/ stays under its line limit | off | off | warn | warn | off | off |
| [function-complexity](function-complexity.md) | a function's cyclomatic complexity stays under its limit | warn | warn | warn | warn | off | off |
| [function-lines](function-lines.md) | a function stays under its line limit | warn | warn | warn | warn | off | off |
| [module-lines](module-lines.md) | a source file stays under its line limit | off | off | warn | warn | off | off |
| [test-lines](test-lines.md) | a crate's test code stays under its line limit | off | off | off | warn | off | off |

## state

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [lock-held-across](lock-held-across.md) | no lock guard is held across an .await or an I/O call | warn | warn | warn | deny | off | off |
| [lock-unwrap](lock-unwrap.md) | a crate keeps its lock().unwrap() calls under a limit | off | off | warn | warn | off | off |
| [nested-locks](nested-locks.md) | no second lock is taken while a guard is alive | off | warn | warn | warn | off | off |
| [static-interior-mutability](static-interior-mutability.md) | a static with interior mutability has one owner, or a reason why a global lock is right | warn | warn | warn | deny | off | off |
| [static-mut](static-mut.md) | no static mut | deny | deny | deny | deny | off | off |
| [thread-local-state](thread-local-state.md) | thread_local! state has one owner, or a reason | off | warn | warn | warn | off | off |

## stk

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [stk-1](stk-1.md) | flip-link is the linker | off | off | off | off | off | deny |
| [stk-2](stk-2.md) | build.rs prints only directives Cargo knows | off | off | off | off | off | deny |
| [stk-3](stk-3.md) | the stack sits below .data and .bss and is not empty | off | off | off | off | off | deny |
| [stk-4](stk-4.md) | cortex-m-rt paint-stack stays off | off | off | off | off | off | deny |
| [stk-5](stk-5.md) | big state is built in place, not by value and moved into a static | off | off | off | off | off | warn |
| [stk-6](stk-6.md) | every stack frame stays within its budget and the deepest call chain within the stack | off | off | off | off | off | warn |

## stop-gap

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [commented-out-code](commented-out-code.md) | no code is left commented out | off | off | off | off | deny | off |
| [dead-features](dead-features.md) | every cargo feature enables something or is tested by a cfg | off | off | off | off | warn | off |
| [lint-table-allow-reason](lint-table-allow-reason.md) | a lint allowed in a Cargo.toml [lints] table says why in a comment | off | off | off | off | deny | off |
| [near-duplicate-fns](near-duplicate-fns.md) | no two functions are near copies of each other | off | off | off | off | warn | off |
| [near-duplicate-modules](near-duplicate-modules.md) | no two modules are near copies of each other | off | off | off | off | deny | off |
| [stop-gap-comments](stop-gap-comments.md) | comments carry no stop-gap markers: TODO, FIXME, HACK, for now, workaround | off | off | off | deny | deny | off |

## structure

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [module-graph-scc](module-graph-scc.md) | no module cycle holds most of a crate's code | off | off | warn | warn | off | off |

## taplo

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [taplo-fmt](taplo-fmt.md) | every TOML file parses and is formatted as taplo writes it | off | warn | warn | deny | off | off |

## typos

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [typos-misspelling](typos-misspelling.md) | no misspelled word in code, comments, docs or file names | warn | warn | warn | deny | off | off |

## udeps

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [udeps-unused-deps](udeps-unused-deps.md) | no crate declares a dependency the compiler never loads | off | off | off | off | off | off |

## usb

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [usb-1](usb-1.md) | high-speed bulk endpoints have 512-byte packets and bIntervals in range | off | off | off | off | off | deny |
| [usb-3](usb-3.md) | bulk endpoints on a hot path are double-buffered | off | off | off | off | off | warn |

## waivers

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [waiver-invalid](waiver-invalid.md) | a waiver comment names known lints, gives a reason and has not expired | deny | deny | deny | deny | off | off |
| [waiver-unused](waiver-unused.md) | every waiver comment allows a finding | warn | warn | warn | deny | off | off |

## workspace

| Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|
| [crate-dir-name](crate-dir-name.md) | a member's directory is named after its crate | off | warn | warn | warn | off | off |
| [macro-crate-suffix](macro-crate-suffix.md) | proc-macro crates and only they end in _macros or _derive | off | warn | warn | warn | off | off |
| [workspace-dependencies](workspace-dependencies.md) | members take every dependency from [workspace.dependencies] | off | warn | deny | deny | off | off |
| [workspace-inheritance](workspace-inheritance.md) | members inherit the shared package keys from [workspace.package] | off | deny | deny | deny | off | off |
| [workspace-lints](workspace-lints.md) | the root declares [workspace.lints] and every member inherits them | off | deny | deny | deny | off | off |
