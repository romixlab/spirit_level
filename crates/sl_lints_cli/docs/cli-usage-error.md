## Why

Scripts and agents tell a wrong call from a failed run by the exit code: 2 means "bad arguments, nothing ran", 1 a failure of the work itself. The message goes to stderr, so stdout carries only data and a caller that reads it sees nothing.

## Example

```console
$ tool --no-such-flag; echo $?
error: unexpected argument '--no-such-flag' found
...
2
```

## How to fix

Let clap report usage errors: `Cli::parse()` exits with 2 and writes to stderr. A tool that catches clap's error itself returns `ExitCode::from(2)` and prints with `eprintln!`. The lint runs the binary with an unknown flag; the parameter `code` is the expected exit code.
