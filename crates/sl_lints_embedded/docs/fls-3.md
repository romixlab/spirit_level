## Why

RP2040/RP2350 flash erase or write issued from core 1, or with core 1 running outside `spawn_core1`'s controlled start-up, or while DMA is reading flash through XIP.

RP's flash program/erase routines execute from RAM precisely because flash can't be read (including XIP-mapped code fetches) while it's being erased or written; core 1 running arbitrary code during a flash op can stall on an XIP read exactly when the flash isn't available to answer it, and DMA reading flash during the op hits the same stall from a different direction.

## Example

Embedded_bedrock TPL-25 is this exact hazard worked through for the RP flash path the template generates.

## How to fix

Park core 1 (or route its own flash needs through the same serialised owner, `fls-2`) and make sure no DMA channel is mid-transfer from flash before issuing an erase or write.

## How it is checked

`src mid`, chip RP: flash erase/write call sites, checked against whether core 1 is confirmed parked (via `spawn_core1`'s synchronisation) and whether any DMA channel is configured to read flash concurrently.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLS-3` (`book/checks.md`, detail in `book/checks/fls.md`). Found on: embedded_bedrock TPL-25.
