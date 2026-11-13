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
