# sl_testkit

The fixture harness every analysis crate tests with (a dev-dependency only).

An analysis crate keeps one folder per lint under `tests/fixtures/<lint-id>/`, with at least a `pass/` and a `fail/` case. Each case is a small cargo project (its `Cargo.toml` has a `[workspace]` table, so it stands alone) and an `expect.toml`:

```toml
# optional: parameters for the lint in this case
params = { warn-above = 10 }

[[finding]]
at = "src/lib.rs:3"          # the primary place, file:line relative to the case
contains = "RefCell"         # optional: a piece of the message
```

`pass/` expects no finding. A case may bring its own `spirit_level.toml` (for a crate map); then that file is the configuration and `params` is not used. `check(&ANALYSIS, "tests/fixtures")` runs every case with only that lint enabled at `deny` and today fixed at 2026-11-13, and fails with a list of every difference. It also fails when a lint of the analysis has no `pass` and `fail` case.

A case with a `git.toml` runs on a temporary copy made into a git repository: every file but those in `untracked` is committed on `main`, then each `[[commit]]` is an empty commit with its `message` on a branch `work`. Lints that read the history or the tracked files test this way.

```toml
untracked = ["scratch.txt"]

[[commit]]
message = "Add the parser"
```

`BLESS=1` writes the findings it got into `expect.toml` (keeping `params`), for a new case.

An adapter of an external tool uses `check_tools` instead. Each case runs on a copy in the temporary directory (without `expect.toml` and `target/`), since the tool may write `Cargo.lock` or build into `target/`. A case whose lint cannot run because its tool, a toolchain, a target or a cargo feature is missing is skipped with a `skipped: <lint>/<case>: <reason>` line; the test is named `tool_fixtures`, and `.config/nextest.toml` shows its output even when it passes. A tool that ran and failed is a failure, never a skip. `SPIRIT_LEVEL_REQUIRE_TOOLS=1` turns skips into failures.
