## Why

D-cache enabled on a Cortex-M7 part with DMA buffers placed in ordinary cacheable RAM, and no explicit cache clean (before a DMA write) or invalidate (before a DMA read) around the transfer, and no MPU region marking the buffer non-cacheable.

The CPU and the DMA engine don't share a cache; the CPU can read stale cached data after DMA has written fresh bytes underneath it, or DMA can read stale RAM while the CPU's dirty cache lines haven't been written back yet. Both look like corrupted or stale data that comes and goes with cache pressure.

## Example

A Cortex-M7 firmware turns the D-cache on while a DMA channel moves data through ordinary RAM, with no cache maintenance and no non-cacheable MPU region:

```rust
fn setup(p: Peripherals, cp: &mut Cp) {
    let _dma = p.DMA1_CH0;
    cp.SCB.enable_dcache(&mut cp.CPUID); // DMA buffers in cacheable RAM from here on
}
```

## How to fix

Put DMA buffers in an MPU region marked non-cacheable, or call the HAL's cache clean/invalidate for the buffer's address range immediately before/after the transfer.

## How it is checked

`src mid`, chip M7 only: the template's generated `main.rs` leaves D-cache commented out for exactly this reason (see embedded_bedrock's `firmware_template_skill/references/stm32-notes.md` "H7 caches"); a firmware that turns it on needs either an MPU non-cacheable region over every DMA buffer or explicit clean/invalidate calls around every transfer.

## Source

Ported from embedded_bedrock's firmware check catalogue, `DMA-2` (`book/checks.md`, detail in `book/checks/dma.md`). Found on: vb135 PLAT-5; template stm32-notes.
