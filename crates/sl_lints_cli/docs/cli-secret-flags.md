## Why

A value passed on the command line sits in the shell history and in `ps` output for every user on the box. A secret taken that way (`--token abc123`, `--password hunter2`) leaks twice over. The right shape reads it from a file, stdin or an environment variable instead.

An option whose long name matches a word in `words` (`token`, `password`, `secret`, `api-key`) and takes a value is flagged, unless the next word in its name is in `safe-suffixes` (`file`, `path`, `env`, `stdin`): `--token-file <PATH>` points at where the secret lives, not at the secret itself, and passes.

```text
--token <TOKEN>        <- reported: the token itself as the value
--db-password <PASS>   <- reported
--token-file <PATH>    <- passes: the flag's value is a path
```

The lint is a candidate: a flag's long name is a guess at what it holds, so it never fails a run at any level and a person reads each hit.

## How to fix

Take the secret from `--token-file <PATH>` (read the file) or from stdin, or read it straight from an environment variable, and drop the flag that takes the secret as a value. List a flag in `safe-suffixes` when its project uses another indirection word for the same pattern.
