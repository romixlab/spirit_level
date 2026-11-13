## Why

A recipe calls a program; it does not hold one. Logic in a shebang recipe or a long chain of shell lines is untested, unportable and invisible to the type checker. It belongs in Rust (a subcommand or a small tool), where it is tested, and the recipe calls it.

## How to fix

Move the logic into a command and let the recipe call it in a line or two. A recipe may have up to `max-lines` command lines; comment lines do not count.
