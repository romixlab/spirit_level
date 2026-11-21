## Why

Flash that a power cut can leave torn (settings, bootloader state) read directly instead of through `ww_ecc_flash`'s NMI-hook-protected reader, on a part whose flash has ECC.

ECC flash computes an ECC code per word at write time; a word that was only partially written when power cut has a data/ECC mismatch, and reading it raw triggers a double-ECC-error trap — an NMI on G0, a BusFault on H7 — right at the read, not at the write that caused it. A reader that doesn't expect this treats a perfectly ordinary power-cut artifact as a fatal fault.

## Example

Typec_dongle API-19, and the settings design in the template both exist because of this; upstream embassy issue #7235 covers the same hazard from the embassy-boot side.

## How to fix

Read torn-writable flash regions only through `ww_ecc_flash` (or an equivalent NMI/BusFault-hooked reader) so a double-ECC error from a half-written word is handled as "this word was never fully written", not as a fault.

## How it is checked

`src mid`, limited to ECC-flash parts (G0, H7, G4, L4 and others per the chip table): a read of settings/bootloader-state flash regions that doesn't go through `ww_ecc_flash`'s NMI-hooked accessor.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLS-1` (`book/checks.md`, detail in `book/checks/fls.md`). Found on: portwright API-19; template settings; embassy#7235.
