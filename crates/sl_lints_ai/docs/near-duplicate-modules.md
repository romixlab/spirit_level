## Why

A rewrite of a module that keeps the old file (`parser.rs` and `parser_new.rs`) doubles the code to maintain and leaves callers on both.

Two files are near copies when each has at least `min-fns` functions with a near copy in the other (as `near-duplicate-fns` measures them, with its parameters) and those functions are at least `min-share` of all compared functions of both files.

## How to fix

Finish the rewrite: move the callers, delete the old module, or merge what both need into one.
