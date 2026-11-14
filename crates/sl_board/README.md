# sl_board

The board descriptions of a firmware repo as the board lints read them:

- `board.toml` files found by the `[board]` table's globs, read through `sl_board_model` (the schema's one loader: spans, revisions and reworks as patches);
- the chip's data from stm32-data's chip JSON (`<chip-data>/<CHIP>.json`): GPIOs, packages, which pins carry each peripheral signal;
- the MCU's pads and nets from each revision's PCB, in boardmap's geometry JSON.

A missing input is reported as such, never as a clean board.

Depends on `sl_board_model`, `sl_docs` (the file walk), `serde` and `serde_json`.
