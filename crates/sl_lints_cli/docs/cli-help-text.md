## Why

`-h` is the map of a tool: one line per command and argument. An entry without a line forces the reader to guess or to open the source, and completion shows the same empty text beside the candidate. A help that exits non-zero, or is not in clap's layout, breaks scripts and the other help lints.

## Example

```text
Commands:
  sync   Bring the mirror up to date
  prune                         <- no help line

Options:
  -n, --name <NAME>             <- no help line
```

## How to fix

Give every subcommand and argument a doc comment (or `about` / `help`); the tool itself gets an `about`. clap's own notes (`[default: x]`, `[possible values: ...]`) do not count as help. A binary that is not a clap CLI is left out with `cli = false` in the crate map ([CLI checks](../cli.md)).
