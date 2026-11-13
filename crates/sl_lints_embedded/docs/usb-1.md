## Why

A High-Speed bulk endpoint must advertise a 512-byte max packet size; a Full-Speed value (64) left over from porting an example, or a `bInterval` the speed doesn't allow, can work on one host's driver stack and misbehave on another's.

## Example

Vb135's USB-HS serial example needed exactly this fix on Windows (commit `c0d19b8`): "Fix USB HS example on Windows (needed to use 512B packet size). bInterval is also incorrectly set to 255 (reported in Linux as 11 max, though seem to work)." Linux's USB stack tolerated the wrong values; Windows did not.

## How to fix

Set the max packet size from the negotiated speed (512 for HS, 64 for FS) rather than a literal copied from an example, and keep `bInterval` inside the range the USB spec allows for the endpoint's speed and transfer type — check against a second host stack (Windows and Linux differ here), not just the one in daily use.

## How it is checked

Src mid — the descriptor values are static and the speed/type combination is known at build time, so a lint can check max packet size and bInterval range per advertised speed.

## Source

Ported from embedded_bedrock's firmware check catalogue, `USB-1` (`book/checks.md`, detail in `book/checks/usb.md`). Found on: vb135 c0d19b8 (Windows).
