## Why

Choosing a board variant's pin map with a `cfg` feature means the binary itself only knows how to drive one variant's wiring; flash the wrong variant's build onto a board and every pin the two variants disagree about gets driven as if it were the other board — a feature that's merely absent on one variant can become a feature actively driving current into whatever *is* there on that variant instead.

## Example

RePort's SD-card design (internal `sd_card.md` notes, board record in flash, PLAT-15) chose the opposite of a build-time feature: the pin map (whether SD lines exist and what they're wired to, `SdPinout`) is read from a board record in flash/OTP at run time, so a single binary is safe to flash onto every board — a board with no SD rework and a blank record defaults to `SdPinout::B200A` (as built) and keeps its SD-capable lines as plain inputs, answering `NoMedium` regardless of which build is on it, so "the wrong build on the wrong board" simply can't drive a line the physical board doesn't have wired that way.

## How to fix

Read the pin map from a board record (flash, OTP, or at minimum a strap read at boot) rather than from a cargo feature baked into the binary, and default unconfigured/unknown boards to every variant pin set as a safe input, never as an assumed output.

## How it is checked

Check: src mid — recognizing "this pin map comes from a `cfg`-gated constant, not a runtime read" needs the firmware model to trace where the pin table's values originate.

## Source

Ported from embedded_bedrock's firmware check catalogue, `PIN-9` (`book/checks.md`, detail in `book/checks/pin.md`). Found on: sd_card.md design notes.
