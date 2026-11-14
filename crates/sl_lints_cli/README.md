# sl_lints_cli

The CLI lints. The analysis builds every CLI binary of the workspace with cargo and runs it in a scratch directory with `-h`, `--help`, `--version`, an unknown flag and `COMPLETE=<shell>`, for the tool and every subcommand. The lints read what it printed:

- `cli-help-examples`: `--help` of the tool and of every leaf command has an `Examples:` block.
- `cli-help-text`: every command and argument has a help line in `-h`.
- `cli-help-clean`: no rustdoc leftovers (code fences, intra-doc links) in the help.
- `cli-completion`: `COMPLETE=<shell> BIN` prints a registration for bash, zsh and fish.
- `cli-plain-when-piped`: no colour escapes when the output is a pipe.
- `cli-color-flag`: a global `--color auto|always|never`.
- `cli-usage-error`: an unknown flag exits with 2, on stderr only.
- `cli-version-line`: `--version` prints the name, version, SHA and commit time.
- `cli-json-output`: every leaf command takes `--json` or `--format` (an estimate).
- `cli-standard-flags`: short flags keep their standard meaning.

The book page `docs/cli.md` at the repository root holds the catalogue of practices these lints come from. Depends on `sl_analysis`, `sl_core`, `sl_model` and `sl_tools`.
