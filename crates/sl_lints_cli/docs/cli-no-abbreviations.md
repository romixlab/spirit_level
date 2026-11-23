## Why

`Command::infer_subcommands(true)` lets a caller type any unambiguous prefix of a subcommand and runs it. It feels convenient until a sibling command is added: a script or a habit that typed `tool rem` for `remove` now resolves to the new `rename`, or to nothing, with no warning that its meaning moved. A subcommand's name is a promise; an unstated prefix is not.

## Example

```console
$ tool rem --help   # today: unambiguous, runs `remove`
Remove a thing
...
$ tool add rename   # tomorrow: `tool rem` is now ambiguous between `remove` and `rename`
```

## How to fix

Don't call `Command::infer_subcommands(true)` (clap's builder API) or set `#[command(infer_subcommands = true)]` on a derive struct. Callers type the full subcommand name, or a stated alias (`visible_alias`), which stays put whatever else is added. The lint runs each subcommand's shortest unique prefix and flags it when that prefix's `-h` prints the subcommand's own help.
