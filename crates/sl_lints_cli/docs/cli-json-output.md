## Why

A script that parses text output breaks when the wording changes. A machine-readable form (`--json`: one JSON value on stdout, JSONL for a stream) is an interface that stays. The guide asks for it on every command.

## How to fix

Add `--json` (or `--format json`) to every command that prints data. A command that prints nothing a script reads (a setup step, `completions`) goes into the parameter `exempt`. The lint is an estimate: it assumes every leaf command prints data, which the help cannot tell. It is off in the shipped tiers; a project turns it on once it lists its exempt commands.
