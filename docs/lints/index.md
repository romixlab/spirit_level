# Lints

Every lint, grouped by category, with its stable code and its level in each shipped tier (`off`: the tier does not enable it). A project changes any of them in its `spirit_level.toml` ([configuration](../configuration.md)).

## agents

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [P0001](P0001.md) | [agent-file-lines](P0001.md) | an agent file stays under its line limit | warn | warn | warn | warn | deny | off |
| [P0002](P0002.md) | [agent-file-links](P0002.md) | links and @imports in agent files resolve | off | off | off | off | deny | off |
| [P0003](P0003.md) | [agent-rules-match-lints](P0003.md) | limits an agent file states are the limits the lints check | off | off | off | warn | deny | off |
| [P0004](P0004.md) | [claude-md-duplicate](P0004.md) | CLAUDE.md points to AGENTS.md instead of copying it | off | off | off | off | deny | off |

## api

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0001](C0001.md) | [pub-surface](C0001.md) | a library's public surface stays small for its size | off | off | off | warn | off | off |

## audit

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0001](X0001.md) | [audit-unmaintained](X0001.md) | no dependency is marked unmaintained or replaced in the RustSec database | off | off | off | off | off | off |
| [X0002](X0002.md) | [audit-unsound](X0002.md) | no dependency is marked unsound in the RustSec database | off | off | off | off | off | off |
| [X0003](X0003.md) | [audit-vulnerabilities](X0003.md) | no dependency has a published security vulnerability | off | off | off | off | off | off |
| [X0004](X0004.md) | [audit-yanked](X0004.md) | Cargo.lock holds no yanked version | off | off | off | off | off | off |

## boot

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0001](F0001.md) | [boot-1](F0001.md) | a firmware entered by a jump detects it and resets once | off | off | off | off | off | deny |
| [F0002](F0002.md) | [boot-10](F0002.md) | backup-domain helpers restore what they found | off | off | off | off | off | warn |
| [F0003](F0003.md) | [boot-2](F0003.md) | the H7 supply configuration is read back, never written blind | off | off | off | off | off | deny |

## build

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0005](X0005.md) | [cargo-doc](X0005.md) | cargo doc builds the workspace's documentation without a warning | warn | warn | deny | deny | off | off |
| [X0006](X0006.md) | [cargo-msrv](X0006.md) | every crate builds with the toolchain of its declared rust-version | off | off | warn | deny | off | off |
| [X0007](X0007.md) | [cargo-no-std](X0007.md) | every crate the map marks no_std builds for targets without std | off | off | off | deny | off | off |
| [X0008](X0008.md) | [cargo-wasm](X0008.md) | every crate the map marks wasm builds for wasm32-unknown-unknown | off | off | off | deny | off | off |

## cfg

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0004](F0004.md) | [cfg-1](F0004.md) | every probe-rs runner and recipe names its probe with --probe VID:PID | off | off | off | off | off | deny |
| [F0005](F0005.md) | [cfg-2](F0005.md) | F0/G0/L0 firmware is flashed by a runner that halts the core first | off | off | off | off | off | warn |
| [F0006](F0006.md) | [cfg-4](F0006.md) | every crate is in the tree from one source | off | off | off | off | off | deny |
| [F0007](F0007.md) | [cfg-5](F0007.md) | the HAL's chip feature names the part on the board | off | off | off | off | off | deny |
| [F0008](F0008.md) | [cfg-6](F0008.md) | thumbv6m firmware that needs compare-and-swap uses portable-atomic with critical-section | off | off | off | off | off | warn |
| [F0009](F0009.md) | [cfg-7](F0009.md) | the build info carries the git SHA and a dirty flag | off | off | off | off | off | warn |

## clippy

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0009](X0009.md) | [clippy-build](X0009.md) | the workspace builds under clippy without compiler errors or warnings | off | off | off | off | off | off |
| [X0010](X0010.md) | [clippy-cargo](X0010.md) | no finding of the cargo-group clippy lints the project turned on | off | off | off | off | off | off |
| [X0011](X0011.md) | [clippy-complexity](X0011.md) | no finding of clippy's complexity group: simple things done simply | off | off | off | off | off | off |
| [X0012](X0012.md) | [clippy-correctness](X0012.md) | no finding of clippy's correctness group: code that is outright wrong | off | off | off | off | off | off |
| [X0013](X0013.md) | [clippy-nursery](X0013.md) | no finding of the nursery clippy lints the project turned on | off | off | off | off | off | off |
| [X0014](X0014.md) | [clippy-pedantic](X0014.md) | no finding of the pedantic clippy lints the project turned on | off | off | off | off | off | off |
| [X0015](X0015.md) | [clippy-perf](X0015.md) | no finding of clippy's perf group: no needlessly slow code | off | off | off | off | off | off |
| [X0016](X0016.md) | [clippy-restriction](X0016.md) | no finding of the restriction clippy lints the project turned on | off | off | off | off | off | off |
| [X0017](X0017.md) | [clippy-style](X0017.md) | no finding of clippy's style group: code written the idiomatic way | off | off | off | off | off | off |
| [X0018](X0018.md) | [clippy-suspicious](X0018.md) | no finding of clippy's suspicious group: code that is most likely wrong | off | off | off | off | off | off |

## clk

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0010](F0010.md) | [clk-2](F0010.md) | the embassy time driver names a timer that nothing else uses | off | off | off | off | off | deny |
| [F0011](F0011.md) | [clk-3](F0011.md) | the clock security system watches HSE and LSE, with a handler | off | off | off | off | off | warn |
| [F0012](F0012.md) | [clk-5](F0012.md) | USB has an exact 48 MHz clock | off | off | off | off | off | deny |

## cnt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0013](F0013.md) | [cnt-1](F0013.md) | every error branch counts what happened with cnt! | off | off | off | off | off | warn |
| [F0014](F0014.md) | [cnt-2](F0014.md) | counter buffers an interrupt updates are never borrowed mutably or read unaligned | off | off | off | off | off | warn |
| [F0015](F0015.md) | [cnt-3](F0015.md) | a proc-macro crate and its runtime crate are pinned to the same exact version | off | off | off | off | off | warn |

## code

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0002](C0002.md) | [allow-without-reason](C0002.md) | every allow or expect of a lint says why | warn | warn | warn | deny | deny | off |
| [C0003](C0003.md) | [cli-subprocess](C0003.md) | workspace crates call each other as libraries, never by running the workspace's own binaries | off | off | deny | deny | off | off |
| [C0004](C0004.md) | [float-determinism](C0004.md) | deterministic crates compute float transcendentals through libm | off | off | off | deny | off | off |
| [C0005](C0005.md) | [panic-in-library](C0005.md) | library code returns errors instead of panicking | off | off | warn | warn | off | off |
| [C0006](C0006.md) | [print-in-library](C0006.md) | library code does not print | warn | warn | warn | warn | off | off |
| [C0007](C0007.md) | [random-state-hash](C0007.md) | deterministic crates use no randomly seeded hash maps | off | off | off | deny | off | off |
| [C0008](C0008.md) | [todo-macros](C0008.md) | no todo!() or unimplemented!() outside tests | warn | warn | warn | deny | deny | off |
| [C0014](C0014.md) | [build-info-env](C0014.md) | library code reads build info at run time, not through env! | off | off | warn | warn | off | off |
| [C0015](C0015.md) | [test-support-gated](C0015.md) | test fakes and testing modules compile only for tests or behind the test-support feature | off | off | warn | warn | off | off |
| [C0017](C0017.md) | [sized-ints-in-contracts](C0017.md) | serde types of a contract crate use sized integers, not usize or isize | off | off | warn | deny | off | off |
| [C0018](C0018.md) | [unit-suffix](C0018.md) | a numeric field or constant that holds a time or a size names its unit | off | off | off | off | warn | off |
| [C0019](C0019.md) | [unbounded-read](C0019.md) | a read to the end of a stream is bounded with take() | off | off | off | warn | warn | off |

## commits

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [P0005](P0005.md) | [commit-body-lines](P0005.md) | a commit body stays short | off | off | off | off | warn | off |
| [P0006](P0006.md) | [commit-prose](P0006.md) | commit messages use no filler, narration or emoji and do not describe the diff | off | off | off | off | warn | off |
| [P0007](P0007.md) | [commit-subject-length](P0007.md) | a commit subject stays under its length limit | off | off | warn | warn | deny | off |
| [P0017](P0017.md) | [fixes-tag](P0017.md) | a Fixes: trailer names its culprit by a hash that resolves and the culprit's subject | off | off | warn | warn | deny | off |
| [P0018](P0018.md) | [fix-without-test](P0018.md) | a commit that fixes code adds a test | off | off | off | warn | warn | off |

## dead-code

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0009](C0009.md) | [dead-pub](C0009.md) | public items of workspace libraries are used outside their crate | off | off | off | warn | off | off |

## deny

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0019](X0019.md) | [deny-advisories](X0019.md) | no dependency has a security advisory, is unmaintained, unsound or yanked | warn | warn | deny | deny | off | off |
| [X0020](X0020.md) | [deny-bans](X0020.md) | the dependency graph holds no banned crate and no duplicate deny.toml refuses | off | off | off | deny | off | off |
| [X0021](X0021.md) | [deny-licenses](X0021.md) | every crate's licence is one deny.toml allows | off | off | warn | deny | off | off |
| [X0022](X0022.md) | [deny-sources](X0022.md) | every crate comes from a registry or git source deny.toml allows | off | off | off | deny | off | off |

## dependencies

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [M0001](M0001.md) | [default-features-off](M0001.md) | third-party dependencies are declared with default-features = false | warn | warn | warn | deny | off | off |
| [M0002](M0002.md) | [duplicate-versions](M0002.md) | no crate is in the dependency graph in two versions | off | warn | warn | deny | off | off |
| [M0003](M0003.md) | [git-dependency-pins](M0003.md) | git dependencies are pinned the way the project's policy says: a branch by default | warn | warn | deny | deny | off | off |
| [M0004](M0004.md) | [no-sys-crates](M0004.md) | the dependency graph has no -sys crates and no C build steps | deny | deny | deny | deny | off | off |
| [M0005](M0005.md) | [path-outside-workspace](M0005.md) | path dependencies stay inside the workspace | deny | deny | deny | deny | off | off |
| [M0027](M0027.md) | [patch-outside-workspace](M0027.md) | no committed [patch] or [replace] override takes a path outside the workspace | deny | deny | deny | deny | off | off |

## dma

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0016](F0016.md) | [dma-1](F0016.md) | DMA buffers sit in memory the DMA master can reach | off | off | off | off | off | deny |
| [F0017](F0017.md) | [dma-2](F0017.md) | DMA buffers in cacheable RAM are cleaned and invalidated, or not cacheable | off | off | off | off | off | deny |
| [F0018](F0018.md) | [dma-4](F0018.md) | every DMA ring consumer counts its overruns | off | off | off | off | off | warn |
| [F0019](F0019.md) | [dma-6](F0019.md) | where an erratum asks for DMA, the code uses DMA instead of polling | off | off | off | off | off | warn |

## embedded

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0020](F0020.md) | [firmware-readable](F0020.md) | every file of a firmware crate that the embedded lints read can be read | off | off | off | off | off | deny |

## err

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0021](F0021.md) | [err-11](F0021.md) | the silicon revision is read from a source that tells every revision apart | off | off | off | off | off | warn |
| [F0022](F0022.md) | [err-2](F0022.md) | every register poke for an erratum quotes the erratum next to it | off | off | off | off | off | deny |
| [F0023](F0023.md) | [err-3](F0023.md) | flash prefetch is off where code crosses into bank 2 on G0B0/G0B1/G0C1 | off | off | off | off | off | deny |
| [F0024](F0024.md) | [err-4](F0024.md) | FDCAN edge filtering and dedicated Tx buffers stay off | off | off | off | off | off | deny |
| [F0025](F0025.md) | [err-5](F0025.md) | FDCAN one-shot mode retransmits after a lost arbitration | off | off | off | off | off | warn |
| [F0026](F0026.md) | [err-6](F0026.md) | RP2350 A2 pads with a pull-down enable their input only for the read | off | off | off | off | off | warn |
| [F0027](F0027.md) | [err-7](F0027.md) | RP2350 ROM reboots use a delay and only the allowed FRCE_OFF bits | off | off | off | off | off | warn |
| [F0028](F0028.md) | [err-8](F0028.md) | G0 ADC calibration waits two ADC clocks after the disable | off | off | off | off | off | warn |

## errors

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0010](C0010.md) | [error-enum-growth](C0010.md) | a big error enum wraps the errors it comes from instead of listing every failure flat | off | off | warn | warn | off | off |

## exe

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0029](F0029.md) | [exe-1](F0029.md) | a task with a hard answer deadline runs on its own executor, not beside bulk work on the thread executor | off | off | off | off | off | deny |
| [F0030](F0030.md) | [exe-10](F0030.md) | one crate provides the executor and its pender | off | off | off | off | off | deny |
| [F0031](F0031.md) | [exe-12](F0031.md) | every async fn awaits something | off | off | off | off | off | warn |
| [F0032](F0032.md) | [exe-13](F0032.md) | a ring-buffered reader in a loop batches its reads | off | off | off | off | off | warn |
| [F0033](F0033.md) | [exe-2](F0033.md) | tasks on an interrupt executor and interrupt handlers never block | off | off | off | off | off | deny |
| [F0034](F0034.md) | [exe-3](F0034.md) | an interrupt executor's priority is set before it is started | off | off | off | off | off | deny |
| [F0035](F0035.md) | [exe-4](F0035.md) | an interrupt executor runs below the interrupts that wake its tasks | off | off | off | off | off | warn |
| [F0036](F0036.md) | [exe-5](F0036.md) | an interrupt executor uses a vector no driver binds and no used peripheral raises | off | off | off | off | off | deny |
| [F0037](F0037.md) | [exe-6](F0037.md) | state shared between an interrupt executor and thread mode sits behind a critical-section mutex | off | off | off | off | off | deny |
| [F0038](F0038.md) | [exe-7](F0038.md) | a loop that drains a queue in a shared task is bounded and yields | off | off | off | off | off | warn |
| [F0039](F0039.md) | [exe-8](F0039.md) | a request handler answers without waiting on a bus or a slow peripheral | off | off | off | off | off | warn |
| [F0040](F0040.md) | [exe-9](F0040.md) | no future that reaches embassy-time is polled with a no-op waker | off | off | off | off | off | deny |

## ext

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0041](F0041.md) | [ext-12](F0041.md) | an RCC enable is read back before the peripheral's first access | off | off | off | off | off | warn |
| [F0042](F0042.md) | [ext-4](F0042.md) | external chip registers are written from shadow copies, not read-modify-write | off | off | off | off | off | warn |
| [F0043](F0043.md) | [ext-7](F0043.md) | register fields are range-checked before they are encoded | off | off | off | off | off | warn |

## fls

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0044](F0044.md) | [fls-1](F0044.md) | flash a power cut can tear is read through ECC-aware access on parts with flash ECC | off | off | off | off | off | deny |
| [F0045](F0045.md) | [fls-2](F0045.md) | the flash peripheral has one owner | off | off | off | off | off | deny |
| [F0046](F0046.md) | [fls-3](F0046.md) | RP flash is erased and written from core 0 with nothing else reading flash | off | off | off | off | off | deny |
| [F0047](F0047.md) | [fls-4](F0047.md) | the image in flash is checked by a periodic CRC, and ECC is on where the part has it | off | off | off | off | off | warn |

## flt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0048](F0048.md) | [flt-1](F0048.md) | the HardFault handler records the crash and resets instead of spinning | off | off | off | off | off | deny |
| [F0049](F0049.md) | [flt-2](F0049.md) | HardFault and DefaultHandler do nothing that can panic or log | off | off | off | off | off | deny |
| [F0050](F0050.md) | [flt-3](F0050.md) | bedrock_crash::exc_return!() is the first statement of HardFault | off | off | off | off | off | warn |
| [F0051](F0051.md) | [flt-4](F0051.md) | UsageFault, MemManage and BusFault have handlers and are enabled where the core has them | off | off | off | off | off | warn |
| [F0052](F0052.md) | [flt-5](F0052.md) | the reset cause is read and reported at boot | off | off | off | off | off | warn |
| [F0053](F0053.md) | [flt-7](F0053.md) | a watchdog runs, fed from a bounded loop of the main work | off | off | off | off | off | deny |
| [F0054](F0054.md) | [flt-8](F0054.md) | a boot-loop rescue falls back to the ROM bootloader after repeated unhealthy boots | off | off | off | off | off | warn |
| [F0055](F0055.md) | [flt-9](F0055.md) | the healthy mark is set on the transport in use | off | off | off | off | off | warn |

## geiger

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0023](X0023.md) | [geiger-unsafe-deps](X0023.md) | every dependency that uses unsafe code is one the project accepted | off | off | off | off | off | off |

## graph

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [G0001](G0001.md) | [core-crate-deps](G0001.md) | a _core crate depends only on other _core crates, the allowed layers and listed vocabulary crates | off | off | warn | deny | off | off |
| [G0002](G0002.md) | [core-crate-types](G0002.md) | a _core crate holds types: no I/O, no threads, no async runtime | off | off | off | deny | off | off |
| [G0003](G0003.md) | [crate-cycles](G0003.md) | workspace crates form no cycle, dev-dependencies included | off | warn | deny | deny | off | off |
| [G0004](G0004.md) | [crate-table](G0004.md) | the crate table in the README is the one generated from the manifests and the map | off | off | off | deny | off | off |
| [G0005](G0005.md) | [external-form](G0005.md) | crates of other projects come by git as the map lists them, used only where their role allows | off | off | off | deny | off | off |
| [G0006](G0006.md) | [forbidden-closure](G0006.md) | a crate never reaches the crates the map forbids it, directly or through others | off | off | off | deny | off | off |
| [G0007](G0007.md) | [layer-edges](G0007.md) | every dependency edge between workspace crates is allowed by the map | off | off | off | deny | off | off |
| [G0008](G0008.md) | [map-complete](G0008.md) | the crate map and the workspace list the same crates | off | off | off | deny | off | off |
| [G0009](G0009.md) | [third-party-placement](G0009.md) | heavy third-party crates are used only by the crates the map names | off | off | off | deny | off | off |
| [G0010](G0010.md) | [unsafe-allow-list](G0010.md) | unsafe code is forbidden except in the crates the map marks unsafe | off | off | off | deny | off | off |

## hakari

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0024](X0024.md) | [hakari-generate](X0024.md) | the workspace-hack package's dependencies are the ones hakari computes | off | off | off | warn | off | off |
| [X0025](X0025.md) | [hakari-manage-deps](X0025.md) | every member hakari manages depends on the workspace-hack package, and no other | off | off | off | warn | off | off |

## host

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0056](F0056.md) | [host-1](F0056.md) | nothing reachable from host input can panic | off | off | off | off | off | deny |
| [F0057](F0057.md) | [host-3](F0057.md) | unknown enum values from the host are refused, not mapped to a default | off | off | off | off | off | warn |
| [F0058](F0058.md) | [host-5](F0058.md) | truncated and oversized requests are counted and dropped | off | off | off | off | off | warn |

## hygiene

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [A0001](A0001.md) | [agent-config-permissions](A0001.md) | committed agent settings grant no blanket permissions | off | off | off | off | deny | off |
| [A0002](A0002.md) | [agent-config-secrets](A0002.md) | committed agent configuration holds no secrets | off | off | off | off | deny | off |
| [A0003](A0003.md) | [agent-session-files](A0003.md) | no agent transcripts or per-person agent files are committed | off | off | off | off | deny | off |
| [A0004](A0004.md) | [leftover-files](A0004.md) | no merge, patch, editor or scratch leftovers are committed | off | off | off | off | deny | off |

## irq

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0059](F0059.md) | [irq-1](F0059.md) | interrupt handlers clear exactly the flags they handled | off | off | off | off | off | deny |
| [F0060](F0060.md) | [irq-2](F0060.md) | write-1-to-clear flags are cleared with a write, not modify() | off | off | off | off | off | deny |
| [F0061](F0061.md) | [irq-4](F0061.md) | no driver disables a shared interrupt vector that other drivers use | off | off | off | off | off | deny |
| [F0062](F0062.md) | [irq-5](F0062.md) | every routed interrupt line is enabled and has a handler bound | off | off | off | off | off | warn |
| [F0063](F0063.md) | [irq-6](F0063.md) | interrupts are disabled through critical sections, never nested by hand | off | off | off | off | off | warn |
| [F0064](F0064.md) | [irq-8](F0064.md) | the default interrupt handler records which IRQ fired | off | off | off | off | off | warn |

## log

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0065](F0065.md) | [log-1](F0065.md) | RTT never blocks | off | off | off | off | off | deny |
| [F0066](F0066.md) | [log-2](F0066.md) | DEFMT_RTT_BUFFER_SIZE and DEFMT_LOG are set in .cargo/config.toml | off | off | off | off | off | warn |

## machete

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0026](X0026.md) | [machete-unused-deps](X0026.md) | no crate declares a dependency its sources never use | warn | warn | warn | deny | off | off |

## manifest

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [M0006](M0006.md) | [crate-docs](M0006.md) | every crate root opens with crate-level documentation | warn | warn | warn | deny | off | off |
| [M0007](M0007.md) | [edition-latest](M0007.md) | every crate is on the current edition | warn | warn | warn | warn | off | off |
| [M0008](M0008.md) | [feature-docs](M0008.md) | every cargo feature has a comment saying what it turns on | off | off | off | warn | off | off |
| [M0009](M0009.md) | [package-description](M0009.md) | every crate has a description in its Cargo.toml | warn | warn | warn | deny | off | off |
| [M0010](M0010.md) | [package-license](M0010.md) | every crate states its licence | warn | warn | warn | warn | off | off |
| [M0011](M0011.md) | [publish-false](M0011.md) | workspace crates are publish = false unless listed as published | off | off | warn | warn | off | off |
| [M0012](M0012.md) | [readme-per-crate](M0012.md) | every crate has a README | warn | deny | deny | deny | off | off |
| [M0013](M0013.md) | [rust-version-set](M0013.md) | every crate declares its minimum Rust version | off | off | warn | deny | off | off |
| [M0025](M0025.md) | [tier-clippy-set](M0025.md) | the project's [lints.clippy] and clippy.toml turn on the Clippy set its tiers expect, every opt-out with a reason | warn | warn | deny | deny | off | off |

## mem

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0067](F0067.md) | [mem-1](F0067.md) | the FLASH region of memory.x leaves out the regions kept for other uses | off | off | off | off | off | deny |
| [F0068](F0068.md) | [mem-10](F0068.md) | no LOAD segment is writable and executable, and .rodata holds nothing writable | off | off | off | off | off | warn |
| [F0069](F0069.md) | [mem-12](F0069.md) | a firmware with defmt links with -Tdefmt.x | off | off | off | off | off | deny |
| [F0070](F0070.md) | [mem-2](F0070.md) | extra RAM sections end without a tail ALIGN or symbol | off | off | off | off | off | warn |
| [F0071](F0071.md) | [mem-3](F0071.md) | the RAM init and zeroing ranges stay clear of .uninit | off | off | off | off | off | deny |
| [F0072](F0072.md) | [mem-6](F0072.md) | .cnt_bkp_buffer sits at the backup-register address | off | off | off | off | off | warn |
| [F0073](F0073.md) | [mem-7](F0073.md) | embassy-boot partition symbols are offsets from the flash origin | off | off | off | off | off | warn |

## perf

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0074](F0074.md) | [perf-2](F0074.md) | hot paths on cores without 64-bit maths avoid it, and float formatting | off | off | off | off | off | warn |
| [F0075](F0075.md) | [perf-3](F0075.md) | RAM functions are #[inline(never)] | off | off | off | off | off | warn |

## pin

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0076](F0076.md) | [pin-1](F0076.md) | pin drivers live as long as the pin is used | off | off | off | off | off | deny |
| [F0077](F0077.md) | [pin-16](F0077.md) | pins held by a dead-battery pull-down are read only after it is released | off | off | off | off | off | warn |
| [F0078](F0078.md) | [pin-2](F0078.md) | the board is put in its safe state first thing after the HAL init | off | off | off | off | off | deny |
| [F0079](F0079.md) | [pin-4](F0079.md) | peripheral pins stay on their function while the peripheral is enabled | off | off | off | off | off | deny |
| [F0080](F0080.md) | [pin-6](F0080.md) | every pin has one owner | off | off | off | off | off | deny |
| [F0081](F0081.md) | [pin-8](F0081.md) | unused pins are analog or pulled, never floating inputs | off | off | off | off | off | warn |
| [F0082](F0082.md) | [pin-9](F0082.md) | the pin map comes from the board at run time, not from a cargo feature | off | off | off | off | off | deny |

## prose

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [P0008](P0008.md) | [changelog-describes-diff](P0008.md) | a CHANGELOG entry says what changed for the user, not which code was edited | off | off | off | off | warn | off |
| [P0009](P0009.md) | [changelog-entry-length](P0009.md) | a CHANGELOG entry is one short line | warn | warn | warn | warn | deny | off |
| [P0010](P0010.md) | [emoji](P0010.md) | docs and comments carry no emoji | off | off | off | off | warn | off |
| [P0011](P0011.md) | [filler-phrases](P0011.md) | docs and comments use no filler phrases | off | off | off | off | deny | off |
| [P0012](P0012.md) | [narration](P0012.md) | docs and comments state facts, not how they were written | off | off | off | off | deny | off |
| [P0013](P0013.md) | [paragraph-sentences](P0013.md) | paragraphs in docs stay short | off | off | off | off | warn | off |
| [P0014](P0014.md) | [restating-comments](P0014.md) | a comment says what the code cannot | off | off | off | off | warn | off |
| [P0015](P0015.md) | [sentence-words](P0015.md) | sentences in docs stay short | off | off | off | off | warn | off |
| [P0016](P0016.md) | [thinking-comments](P0016.md) | comments state conclusions, not a stream of thought | off | off | off | off | deny | off |

## pwr

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0083](F0083.md) | [pwr-1](F0083.md) | a brown-out reset level or voltage detector fits the board's supply | off | off | off | off | off | warn |
| [F0084](F0084.md) | [pwr-3](F0084.md) | the backup domain is reset at boot when nothing keeps state there | off | off | off | off | off | warn |
| [F0085](F0085.md) | [pwr-4](F0085.md) | a firmware that keeps state in the backup domain never changes the RTC clock source | off | off | off | off | off | warn |

## repo

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [M0014](M0014.md) | [changelog-unreleased](M0014.md) | CHANGELOG.md has an Unreleased section | warn | warn | warn | warn | off | off |
| [M0015](M0015.md) | [justfile-recipes](M0015.md) | the justfile has the standard recipes | warn | warn | warn | warn | off | off |
| [M0016](M0016.md) | [justfile-simple](M0016.md) | justfile recipes only call programs: a few lines, no inline scripts | off | off | off | warn | off | off |
| [M0017](M0017.md) | [profile-defaults](M0017.md) | the dev profile splits debug info and a deploy profile builds incrementally | warn | warn | warn | warn | off | off |
| [M0018](M0018.md) | [repo-files](M0018.md) | the repo carries its rule files | warn | warn | warn | warn | deny | off |
| [M0019](M0019.md) | [toolchain-pinned](M0019.md) | the repo pins its toolchain in rust-toolchain.toml | off | off | warn | deny | off | off |
| [M0026](M0026.md) | [profile-no-lto](M0026.md) | the profiles of the edit-build-test loop and the deploy profile build without LTO | warn | warn | warn | warn | off | off |
| [M0028](M0028.md) | [test-binaries](M0028.md) | a crate builds its integration tests as one binary | off | off | warn | warn | off | off |

## rustfmt

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0027](X0027.md) | [rustfmt-unformatted](X0027.md) | every Rust file is formatted the way cargo fmt writes it | off | off | off | off | off | off |

## semver

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0028](X0028.md) | [semver-checks-breaking](X0028.md) | a stable crate changes its public API only with the version bump the change needs | off | off | off | deny | off | off |

## shear

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0029](X0029.md) | [shear-misplaced-deps](X0029.md) | a dependency only tests use is a dev-dependency | off | off | off | off | off | off |
| [X0030](X0030.md) | [shear-unused-deps](X0030.md) | no crate declares a dependency it never uses, and every workspace dependency is used | off | off | off | off | off | off |

## siz

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0086](F0086.md) | [siz-1](F0086.md) | flash and RAM regions stay under a fill limit | off | off | off | off | off | warn |
| [F0087](F0087.md) | [siz-3](F0087.md) | no per-package opt-level beside lto = "fat" | off | off | off | off | off | warn |
| [F0088](F0088.md) | [siz-4](F0088.md) | the release profile sets codegen-units = 1, lto, panic = "abort" and a measured opt-level | off | off | off | off | off | warn |
| [F0089](F0089.md) | [siz-5](F0089.md) | core::fmt stays out of the firmware | off | off | off | off | off | warn |
| [F0090](F0090.md) | [siz-6](F0090.md) | no large .data initializer | off | off | off | off | off | warn |

## size

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [S0001](S0001.md) | [crate-lines](S0001.md) | a crate's src/ stays under its line limit | off | off | warn | warn | off | off |
| [S0002](S0002.md) | [function-complexity](S0002.md) | a function's cyclomatic complexity stays under its limit | warn | warn | warn | warn | off | off |
| [S0003](S0003.md) | [function-lines](S0003.md) | a function stays under its line limit | warn | warn | warn | warn | off | off |
| [S0004](S0004.md) | [module-lines](S0004.md) | a source file stays under its line limit | off | off | warn | warn | off | off |
| [S0005](S0005.md) | [test-lines](S0005.md) | a crate's test code stays under its line limit | off | off | off | warn | off | off |

## state

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [T0001](T0001.md) | [lock-held-across](T0001.md) | no lock guard is held across an .await or an I/O call | warn | warn | warn | deny | off | off |
| [T0002](T0002.md) | [lock-unwrap](T0002.md) | a crate keeps its lock().unwrap() calls under a limit | off | off | warn | warn | off | off |
| [T0003](T0003.md) | [nested-locks](T0003.md) | no second lock is taken while a guard is alive | off | warn | warn | warn | off | off |
| [T0004](T0004.md) | [static-interior-mutability](T0004.md) | a static with interior mutability has one owner, or a reason why a global lock is right | warn | warn | warn | deny | off | off |
| [T0005](T0005.md) | [static-mut](T0005.md) | no static mut | deny | deny | deny | deny | off | off |
| [T0006](T0006.md) | [thread-local-state](T0006.md) | thread_local! state has one owner, or a reason | off | warn | warn | warn | off | off |

## stk

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0091](F0091.md) | [stk-1](F0091.md) | flip-link is the linker | off | off | off | off | off | deny |
| [F0092](F0092.md) | [stk-2](F0092.md) | build.rs prints only directives Cargo knows | off | off | off | off | off | deny |
| [F0093](F0093.md) | [stk-3](F0093.md) | the stack sits below .data and .bss and is not empty | off | off | off | off | off | deny |
| [F0094](F0094.md) | [stk-4](F0094.md) | cortex-m-rt paint-stack stays off | off | off | off | off | off | deny |
| [F0095](F0095.md) | [stk-5](F0095.md) | big state is built in place, not by value and moved into a static | off | off | off | off | off | warn |
| [F0096](F0096.md) | [stk-6](F0096.md) | every stack frame stays within its budget and the deepest call chain within the stack | off | off | off | off | off | warn |

## stop-gap

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [A0005](A0005.md) | [commented-out-code](A0005.md) | no code is left commented out | off | off | off | off | deny | off |
| [A0006](A0006.md) | [dead-features](A0006.md) | every cargo feature enables something or is tested by a cfg | off | off | off | off | warn | off |
| [A0007](A0007.md) | [lint-table-allow-reason](A0007.md) | a lint allowed in a Cargo.toml [lints] table says why in a comment | off | off | off | off | deny | off |
| [A0008](A0008.md) | [near-duplicate-fns](A0008.md) | no two functions are near copies of each other | off | off | off | off | warn | off |
| [A0009](A0009.md) | [near-duplicate-modules](A0009.md) | no two modules are near copies of each other | off | off | off | off | deny | off |
| [A0010](A0010.md) | [stop-gap-comments](A0010.md) | comments carry no stop-gap markers: TODO, HACK, for now, workaround | off | off | off | deny | deny | off |
| [A0011](A0011.md) | [fixme-before-land](A0011.md) | no FIXME is left when a branch lands | off | off | off | deny | deny | off |

## structure

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0011](C0011.md) | [module-graph-scc](C0011.md) | no module cycle holds most of a crate's code | off | off | warn | warn | off | off |
| [C0016](C0016.md) | [orphan-source-file](C0016.md) | every .rs file under src/ is reached by a mod declaration | off | off | warn | deny | off | off |

## taplo

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0031](X0031.md) | [taplo-fmt](X0031.md) | every TOML file parses and is formatted as taplo writes it | off | warn | warn | deny | off | off |

## typos

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0032](X0032.md) | [typos-misspelling](X0032.md) | no misspelled word in code, comments, docs or file names | warn | warn | warn | deny | off | off |

## udeps

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [X0033](X0033.md) | [udeps-unused-deps](X0033.md) | no crate declares a dependency the compiler never loads | off | off | off | off | off | off |

## usb

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [F0097](F0097.md) | [usb-1](F0097.md) | high-speed bulk endpoints have 512-byte packets and bIntervals in range | off | off | off | off | off | deny |
| [F0098](F0098.md) | [usb-3](F0098.md) | bulk endpoints on a hot path are double-buffered | off | off | off | off | off | warn |

## waivers

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [C0012](C0012.md) | [waiver-invalid](C0012.md) | a waiver comment names known lints, gives a reason and has not expired | deny | deny | deny | deny | off | off |
| [C0013](C0013.md) | [waiver-unused](C0013.md) | every waiver comment allows a finding | warn | warn | warn | deny | off | off |

## workspace

| Code | Lint | Rule | small | small-workspace | medium | big | ai | embedded |
|---|---|---|---|---|---|---|---|---|
| [M0020](M0020.md) | [crate-dir-name](M0020.md) | a member's directory is named after its crate | off | warn | warn | warn | off | off |
| [M0021](M0021.md) | [macro-crate-suffix](M0021.md) | proc-macro crates and only they end in _macros or _derive | off | warn | warn | warn | off | off |
| [M0022](M0022.md) | [workspace-dependencies](M0022.md) | members take every dependency from [workspace.dependencies] | off | warn | deny | deny | off | off |
| [M0023](M0023.md) | [workspace-inheritance](M0023.md) | members inherit the shared package keys from [workspace.package] | off | deny | deny | deny | off | off |
| [M0024](M0024.md) | [workspace-lints](M0024.md) | the root declares [workspace.lints] and every member inherits them | off | deny | deny | deny | off | off |
| [M0029](M0029.md) | [macro-compile-fail](M0029.md) | every proc-macro crate has compile-fail tests | off | off | warn | warn | off | off |
