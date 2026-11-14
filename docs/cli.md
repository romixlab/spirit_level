# CLI checks

The `cli` lints check a command-line tool against the practices of the [CLI UX guide](https://github.com/vhrdtech/vhrd_cli/blob/main/docs/ux-guide.md) of vhrd_cli: help with examples, completion, `--version`, colour rules, machine-readable output and exit codes. They hold for any CLI built on [clap](https://docs.rs/clap); a CLI does not need vhrd_cli's crates to pass them.

## How spl sees a CLI

spl builds the CLI and runs it, the way a person or a script meets it. It reads what the tool prints, not how the source spells it. A derive struct, the builder API and commands that register themselves through linkme all look the same from outside, and the checks see help text that clap assembles at run time, such as a flattened struct's documentation.

For each CLI binary, spl runs `cargo build --message-format json -p CRATE --bins` in the project and takes the executables cargo reports. It then starts each binary only with arguments that print and exit:

| Probe | What it shows |
|---|---|
| `BIN -h` and `BIN --help` | the command tree, one line per item, the long help |
| `BIN SUB -h` and `BIN SUB --help` | the same for every subcommand listed under `Commands:`, down to `depth-max` levels |
| `BIN --version` | the version line |
| `BIN --spirit-level-unknown-flag` | the answer to a usage error |
| `COMPLETE=bash BIN --help` (and zsh, fish) | the completion registration; a CLI without completion prints its help instead |

Each probe runs with a clean environment: only `PATH`, `TERM=dumb`, and a `HOME` and XDG directories inside an empty scratch directory, which is also the working directory. Standard input is closed and both outputs are pipes. A probe that does not exit within `timeout-secs` is killed and reported. Help, version and a usage error never reach the tool's own code in a clap CLI, so the probes change nothing outside the scratch directory.

The probe reads clap's help layout: a `Usage:` line, then sections such as `Commands:`, `Arguments:` and `Options:` with one indented entry per item. A binary whose help has no `Usage:` line is not a clap CLI, and its help lints are reported as not run.

## Settings

The `[cli]` table of `spirit_level.toml` holds the settings every CLI lint shares. Each key is optional.

```toml
[cli]
timeout-secs = 10                  # one run of a tool is killed after this long
depth-max = 4                      # subcommand levels probed below the tool
commands-max = 1000                # commands probed per tool at most
build-args = ["--locked"]          # more arguments to `cargo build`
target-dir = "target/cli-probe"    # cargo's target directory, relative to the root (default: cargo's own)
commands-heading = "Commands:"     # the help heading under which clap lists subcommands
```

The values above are the defaults, except `build-args` and `target-dir`, which are empty by default.

## Which crates are CLIs

The crate map says it with the key `cli` ([the crate map](map.md)):

```toml
[map.crates]
foo_cli = { layer = "app", cli = true }    # every binary of the crate is checked
foo_bench = { layer = "app", cli = false } # its binaries are left out
```

Without the key, or without a map, a workspace crate with a binary target and a dependency on `clap` is a CLI. A project without a map is therefore checked too, and a crate whose binary is not meant for people (a test helper, a server) is left out with `cli = false`.

## vhrd_cli and any clap CLI

Every check holds for any clap CLI. vhrd_cli's crates make passing them cheap:

| Crate | What it gives | The lints it satisfies |
|---|---|---|
| `vc_build` | `--version` with SHA and commit time from a worktree-safe build script | `cli-version-line` |
| `vc_ui` | a global `--color`, colour off without a terminal, the error line, the exit codes | `cli-color-flag`, `cli-plain-when-piped`, `cli-usage-error` |
| `vc_complete` | dynamic completion for bash, zsh and fish, quiet on errors | `cli-completion` |

A CLI with its own build script or completion adapter passes the same checks when it prints the same thing. vhrd_cli's `repo_checks` crate checks vhrd_cli's own documents (the guide's path, sections and links); it looks at no CLI, so nothing here repeats it.

## The catalogue

One row per practice of the guide: the section that states it, the lint that checks it, and how sure that lint is. `by review` means no lint can check the practice honestly; a reviewer checks it. Certain lints compare the tool's output with a rule. Estimate lints rest on an assumption that their message states.

| Practice | Guide | Checked by | How sure |
|---|---|---|---|
| A cut list names what it left out and the flag that shows the rest | §2 | by review | |
| A disabled action stays listed with its reason | §2 | by review | |
| Raw input is calibrated by measuring, then confirmed | §2 | by review | |
| A step says why it runs; ids are shown with their names | §2 | by review | |
| Commands are `noun verb`, with the same verbs for every noun | §3 | by review | |
| Flags over positional arguments | §3 | by review | |
| Short flags keep their standard meaning (`-n` is `--dry-run`) | §3 | [cli-standard-flags](lints/C0029.md) | certain |
| Global flags work before and after the subcommand | §3 | [cli-color-flag](lints/C0025.md) for `--color`; clap for the rest | certain |
| No prefix abbreviations of subcommands | §3 | by review | |
| Secrets never come from a flag value | §3 | by review | |
| Plugins live on `PATH` and show in help and completion | §3 | by review | |
| Commands register themselves per module, with a registry test | §3 | by review | |
| The CLI is a thin crate over a library | §3 | [third-party-placement](lints/G0009.md), [forbidden-closure](lints/G0006.md), [cli-subprocess](lints/C0003.md) | certain |
| `-h` gives one line per item; every command and argument has help | §4 | [cli-help-text](lints/C0021.md) | certain |
| `--help` of the tool and of every leaf command ends in `Examples:` | §4 | [cli-help-examples](lints/C0020.md) | certain |
| Help text is prose for the terminal, without rustdoc leftovers | §4 | [cli-help-clean](lints/C0022.md) | certain |
| `-h` works anywhere on the line | §4 | clap | |
| A `next:` hint goes to stderr when stdout is not a terminal | §4 | by review | |
| A typo gets a suggestion and is not run | §4 | clap's `suggestions` feature | |
| Completion for bash, zsh and fish, registered from the binary | §5 | [cli-completion](lints/C0023.md) | certain |
| Completion offers live values, each with a help text | §5 | by review | |
| Completion never falls back to file names (`-o bashdefault`) | §5 | [cli-completion](lints/C0023.md) | certain |
| Slow values come from a cache; plugins answer a probe; a picker on Tab | §5 | by review | |
| stdout carries the data, stderr everything else | §6 | [print-in-library](lints/C0006.md) keeps printing in the binary; the rest by review | |
| Colour is off when the output is not a terminal | §6 | [cli-plain-when-piped](lints/C0024.md) | certain |
| A global `--color auto\|always\|never` | §6 | [cli-color-flag](lints/C0025.md) | certain |
| `NO_COLOR` turns colour off | §6 | by review: a pipe already turns it off | |
| One style set, few colours, red only for errors | §6 | by review | |
| Column widths are measured on the visible text | §6 | by review | |
| Short, line-oriented output; dates as `7 Oct 2026` | §6 | by review | |
| Every command has a machine-readable form | §6 | [cli-json-output](lints/C0028.md) | estimate |
| JSON carries a version field; outside strings are escaped | §6 | by review | |
| A frame and a pager only at a terminal | §6 | by review | |
| An error is one stderr line that names the tool and what to do | §7 | by review | |
| No `unwrap` on outside data | §7 | [panic-in-library](lints/C0005.md) in libraries; by review in the binary | certain |
| A failed step shows its last lines and the log path | §7 | by review | |
| A usage error exits with 2, on stderr only | §7 | [cli-usage-error](lints/C0026.md) | certain |
| Exit codes 3-63 are documented; 75 and 78 keep their meaning | §7 | by review | |
| Under JSON output an error is one JSON line on stderr | §7 | by review | |
| Progress for any step over a few seconds | §8 | by review | |
| Deploys read like an installer; transfers send only what changed | §8 | by review | |
| Ctrl-C answers at once; network calls time out | §8 | by review | |
| Prompts only at a terminal, each with a flag; `--dry-run` changes nothing | §9 | by review | |
| Layered settings; every tunable is a typed setting | §10 | by review | |
| `--version` prints `tool 0.4.2 (a1b2c3d-dirty, commit 3 Oct 2026 18:20 UTC)` | §11 | [cli-version-line](lints/C0027.md) | certain |
| The version comes from a worktree-safe build script without a clock | §11 | [cli-version-line](lints/C0027.md) catches `unknown` from a git checkout | certain |
| Libraries read build info at run time | | [build-info-env](lints/C0014.md) | certain |
| Every invocation records its usage per subcommand | §11 | by review | |
| Every learnable feature lands in the skill tree | §11 | by review | |
| Tests cover help, completion, colour, exit codes and JSON | §12 | by review | |

## Levels

The shipped tiers enable the CLI lints from `small` on, because a single crate is often a CLI. The lints that fail rarely and point at a plain defect (`cli-help-clean`, `cli-usage-error`, `cli-plain-when-piped`) deny from `medium` on. The lints that ask for work in most existing CLIs (examples, help lines, completion, `--color`, the version line, standard flags) warn. `cli-json-output` is off: which commands print data is a judgement its estimate cannot make for every tool.
