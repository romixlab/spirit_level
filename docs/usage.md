# Usage records

Every run of spl can count which command and which flags it was given, in a small file on the machine that ran it. `spl usage` shows the counts for every part of the command line, the never-used ones included. The point is to find the parts nobody uses, so they can be removed.

Opt-in, off by default: nothing is recorded, and no directory is created, until it is enabled. Nothing leaves the machine either way. There is no network access, no upload and no identifier: the file holds counts per command and flag, by day.

## What is counted

One run gives one signal for its command and one for each flag it was given:

| The run | Its signals |
|---|---|
| `spl tiers` | `spl tiers` |
| `spl check --fix ../other_repo` | `spl check`, `spl check --fix` |
| `spl check --format json --only S0001` | `spl check`, `spl check --format=json`, `spl check --only` |
| `spl --color never tools needed --missing` | `spl tools needed`, `spl --color=never`, `spl tools needed --missing` |

- A flag counts when it is on the command line or set by its environment variable, such as `--today` through `SPIRIT_LEVEL_TODAY`. A default is not a use.
- A flag with a fixed set of values, such as `--format` or `--group-by`, counts per value.
- Paths, lint names, tier names and other free text are never written: only that the flag was given.
- `--help`, `--version` and a run with a usage error end before the record and are not counted.

Each run also records where it came from:

| Source | When |
|---|---|
| `agent` | inside an agent session: `CLAUDECODE` is set |
| `terminal` | standard input is a terminal |
| `script` | anything else: a recipe, a script, CI |

`VC_USAGE_SOURCE=terminal`, `script` or `agent` names the source instead, for a harness that runs spl on someone's behalf.

## Where the records are

Off by default.

| Setting | Directory |
|---|---|
| `SPL_USAGE_DIR=DIR` | `DIR` |
| `SPL_USAGE=on` | `$XDG_STATE_HOME/spl/usage`, by default `~/.local/state/spl/usage` |
| `SPL_USAGE_DIR=off` | none: recording is off, even with `SPL_USAGE=on` |
| neither set | none: recording is off |

The directory holds two files:

- `log.tsv`: one line per signal of each run, `day`, `source`, `signal`, separated by tabs. A run appends its lines with one write.
- `counts.tsv`: one line per signal with its runs from each source and the first and last day.

## Cost and bounds

Recording never fails or slows a run. A run appends a few short lines to the log; an error on the way (a full disk, a read-only home) loses that record and prints nothing.

The files stay small:

- When the log reaches 64 KiB (about a thousand runs), the run that finds it so folds it into the counts file under a lock. If another run holds the lock, the record is dropped, so the log never grows past the limit by more than one run.
- The counts file keeps at most 2,048 signals. When a new version adds more than that over time, the ones used longest ago go.
- A signal longer than 512 bytes is not recorded.

The limits are `LOG_BYTES_MAX`, `SIGNALS_MAX` and `SIGNAL_BYTES_MAX` of the `vc_usage` crate, which does the recording.

## Reading the counts

```bash
spl usage
spl usage --unused
spl usage --format json
```

`spl usage` lists every command and flag of this version of spl in the order of its help, with the runs in all and per source and the last day. One never used shows `0` and `never`. A signal from the records that this version no longer has comes last, marked `(not in this version)`.

`--unused` lists only the never-used ones. `--format json` prints the same rows as one document (`schema`, `dir`, `signals`), each with `signal`, `runs`, `terminal`, `script`, `agent`, `first`, `last` and `known`.

`spl usage` exits 2 when recording is off or the records cannot be read; off, it says how to enable it.

## For other CLIs

The recording comes from the `vc_usage` crate of [vhrd_cli](https://github.com/vhrdtech/vhrd_cli), so any clap CLI can keep the same records: `vc_usage::record(&Cli::command(), &matches, None)` right after parsing, and a `usage` command on `vc_usage::known`, `rows` and `text`. The signals come from clap's own command tree, so a new command or flag is counted, and listed as unused, without a line of its own.
