## Why

Short flags are muscle memory across tools: `-n` is a dry run, `-q` quiet, `-y` yes. A tool where `-n` means `--name` turns a habit into a mistake, and `-n` on a destructive command then runs it for real.

## Example

```text
Options:
  -n, --name <NAME>   <- -n stands for --dry-run
```

## How to fix

Give the flag another short form, or none. The parameter `pairs` lists each short flag and the long flag it stands for (`-n=--dry-run`); `-f` is not in the default list, because some tools use it for `--force` and others for `--format`.
