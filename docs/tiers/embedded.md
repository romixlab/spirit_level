# embedded

Bare-metal firmware: executors, interrupts, DMA, stack, memory, size, faults, boot, power, flash, pins, clocks, errata, USB, host input, logging, counters, build configuration.

Use it with `extends = "embedded"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [C0005](../lints/C0005.md) | panic-in-library | `deny` | a service module (LED, log, telemetry) never reaches the panic handler in release: the whole MCU resets for it | embedded |
| [C0030](../lints/C0030.md) | compound-assert | `warn` | assert!(a && b) is two assertions: a crash record from the field does not say which condition broke | embedded |
| [C0031](../lints/C0031.md) | assert-message | `warn` | a failed assertion says in words what broke, which is what the crash record carries back; with defmt it costs no flash | embedded |
| [F0001](../lints/F0001.md) | boot-1 | `deny` | bedrock error: Firmware entered by a jump (ROM DfuSe leave, bootloader) not detected: VTOR, PLL, SysTick, enabled IRQs of the ROM are still set; detect (RCC_CFGR SWS, VTOR, NVIC) and reset once | embedded |
| [F0002](../lints/F0002.md) | boot-10 | `warn` | bedrock warning: Backup-domain helper that turns the RTC clock or `PWR_CR1.DBP` off afterwards instead of restoring what it found | embedded |
| [F0003](../lints/F0003.md) | boot-2 | `deny` | bedrock error: H7 supply config written blind: an old ROM bootloader (before V9.3) locks PWR_CR3 until a power cycle; read it back, use what it holds, count a mismatch | embedded |
| [F0004](../lints/F0004.md) | cfg-1 | `deny` | bedrock error: probe-rs runner or a recipe without `--probe VID:PID`: with two probes on the PC another board gets flashed | embedded |
| [F0005](../lints/F0005.md) | cfg-2 | `warn` | bedrock warning: `probe-rs download` then `reset` on F0/G0/L0: probe-rs writes RCC_APBENR1 back on a running core (USB clock off); use a runner that halts first | embedded |
| [F0006](../lints/F0006.md) | cfg-4 | `deny` | bedrock error: A crate in the tree twice (git and path copies of WireWeaver, embassy): patch it to one source | embedded |
| [F0007](../lints/F0007.md) | cfg-5 | `deny` | bedrock error: Chip feature (package, flash size) of the HAL not the part on the board | embedded |
| [F0008](../lints/F0008.md) | cfg-6 | `warn` | bedrock warning: thumbv6m without `portable-atomic` + `critical-section` where a crate needs CAS (`static_cell`) | embedded |
| [F0009](../lints/F0009.md) | cfg-7 | `warn` | bedrock warning: Dirty build flashed without a mark: build info carries the git SHA and a dirty flag, read back from the device | embedded |
| [F0010](../lints/F0010.md) | clk-2 | `deny` | bedrock error: embassy time driver `time-driver-any`, or the named timer also used for PWM or capture | embedded |
| [F0011](../lints/F0011.md) | clk-3 | `warn` | bedrock warning: No clock security: CSS on HSE (and LSE) with a handler | embedded |
| [F0012](../lints/F0012.md) | clk-5 | `deny` | bedrock error: USB enabled on a part without HSI48 / CRS and no exact 48 MHz source configured | embedded |
| [F0013](../lints/F0013.md) | cnt-1 | `warn` | bedrock warning: Error branch (an `Err` handled, a timeout, a retry, a dropped frame, an overrun) without a `cnt!` counter: unexpected events must be counted | embedded |
| [F0014](../lints/F0014.md) | cnt-2 | `warn` | bedrock warning: `&mut` to a counter buffer that an interrupt can update, unaligned u64 access | embedded |
| [F0015](../lints/F0015.md) | cnt-3 | `warn` | bedrock warning: Proc-macro crate and its runtime crate not pinned to the exact same version (`=x.y.z`) | embedded |
| [F0016](../lints/F0016.md) | dma-1 | `deny` | bedrock error: DMA buffer in memory the DMA master can't reach (H7 DTCM for IDMA, SDMMC, USB OTG; flash for some masters): it hangs or reads zeros | embedded |
| [F0017](../lints/F0017.md) | dma-2 | `deny` | bedrock error: D-cache on with DMA buffers in cacheable RAM and no clean / invalidate or MPU non-cacheable region | embedded |
| [F0018](../lints/F0018.md) | dma-4 | `warn` | bedrock warning: DMA ring overrun not counted: every ring consumer counts overruns with `cnt!` | embedded |
| [F0019](../lints/F0019.md) | dma-6 | `warn` | bedrock warning: Erratum asks for DMA where the code polls (G0 UCPD TX underflow then TXHRST corrupts the next CRC: ES0548 2.17.1) | embedded |
| [F0020](../lints/F0020.md) | firmware-readable | `deny` | a firmware file the embedded lints cannot read hides every finding in it | embedded |
| [F0021](../lints/F0021.md) | err-11 | `warn` | bedrock suggestion, reported without failing the run: Silicon revision read from a register that can't tell all revisions apart (RP2350 `CHIP_ID.REVISION`: A3 and A4 both 3; use the bootrom version) | embedded |
| [F0022](../lints/F0022.md) | err-2 | `deny` | bedrock error: Register poke for an erratum without the erratum quoted (number, title, revisions, workaround) next to it | embedded |
| [F0023](../lints/F0023.md) | err-3 | `deny` | bedrock error: G0B0/G0B1/G0C1 flash prefetch on while code crosses into bank 2 (ES0548 2.2.10, ES0547 2.2.8: HardFault on cross-bank branches) | embedded |
| [F0024](../lints/F0024.md) | err-4 | `deny` | bedrock error: FDCAN edge filtering (`CCCR.EFBI`) on, or dedicated Tx buffers beside the Tx FIFO (ES0548 2.15.1/2.15.2, ES0491 2.22.1/2.22.2) | embedded |
| [F0025](../lints/F0025.md) | err-5 | `warn` | bedrock warning: FDCAN DAR (one-shot) without retransmit after a lost arbitration on the first two ID bits (ES0491 2.22.3) | embedded |
| [F0026](../lints/F0026.md) | err-6 | `warn` | bedrock warning: RP2350 A2 pad with pull-down and input buffer left on (RP2350-E9: the pin stays high): enable the input only for the read | embedded |
| [F0027](../lints/F0027.md) | err-7 | `warn` | bedrock warning: RP2350 ROM reboot with delay 0 or FRCE_OFF bits beyond PROC1 (RP2350-E30, E19) | embedded |
| [F0028](../lints/F0028.md) | err-8 | `warn` | bedrock warning: G0 ADC calibration without the wait of 2 ADC kernel clocks after disable (freezes) | embedded |
| [F0029](../lints/F0029.md) | exe-1 | `deny` | bedrock error: A task with a hard answer deadline (a PHY that arms RX DMA only inside `receive().await`, a software GoodCRC, a DMA ring consumer) runs on the thread executor beside bulk work (USB server, logging) | embedded |
| [F0030](../lints/F0030.md) | exe-10 | `deny` | bedrock error: Two crates provide the executor and `__pender` (embassy-rp or embassy-stm32 `executor-*` features and embassy-executor `platform-cortex-m` together) | embedded |
| [F0031](../lints/F0031.md) | exe-12 | `warn` | bedrock suggestion, reported without failing the run: `async fn` that never awaits (handlers especially): each one is a state in the dispatcher future and costs flash | embedded |
| [F0032](../lints/F0032.md) | exe-13 | `warn` | bedrock warning: Ring-buffered reader (`RingBufferedUartRx::read`) in a loop without batching: a busy line gives 1-byte reads and 100 % CPU; wait about 1 ms after a short read | embedded |
| [F0033](../lints/F0033.md) | exe-2 | `deny` | bedrock error: Blocking call (`block_for`, `blocking_*` HAL calls, `nb::block!`, busy-wait loops, `embassy_futures::block_on`) in a task on an interrupt executor or in an interrupt handler | embedded |
| [F0034](../lints/F0034.md) | exe-3 | `deny` | bedrock error: Interrupt executor started before its IRQ priority is set: `start()` unmasks the IRQ at the default (highest) priority | embedded |
| [F0035](../lints/F0035.md) | exe-4 | `warn` | bedrock warning: Interrupt executor priority not below the peripheral, DMA and time-driver interrupts that wake its tasks (it must preempt thread mode, never them) | embedded |
| [F0036](../lints/F0036.md) | exe-5 | `deny` | bedrock error: Interrupt executor on a vector that a driver also binds (`bind_interrupts!`) or that a used peripheral raises: use the vector of a peripheral that is unused, and say so in a comment | embedded |
| [F0037](../lints/F0037.md) | exe-6 | `deny` | bedrock error: State shared between a task on an interrupt executor and thread-mode code behind `ThreadModeRawMutex` / `NoopRawMutex` or a `static mut`: use `CriticalSectionRawMutex` | embedded |
| [F0038](../lints/F0038.md) | exe-7 | `warn` | bedrock warning: Loop in a shared task that works until a queue is empty (or logs in a loop) without a bound and a yield: above the input rate it never ends and starves the executor | embedded |
| [F0039](../lints/F0039.md) | exe-8 | `warn` | bedrock warning: Request server waits on a bus or a slow peripheral inside a handler: defer the transfer to the task that owns the bus and answer from the server loop | embedded |
| [F0040](../lints/F0040.md) | exe-9 | `deny` | bedrock error: Future polled with a no-op waker (`poll_once`, `noop_waker`) that can reach `embassy_time`: embassy-time panics "Found waker not created by the Embassy executor" | embedded |
| [F0041](../lints/F0041.md) | ext-12 | `warn` | bedrock warning: RCC enable not read back before the first access to the peripheral | embedded |
| [F0042](../lints/F0042.md) | ext-4 | `warn` | bedrock warning: Read-modify-write of an external chip's registers over the bus, clear-on-read registers read more than once: write from shadow copies | embedded |
| [F0043](../lints/F0043.md) | ext-7 | `warn` | bedrock warning: Register field encoded without a range check (`x - 1` underflows, masked to 0) | embedded |
| [F0044](../lints/F0044.md) | fls-1 | `deny` | bedrock error: Flash that a power cut can tear (settings, bootloader state) read raw on an ECC part: a double-ECC word is an NMI (G0) or BusFault (H7) on every boot; read through `ww_ecc_flash` with the NMI hook | embedded |
| [F0045](../lints/F0045.md) | fls-2 | `deny` | bedrock error: More than one owner of the flash peripheral (updater and settings each take `FLASH`) | embedded |
| [F0046](../lints/F0046.md) | fls-3 | `deny` | bedrock error: RP flash erase or write from core 1, or with core 1 started outside `spawn_core1`, or while DMA reads flash (XIP stalls) | embedded |
| [F0047](../lints/F0047.md) | fls-4 | `warn` | bedrock warning: No periodic CRC of the image in flash, no ECC on flash and RAM where the part has it | embedded |
| [F0048](../lints/F0048.md) | flt-1 | `deny` | bedrock error: HardFault handler that spins (`loop {}`) in a shipped build: record (crash capture) and reset | embedded |
| [F0049](../lints/F0049.md) | flt-2 | `deny` | bedrock error: defmt, `cnt!`, `unwrap` or anything that can panic or nest-log inside `HardFault` / `DefaultHandler`: record the IRQ number raw instead | embedded |
| [F0050](../lints/F0050.md) | flt-3 | `warn` | bedrock warning: `bedrock_crash::exc_return!()` not the first statement of HardFault (LR changes) | embedded |
| [F0051](../lints/F0051.md) | flt-4 | `warn` | bedrock warning: No UsageFault, MemManage and BusFault handlers (or not enabled in SHCSR) where the core has them | embedded |
| [F0052](../lints/F0052.md) | flt-5 | `warn` | bedrock warning: Reset cause not read and reported at boot (power-on, brown-out, watchdog, software, pin, lockup) | embedded |
| [F0053](../lints/F0053.md) | flt-7 | `deny` | bedrock error: No watchdog, or a watchdog fed from an interrupt or from a loop not bounded by a timeout; window not calibrated to the real period | embedded |
| [F0054](../lints/F0054.md) | flt-8 | `warn` | bedrock warning: No boot-loop rescue: count boots without a healthy mark in a backup register, fall back to the ROM bootloader after a few | embedded |
| [F0055](../lints/F0055.md) | flt-9 | `warn` | bedrock warning: Healthy mark never set on the transport in use (`usb_configured()` not called with USB transport): every boot waits the full timeout | embedded |
| [F0056](../lints/F0056.md) | host-1 | `deny` | bedrock error: Panic reachable from host input: `unwrap`, `expect`, slice indexing, `unreachable!`, arithmetic overflow on values from a request (pin numbers, lengths, rates) | embedded |
| [F0057](../lints/F0057.md) | host-3 | `warn` | bedrock warning: Unknown enum value from a newer host mapped to a default (`_ => ..`) instead of refused | embedded |
| [F0058](../lints/F0058.md) | host-5 | `warn` | bedrock warning: Truncated or oversized request not counted and dropped | embedded |
| [F0059](../lints/F0059.md) | irq-1 | `deny` | bedrock error: Interrupt flags cleared wholesale (all-ones or the read value written back to a W1C register): an event between the read and the clear is lost; clear exactly the flags handled | embedded |
| [F0060](../lints/F0060.md) | irq-2 | `deny` | bedrock error: `rc_w1` / W1C flags cleared with `modify()` (read-modify-write) instead of a write of the one bit: it clears flags that were set meanwhile | embedded |
| [F0061](../lints/F0061.md) | irq-4 | `deny` | bedrock error: Several drivers on one shared NVIC vector, and a driver that disables "its" interrupt on drop: the others go deaf (G0B1 `USB_UCPD1_2`) | embedded |
| [F0062](../lints/F0062.md) | irq-5 | `warn` | bedrock warning: Interrupt line routed (ILS) but not enabled (ILE), or enabled without a handler bound | embedded |
| [F0063](../lints/F0063.md) | irq-6 | `warn` | bedrock warning: Nested disable / enable of interrupts by hand: an inner enable turns them on under an outer section; use `critical_section::with` | embedded |
| [F0064](../lints/F0064.md) | irq-8 | `warn` | bedrock warning: Unused interrupt vectors not caught: `DefaultHandler` must record the IRQ number (raw: a static, the crash record or a counter; not logging it, FLT-2) | embedded |
| [F0065](../lints/F0065.md) | log-1 | `deny` | bedrock error: RTT that can block: probe-rs sets the channel to blocking on attach; force NoBlockSkip, whole frames or skipped and counted | embedded |
| [F0066](../lints/F0066.md) | log-2 | `warn` | bedrock warning: `DEFMT_RTT_BUFFER_SIZE` and `DEFMT_LOG` not set explicitly in `.cargo/config.toml` (init bursts lost: 773 of 1000 lines) | embedded |
| [F0067](../lints/F0067.md) | mem-1 | `deny` | bedrock error: `FLASH` length in memory.x covers a region kept for something else (settings, bootloader state, DFU): the image can grow into it | embedded |
| [F0068](../lints/F0068.md) | mem-10 | `warn` | bedrock warning: RWX LOAD segment or a writable symbol in `.rodata` (`static mut` in a read-only section) | embedded |
| [F0069](../lints/F0069.md) | mem-12 | `deny` | bedrock error: defmt in the dependencies without `-Tdefmt.x` (no `.defmt` section) | embedded |
| [F0070](../lints/F0070.md) | mem-2 | `warn` | bedrock warning: Extra RAM section with a tail `. = ALIGN(n)` or a symbol in it: lld keeps an empty section that flip-link counts as used | embedded |
| [F0071](../lints/F0071.md) | mem-3 | `deny` | bedrock error: RAM init or zeroing range overlaps `.uninit` (crash record, handoff data) | embedded |
| [F0072](../lints/F0072.md) | mem-6 | `warn` | bedrock warning: `.cnt_bkp_buffer` not at the backup-register address | embedded |
| [F0073](../lints/F0073.md) | mem-7 | `warn` | bedrock warning: Flash partition symbols for embassy-boot written as absolute addresses where offsets are expected (`- ORIGIN(FLASH)`) | embedded |
| [F0074](../lints/F0074.md) | perf-2 | `warn` | bedrock suggestion, reported without failing the run: 64-bit or i128 maths, float formatting, or `%` / `/` by a runtime value per element on a hot path of a core without them (`__aeabi_lmul`, `__aeabi_uldivmod`, `__multi3`) | embedded |
| [F0075](../lints/F0075.md) | perf-3 | `warn` | bedrock warning: RAM function (`#[link_section = ".data.*"]`) without `#[inline(never)]`: LTO inlines it back into flash | embedded |
| [F0076](../lints/F0076.md) | pin-1 | `deny` | bedrock error: Pin driver dropped (`let _ = Output::new(..)`, a temporary, end of `main`'s scope): embassy disconnects the pin on drop | embedded |
| [F0077](../lints/F0077.md) | pin-16 | `warn` | bedrock warning: Pin read that a dead-battery pull-down still holds (G0 UCPD DBCC pins until the SYSCFG strobe; never clear the strobe again) | embedded |
| [F0078](../lints/F0078.md) | pin-2 | `deny` | bedrock error: Board not put in its safe state first thing after the HAL init (output enables, shift-register chains, power switches), before USB or anything slow | embedded |
| [F0079](../lints/F0079.md) | pin-4 | `deny` | bedrock error: Peripheral pins moved to GPIO and back under an enabled peripheral (RP2350: the peripheral reads the OR of all pins with its FUNCSEL; the I2C block saw a start without a stop): keep FUNCSEL, use OEOVER / OUTOVER | embedded |
| [F0080](../lints/F0080.md) | pin-6 | `deny` | bedrock error: Two owners of a pin (a diagnostic or test API and a driver task): one owner per pin, the other refused | embedded |
| [F0081](../lints/F0081.md) | pin-8 | `warn` | bedrock warning: Unused pins left floating inputs: analog (STM32) or a defined pull | embedded |
| [F0082](../lints/F0082.md) | pin-9 | `deny` | bedrock error: Pin map chosen by a cargo feature for a board variant: a build for one variant flashed onto another drives the wrong lines; read the board record at run time, default to inputs | embedded |
| [F0083](../lints/F0083.md) | pwr-1 | `warn` | bedrock warning: No brown-out reset level or programmable voltage detector (PVD) for the supply the board really has | embedded |
| [F0084](../lints/F0084.md) | pwr-3 | `warn` | bedrock warning: Backup domain not reset at boot when the RTC and backup registers are unused (corrupted after a brown-out without POR) | embedded |
| [F0085](../lints/F0085.md) | pwr-4 | `warn` | bedrock warning: RTC clock source changed in the clock config of a firmware that keeps state in the backup domain: embassy resets the domain | embedded |
| [F0086](../lints/F0086.md) | siz-1 | `warn` | bedrock warning: Flash or RAM use over a threshold (default 90 % of the region) in a shipped profile | embedded |
| [F0087](../lints/F0087.md) | siz-3 | `warn` | bedrock suggestion, reported without failing the run: Per-package `opt-level` together with `lto = "fat"`: the override does nothing | embedded |
| [F0088](../lints/F0088.md) | siz-4 | `warn` | bedrock suggestion, reported without failing the run: Release profile without `codegen-units = 1`, `lto`, `panic = "abort"`, and an `opt-level` chosen by measurement (z vs 3 is the largest lever: Donguru 501.7 → 321.8 KiB, 10-25 % slower USB) | embedded |
| [F0089](../lints/F0089.md) | siz-5 | `warn` | bedrock suggestion, reported without failing the run: `core::fmt` pulled in: `Result::unwrap` / `expect` (through `&dyn Debug`), `Debug2Format`, a panic handler that formats `PanicInfo` (4-5 KiB) | embedded |
| [F0090](../lints/F0090.md) | siz-6 | `warn` | bedrock suggestion, reported without failing the run: Large `.data` initializer (a static with a non-zero const value): zero-init and set it up in place | embedded |
| [F0091](../lints/F0091.md) | stk-1 | `deny` | bedrock error: flip-link not the linker: `linker = "flip-link"` under `[target.<triple>]` in `.cargo/config.toml` | embedded |
| [F0092](../lints/F0092.md) | stk-2 | `deny` | bedrock error: `cargo:rustc-linker=` (or another unknown `cargo:` key) printed by build.rs: not a build-script directive, Cargo ignores it silently | embedded |
| [F0093](../lints/F0093.md) | stk-3 | `deny` | bedrock error: Stack not below `.data` / `.bss` in the ELF (`_stack_start` != start of `.data`), or a zero-size stack | embedded |
| [F0094](../lints/F0094.md) | stk-4 | `deny` | bedrock error: cortex-m-rt `paint-stack` on before its BusFault is understood | embedded |
| [F0095](../lints/F0095.md) | stk-5 | `warn` | bedrock warning: Big state built by value and moved into a static (`StaticCell::init(Big::new())`): the frame holds it whole; build it in place (`const fn new`, init into `MaybeUninit`) | embedded |
| [F0096](../lints/F0096.md) | stk-6 | `warn` | bedrock warning: Stack frame over a budget (per function) or a worst-case stack depth over the stack region | embedded |
| [F0097](../lints/F0097.md) | usb-1 | `deny` | bedrock error: High-speed bulk endpoint max packet not 512, bInterval out of range for the speed | embedded |
| [F0098](../lints/F0098.md) | usb-3 | `warn` | bedrock suggestion, reported without failing the run: Single-buffered bulk endpoints on a hot path behind a hub TT (G0 usb_v4: 500 vs 947 kB/s) | embedded |
| [F0099](../lints/F0099.md) | brd-1 | `deny` | a description that does not read, or names chip data or a PCB that does not read, hides every board finding | embedded |
| [F0100](../lints/F0100.md) | brd-2 | `deny` | a pin with two jobs in one configuration is two drivers on one line | embedded |
| [F0101](../lints/F0101.md) | brd-3 | `deny` | two used pins joined on the board drive against each other | embedded |
| [F0102](../lints/F0102.md) | brd-4 | `deny` | a pin, package, peripheral or signal the chip does not have is a typo the generated code would carry | embedded |
| [F0103](../lints/F0103.md) | brd-5 | `deny` | a signal on a pin that cannot carry it never reaches the peripheral | embedded |
| [F0104](../lints/F0104.md) | brd-6 | `deny` | a pin the PCB does not connect as described is firmware driving the wrong line (B135B's LED on B135A's pins) | embedded |
| [F0105](../lints/F0105.md) | brd-7 | `warn` | a connected pin the description leaves out is a fact missing, not a fault: the description grows to cover it | embedded |
| [F0106](../lints/F0106.md) | cls-1 | `warn` | every task file says what a panic in it may do; warn while no firmware declares [embedded.classes] yet, then deny | embedded |
| [F0107](../lints/F0107.md) | cls-2 | `deny` | a service module never reaches the panic handler in release; Clippy finds its panics at build time. Fires only where classes are declared | embedded |
| [F0108](../lints/F0108.md) | flt-11 | `warn` | a panic in control code turns the outputs off before it records and resets; warn while no board generates safe_state() yet, then deny | embedded |
| [F0109](../lints/F0109.md) | flt-12 | `warn` | a crash loop must not re-enable the outputs on every boot; an estimate by statement order, and no firmware reads boots_since at boot yet | embedded |

## Clippy set (with what it extends)

The project turns these on in its own `[workspace.lints.clippy]` (`allow` rows are accepted opt-outs, pasted with their reason); [tier-clippy-set](../lints/M0025.md) checks it.

| Clippy lint | Level | Reason | Set in |
|---|---|---|---|
| `as_ptr_cast_mut` | `deny` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | embedded |
| `infinite_loop` | `warn` | a loop that never ends returns `!`, so the hang is a type, not a surprise | embedded |
| `large_include_file` | `warn` | an include_bytes! of a large file fills the flash unseen | embedded |
| `large_stack_frames` | `warn` | a large frame overflows a small stack with no guard page | embedded |
| `undocumented_unsafe_blocks` | `deny` | firmware is full of unsafe; every block says in a SAFETY: comment why it is sound | embedded |
| `unnecessary_safety_doc` | `deny` | a # Safety section on a safe function sends readers looking for a contract that is not there | embedded |

## The file

```toml
# Beside the size tiers: a firmware extends ["small", "embedded"]. Every automatic check of embedded_bedrock's
# firmware check catalogue, at bedrock's severity: error is deny, warning is warn, suggestion is warn (reported,
# so the size and speed levers stay visible, but never failing the run on its own). A workspace with host crates
# beside its firmware puts this tier on the firmware crates only: [[crate-tiers]] or the map key tiers.
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

# The Clippy set of a firmware, on top of the size tier it is combined with: unsafe is documented and denied
# where unsound, stack and flash are watched, the main loop is the only endless one.

# The board description (board.toml): checked on every revision and rework, against itself, the chip's data and the
# PCB's nets (the board lints, docs/boards.md).

[lints.brd-1]
level = "deny"
reason = "a description that does not read, or names chip data or a PCB that does not read, hides every board finding"

[lints.brd-2]
level = "deny"
reason = "a pin with two jobs in one configuration is two drivers on one line"

[lints.brd-3]
level = "deny"
reason = "two used pins joined on the board drive against each other"

[lints.brd-4]
level = "deny"
reason = "a pin, package, peripheral or signal the chip does not have is a typo the generated code would carry"

[lints.brd-5]
level = "deny"
reason = "a signal on a pin that cannot carry it never reaches the peripheral"

[lints.brd-6]
level = "deny"
reason = "a pin the PCB does not connect as described is firmware driving the wrong line (B135B's LED on B135A's pins)"

[lints.brd-7]
level = "warn"
reason = "a connected pin the description leaves out is a fact missing, not a fault: the description grows to cover it"

# The assertion policy: what a broken invariant may do depends on the module's task class ([embedded.classes]).

[lints.cls-1]
level = "warn"
reason = "every task file says what a panic in it may do; warn while no firmware declares [embedded.classes] yet, then deny"

[lints.cls-2]
level = "deny"
reason = "a service module never reaches the panic handler in release; Clippy finds its panics at build time. Fires only where classes are declared"

[lints.flt-11]
level = "warn"
reason = "a panic in control code turns the outputs off before it records and resets; warn while no board generates safe_state() yet, then deny"

[lints.flt-12]
level = "warn"
reason = "a crash loop must not re-enable the outputs on every boot; an estimate by statement order, and no firmware reads boots_since at boot yet"

[lints.pan-1]
level = "allow"
reason = "the strict level: every panic site of the release ELF acknowledged in panics.toml; a firmware turns it on for releases, with `spl panics --ack` to start the file"

[lints.panic-in-library]
level = "deny"
reason = "a service module (LED, log, telemetry) never reaches the panic handler in release: the whole MCU resets for it"
classes = ["service"]
methods = ["unwrap", "expect"]
macros = ["panic", "todo", "unimplemented", "unreachable", "assert", "assert_eq", "assert_ne"]
bare-macros = []

[lints.compound-assert]
level = "warn"
reason = "assert!(a && b) is two assertions: a crash record from the field does not say which condition broke"

[lints.assert-message]
level = "warn"
reason = "a failed assertion says in words what broke, which is what the crash record carries back; with defmt it costs no flash"

[clippy.lints]
undocumented_unsafe_blocks = { level = "deny", reason = "firmware is full of unsafe; every block says in a SAFETY: comment why it is sound" }
unnecessary_safety_doc = { level = "deny", reason = "a # Safety section on a safe function sends readers looking for a contract that is not there" }
as_ptr_cast_mut = { level = "deny", reason = "`as_ptr() as *mut` writes through a shared borrow: undefined behaviour" }
large_stack_frames = { level = "warn", reason = "a large frame overflows a small stack with no guard page" }
large_include_file = { level = "warn", reason = "an include_bytes! of a large file fills the flash unseen" }
infinite_loop = { level = "warn", reason = "a loop that never ends returns `!`, so the hang is a type, not a surprise" }
```
