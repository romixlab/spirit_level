# embedded

Bare-metal firmware: executors, interrupts, DMA, stack, memory, size, faults, boot, power, flash, pins, clocks, errata, USB, host input, logging, counters, build configuration.

Use it with `extends = "embedded"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Lint | Level | Reason | Set in |
|---|---|---|---|
| [boot-1](../lints/boot-1.md) | `deny` | bedrock error: Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected: VTOR, PLL, SysTick, enabled IRQs of the ROM are still set; detect (RCC_CFGR SWS, VTOR, NVIC) and reset once | embedded |
| [boot-10](../lints/boot-10.md) | `warn` | bedrock warning: Backup-domain helper that turns the RTC clock or `PWR_CR1.DBP` off afterwards instead of restoring what it found | embedded |
| [boot-2](../lints/boot-2.md) | `deny` | bedrock error: H7 supply config written blind: an old ROM bootloader (before V9.3) locks PWR_CR3 until a power cycle; read it back, use what it holds, count a mismatch | embedded |
| [cfg-1](../lints/cfg-1.md) | `deny` | bedrock error: probe-rs runner or a recipe without `--probe VID:PID`: with two probes on the PC another board gets flashed | embedded |
| [cfg-2](../lints/cfg-2.md) | `warn` | bedrock warning: `probe-rs download` then `reset` on F0/G0/L0: probe-rs writes RCC_APBENR1 back on a running core (USB clock off); use a runner that halts first | embedded |
| [cfg-4](../lints/cfg-4.md) | `deny` | bedrock error: A crate in the tree twice (git and path copies of WireWeaver, embassy): patch it to one source | embedded |
| [cfg-5](../lints/cfg-5.md) | `deny` | bedrock error: Chip feature (package, flash size) of the HAL not the part on the board | embedded |
| [cfg-6](../lints/cfg-6.md) | `warn` | bedrock warning: thumbv6m without `portable-atomic` + `critical-section` where a crate needs CAS (`static_cell`) | embedded |
| [cfg-7](../lints/cfg-7.md) | `warn` | bedrock warning: Dirty build flashed without a mark: build info carries the git SHA and a dirty flag, read back from the device | embedded |
| [clk-2](../lints/clk-2.md) | `deny` | bedrock error: embassy time driver `time-driver-any`, or the named timer also used for PWM or capture | embedded |
| [clk-3](../lints/clk-3.md) | `warn` | bedrock warning: No clock security: CSS on HSE (and LSE) with a handler | embedded |
| [clk-5](../lints/clk-5.md) | `deny` | bedrock error: USB enabled on a part without HSI48 / CRS and no exact 48 MHz source configured | embedded |
| [cnt-1](../lints/cnt-1.md) | `warn` | bedrock warning: Error branch (an `Err` handled, a timeout, a retry, a dropped frame, an overrun) without a `cnt!` counter: unexpected events must be counted | embedded |
| [cnt-2](../lints/cnt-2.md) | `warn` | bedrock warning: `&mut` to a counter buffer that an interrupt can update, unaligned u64 access | embedded |
| [cnt-3](../lints/cnt-3.md) | `warn` | bedrock warning: Proc-macro crate and its runtime crate not pinned to the exact same version (`=x.y.z`) | embedded |
| [dma-1](../lints/dma-1.md) | `deny` | bedrock error: DMA buffer in memory the DMA master can't reach (H7 DTCM for IDMA, SDMMC, USB OTG; flash for some masters): it hangs or reads zeros | embedded |
| [dma-2](../lints/dma-2.md) | `deny` | bedrock error: D-cache on with DMA buffers in cacheable RAM and no clean / invalidate or MPU non-cacheable region | embedded |
| [dma-4](../lints/dma-4.md) | `warn` | bedrock warning: DMA ring overrun not counted: every ring consumer counts overruns with `cnt!` | embedded |
| [dma-6](../lints/dma-6.md) | `warn` | bedrock warning: Erratum asks for DMA where the code polls (G0 UCPD TX underflow then TXHRST corrupts the next CRC: ES0548 2.17.1) | embedded |
| [err-11](../lints/err-11.md) | `warn` | bedrock suggestion, reported without failing the run: Silicon revision read from a register that can't tell all revisions apart (RP2350 `CHIP_ID.REVISION`: A3 and A4 both 3; use the bootrom version) | embedded |
| [err-2](../lints/err-2.md) | `deny` | bedrock error: Register poke for an erratum without the erratum quoted (number, title, revisions, workaround) next to it | embedded |
| [err-3](../lints/err-3.md) | `deny` | bedrock error: G0B0/G0B1/G0C1 flash prefetch on while code crosses into bank 2 (ES0548 2.2.10, ES0547 2.2.8: HardFault on cross-bank branches) | embedded |
| [err-4](../lints/err-4.md) | `deny` | bedrock error: FDCAN edge filtering (`CCCR.EFBI`) on, or dedicated Tx buffers beside the Tx FIFO (ES0548 2.15.1/2.15.2, ES0491 2.22.1/2.22.2) | embedded |
| [err-5](../lints/err-5.md) | `warn` | bedrock warning: FDCAN DAR (one-shot) without retransmit after a lost arbitration on the first two ID bits (ES0491 2.22.3) | embedded |
| [err-6](../lints/err-6.md) | `warn` | bedrock warning: RP2350 A2 pad with pull-down and input buffer left on (RP2350-E9: the pin stays high): enable the input only for the read | embedded |
| [err-7](../lints/err-7.md) | `warn` | bedrock warning: RP2350 ROM reboot with delay 0 or FRCE_OFF bits beyond PROC1 (RP2350-E30, E19) | embedded |
| [err-8](../lints/err-8.md) | `warn` | bedrock warning: G0 ADC calibration without the wait of 2 ADC kernel clocks after disable (freezes) | embedded |
| [exe-1](../lints/exe-1.md) | `deny` | bedrock error: A task with a hard answer deadline (a PHY that arms RX DMA only inside `receive().await`, a software GoodCRC, a DMA ring consumer) runs on the thread executor beside bulk work (USB server, logging) | embedded |
| [exe-10](../lints/exe-10.md) | `deny` | bedrock error: Two crates provide the executor and `__pender` (embassy-rp or embassy-stm32 `executor-*` features and embassy-executor `platform-cortex-m` together) | embedded |
| [exe-12](../lints/exe-12.md) | `warn` | bedrock suggestion, reported without failing the run: `async fn` that never awaits (handlers especially): each one is a state in the dispatcher future and costs flash | embedded |
| [exe-13](../lints/exe-13.md) | `warn` | bedrock warning: Ring-buffered reader (`RingBufferedUartRx::read`) in a loop without batching: a busy line gives 1-byte reads and 100 % CPU; wait about 1 ms after a short read | embedded |
| [exe-2](../lints/exe-2.md) | `deny` | bedrock error: Blocking call (`block_for`, `blocking_*` HAL calls, `nb::block!`, busy-wait loops, `embassy_futures::block_on`) in a task on an interrupt executor or in an interrupt handler | embedded |
| [exe-3](../lints/exe-3.md) | `deny` | bedrock error: Interrupt executor started before its IRQ priority is set: `start()` unmasks the IRQ at the default (highest) priority | embedded |
| [exe-4](../lints/exe-4.md) | `warn` | bedrock warning: Interrupt executor priority not below the peripheral, DMA and time-driver interrupts that wake its tasks (it must preempt thread mode, never them) | embedded |
| [exe-5](../lints/exe-5.md) | `deny` | bedrock error: Interrupt executor on a vector that a driver also binds (`bind_interrupts!`) or that a used peripheral raises: use the vector of a peripheral that is unused, and say so in a comment | embedded |
| [exe-6](../lints/exe-6.md) | `deny` | bedrock error: State shared between a task on an interrupt executor and thread-mode code behind `ThreadModeRawMutex` / `NoopRawMutex` or a `static mut`: use `CriticalSectionRawMutex` | embedded |
| [exe-7](../lints/exe-7.md) | `warn` | bedrock warning: Loop in a shared task that works until a queue is empty (or logs in a loop) without a bound and a yield: above the input rate it never ends and starves the executor | embedded |
| [exe-8](../lints/exe-8.md) | `warn` | bedrock warning: Request server waits on a bus or a slow peripheral inside a handler: defer the transfer to the task that owns the bus and answer from the server loop | embedded |
| [exe-9](../lints/exe-9.md) | `deny` | bedrock error: Future polled with a no-op waker (`poll_once`, `noop_waker`) that can reach `embassy_time`: embassy-time panics "Found waker not created by the Embassy executor" | embedded |
| [ext-12](../lints/ext-12.md) | `warn` | bedrock warning: RCC enable not read back before the first access to the peripheral | embedded |
| [ext-4](../lints/ext-4.md) | `warn` | bedrock warning: Read-modify-write of an external chip's registers over the bus, clear-on-read registers read more than once: write from shadow copies | embedded |
| [ext-7](../lints/ext-7.md) | `warn` | bedrock warning: Register field encoded without a range check (`x - 1` underflows, masked to 0) | embedded |
| [firmware-readable](../lints/firmware-readable.md) | `deny` | a firmware file the embedded lints cannot read hides every finding in it | embedded |
| [fls-1](../lints/fls-1.md) | `deny` | bedrock error: Flash that a power cut can tear (settings, bootloader state) read raw on an ECC part: a double-ECC word is an NMI (G0) or BusFault (H7) on every boot; read through `ww_ecc_flash` with the NMI hook | embedded |
| [fls-2](../lints/fls-2.md) | `deny` | bedrock error: More than one owner of the flash peripheral (updater and settings each take `FLASH`) | embedded |
| [fls-3](../lints/fls-3.md) | `deny` | bedrock error: RP flash erase or write from core 1, or with core 1 started outside `spawn_core1`, or while DMA reads flash (XIP stalls) | embedded |
| [fls-4](../lints/fls-4.md) | `warn` | bedrock warning: No periodic CRC of the image in flash, no ECC on flash and RAM where the part has it | embedded |
| [flt-1](../lints/flt-1.md) | `deny` | bedrock error: HardFault handler that spins (`loop {}`) in a shipped build: record (crash capture) and reset | embedded |
| [flt-2](../lints/flt-2.md) | `deny` | bedrock error: defmt, `cnt!`, `unwrap` or anything that can panic or nest-log inside `HardFault` / `DefaultHandler`: record the IRQ number raw instead | embedded |
| [flt-3](../lints/flt-3.md) | `warn` | bedrock warning: `bedrock_crash::exc_return!()` not the first statement of HardFault (LR changes) | embedded |
| [flt-4](../lints/flt-4.md) | `warn` | bedrock warning: No UsageFault, MemManage and BusFault handlers (or not enabled in SHCSR) where the core has them | embedded |
| [flt-5](../lints/flt-5.md) | `warn` | bedrock warning: Reset cause not read and reported at boot (power-on, brown-out, watchdog, software, pin, lockup) | embedded |
| [flt-7](../lints/flt-7.md) | `deny` | bedrock error: No watchdog, or a watchdog fed from an interrupt or from a loop not bounded by a timeout; window not calibrated to the real period | embedded |
| [flt-8](../lints/flt-8.md) | `warn` | bedrock warning: No boot-loop rescue: count boots without a healthy mark in a backup register, fall back to the ROM bootloader after a few | embedded |
| [flt-9](../lints/flt-9.md) | `warn` | bedrock warning: Healthy mark never set on the transport in use (`usb_configured()` not called with USB transport): every boot waits the full timeout | embedded |
| [host-1](../lints/host-1.md) | `deny` | bedrock error: Panic reachable from host input: `unwrap`, `expect`, slice indexing, `unreachable!`, arithmetic overflow on values from a request (pin numbers, lengths, rates) | embedded |
| [host-3](../lints/host-3.md) | `warn` | bedrock warning: Unknown enum value from a newer host mapped to a default (`_ => ..`) instead of refused | embedded |
| [host-5](../lints/host-5.md) | `warn` | bedrock warning: Truncated or oversized request not counted and dropped | embedded |
| [irq-1](../lints/irq-1.md) | `deny` | bedrock error: Interrupt flags cleared wholesale (all-ones or the read value written back to a W1C register): an event between the read and the clear is lost; clear exactly the flags handled | embedded |
| [irq-2](../lints/irq-2.md) | `deny` | bedrock error: `rc_w1` / W1C flags cleared with `modify()` (read-modify-write) instead of a write of the one bit: it clears flags that were set meanwhile | embedded |
| [irq-4](../lints/irq-4.md) | `deny` | bedrock error: Several drivers on one shared NVIC vector, and a driver that disables "its" interrupt on drop: the others go deaf (G0B1 `USB_UCPD1_2`) | embedded |
| [irq-5](../lints/irq-5.md) | `warn` | bedrock warning: Interrupt line routed (ILS) but not enabled (ILE), or enabled without a handler bound | embedded |
| [irq-6](../lints/irq-6.md) | `warn` | bedrock warning: Nested disable / enable of interrupts by hand: an inner enable turns them on under an outer section; use `critical_section::with` | embedded |
| [irq-8](../lints/irq-8.md) | `warn` | bedrock warning: Unused interrupt vectors not caught: `DefaultHandler` must record the IRQ number (raw: a static, the crash record or a counter; not logging it, FLT-2) | embedded |
| [log-1](../lints/log-1.md) | `deny` | bedrock error: RTT that can block: probe-rs sets the channel to blocking on attach; force NoBlockSkip, whole frames or skipped and counted | embedded |
| [log-2](../lints/log-2.md) | `warn` | bedrock warning: `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` not set explicitly in `.cargo/config.toml` (init bursts lost: 773 of 1000 lines) | embedded |
| [mem-1](../lints/mem-1.md) | `deny` | bedrock error: `FLASH` length in memory.x covers a region kept for something else (settings, bootloader state, DFU): the image can grow into it | embedded |
| [mem-10](../lints/mem-10.md) | `warn` | bedrock warning: RWX LOAD segment or a writable symbol in `.rodata` (`static mut` in a read-only section) | embedded |
| [mem-12](../lints/mem-12.md) | `deny` | bedrock error: defmt in the dependencies without `-Tdefmt.x` (no `.defmt` section) | embedded |
| [mem-2](../lints/mem-2.md) | `warn` | bedrock warning: Extra RAM section with a tail `. = ALIGN(n)` or a symbol in it: lld keeps an empty section that flip-link counts as used | embedded |
| [mem-3](../lints/mem-3.md) | `deny` | bedrock error: RAM init or zeroing range overlaps `.uninit` (crash record, handoff data) | embedded |
| [mem-6](../lints/mem-6.md) | `warn` | bedrock warning: `.cnt_bkp_buffer` not at the backup-register address | embedded |
| [mem-7](../lints/mem-7.md) | `warn` | bedrock warning: Flash partition symbols for embassy-boot written as absolute addresses where offsets are expected (`- ORIGIN(FLASH)`) | embedded |
| [perf-2](../lints/perf-2.md) | `warn` | bedrock suggestion, reported without failing the run: 64-bit or i128 maths, float formatting, or `%` / `/` by a runtime value per element on a hot path of a core without them (`__aeabi_lmul`, `__aeabi_uldivmod`, `__multi3`) | embedded |
| [perf-3](../lints/perf-3.md) | `warn` | bedrock warning: RAM function (`#[link_section = ".data.*"]`) without `#[inline(never)]`: LTO inlines it back into flash | embedded |
| [pin-1](../lints/pin-1.md) | `deny` | bedrock error: Pin driver dropped (`let _ = Output::new(..)`, a temporary, end of `main`'s scope): embassy disconnects the pin on drop | embedded |
| [pin-16](../lints/pin-16.md) | `warn` | bedrock warning: Pin read that a dead-battery pull-down still holds (G0 UCPD DBCC pins until the SYSCFG strobe; never clear the strobe again) | embedded |
| [pin-2](../lints/pin-2.md) | `deny` | bedrock error: Board not put in its safe state first thing after the HAL init (output enables, shift-register chains, power switches), before USB or anything slow | embedded |
| [pin-4](../lints/pin-4.md) | `deny` | bedrock error: Peripheral pins moved to GPIO and back under an enabled peripheral (RP2350: the peripheral reads the OR of all pins with its FUNCSEL; the I2C block saw a start without a stop): keep FUNCSEL, use OEOVER / OUTOVER | embedded |
| [pin-6](../lints/pin-6.md) | `deny` | bedrock error: Two owners of a pin (a diagnostic or test API and a driver task): one owner per pin, the other refused | embedded |
| [pin-8](../lints/pin-8.md) | `warn` | bedrock warning: Unused pins left floating inputs: analog (STM32) or a defined pull | embedded |
| [pin-9](../lints/pin-9.md) | `deny` | bedrock error: Pin map chosen by a cargo feature for a board variant: a build for one variant flashed onto another drives the wrong lines; read the board record at run time, default to inputs | embedded |
| [pwr-1](../lints/pwr-1.md) | `warn` | bedrock warning: No brown-out reset level or programmable voltage detector (PVD) for the supply the board really has | embedded |
| [pwr-3](../lints/pwr-3.md) | `warn` | bedrock warning: Backup domain not reset at boot when the RTC and backup registers are unused (corrupted after a brown-out without POR) | embedded |
| [pwr-4](../lints/pwr-4.md) | `warn` | bedrock warning: RTC clock source changed in the clock config of a firmware that keeps state in the backup domain: embassy resets the domain | embedded |
| [siz-1](../lints/siz-1.md) | `warn` | bedrock warning: Flash or RAM use over a threshold (default 90 % of the region) in a shipped profile | embedded |
| [siz-3](../lints/siz-3.md) | `warn` | bedrock suggestion, reported without failing the run: Per-package `opt-level` together with `lto = "fat"`: the override does nothing | embedded |
| [siz-4](../lints/siz-4.md) | `warn` | bedrock suggestion, reported without failing the run: Release profile without `codegen-units = 1`, `lto`, `panic = "abort"`, and an `opt-level` chosen by measurement (z vs 3 is the largest lever: Donguru 501.7 → 321.8 KiB, 10-25 % slower USB) | embedded |
| [siz-5](../lints/siz-5.md) | `warn` | bedrock suggestion, reported without failing the run: `core::fmt` pulled in: `Result::unwrap` / `expect` (through `&dyn Debug`), `Debug2Format`, a panic handler that formats `PanicInfo` (4-5 KiB) | embedded |
| [siz-6](../lints/siz-6.md) | `warn` | bedrock suggestion, reported without failing the run: Large `.data` initializer (a static with a non-zero const value): zero-init and set it up in place | embedded |
| [stk-1](../lints/stk-1.md) | `deny` | bedrock error: flip-link not the linker: `linker = "flip-link"` under `[target.<triple>]` in `.cargo/config.toml` | embedded |
| [stk-2](../lints/stk-2.md) | `deny` | bedrock error: `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs: not a build-script directive, Cargo ignores it silently | embedded |
| [stk-3](../lints/stk-3.md) | `deny` | bedrock error: Stack not below `.data` / `.bss` in the ELF (`_stack_start` != start of `.data`), or a zero-size stack | embedded |
| [stk-4](../lints/stk-4.md) | `deny` | bedrock error: cortex-m-rt `paint-stack` on before its BusFault is understood | embedded |
| [stk-5](../lints/stk-5.md) | `warn` | bedrock warning: Big state built by value and moved into a static (`StaticCell::init(Big::new())`): the frame holds it whole; build it in place (`const fn new`, init into `MaybeUninit`) | embedded |
| [stk-6](../lints/stk-6.md) | `warn` | bedrock warning: Stack frame over a budget (per function) or a worst-case stack depth over the stack region | embedded |
| [usb-1](../lints/usb-1.md) | `deny` | bedrock error: High-speed bulk endpoint max packet not 512, bInterval out of range for the speed | embedded |
| [usb-3](../lints/usb-3.md) | `warn` | bedrock suggestion, reported without failing the run: Single-buffered bulk endpoints on a hot path behind a hub TT (G0 usb_v4: 500 vs 947 kB/s) | embedded |

## The file

```toml
# Beside the size tiers: a firmware extends ["small", "embedded"]. Every automatic check of embedded_bedrock's
# firmware check catalogue, at bedrock's severity: error is deny, warning is warn, suggestion is warn (reported,
# so the size and speed levers stay visible, but never failing the run on its own).
[tier]
description = "Bare-metal firmware: executors, interrupts, DMA, stack, memory, size, faults, boot, power, flash, pins, clocks, errata, USB, host input, logging, counters, build configuration"

[lints.firmware-readable]
level = "deny"
reason = "a firmware file the embedded lints cannot read hides every finding in it"

[lints.exe-1]
level = "deny"
reason = "bedrock error: A task with a hard answer deadline (a PHY that arms RX DMA only inside `receive().await`, a software GoodCRC, a DMA ring consumer) runs on the thread executor beside bulk work (USB server, logging)"

[lints.exe-2]
level = "deny"
reason = "bedrock error: Blocking call (`block_for`, `blocking_*` HAL calls, `nb::block!`, busy-wait loops, `embassy_futures::block_on`) in a task on an interrupt executor or in an interrupt handler"

[lints.exe-3]
level = "deny"
reason = "bedrock error: Interrupt executor started before its IRQ priority is set: `start()` unmasks the IRQ at the default (highest) priority"

[lints.exe-4]
level = "warn"
reason = "bedrock warning: Interrupt executor priority not below the peripheral, DMA and time-driver interrupts that wake its tasks (it must preempt thread mode, never them)"

[lints.exe-5]
level = "deny"
reason = "bedrock error: Interrupt executor on a vector that a driver also binds (`bind_interrupts!`) or that a used peripheral raises: use the vector of a peripheral that is unused, and say so in a comment"

[lints.exe-6]
level = "deny"
reason = "bedrock error: State shared between a task on an interrupt executor and thread-mode code behind `ThreadModeRawMutex` / `NoopRawMutex` or a `static mut`: use `CriticalSectionRawMutex`"

[lints.exe-7]
level = "warn"
reason = "bedrock warning: Loop in a shared task that works until a queue is empty (or logs in a loop) without a bound and a yield: above the input rate it never ends and starves the executor"

[lints.exe-8]
level = "warn"
reason = "bedrock warning: Request server waits on a bus or a slow peripheral inside a handler: defer the transfer to the task that owns the bus and answer from the server loop"

[lints.exe-9]
level = "deny"
reason = "bedrock error: Future polled with a no-op waker (`poll_once`, `noop_waker`) that can reach `embassy_time`: embassy-time panics \"Found waker not created by the Embassy executor\""

[lints.exe-10]
level = "deny"
reason = "bedrock error: Two crates provide the executor and `__pender` (embassy-rp or embassy-stm32 `executor-*` features and embassy-executor `platform-cortex-m` together)"

[lints.exe-12]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: `async fn` that never awaits (handlers especially): each one is a state in the dispatcher future and costs flash"

[lints.exe-13]
level = "warn"
reason = "bedrock warning: Ring-buffered reader (`RingBufferedUartRx::read`) in a loop without batching: a busy line gives 1-byte reads and 100 % CPU; wait about 1 ms after a short read"

[lints.irq-1]
level = "deny"
reason = "bedrock error: Interrupt flags cleared wholesale (all-ones or the read value written back to a W1C register): an event between the read and the clear is lost; clear exactly the flags handled"

[lints.irq-2]
level = "deny"
reason = "bedrock error: `rc_w1` / W1C flags cleared with `modify()` (read-modify-write) instead of a write of the one bit: it clears flags that were set meanwhile"

[lints.irq-4]
level = "deny"
reason = "bedrock error: Several drivers on one shared NVIC vector, and a driver that disables \"its\" interrupt on drop: the others go deaf (G0B1 `USB_UCPD1_2`)"

[lints.irq-5]
level = "warn"
reason = "bedrock warning: Interrupt line routed (ILS) but not enabled (ILE), or enabled without a handler bound"

[lints.irq-6]
level = "warn"
reason = "bedrock warning: Nested disable / enable of interrupts by hand: an inner enable turns them on under an outer section; use `critical_section::with`"

[lints.irq-8]
level = "warn"
reason = "bedrock warning: Unused interrupt vectors not caught: `DefaultHandler` must record the IRQ number (raw: a static, the crash record or a counter; not logging it, FLT-2)"

[lints.dma-1]
level = "deny"
reason = "bedrock error: DMA buffer in memory the DMA master can't reach (H7 DTCM for IDMA, SDMMC, USB OTG; flash for some masters): it hangs or reads zeros"

[lints.dma-2]
level = "deny"
reason = "bedrock error: D-cache on with DMA buffers in cacheable RAM and no clean / invalidate or MPU non-cacheable region"

[lints.dma-4]
level = "warn"
reason = "bedrock warning: DMA ring overrun not counted: every ring consumer counts overruns with `cnt!`"

[lints.dma-6]
level = "warn"
reason = "bedrock warning: Erratum asks for DMA where the code polls (G0 UCPD TX underflow then TXHRST corrupts the next CRC: ES0548 2.17.1)"

[lints.stk-1]
level = "deny"
reason = "bedrock error: flip-link not the linker: `linker = \"flip-link\"` under `[target.<triple>]` in `.cargo/config.toml`"

[lints.stk-2]
level = "deny"
reason = "bedrock error: `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs: not a build-script directive, Cargo ignores it silently"

[lints.stk-3]
level = "deny"
reason = "bedrock error: Stack not below `.data` / `.bss` in the ELF (`_stack_start` != start of `.data`), or a zero-size stack"

[lints.stk-4]
level = "deny"
reason = "bedrock error: cortex-m-rt `paint-stack` on before its BusFault is understood"

[lints.stk-5]
level = "warn"
reason = "bedrock warning: Big state built by value and moved into a static (`StaticCell::init(Big::new())`): the frame holds it whole; build it in place (`const fn new`, init into `MaybeUninit`)"

[lints.stk-6]
level = "warn"
reason = "bedrock warning: Stack frame over a budget (per function) or a worst-case stack depth over the stack region"

[lints.mem-1]
level = "deny"
reason = "bedrock error: `FLASH` length in memory.x covers a region kept for something else (settings, bootloader state, DFU): the image can grow into it"

[lints.mem-2]
level = "warn"
reason = "bedrock warning: Extra RAM section with a tail `. = ALIGN(n)` or a symbol in it: lld keeps an empty section that flip-link counts as used"

[lints.mem-3]
level = "deny"
reason = "bedrock error: RAM init or zeroing range overlaps `.uninit` (crash record, handoff data)"

[lints.mem-6]
level = "warn"
reason = "bedrock warning: `.cnt_bkp_buffer` not at the backup-register address"

[lints.mem-7]
level = "warn"
reason = "bedrock warning: Flash partition symbols for embassy-boot written as absolute addresses where offsets are expected (`- ORIGIN(FLASH)`)"

[lints.mem-10]
level = "warn"
reason = "bedrock warning: RWX LOAD segment or a writable symbol in `.rodata` (`static mut` in a read-only section)"

[lints.mem-12]
level = "deny"
reason = "bedrock error: defmt in the dependencies without `-Tdefmt.x` (no `.defmt` section)"

[lints.siz-1]
level = "warn"
reason = "bedrock warning: Flash or RAM use over a threshold (default 90 % of the region) in a shipped profile"

[lints.siz-3]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: Per-package `opt-level` together with `lto = \"fat\"`: the override does nothing"

[lints.siz-4]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: Release profile without `codegen-units = 1`, `lto`, `panic = \"abort\"`, and an `opt-level` chosen by measurement (z vs 3 is the largest lever: Donguru 501.7 → 321.8 KiB, 10-25 % slower USB)"

[lints.siz-5]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: `core::fmt` pulled in: `Result::unwrap` / `expect` (through `&dyn Debug`), `Debug2Format`, a panic handler that formats `PanicInfo` (4-5 KiB)"

[lints.siz-6]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: Large `.data` initializer (a static with a non-zero const value): zero-init and set it up in place"

[lints.flt-1]
level = "deny"
reason = "bedrock error: HardFault handler that spins (`loop {}`) in a shipped build: record (crash capture) and reset"

[lints.flt-2]
level = "deny"
reason = "bedrock error: defmt, `cnt!`, `unwrap` or anything that can panic or nest-log inside `HardFault` / `DefaultHandler`: record the IRQ number raw instead"

[lints.flt-3]
level = "warn"
reason = "bedrock warning: `bedrock_crash::exc_return!()` not the first statement of HardFault (LR changes)"

[lints.flt-4]
level = "warn"
reason = "bedrock warning: No UsageFault, MemManage and BusFault handlers (or not enabled in SHCSR) where the core has them"

[lints.flt-5]
level = "warn"
reason = "bedrock warning: Reset cause not read and reported at boot (power-on, brown-out, watchdog, software, pin, lockup)"

[lints.flt-7]
level = "deny"
reason = "bedrock error: No watchdog, or a watchdog fed from an interrupt or from a loop not bounded by a timeout; window not calibrated to the real period"

[lints.flt-8]
level = "warn"
reason = "bedrock warning: No boot-loop rescue: count boots without a healthy mark in a backup register, fall back to the ROM bootloader after a few"

[lints.flt-9]
level = "warn"
reason = "bedrock warning: Healthy mark never set on the transport in use (`usb_configured()` not called with USB transport): every boot waits the full timeout"

[lints.boot-1]
level = "deny"
reason = "bedrock error: Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected: VTOR, PLL, SysTick, enabled IRQs of the ROM are still set; detect (RCC_CFGR SWS, VTOR, NVIC) and reset once"

[lints.boot-2]
level = "deny"
reason = "bedrock error: H7 supply config written blind: an old ROM bootloader (before V9.3) locks PWR_CR3 until a power cycle; read it back, use what it holds, count a mismatch"

[lints.boot-10]
level = "warn"
reason = "bedrock warning: Backup-domain helper that turns the RTC clock or `PWR_CR1.DBP` off afterwards instead of restoring what it found"

[lints.pwr-1]
level = "warn"
reason = "bedrock warning: No brown-out reset level or programmable voltage detector (PVD) for the supply the board really has"

[lints.pwr-3]
level = "warn"
reason = "bedrock warning: Backup domain not reset at boot when the RTC and backup registers are unused (corrupted after a brown-out without POR)"

[lints.pwr-4]
level = "warn"
reason = "bedrock warning: RTC clock source changed in the clock config of a firmware that keeps state in the backup domain: embassy resets the domain"

[lints.fls-1]
level = "deny"
reason = "bedrock error: Flash that a power cut can tear (settings, bootloader state) read raw on an ECC part: a double-ECC word is an NMI (G0) or BusFault (H7) on every boot; read through `ww_ecc_flash` with the NMI hook"

[lints.fls-2]
level = "deny"
reason = "bedrock error: More than one owner of the flash peripheral (updater and settings each take `FLASH`)"

[lints.fls-3]
level = "deny"
reason = "bedrock error: RP flash erase or write from core 1, or with core 1 started outside `spawn_core1`, or while DMA reads flash (XIP stalls)"

[lints.fls-4]
level = "warn"
reason = "bedrock warning: No periodic CRC of the image in flash, no ECC on flash and RAM where the part has it"

[lints.pin-1]
level = "deny"
reason = "bedrock error: Pin driver dropped (`let _ = Output::new(..)`, a temporary, end of `main`'s scope): embassy disconnects the pin on drop"

[lints.pin-2]
level = "deny"
reason = "bedrock error: Board not put in its safe state first thing after the HAL init (output enables, shift-register chains, power switches), before USB or anything slow"

[lints.pin-4]
level = "deny"
reason = "bedrock error: Peripheral pins moved to GPIO and back under an enabled peripheral (RP2350: the peripheral reads the OR of all pins with its FUNCSEL; the I2C block saw a start without a stop): keep FUNCSEL, use OEOVER / OUTOVER"

[lints.pin-6]
level = "deny"
reason = "bedrock error: Two owners of a pin (a diagnostic or test API and a driver task): one owner per pin, the other refused"

[lints.pin-8]
level = "warn"
reason = "bedrock warning: Unused pins left floating inputs: analog (STM32) or a defined pull"

[lints.pin-9]
level = "deny"
reason = "bedrock error: Pin map chosen by a cargo feature for a board variant: a build for one variant flashed onto another drives the wrong lines; read the board record at run time, default to inputs"

[lints.pin-16]
level = "warn"
reason = "bedrock warning: Pin read that a dead-battery pull-down still holds (G0 UCPD DBCC pins until the SYSCFG strobe; never clear the strobe again)"

[lints.clk-2]
level = "deny"
reason = "bedrock error: embassy time driver `time-driver-any`, or the named timer also used for PWM or capture"

[lints.clk-3]
level = "warn"
reason = "bedrock warning: No clock security: CSS on HSE (and LSE) with a handler"

[lints.clk-5]
level = "deny"
reason = "bedrock error: USB enabled on a part without HSI48 / CRS and no exact 48 MHz source configured"

[lints.err-2]
level = "deny"
reason = "bedrock error: Register poke for an erratum without the erratum quoted (number, title, revisions, workaround) next to it"

[lints.err-3]
level = "deny"
reason = "bedrock error: G0B0/G0B1/G0C1 flash prefetch on while code crosses into bank 2 (ES0548 2.2.10, ES0547 2.2.8: HardFault on cross-bank branches)"

[lints.err-4]
level = "deny"
reason = "bedrock error: FDCAN edge filtering (`CCCR.EFBI`) on, or dedicated Tx buffers beside the Tx FIFO (ES0548 2.15.1/2.15.2, ES0491 2.22.1/2.22.2)"

[lints.err-5]
level = "warn"
reason = "bedrock warning: FDCAN DAR (one-shot) without retransmit after a lost arbitration on the first two ID bits (ES0491 2.22.3)"

[lints.err-6]
level = "warn"
reason = "bedrock warning: RP2350 A2 pad with pull-down and input buffer left on (RP2350-E9: the pin stays high): enable the input only for the read"

[lints.err-7]
level = "warn"
reason = "bedrock warning: RP2350 ROM reboot with delay 0 or FRCE_OFF bits beyond PROC1 (RP2350-E30, E19)"

[lints.err-8]
level = "warn"
reason = "bedrock warning: G0 ADC calibration without the wait of 2 ADC kernel clocks after disable (freezes)"

[lints.err-11]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: Silicon revision read from a register that can't tell all revisions apart (RP2350 `CHIP_ID.REVISION`: A3 and A4 both 3; use the bootrom version)"

[lints.ext-4]
level = "warn"
reason = "bedrock warning: Read-modify-write of an external chip's registers over the bus, clear-on-read registers read more than once: write from shadow copies"

[lints.ext-7]
level = "warn"
reason = "bedrock warning: Register field encoded without a range check (`x - 1` underflows, masked to 0)"

[lints.ext-12]
level = "warn"
reason = "bedrock warning: RCC enable not read back before the first access to the peripheral"

[lints.usb-1]
level = "deny"
reason = "bedrock error: High-speed bulk endpoint max packet not 512, bInterval out of range for the speed"

[lints.usb-3]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: Single-buffered bulk endpoints on a hot path behind a hub TT (G0 usb_v4: 500 vs 947 kB/s)"

[lints.host-1]
level = "deny"
reason = "bedrock error: Panic reachable from host input: `unwrap`, `expect`, slice indexing, `unreachable!`, arithmetic overflow on values from a request (pin numbers, lengths, rates)"

[lints.host-3]
level = "warn"
reason = "bedrock warning: Unknown enum value from a newer host mapped to a default (`_ => ..`) instead of refused"

[lints.host-5]
level = "warn"
reason = "bedrock warning: Truncated or oversized request not counted and dropped"

[lints.log-1]
level = "deny"
reason = "bedrock error: RTT that can block: probe-rs sets the channel to blocking on attach; force NoBlockSkip, whole frames or skipped and counted"

[lints.log-2]
level = "warn"
reason = "bedrock warning: `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` not set explicitly in `.cargo/config.toml` (init bursts lost: 773 of 1000 lines)"

[lints.cnt-1]
level = "warn"
reason = "bedrock warning: Error branch (an `Err` handled, a timeout, a retry, a dropped frame, an overrun) without a `cnt!` counter: unexpected events must be counted"

[lints.cnt-2]
level = "warn"
reason = "bedrock warning: `&mut` to a counter buffer that an interrupt can update, unaligned u64 access"

[lints.cnt-3]
level = "warn"
reason = "bedrock warning: Proc-macro crate and its runtime crate not pinned to the exact same version (`=x.y.z`)"

[lints.cfg-1]
level = "deny"
reason = "bedrock error: probe-rs runner or a recipe without `--probe VID:PID`: with two probes on the PC another board gets flashed"

[lints.cfg-2]
level = "warn"
reason = "bedrock warning: `probe-rs download` then `reset` on F0/G0/L0: probe-rs writes RCC_APBENR1 back on a running core (USB clock off); use a runner that halts first"

[lints.cfg-4]
level = "deny"
reason = "bedrock error: A crate in the tree twice (git and path copies of WireWeaver, embassy): patch it to one source"

[lints.cfg-5]
level = "deny"
reason = "bedrock error: Chip feature (package, flash size) of the HAL not the part on the board"

[lints.cfg-6]
level = "warn"
reason = "bedrock warning: thumbv6m without `portable-atomic` + `critical-section` where a crate needs CAS (`static_cell`)"

[lints.cfg-7]
level = "warn"
reason = "bedrock warning: Dirty build flashed without a mark: build info carries the git SHA and a dirty flag, read back from the device"

[lints.perf-2]
level = "warn"
reason = "bedrock suggestion, reported without failing the run: 64-bit or i128 maths, float formatting, or `%` / `/` by a runtime value per element on a hot path of a core without them (`__aeabi_lmul`, `__aeabi_uldivmod`, `__multi3`)"

[lints.perf-3]
level = "warn"
reason = "bedrock warning: RAM function (`#[link_section = \".data.*\"]`) without `#[inline(never)]`: LTO inlines it back into flash"
```
