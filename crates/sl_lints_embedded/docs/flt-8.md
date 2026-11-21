## Why

No counter, in a backup register, of boots since the last time the firmware marked itself healthy — and so no fallback into the ROM bootloader after a few unhealthy boots in a row.

A firmware that crashes before it can mark itself healthy (a bad update, a new regression) boot-loops forever with no rescue path other than a debug probe; a counted fallback into the ROM bootloader after a few failed boots turns "the board is bricked" into "the board is recoverable over USB".

## Example

Typec_dongle FW-45, and embedded_bedrock DEV-16 which specifies the device-wide boot-loop rescue contract every board should meet.

## How to fix

Count boots in a backup register, clear it once the firmware marks itself healthy, and jump to the ROM bootloader (or hold there) once the count passes a small threshold (DEV-16's contract).

## How it is checked

`src easy`: a backup-register boot counter incremented at boot and cleared on a healthy mark, with a check that falls into the ROM bootloader past a threshold.

## Source

Ported from embedded_bedrock's firmware check catalogue, `FLT-8` (`book/checks.md`, detail in `book/checks/flt.md`). Found on: portwright FW-45; embedded_bedrock DEV-16.
