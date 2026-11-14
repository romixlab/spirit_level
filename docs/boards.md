# Board descriptions

A firmware repo describes each board it runs on in one typed `board.toml`: which MCU pin does what (GPIOs with their level from reset, peripherals with their signals, LEDs, buttons, reserved and joined pins), for every revision and rework of the board. The board lints (`brd-*`, in the [embedded](tiers/embedded.md) tier) hold each description against itself, against the chip's data and against the PCB's nets, on every configuration: each revision, each revision with each of its reworks, and with all of them.

The schema and its loader are spirit_level's crate `sl_board_model`; embedded_bedrock's book page `board-description.md` explains the file field by field.

## The lints

| Lint | Code | Holds | Level in `embedded` |
|---|---|---|---|
| [brd-1](lints/F0099.md) | F0099 | the description reads and follows the schema; the chip data and PCB it names read | deny |
| [brd-2](lints/F0100.md) | F0100 | every pin has one job in each configuration (`pin-twice`, `free-used`, `short-twice`) | deny |
| [brd-3](lints/F0101.md) | F0101 | of pins joined on the board, at most one is used (`short-both`) | deny |
| [brd-4](lints/F0102.md) | F0102 | pins, package, peripherals and signals exist on the chip (`pin-unknown`, `package-unknown`, `periph-unknown`, `signal-unknown`) | deny |
| [brd-5](lints/F0103.md) | F0103 | every signal sits on a pin that can carry it (`signal-pin`) | deny |
| [brd-6](lints/F0104.md) | F0104 | the pins are connected on the PCB as described (`pcb-pad`, `pcb-unconnected`, `pcb-net`, `pcb-short`, `pcb-shared`) | deny |
| [brd-7](lints/F0105.md) | F0105 | every connected MCU pin is described, and no `[free]` pin is unconnected (`pcb-undescribed`, `pcb-free-unconnected`) | warn |

Each finding names its check in the item and points at the line and column in `board.toml`; a finding about the PCB as a whole points at the `pcb` key of the configuration. A finding that holds in some configurations only says which (`(in rev A)`). The lints are grouped by what a project would waive or set apart: the description's own consistency, the chip, and the PCB, with coverage of the PCB a warning because a missing description entry is a fact still to write, not a fault.

## Configuration

```toml
# spirit_level.toml of a firmware repo
extends = ["small", "embedded"]

[board]
descriptions = ["boards/*/board.toml", "board.toml"]   # globs relative to the root (these are the defaults)
chip-data = "boards/chips"                             # one <CHIP>.json per chip (the default)
```

| Input | Where it comes from |
|---|---|
| The descriptions | the `descriptions` globs; nested cargo projects that are not workspace members are not searched |
| Chip data | stm32-data's chip JSON (`data/chips/<CHIP>.json` of embassy-rs/stm32-data-generated), committed under `chip-data`; the firmware template's generator caches the same files in `~/.cache/bedrock_gen/chips` |
| The PCB | `pcb = "..."` in the description's `[board]`, a revision or a rework, relative to the description: boardmap's geometry JSON (`boardmap extract` writes it from a `.kicad_pcb`), with the MCU named by `[board] mcu` |

Chip data and the PCB are read from the repo, never from the host, so a check gives the same result on every machine.

## Missing inputs

A missing input never passes silently. Without the chip data, or for a chip stm32-data does not cover, `brd-4` and `brd-5` do not run, and the report says why: `skipped: chip data missing: boards/chips/STM32H725IG.json is not there`. Without the PCB file, or with a `.kicad_pcb` instead of geometry JSON, `brd-6` and `brd-7` do not run the same way. A lint that does not run reports nothing for any description of the run, so a partial result is never shown as the whole. An input that is there but does not read is a `brd-1` finding at the key that names it.

## Where the model lives

The schema has one owner and one loader: spirit_level's `sl_board_model` (the schema, loading with spans, revisions and reworks as patches). It is the one crate of this workspace that is published, so that a generator of board code elsewhere reads the same file the same way; it depends on `serde` and `toml` only.

| Crate | Holds |
|---|---|
| `sl_board_model` | the schema and the loader; published |
| `sl_board` | finding the descriptions, stm32-data chip JSON, the MCU's nets from the PCB geometry |
| `sl_lints_board` | the `brd-*` lints |

- spirit_level owns the firmware checks, so the checks and the model they read live here: one `spl check` covers the board and the code, and the schema has no second loader to drift from.
- embedded_bedrock's generator (`bedrock-board gen`, which turns `board.toml` into a firmware's committed `board.rs`) takes `sl_board_model` from this repository's default branch until it is on crates.io.
- The model started in embedded_bedrock (`bedrock_board_model`, 0.3.1) with the checks as `bedrock-board check`; both moved here.

## What the description does not feed yet

The source lints that guess board facts, `pin-2` (safe state after the HAL init), `pin-8` (unused pins), `pin-9` (pin map by cargo feature) and `clk-2` (the time driver's timer), read the firmware's source and do not read `board.toml`. What they check is the init code and the timers the firmware uses, which the description does not state.
