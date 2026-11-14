## Why

People and agents learn a tool from its examples first. A long help that ends in a few real command lines answers "how do I call this" without a trip to a README. The guide asks for an `Examples:` block in the help of the tool and of every leaf command: a command that does the work, not a group of subcommands.

## Example

```text
$ tool task add --help
Add a task to a project
...
Examples:
  tool task add P1 "Order parts"
  tool task add P1 "Fix the build" --due 2026-12-01
```

## How to fix

End the long help with `Examples:` and a few lines (clap: `after_long_help`, or `after_help` to show them in `-h` too). Commands that need none go into the parameter `exempt` as paths without the binary's name: `"completions"`, `"* list"`. The lint runs `--help` of the built binary ([CLI checks](../cli.md)).
