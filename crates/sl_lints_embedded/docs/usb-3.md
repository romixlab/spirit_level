## Why

A single-buffered bulk endpoint can't accept (or supply) the next packet while firmware is still handling the previous one, so there is dead time on the wire between packets; through a hub's Transaction Translator (TT) that dead time is far more expensive than directly on the host's own root port, because the TT serializes Full-Speed traffic from multiple downstream devices.

## Example

Typec_dongle's WireWeaver USB throughput (FW-24, on the G0's `usb_v4` peripheral) went from 373 to 664-676 kB/s host→device purely from switching to double-buffered bulk endpoints (`embassy-rs/embassy#7275`), alongside `opt-level = 3` and a protocol fast path — nearly double, from firmware-side buffering changes alone, with the same wire protocol and the same host.

## How to fix

Double-buffer (or use more than one buffer per direction) any bulk endpoint on a path that matters for throughput, especially one a hub TT sits in front of; chip: STM32 `usb_v4` parts (G0, among others) need this explicitly since the peripheral itself only gives one buffer per endpoint direction unless the double-buffer feature is used.

## How it is checked

Src easy — whether an endpoint is constructed with single or double buffering is a static property of the driver call, chip: STM32 `usb_v4`.

## Source

Ported from embedded_bedrock's firmware check catalogue, `USB-3` (`book/checks.md`, detail in `book/checks/usb.md`). Found on: portwright FW-24.
