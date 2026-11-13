## Why

No periodic check of the running image's CRC in flash, and no use of the part's own ECC on flash/RAM where the chip actually has it.

Flash bits can flip over the firmware's lifetime (radiation, wear, marginal cells); without a periodic CRC of the image itself, or without ECC turned on where available, the first sign of a flipped bit is whatever misbehaviour the corrupted instruction or data happens to cause — not a clean, early detection.

## Example

A firmware computes a CRC of the USB packets it receives but never checks its own image in flash, so a flipped bit in code runs until it crashes:

```rust
let p = embassy_stm32::init(Default::default());
let crc = packet_crc(&[1, 2, 3]); // a packet's CRC, not the image's
```

## How to fix

Add a low-priority periodic CRC check over the application image; turn on the chip's flash/RAM ECC where available rather than leaving it off by default.

## How it is checked

`src easy`: a periodic task computing a CRC over the running image's flash range, and ECC enabled on parts whose chip data lists it.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLS-4` (`book/checks.md`, detail in `book/checks/fls.md`). Found on: notion.
