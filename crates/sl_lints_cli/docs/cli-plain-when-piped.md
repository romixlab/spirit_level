## Why

Colour escapes in a pipe or a file are noise: `grep` misses words, diffs show `^[[1m`, and scripts parse garbage. A tool colours a stream only when that stream is a terminal and `NO_COLOR` is not set.

## How to fix

Decide colour per stream: anstream does it for `print!` through its `stdout()`, clap does it for help and errors with `ColorChoice::Auto`, and vhrd_cli's `Paint::stdout()` and `Paint::stderr()` do it for painted text. Never force `ColorChoice::Always` as a default. The lint runs the help of every command, `--version` and a usage error with both streams on pipes, and reports the first escape it finds.
