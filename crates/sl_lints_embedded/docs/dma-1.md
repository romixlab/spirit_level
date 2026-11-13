## Why

A buffer handed to a DMA transfer that sits in a memory region the chosen DMA master has no bus access to: H7 DTCM for IDMA, SDMMC or USB OTG; flash for a master that can't read code memory on that part.

The DMA engine doesn't fault on an unreachable address the way the CPU would — it either hangs the transfer or silently returns zeros, which looks like "the peripheral isn't working" rather than "the buffer is in the wrong RAM bank".

## Example

Vb135's PLAT-5: a buffer placed in DTCM for a master that can't reach it there on H7, where `main_ram` defaults to AXISRAM for exactly this reason (see [memory-layout.md](../../firmware_template_skill/references/memory-layout.md) "RAM").

## How to fix

Place the buffer in a RAM bank the master can reach (on H7, prefer AXISRAM over DTCM for anything DMA touches); the template already defaults `main_ram` that way.

## How it is checked

`elf mid`, needs a chip table of which DMA masters can reach which memory region: for each DMA transfer, the buffer's linked address must be inside a region the configured master can access.

## Source

Ported from embedded_bedrock's firmware check catalogue, `DMA-1` (`book/checks.md`, detail in `book/checks/dma.md`). Found on: vb135 PLAT-5; notion.
