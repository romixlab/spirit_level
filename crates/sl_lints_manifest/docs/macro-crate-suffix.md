## Why

`_macros` and `_derive` are reserved for proc-macro crates (Bevy, rerun): a reader knows from the name that the crate runs at compile time and has a compile-fail test crate beside it. A regular crate with such a name, or a proc-macro crate without one, breaks that.

## How to fix

Rename the crate.
