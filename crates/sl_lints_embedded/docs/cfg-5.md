## Why

HAL crates like `embassy-rp` gate the exact package variant (flash size, pin count, die) behind a cargo feature (`rp235xa` vs `rp235xb`), and the wrong one builds and links cleanly — it just targets a different pinout or flash geometry than the chip actually on the board.

## Example

Usb_io_fw commit `19203e6` ("fix: set RP235xB (QFN-80), set correct LED pin") changed `b200_usb_io/Cargo.toml`'s `embassy-rp` feature from `rp235xa` to `rp235xb` — the board uses the QFN-80 package, not the package the `rp235xa` feature assumes — alongside a GPIO fix for the status LED pin that depended on the same mismatch.

## How to fix

Read the chip feature straight from the board's part number (the template's `chip-info` / board preset already does this for generated firmware); when hand-editing a HAL dependency's features after a board respin, cross-check the package suffix against the schematic's MCU designator, not against what built last time.

## How it is checked

Check: cfg mid — needs the board record (part number, package) cross-referenced against the HAL crate's selected feature; a lint can flag a feature that doesn't match the `chip:` the firmware's own board data names.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CFG-5` (`book/checks.md`, detail in `book/checks/cfg.md`). Found on: usb_io_fw 19203e6 (rp235xb).
