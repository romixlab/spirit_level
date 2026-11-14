# sl_lints_embedded

The lints on bare-metal firmware: every automatic check of embedded_bedrock's firmware check catalogue (bedrock_lint), with its id in kebab-case (`EXE-3` is `exe-3`) and its category (`exe`, `irq`, `stk`, ...), plus `firmware-readable` for the files the model could not read.

One analysis, `EMBEDDED`, reads each firmware crate of the project once through `sl_firmware` and runs the enabled checks on it. A check is one file under `src/checks/` with its `lint!` declaration, what it needs from a crate (sources, build script, memory.x, regions, chip, ELF, Thumb code) and its `run` function; a crate without what a check needs is not checked by it. The lints that read the linked ELF need `--elf`.

Each lint's page is `docs/<id>.md`. The fixtures run the firmware projects under `tests/projects/` (bedrock_lint's fixtures); a project's `elf.toml` is written as its ELF by the test. The tier is `tiers/embedded.toml` (repo root); the book's `embedded-migration.md` maps bedrock_lint to it.
