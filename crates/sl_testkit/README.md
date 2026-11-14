# sl_testkit

The fixture harness every analysis crate tests with (a dev-dependency only).

## Cases

An analysis crate keeps one folder per lint under `tests/fixtures/<lint-id>/`, with at least a `pass/` and a `fail/` case. Each case is a small cargo project (its `Cargo.toml` has a `[workspace]` table, so it stands alone) and an `expect.toml`:

```toml
# optional: parameters for the lint in this case
params = { warn-above = 10 }

[[finding]]
at = "src/lib.rs:3"          # the primary place, file:line relative to the case
contains = "RefCell"         # optional: a piece of the message
confidence = "estimate"      # optional: estimate or candidate; absent means certain
```

`pass/` expects no finding. A case may bring its own `spirit_level.toml` (for a crate map); then that file is the configuration and `params` is not used. `check(&ANALYSIS, "tests/fixtures")` runs every case with only that lint enabled at `deny` and today fixed at 2026-11-13, and fails with a list of every difference. It also fails when a lint of the analysis has no `pass` and `fail` case.

## Cases with a history

A case with a `git.toml` runs on a temporary copy made into a git repository: every file but those in `untracked` is committed on `main`, then each `[[commit]]` is a commit on a branch `work` with its `message` (`{sha:N}` is the 12-hex hash of commit N, 0 the initial one) and the files of its `files` table (path and text) written and its `remove` list deleted (neither: an empty commit). Lints that read the history or the tracked files test this way.

```toml
untracked = ["scratch.txt"]

[[commit]]
message = "Add the parser"
```

## Shared projects and linked ELFs

Several cases may run one project: `project = "../../../projects/clean"` in `expect.toml` (relative to the case) names it, so a firmware shared by many lints is kept once under `tests/projects/`.

Lints that read a linked ELF get it through `check_with(&ANALYSIS, dir, &Hooks { elf: Some(&make) })`: `make` gets the case's project directory and returns the ELF's bytes (or None), the harness writes them to a temporary file and passes it as `--elf` would. A span in that ELF is written `<elf>` in `expect.toml`.

A case with `landing = true` in its `expect.toml` runs as a landing (`spl check --landing`).

`BLESS=1` writes the findings it got into `expect.toml` (keeping `params`), for a new case.

## Cases of external tools

An adapter of an external tool uses `check_tools` instead. Each case runs on a copy in the temporary directory (without `expect.toml` and `target/`), since the tool may write `Cargo.lock` or build into `target/`. A case whose lint cannot run because its tool, a toolchain, a target or a cargo feature is missing is skipped with a `skipped: <lint>/<case>: <reason>` line; the test is named `tool_fixtures`, and `.config/nextest.toml` shows its output even when it passes. A tool that ran and failed is a failure, never a skip. `SPIRIT_LEVEL_REQUIRE_TOOLS=1` turns skips into failures.

Tool cases never touch the network. The copy gets a `.cargo/config.toml` that replaces crates.io with `vendor/`, a cargo directory source of the few published crates the fixtures use, and turns cargo offline; a fixture that needs another crate adds it there (`cargo vendor`, then trim to `Cargo.toml`, the sources and the licences, with `"files": {}` in `.cargo-checksum.json`). A case that cannot run without the network (a fetched advisory database) says `network = true` in its `expect.toml`: it is skipped with a `skipped:` line unless `SPIRIT_LEVEL_NETWORK=1` is set, and then runs against crates.io. `CARGO_HOME=$(mktemp -d)` on a test binary shows a case that still reaches for the network.
