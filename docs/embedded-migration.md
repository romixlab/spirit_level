# Migrating from bedrock-lint

spirit_level carries every automatic check of embedded_bedrock's firmware lint, `bedrock-lint`: 97 checks, each a lint with the same id in kebab-case (`EXE-3` is [`exe-3`](lints/exe-3.md)), the same fixtures and the same findings. They live in the `embedded` tier, which stands beside the size tiers, and one more lint, [`firmware-readable`](lints/firmware-readable.md), reports the files bedrock-lint printed as loader notes.

## Running it

| bedrock-lint | spirit_level |
|---|---|
| `bedrock-lint check fw/board` in the firmware crate | `spl check` in the workspace: every firmware crate of it is checked |
| `--elf target/<triple>/release/<name>` | `--elf target/<triple>/release/<name>`, repeated for several firmware crates; an ELF goes with the crate that has a binary of its file name, or with the only firmware crate |
| `--only EXE-3,STK` | `--only exe-3 --only stk` (a lint id or a category) |
| `--format sarif` | `--format json`; SARIF is planned (FEATURES `ENG-5`) |
| `--fail-on warning` (the default) | a `deny` finding fails the run, a `warn` one never does: set the warnings that must fail to `deny` in `spirit_level.toml` |
| `--root` (where discovery stops) | discovery stops at the workspace root |
| `checks list --all`, `explain EXE-3` | `spl lints`, `spl explain exe-3`, and the [lint pages](lints/index.md) |
| `code-size <ELF> --by-crate` | not ported: a report, not a check |

A firmware crate is a workspace member with a binary whose root is `#![no_std]`, or with a `memory.x`, a `bedrock_fw.json` or a `-none-` target in its `.cargo/config.toml`. `[package.metadata.spirit-level] firmware = true` (or `false`) in the crate's `Cargo.toml` decides it by hand.

## Configuration

`bedrock-lint.toml` in the firmware crate becomes the workspace's `spirit_level.toml`:

```toml
# bedrock-lint.toml
baseline = "bedrock-lint.baseline.json"

[severity]
"EXE-12" = "warning"

[skip]
"STK-4" = "paint-stack is checked on the bench"

[limits]
region-percent = 95
frame = 4096
```

```toml
# spirit_level.toml
extends = ["small", "embedded"]

[lints.exe-12]
level = "warn"
reason = "the generated server trait wants async handlers"

[lints.stk-4]
level = "allow"
reason = "paint-stack is checked on the bench"

[lints.siz-1]
region-percent = 95
reason = "the bootloader leaves no room to grow"

[lints.stk-6]
frame = 4096
reason = "the USB stack's frame is measured at 3.9 KiB"
```

| bedrock-lint.toml | spirit_level.toml |
|---|---|
| `[severity] "ID" = "error"` / `"warning"` / `"suggestion"` | `[lints.<id>] level = "deny"` / `"warn"` / `"warn"`, with a `reason` |
| `[skip] "ID" = "reason"` | `[lints.<id>] level = "allow"` with the reason; for one crate or some paths, an [exception](configuration.md) |
| `[limits] region-percent` (default 90) | parameter `region-percent` of [`siz-1`](lints/siz-1.md) |
| `[limits] frame` (2048) | parameter `frame` of [`stk-6`](lints/stk-6.md) |
| `[limits] move-frame` (1024) | parameter `move-frame` of [`stk-5`](lints/stk-5.md) |
| `[limits] data-init` (1024) | parameter `data-init` of [`siz-6`](lints/siz-6.md) |
| `[limits] fmt-bytes` (1024) | parameter `fmt-bytes` of [`siz-5`](lints/siz-5.md) |
| `baseline = "..."`, `--baseline`, `--write-baseline` | no baseline yet (planned: FEATURES `ENG-6`); until then a known finding gets a waiver or an exception with its reason, and `expires` when it is meant to be fixed |

Overrides may expire (`expires = "2027-03-31"` or a release); see [configuration](configuration.md).

## Waivers

`// bedrock-lint: allow(EXE-3) the ROM leaves this vector at P2` becomes `// spirit-level: allow(exe-3) the ROM leaves this vector at P2`, in Rust sources (`build.rs` included) and in `Cargo.toml` files, with the same placement rules ([waivers](waivers.md)). A waiver without a reason is [`waiver-invalid`](lints/waiver-invalid.md), one that allows nothing [`waiver-unused`](lints/waiver-unused.md). For a finding in `.cargo/config.toml`, `memory.x` or `bedrock_fw.json`, use an exception by path in `spirit_level.toml`.

## What runs where

bedrock-lint reported a check as not run when the firmware lacked what it needs (no ELF, no `memory.x`, no chip). In spirit_level only the ELF is a need of the run: the lints that read it say "not run" without `--elf`. The other needs are per crate: a lint checks the firmware crates that have a `memory.x`, a build script, a known chip, Thumb code in the ELF, and finds nothing in the others. The chip comes from `bedrock_fw.json`, else from the HAL's chip feature (`embassy-stm32`, `embassy-rp`, `embassy-nrf`).

Places are relative to the workspace root (`fw/src/main.rs`), not to the firmware crate.

## Every bedrock check

| bedrock | spirit_level | Severity → level | Check |
|---|---|---|---|
| `EXE-1` | [`exe-1`](lints/exe-1.md) | error → `deny` | A task with a hard answer deadline (a PHY that arms RX DMA only inside `receive().await`, a software … |
| `EXE-2` | [`exe-2`](lints/exe-2.md) | error → `deny` | Blocking call (`block_for`, `blocking_*` HAL calls, `nb::block!`, busy-wait loops, … |
| `EXE-3` | [`exe-3`](lints/exe-3.md) | error → `deny` | Interrupt executor started before its IRQ priority is set: `start()` unmasks the IRQ at the default … |
| `EXE-4` | [`exe-4`](lints/exe-4.md) | warning → `warn` | Interrupt executor priority not below the peripheral, DMA and time-driver interrupts that wake its tasks … |
| `EXE-5` | [`exe-5`](lints/exe-5.md) | error → `deny` | Interrupt executor on a vector that a driver also binds (`bind_interrupts!`) or that a used peripheral … |
| `EXE-6` | [`exe-6`](lints/exe-6.md) | error → `deny` | State shared between a task on an interrupt executor and thread-mode code behind `ThreadModeRawMutex` / … |
| `EXE-7` | [`exe-7`](lints/exe-7.md) | warning → `warn` | Loop in a shared task that works until a queue is empty (or logs in a loop) without a bound and a yield: … |
| `EXE-8` | [`exe-8`](lints/exe-8.md) | warning → `warn` | Request server waits on a bus or a slow peripheral inside a handler: defer the transfer to the task that … |
| `EXE-9` | [`exe-9`](lints/exe-9.md) | error → `deny` | Future polled with a no-op waker (`poll_once`, `noop_waker`) that can reach `embassy_time`: embassy-time … |
| `EXE-10` | [`exe-10`](lints/exe-10.md) | error → `deny` | Two crates provide the executor and `__pender` (embassy-rp or embassy-stm32 `executor-*` features and … |
| `EXE-11` | not ported: bedrock has no automatic check for it yet (catalogue: `elf hard`) | warning | Task moved to an interrupt executor without a new look at the stack: it runs on top of the deepest … |
| `EXE-12` | [`exe-12`](lints/exe-12.md) | suggestion → `warn` | `async fn` that never awaits (handlers especially): each one is a state in the dispatcher future and costs … |
| `EXE-13` | [`exe-13`](lints/exe-13.md) | warning → `warn` | Ring-buffered reader (`RingBufferedUartRx::read`) in a loop without batching: a busy line gives 1-byte … |
| `IRQ-1` | [`irq-1`](lints/irq-1.md) | error → `deny` | Interrupt flags cleared wholesale (all-ones or the read value written back to a W1C register): an event … |
| `IRQ-2` | [`irq-2`](lints/irq-2.md) | error → `deny` | `rc_w1` / W1C flags cleared with `modify()` (read-modify-write) instead of a write of the one bit: it … |
| `IRQ-3` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Handler that doesn't check its source flag before acting, or clears it late (double interrupt) |
| `IRQ-4` | [`irq-4`](lints/irq-4.md) | error → `deny` | Several drivers on one shared NVIC vector, and a driver that disables "its" interrupt on drop: the others … |
| `IRQ-5` | [`irq-5`](lints/irq-5.md) | warning → `warn` | Interrupt line routed (ILS) but not enabled (ILE), or enabled without a handler bound |
| `IRQ-6` | [`irq-6`](lints/irq-6.md) | warning → `warn` | Nested disable / enable of interrupts by hand: an inner enable turns them on under an outer section; use … |
| `IRQ-7` | not ported: bedrock has no automatic check for it yet (catalogue: `elf mid`) | suggestion | Interrupt does more than capture (formatting, encoding, logging): a few stores and a counter, the rest in … |
| `IRQ-8` | [`irq-8`](lints/irq-8.md) | warning → `warn` | Unused interrupt vectors not caught: `DefaultHandler` must record the IRQ number (raw: a static, the crash … |
| `DMA-1` | [`dma-1`](lints/dma-1.md) | error → `deny` | DMA buffer in memory the DMA master can't reach (H7 DTCM for IDMA, SDMMC, USB OTG; flash for some … |
| `DMA-2` | [`dma-2`](lints/dma-2.md) | error → `deny` | D-cache on with DMA buffers in cacheable RAM and no clean / invalidate or MPU non-cacheable region |
| `DMA-3` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Receive DMA armed only inside an await (embassy UCPD `receive()`, one-shot UART reads): data that arrives … |
| `DMA-4` | [`dma-4`](lints/dma-4.md) | warning → `warn` | DMA ring overrun not counted: every ring consumer counts overruns with `cnt!` |
| `DMA-5` | not ported: bedrock has no automatic check for it yet (catalogue: `elf mid`) | suggestion | A per-element `%` in the ring copy on a core without a divider (embassy `ReadableDmaRingBuffer` per-byte … |
| `DMA-6` | [`dma-6`](lints/dma-6.md) | warning → `warn` | Erratum asks for DMA where the code polls (G0 UCPD TX underflow then TXHRST corrupts the next CRC: ES0548 … |
| `DMA-7` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | "Things just hang": suspect DMA into a wrong memory region first |
| `STK-1` | [`stk-1`](lints/stk-1.md) | error → `deny` | flip-link not the linker: `linker = "flip-link"` under `[target.<triple>]` in `.cargo/config.toml` |
| `STK-2` | [`stk-2`](lints/stk-2.md) | error → `deny` | `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs: not a build-script directive, … |
| `STK-3` | [`stk-3`](lints/stk-3.md) | error → `deny` | Stack not below `.data` / `.bss` in the ELF (`_stack_start` != start of `.data`), or a zero-size stack |
| `STK-4` | [`stk-4`](lints/stk-4.md) | error → `deny` | cortex-m-rt `paint-stack` on before its BusFault is understood |
| `STK-5` | [`stk-5`](lints/stk-5.md) | warning → `warn` | Big state built by value and moved into a static (`StaticCell::init(Big::new())`): the frame holds it … |
| `STK-6` | [`stk-6`](lints/stk-6.md) | warning → `warn` | Stack frame over a budget (per function) or a worst-case stack depth over the stack region |
| `STK-7` | not ported: checked on hardware | error | Stack overflow never provoked on hardware after a RAM layout change: it must end in a fault, not in … |
| `MEM-1` | [`mem-1`](lints/mem-1.md) | error → `deny` | `FLASH` length in memory.x covers a region kept for something else (settings, bootloader state, DFU): the … |
| `MEM-2` | [`mem-2`](lints/mem-2.md) | warning → `warn` | Extra RAM section with a tail `. = ALIGN(n)` or a symbol in it: lld keeps an empty section that flip-link … |
| `MEM-3` | [`mem-3`](lints/mem-3.md) | error → `deny` | RAM init or zeroing range overlaps `.uninit` (crash record, handoff data) |
| `MEM-4` | not ported: bedrock has no automatic check for it yet (catalogue: `src mid`) | error | Extra SRAM bank used before its RCC clock enable |
| `MEM-5` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid + elf mid`) | error | Backup-register map overlaps: `bkp_cnt!` buffer, boot word, boot reason and board state on the same … |
| `MEM-6` | [`mem-6`](lints/mem-6.md) | warning → `warn` | `.cnt_bkp_buffer` not at the backup-register address |
| `MEM-7` | [`mem-7`](lints/mem-7.md) | warning → `warn` | Flash partition symbols for embassy-boot written as absolute addresses where offsets are expected (`- … |
| `MEM-8` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid, chip table`) | warning | Settings store smaller than two erase pages, or on the code's bank of a dual-bank part (an erase stalls … |
| `MEM-9` | not ported: bedrock has no automatic check for it yet (catalogue: `elf mid, chip table`) | error | Data that must survive the ROM bootloader kept in RAM the ROM uses (H7: AXI SRAM from 0x24000000, DTCM … |
| `MEM-10` | [`mem-10`](lints/mem-10.md) | warning → `warn` | RWX LOAD segment or a writable symbol in `.rodata` (`static mut` in a read-only section) |
| `MEM-11` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg easy`) | warning | Linker warnings ignored (section alignment, `.text` not 8-aligned after `.start_block` on RP235x) |
| `MEM-12` | [`mem-12`](lints/mem-12.md) | error → `deny` | defmt in the dependencies without `-Tdefmt.x` (no `.defmt` section) |
| `SIZ-1` | [`siz-1`](lints/siz-1.md) | warning → `warn` | Flash or RAM use over a threshold (default 90 % of the region) in a shipped profile |
| `SIZ-2` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg easy`) | warning | Dev profile not linked in CI: debug assertions and overflow checks add 18-30 KiB and overflow unnoticed |
| `SIZ-3` | [`siz-3`](lints/siz-3.md) | suggestion → `warn` | Per-package `opt-level` together with `lto = "fat"`: the override does nothing |
| `SIZ-4` | [`siz-4`](lints/siz-4.md) | suggestion → `warn` | Release profile without `codegen-units = 1`, `lto`, `panic = "abort"`, and an `opt-level` chosen by … |
| `SIZ-5` | [`siz-5`](lints/siz-5.md) | suggestion → `warn` | `core::fmt` pulled in: `Result::unwrap` / `expect` (through `&dyn Debug`), `Debug2Format`, a panic handler … |
| `SIZ-6` | [`siz-6`](lints/siz-6.md) | suggestion → `warn` | Large `.data` initializer (a static with a non-zero const value): zero-init and set it up in place |
| `SIZ-7` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid`) | suggestion | A shared crate turns on wide types or features for every firmware (`ww_si` u64: 7.7 KB) |
| `SIZ-8` | not ported: bedrock has no automatic check for it yet (catalogue: `elf mid`) | suggestion | A cargo feature meant to cut code saves nothing (bodies gated, dispatch and ser/des not): measure what … |
| `SIZ-9` | not ported: bedrock has no automatic check for it yet (catalogue: `elf hard`) | suggestion | Generic code compiled once per const parameter (`ObserveTable<N>`), unique constants that stop identical … |
| `SIZ-10` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | Size comments in Cargo.toml or code without a date and profile: they go stale and mislead |
| `FLT-1` | [`flt-1`](lints/flt-1.md) | error → `deny` | HardFault handler that spins (`loop {}`) in a shipped build: record (crash capture) and reset |
| `FLT-2` | [`flt-2`](lints/flt-2.md) | error → `deny` | defmt, `cnt!`, `unwrap` or anything that can panic or nest-log inside `HardFault` / `DefaultHandler`: … |
| `FLT-3` | [`flt-3`](lints/flt-3.md) | warning → `warn` | `bedrock_crash::exc_return!()` not the first statement of HardFault (LR changes) |
| `FLT-4` | [`flt-4`](lints/flt-4.md) | warning → `warn` | No UsageFault, MemManage and BusFault handlers (or not enabled in SHCSR) where the core has them |
| `FLT-5` | [`flt-5`](lints/flt-5.md) | warning → `warn` | Reset cause not read and reported at boot (power-on, brown-out, watchdog, software, pin, lockup) |
| `FLT-6` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Crash record or RAM handoff read after a power-on or brown-out (RAM content undefined, ECC RAM faults) or … |
| `FLT-7` | [`flt-7`](lints/flt-7.md) | error → `deny` | No watchdog, or a watchdog fed from an interrupt or from a loop not bounded by a timeout; window not … |
| `FLT-8` | [`flt-8`](lints/flt-8.md) | warning → `warn` | No boot-loop rescue: count boots without a healthy mark in a backup register, fall back to the ROM … |
| `FLT-9` | [`flt-9`](lints/flt-9.md) | warning → `warn` | Healthy mark never set on the transport in use (`usb_configured()` not called with USB transport): every … |
| `FLT-10` | not ported: checked on hardware | error | Forced HardFault and panic not tried on hardware: the next boot reports them (`last reset: Fault / Panic`) … |
| `FLT-11` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | `exc_return` relied on for crash capture on Cortex-M0+ (reads 0 there) |
| `FLT-12` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Runaway conditions not handled (AN1015): control registers of critical peripherals never re-written or … |
| `FLT-13` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Critical data used without a validity check (CRC, redundant copy, range) |
| `FLT-14` | not ported: checked on hardware | error | Reset-cause flags read as fresh when a core-only reset leaves them: RP2040 / RP2350 `CHIP_RESET` keeps … |
| `BOOT-1` | [`boot-1`](lints/boot-1.md) | error → `deny` | Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected: VTOR, PLL, SysTick, enabled IRQs of … |
| `BOOT-2` | [`boot-2`](lints/boot-2.md) | error → `deny` | H7 supply config written blind: an old ROM bootloader (before V9.3) locks PWR_CR3 until a power cycle; … |
| `BOOT-3` | not ported: a review check (a person or an agent reads the code or the datasheet) | error | Control signal on a pin the ROM bootloader drives (AN2606: USART, USB, CAN, I2C pins of the ROM's interfaces) |
| `BOOT-4` | not ported: a review check (a person or an agent reads the code or the datasheet) | error | No probe-less recovery path: ROM bootloader (BOOT pin, `boot reboot --updater`) or a DFU in the firmware … |
| `BOOT-5` | not ported: a test the firmware should have | error | Bootloader state written in an order a power cut can tear (embassy-boot STATE erase before zeroing the … |
| `BOOT-6` | not ported: a test the firmware should have | warning | A confirm or mark write that rewrites state when nothing changed |
| `BOOT-7` | not ported: checked on hardware | error | Update not tried from the previous released version, over the field path (no probe) |
| `BOOT-8` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid (API snapshots)`) | warning | Wire format of the update path changed without a version bump: an older firmware must still understand … |
| `BOOT-9` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid`) | warning | Dual-bank bootloader passed `bank1_region` only while the DFU region sits in bank 2 |
| `BOOT-10` | [`boot-10`](lints/boot-10.md) | warning → `warn` | Backup-domain helper that turns the RTC clock or `PWR_CR1.DBP` off afterwards instead of restoring what it … |
| `PWR-1` | [`pwr-1`](lints/pwr-1.md) | warning → `warn` | No brown-out reset level or programmable voltage detector (PVD) for the supply the board really has |
| `PWR-2` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid + review`) | error | H7 SMPS / LDO supply config not the one of the schematic: the MCU may not start or may be damaged |
| `PWR-3` | [`pwr-3`](lints/pwr-3.md) | warning → `warn` | Backup domain not reset at boot when the RTC and backup registers are unused (corrupted after a brown-out … |
| `PWR-4` | [`pwr-4`](lints/pwr-4.md) | warning → `warn` | RTC clock source changed in the clock config of a firmware that keeps state in the backup domain: embassy … |
| `PWR-5` | not ported: checked on hardware | warning | How long the MCU runs after a supply cut not measured where it matters (writes, erases): B129 G0 keeps … |
| `FLS-1` | [`fls-1`](lints/fls-1.md) | error → `deny` | Flash that a power cut can tear (settings, bootloader state) read raw on an ECC part: a double-ECC word is … |
| `FLS-2` | [`fls-2`](lints/fls-2.md) | error → `deny` | More than one owner of the flash peripheral (updater and settings each take `FLASH`) |
| `FLS-3` | [`fls-3`](lints/fls-3.md) | error → `deny` | RP flash erase or write from core 1, or with core 1 started outside `spawn_core1`, or while DMA reads … |
| `FLS-4` | [`fls-4`](lints/fls-4.md) | warning → `warn` | No periodic CRC of the image in flash, no ECC on flash and RAM where the part has it |
| `FLS-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | Slow erase: check the programming parallelism (higher peak current) |
| `FLS-6` | not ported: a test the firmware should have | warning | Power-fail behaviour of the flash store not tested with cuts during erase and write on the family in use … |
| `FLS-7` | not ported: bedrock has no automatic check for it yet (catalogue: `elf easy`) | warning | Unused program memory not filled with a trap (or protected) |
| `PIN-1` | [`pin-1`](lints/pin-1.md) | error → `deny` | Pin driver dropped (`let _ = Output::new(..)`, a temporary, end of `main`'s scope): embassy disconnects … |
| `PIN-2` | [`pin-2`](lints/pin-2.md) | error → `deny` | Board not put in its safe state first thing after the HAL init (output enables, shift-register chains, … |
| `PIN-3` | not ported: a test the firmware should have | error | A level translator or bus switch enabled before the MCU side is ready (pins released, at the right level, … |
| `PIN-4` | [`pin-4`](lints/pin-4.md) | error → `deny` | Peripheral pins moved to GPIO and back under an enabled peripheral (RP2350: the peripheral reads the OR of … |
| `PIN-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Input relies on an external pull the BOM may not fit (DNP), with the pad pull off |
| `PIN-6` | [`pin-6`](lints/pin-6.md) | error → `deny` | Two owners of a pin (a diagnostic or test API and a driver task): one owner per pin, the other refused |
| `PIN-7` | not ported: a test the firmware should have | error | Direction pin of a level translator turned while its IO pin drives: stop driving first |
| `PIN-8` | [`pin-8`](lints/pin-8.md) | warning → `warn` | Unused pins left floating inputs: analog (STM32) or a defined pull |
| `PIN-9` | [`pin-9`](lints/pin-9.md) | error → `deny` | Pin map chosen by a cargo feature for a board variant: a build for one variant flashed onto another drives … |
| `PIN-10` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Lines driven onto a device that may be unpowered (back-feed), and not released to Hi-Z on every session … |
| `PIN-11` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Hardware feature used without checking the board record says the rework or variant is there |
| `PIN-12` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Pin direction set only by an optional protocol command instead of on every path that uses it |
| `PIN-13` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A pin that must hold across an MCU reset driven by a GPIO (goes to reset state): use a backup-domain … |
| `PIN-14` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Input sampled once: filter by sampling several times where noise can fake an edge |
| `PIN-15` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | IO speed, drive and load against the frequency used (edge rates, EMC) |
| `PIN-16` | [`pin-16`](lints/pin-16.md) | warning → `warn` | Pin read that a dead-battery pull-down still holds (G0 UCPD DBCC pins until the SYSCFG strobe; never clear … |
| `CLK-1` | not ported: checked on hardware | warning | Core clock not measured once (DWT cycles against the time driver): prescaler math goes wrong silently (H7 … |
| `CLK-2` | [`clk-2`](lints/clk-2.md) | error → `deny` | embassy time driver `time-driver-any`, or the named timer also used for PWM or capture |
| `CLK-3` | [`clk-3`](lints/clk-3.md) | warning → `warn` | No clock security: CSS on HSE (and LSE) with a handler |
| `CLK-4` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | LSE start-up (seconds on H7) blocks boot or a feature |
| `CLK-5` | [`clk-5`](lints/clk-5.md) | error → `deny` | USB enabled on a part without HSI48 / CRS and no exact 48 MHz source configured |
| `CLK-6` | not ported: a test the firmware should have | warning | A kernel clock that can't divide the rates needed exactly (FDCAN 135 MHz for 2/4/8 Mbit/s, USART OVER8 … |
| `CLK-7` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Clock run at the part's maximum just because it can (lifetime at small nodes, ECC RAM limits: H7 TCM ECC … |
| `CLK-8` | not ported: a test the firmware should have | warning | Fractional PLL math not checked: embassy refuses a core above the limit at boot, a FRACN at 0 is integer … |
| `CLK-10` | not ported: bedrock has no automatic check for it yet (catalogue: `src mid`) | warning | FRACN changed on the fly with PLLxFRACEN cleared and set back to back: the new value is sometimes not … |
| `ERR-1` | not ported: a review check (a person or an agent reads the code or the datasheet) | error | Errata sheet for the silicon revision on the board not read for the peripherals, clocks, flash, DMA, low … |
| `ERR-2` | [`err-2`](lints/err-2.md) | error → `deny` | Register poke for an erratum without the erratum quoted (number, title, revisions, workaround) next to it |
| `ERR-3` | [`err-3`](lints/err-3.md) | error → `deny` | G0B0/G0B1/G0C1 flash prefetch on while code crosses into bank 2 (ES0548 2.2.10, ES0547 2.2.8: HardFault on … |
| `ERR-4` | [`err-4`](lints/err-4.md) | error → `deny` | FDCAN edge filtering (`CCCR.EFBI`) on, or dedicated Tx buffers beside the Tx FIFO (ES0548 2.15.1/2.15.2, … |
| `ERR-5` | [`err-5`](lints/err-5.md) | warning → `warn` | FDCAN DAR (one-shot) without retransmit after a lost arbitration on the first two ID bits (ES0491 2.22.3) |
| `ERR-6` | [`err-6`](lints/err-6.md) | warning → `warn` | RP2350 A2 pad with pull-down and input buffer left on (RP2350-E9: the pin stays high): enable the input … |
| `ERR-7` | [`err-7`](lints/err-7.md) | warning → `warn` | RP2350 ROM reboot with delay 0 or FRCE_OFF bits beyond PROC1 (RP2350-E30, E19) |
| `ERR-8` | [`err-8`](lints/err-8.md) | warning → `warn` | G0 ADC calibration without the wait of 2 ADC kernel clocks after disable (freezes) |
| `ERR-9` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | I2C timing not checked against the kernel / APB clock ratio errata (ES0548 2.11.1, 2.11.4) |
| `ERR-10` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Cache on without reading the cache errata of the part |
| `ERR-11` | [`err-11`](lints/err-11.md) | suggestion → `warn` | Silicon revision read from a register that can't tell all revisions apart (RP2350 `CHIP_ID.REVISION`: A3 … |
| `EXT-1` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | An I2C NAK taken as "device lost": chips NAK while busy; use a positive signature (ID reads all-zero … |
| `EXT-2` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | External chip on a rail that can sag without an MCU reset, and no watchdog that notices it lost its config … |
| `EXT-3` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A configurable chip identified by its ID registers before it is configured (no factory default): use a strap |
| `EXT-4` | [`ext-4`](lints/ext-4.md) | warning → `warn` | Read-modify-write of an external chip's registers over the bus, clear-on-read registers read more than … |
| `EXT-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A sense line assumed wired (PHY VBUS pin, card detect polarity): read the board and the connector datasheet |
| `EXT-6` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A settle or start-up time skipped on one path (VBUS after a hard reset, reset to first command, power-on … |
| `EXT-7` | [`ext-7`](lints/ext-7.md) | warning → `warn` | Register field encoded without a range check (`x - 1` underflows, masked to 0) |
| `EXT-8` | not ported: a test the firmware should have | warning | Config value set but never written to the hardware by the apply path, or not stored and re-applied stale … |
| `EXT-9` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Shared peripheral memory (H7 FDCAN message RAM) zeroed whole by one instance's init |
| `EXT-10` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Register map or masks of another IP variant (G0 FDCAN with the H7 map, reserved bits in masks) |
| `EXT-11` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | A HAL driver taken without checking it against what the product needs (timing accuracy, NAK index, … |
| `EXT-12` | [`ext-12`](lints/ext-12.md) | warning → `warn` | RCC enable not read back before the first access to the peripheral |
| `USB-1` | [`usb-1`](lints/usb-1.md) | error → `deny` | High-speed bulk endpoint max packet not 512, bInterval out of range for the speed |
| `USB-2` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A transfer that is an exact multiple of the max packet without a ZLP, or framing that waits for a short … |
| `USB-3` | [`usb-3`](lints/usb-3.md) | suggestion → `warn` | Single-buffered bulk endpoints on a hot path behind a hub TT (G0 usb_v4: 500 vs 947 kB/s) |
| `USB-4` | not ported: checked on hardware | warning | Sending stuck when the host driver closes or the host dies without a disconnect |
| `USB-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | OUT packet read from USB SRAM within 800 ns of its CTR on a part with the "buffer description table update … |
| `HOST-1` | [`host-1`](lints/host-1.md) | error → `deny` | Panic reachable from host input: `unwrap`, `expect`, slice indexing, `unreachable!`, arithmetic overflow … |
| `HOST-2` | not ported: a review check (a person or an agent reads the code or the datasheet) | error | Host can drive a pin into a level or mode that damages the board: every electrical rule enforced by the … |
| `HOST-3` | [`host-3`](lints/host-3.md) | warning → `warn` | Unknown enum value from a newer host mapped to a default (`_ => ..`) instead of refused |
| `HOST-4` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A host that stops reading can stall a release or cleanup path: every wait on the host has a timeout |
| `HOST-5` | [`host-5`](lints/host-5.md) | warning → `warn` | Truncated or oversized request not counted and dropped |
| `HOST-6` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A host-settable range wider than the firmware can serve (bounded by DMA geometry, buffers): limits derived … |
| `HOST-7` | not ported: a test the firmware should have | warning | One unimplemented field voids a batched update; a deferred answer path (credit, grant) only sent from … |
| `HOST-8` | not ported: a test the firmware should have | warning | Loss reported out of order with the data it belongs to |
| `LOG-1` | [`log-1`](lints/log-1.md) | error → `deny` | RTT that can block: probe-rs sets the channel to blocking on attach; force NoBlockSkip, whole frames or … |
| `LOG-2` | [`log-2`](lints/log-2.md) | warning → `warn` | `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` not set explicitly in `.cargo/config.toml` (init bursts lost: 773 … |
| `LOG-3` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | Hot-path logging at info or above |
| `CNT-1` | [`cnt-1`](lints/cnt-1.md) | warning → `warn` | Error branch (an `Err` handled, a timeout, a retry, a dropped frame, an overrun) without a `cnt!` counter: … |
| `CNT-2` | [`cnt-2`](lints/cnt-2.md) | warning → `warn` | `&mut` to a counter buffer that an interrupt can update, unaligned u64 access |
| `CNT-3` | [`cnt-3`](lints/cnt-3.md) | warning → `warn` | Proc-macro crate and its runtime crate not pinned to the exact same version (`=x.y.z`) |
| `CFG-1` | [`cfg-1`](lints/cfg-1.md) | error → `deny` | probe-rs runner or a recipe without `--probe VID:PID`: with two probes on the PC another board gets flashed |
| `CFG-2` | [`cfg-2`](lints/cfg-2.md) | warning → `warn` | `probe-rs download` then `reset` on F0/G0/L0: probe-rs writes RCC_APBENR1 back on a running core (USB … |
| `CFG-3` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg easy`) | warning | Feature sets not all built and linted (default, `--all-features`, `--no-default-features`): gated code rots |
| `CFG-4` | [`cfg-4`](lints/cfg-4.md) | error → `deny` | A crate in the tree twice (git and path copies of WireWeaver, embassy): patch it to one source |
| `CFG-5` | [`cfg-5`](lints/cfg-5.md) | error → `deny` | Chip feature (package, flash size) of the HAL not the part on the board |
| `CFG-6` | [`cfg-6`](lints/cfg-6.md) | warning → `warn` | thumbv6m without `portable-atomic` + `critical-section` where a crate needs CAS (`static_cell`) |
| `CFG-7` | [`cfg-7`](lints/cfg-7.md) | warning → `warn` | Dirty build flashed without a mark: build info carries the git SHA and a dirty flag, read back from the device |
| `CFG-8` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg mid`) | warning | Git consumers not built: a dependency from git (not path) breaks where path builds pass |
| `CFG-9` | not ported: bedrock has no automatic check for it yet (catalogue: `cfg easy`) | warning | Committed path dependency or `[patch]` to a sibling checkout (`../wire_weaver`): the lock follows each … |
| `PERF-1` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Speed or size claims without numbers at the opt-level that ships, in their own units with the clock … |
| `PERF-2` | [`perf-2`](lints/perf-2.md) | suggestion → `warn` | 64-bit or i128 maths, float formatting, or `%` / `/` by a runtime value per element on a hot path of a … |
| `PERF-3` | [`perf-3`](lints/perf-3.md) | warning → `warn` | RAM function (`#[link_section = ".data.*"]`) without `#[inline(never)]`: LTO inlines it back into flash |
| `PERF-4` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | Hot path through a bitwise CRC, a bit-by-bit serializer, or GPIO wrappers per edge (embassy `Flex` ~50 … |
| `PERF-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | Wake cost of an event loop with nothing to do (`select4` with timers and waker registrations per turn on M0+) |
| `TEST-1` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | Host test that races a free-running tick or sleeps "long enough" |
| `TEST-2` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A simulator that answers inline hides bugs of the deferred path: test both |
| `TEST-3` | not ported: a test the firmware should have | warning | Sequencing rules of the board (enable order, safe off) without host tests of the transition model |
| `TEST-4` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A board left running its last test after probe-rs exits: reset before each run |
| `TEST-5` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | One probe, one owner: another session's probe-rs or gdb on the same board makes faults that aren't the … |
| `TEST-6` | not ported: a review check (a person or an agent reads the code or the datasheet) | suggestion | A fault that depends on timing or layout bisected before the errata were checked |
| `TEST-7` | not ported: a review check (a person or an agent reads the code or the datasheet) | warning | A peripheral or chip blamed before our own side was measured (M1 blamed the sink's FUSB302B for our RXDR … |
