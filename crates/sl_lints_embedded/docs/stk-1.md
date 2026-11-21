## Why

`.cargo/config.toml`'s `[target.<triple>]` missing `linker = "flip-link"`, so the firmware links with the plain linker instead.

flip-link moves the stack below `.data`/`.bss` so a stack overflow runs into an unmapped guard instead of silently corrupting the first bytes of RAM. Without it, an overflow corrupts whatever static data happens to sit above the stack and the symptom is wherever that data is read next — rarely at the overflow site.

## Example

Found independently in embedded_bedrock (ROB-12), portwright (FW-1) and usb_io_fw (PLAT-1): three projects hit the same missing-linker mistake before it became a generator default and a lint.

## How to fix

Add `linker = "flip-link"` under `[target.<triple>]`; the generator sets this by default for every firmware it produces.

## How it is checked

`cfg easy`: read `.cargo/config.toml` for the target triple in use and check `linker` is set to `"flip-link"`.

## Source

Ported from embedded_bedrock's firmware check catalogue, `STK-1` (`book/checks.md`, detail in `book/checks/stk.md`). Found on: embedded_bedrock ROB-12; portwright FW-1; usb_io_fw PLAT-1.
