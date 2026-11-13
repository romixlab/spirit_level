## Why

A source file of thousands of lines holds several subjects; reviews, merges and readers suffer first. tpm_cli's `main.rs` of 8,700 lines changed in 67% of all landings because it held every command's clap types.

## How to fix

Split the module by subject: a type and its impls, a command, a parser each in a file of its own.
