## Why

A shipped build whose flash or RAM use is over a threshold (default 90 %) of its region, read from the linked ELF.

Past 90 % there's little headroom left for the next feature, the next dependency bump, or a safety margin the project may need for things like `stk-6`'s stack budget; finding this out at 99.6 % full, mid release, leaves no room to fix it quickly.

## Example

Vb135 PLAT-7, and portwright at 99.6 % flash — both found after the fact rather than caught by a budget check at build time.

## How to fix

Nothing automatic — this check exists to surface the number early; reducing use is `siz-4` through `SIZ-9`'s job, or the budget needs raising (a bigger part, or dropping `--settings`/`--bootloader` headroom elsewhere).

## How it is checked

`elf easy`: sum section sizes per memory region from the linked ELF and compare against the region's size.

## Source

Ported from embedded_bedrock's firmware check catalogue, `SIZ-1` (`book/checks.md`, detail in `book/checks/siz.md`). Found on: internal firmware playbook; vb135 PLAT-7; portwright (99.6 %).
