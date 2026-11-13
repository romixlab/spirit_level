## Why

A `.cargo/config.toml` runner or a `just flash` / `just run` recipe that calls `probe-rs run` or `probe-rs download` without pinning the probe by its VID:PID flashes whichever probe-rs happens to pick first. With one probe and one board on the PC nothing shows; plug in a second board for another firmware (or let a sibling session's probe sit attached) and the wrong board gets flashed, or reset mid-session.

## Example

The generated template's `.cargo/config.toml.j2` already gets this right and is the reference: its runner line is `probe-rs run --chip {chip} --probe {probe}{extra}`, with the comment "Only {probe_name} ({probe}), never another probe on the PC: one with someone else's board would be flashed." and a second note that `just run` / `just cnt` add the probe's own serial for a PC with two of the same board. A hand-written recipe that drops the `--probe` flag (copied from an older README, or from `cargo-flash` examples that assume a single probe) loses that guarantee silently.

## How to fix

Always carry `--probe VID:PID` (or the probe's serial, for two identical boards) through every runner and recipe; never rely on "there's only one probe right now."

## How it is checked

Check: cfg easy — a lint over `.cargo/config.toml` and the justfile's flash/run targets greps for `probe-rs (run|download)` without a following `--probe`.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CFG-1` (`book/checks.md`, detail in `book/checks/cfg.md`). Found on: nucleo_g0b1re_template; template conventions.
