## Why

`embassy-executor`'s `platform-cortex-m` feature enabled together with an `executor-*` feature of `embassy-rp` or `embassy-stm32` (or any combination that makes two crates define the executor's `__pender` symbol).

Only one crate may define `__pender`; with two, the link either fails outright or (worse) picks one silently depending on link order, so the executor that actually runs may not be the one the code was written against.

## Example

Both usb_io_fw's AGENTS.md and the template's `frameworks.md` call this out explicitly, because the chip HAL crates offer their own `executor-*` feature as a convenience and it's easy to turn on by habit next to `platform-cortex-m`.

## How to fix

Pick one source for the executor (usually the chip HAL's `executor-*` feature on RP and STM32 targets) and drop `platform-cortex-m`, or vice versa.

## How it is checked

`cfg easy`: read the resolved feature set of `Cargo.toml` / `Cargo.lock` and flag the combination directly — no firmware model needed.

## Source

Ported from embedded_bedrock's firmware check catalogue, `EXE-10` (`book/checks.md`, detail in `book/checks/exe.md`). Found on: usb_io_fw AGENTS.md; template frameworks.md.
