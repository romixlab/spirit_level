## Why

`defmt`/`defmt-rtt` in the dependency tree without `-Tdefmt.x` passed to the linker, so no `.defmt` section is produced.

Without `.defmt`, the interned format strings defmt needs to decode RTT output at the host side don't exist in the image; probe-rs (or any defmt decoder) either refuses to attach meaningfully or shows nothing recognisable, and the failure looks like "logging isn't working" rather than "the linker argument is missing".

## Example

Documented in book/issues.md as a recurring first-boot surprise on a freshly hand-edited `.cargo/config.toml`.

## How to fix

Add `-C link-arg=-Tdefmt.x` to the target's `rustflags` in `.cargo/config.toml`; the generator always includes it when defmt is selected.

## How it is checked

`cfg easy` (defmt present in `Cargo.toml` implies `-Tdefmt.x` must be in `rustflags`) plus `elf easy` (confirm `.defmt` actually exists in the linked output).

## Source

Ported from embedded_bedrock's firmware check catalogue, `MEM-12` (`book/checks.md`, detail in `book/checks/mem.md`). Found on: book/issues.md.
