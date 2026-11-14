# Dependents

Before a breaking change to the compiler, the Rust project runs crater: it builds and tests the crates that depend on it. `spl dependents` does the same in small for a library that other repos build on. It lists the repos checked out beside it that use the library, and in each the crates that reach a changed crate, with the command that retests them.

It lists; it does not judge. Nothing is built, nothing is written, and the run exits 0 whatever it finds.

## Turning it on

It is off until the library's `spirit_level.toml` names where its dependents are checked out:

```toml
[dependents]
scan = ["~/git"]                # folders whose subfolders are checkouts
repos = ["../io_weaver"]        # single checkouts
base = "main"                   # optional: what "changed" is measured against
test = "cargo nextest run"      # optional: the retest command; `-p CRATE` is appended per crate
```

| Key | Default | Meaning |
|---|---|---|
| `scan` | none | Folders whose direct subfolders are checkouts. A subfolder counts when it holds a `Cargo.lock`; hidden folders and the library itself are skipped. |
| `repos` | none | Single checkouts. A missing or unreadable `Cargo.lock` here is reported, not skipped. |
| `base` | the default branch | The revision changes are measured against. The default is the local branch `origin/HEAD` names (`main`, `master`), else `origin/HEAD` itself, else `main`. |
| `test` | `cargo nextest run` | The retest command. |

Paths are relative to the file the table is in; `~/` is the home directory. At least one of `scan` and `repos` is required. A tier never sets the table: dependents are a fact of one project.

Without the table, `spl dependents` prints that it is off and exits 0 (`{"enabled": false}` with `--format json`), so a script can run it on every repo.

## What changed

The changed crates are those with a file changed since the merge base of `base` and `HEAD`: commits on the branch, the working tree and untracked files, as `just test --changed` counts them. A file belongs to the crate whose directory holds it. A change to the root `Cargo.toml` counts every crate as changed, since workspace dependencies and features reach them all; other files outside the crates (docs, the library's own `Cargo.lock`) reach no dependent.

To the changed crates come the library's crates that depend on them through normal and build edges. Dev-dependencies do not carry a change to a dependent.

`--all` counts every crate as changed and needs no git: the list of everything that uses the library at all.

## Where the dependents come from

The checkouts on disk, read through their `Cargo.lock`. A lock is the resolved graph the dependent builds today, it is read without cargo or the network, and it holds the library's commit. A dependent whose lock names none of the library's crates is skipped; for one that does, `cargo metadata --no-deps` gives its own crates, also offline.

A list of repos kept by another tool (a `repos.toml` with URLs) is not read: spirit_level reads no other tool's format, scanning the folder finds the same checkouts and those no list names, and a repo that is not checked out cannot be retested anyway.

## Which crates

A crate of a dependent is listed when its lock graph reaches an affected crate of the library, directly or through other crates: its own workspace's crates, third-party crates or another library of the same family. The lock does not tell dev-dependencies apart, so a crate whose tests use the library is listed too.

For each crate the list names the library's crates it enters through (its `uses`), and when it uses none of them directly, its own dependencies the change reaches it through (`through`).

## The library's commit in a lock

Each dependent says where its lock takes the library from:

| Source | Shown |
|---|---|
| git | The locked commit against the tip of the branch the source follows (the library's local branch, else `origin/`), or against `base` when the source names no branch: the tip, N commits behind, or not on that branch. |
| path | The path a manifest gives. A path inside the library's checkout builds against the change as it is. A lock with a path no manifest names (a `[patch]` in a cargo config, or a stale lock) says so. |
| registry | The released version. |

A lock behind the tip means the dependent has not taken earlier changes either: its next `cargo update` brings them all, and a retest against the new change tests those too.

## Output

```text
$ spl dependents
lib against main: changed lib_io; depending on them in lib: lib_cli

app_a  ~/git/app_a
  lock: https://github.com/example/lib at a2e5cd0, 1 commit behind main: its tests run against an older library
  a_app   lib_io through a_core
  a_core  lib_io
  next: cd ~/git/app_a && cargo nextest run -p a_app -p a_core

Not affected (they use only other crates): app_b

2 crates to retest in 1 repo
```

`--format json` gives the same as one document (`"schema": "spirit_level.dependents/1"`): `changed`, `affected`, and per dependent in `retest` its `root`, `crates` (`name`, `uses`, `through`), `command` (run at `root`) and `sources` (`kind` git, path or registry, with `commit` and `state` current, behind or unknown for git). `unaffected` lists the dependents that use only other crates of the library, `errors` the checkouts that could not be read.

## Why a command and not a lint

- Its result is a list per repo with commands, not findings with a span and a level.
- It reads other repos and asks git about a base revision. `spl check` reads only the project, the same on every machine.
- A finding that is always there whenever a crate changed has no level that means anything.

## Using it as a gate

A landing gate runs `spl dependents --format json` on the library's branch. Where `retest` is not empty, it runs each `command` at `root` against the changed library, or holds a breaking change until each listed dependent was retested. Running the tests against the change needs the library patched into a scratch copy of each dependent; that is planned (`DEP-2` in FEATURES.md).
