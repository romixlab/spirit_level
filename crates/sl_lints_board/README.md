# sl_lints_board

The board lints `brd-*`: a firmware repo's board descriptions (`board.toml`, read through `sl_board`) checked on every revision, every revision with each of its reworks and with all of them:

- the description itself: it reads and follows the schema (`brd-1`), every pin has one job (`brd-2`), of pins joined on the board at most one is used (`brd-3`);
- against the chip's data (stm32-data): pins, package, peripherals and signals exist (`brd-4`), every signal sits on a pin that can carry it (`brd-5`);
- against the PCB's nets: the pins are connected as described (`brd-6`), every connected pin is described (`brd-7`).

The lints that need chip data or a PCB report themselves as not run, with the reason, when it is missing. The group's page in the book is `docs/boards.md`.

Depends on `sl_analysis`, `sl_board` and `sl_core`.
