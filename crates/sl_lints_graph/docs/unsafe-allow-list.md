## Why

`unsafe` code is where memory bugs live. The workspace forbids it (`unsafe_code = "forbid"` in `[workspace.lints.rust]`), and only the crates the map marks `unsafe = true` override that, each a small crate with a safe API around its unsafe core. The lint holds both sides: the forbid is set, every crate with an `unsafe` block, fn, impl or trait (or an `allow(unsafe_code)`) is on the list, and every crate on the list still has unsafe code, so the list never grows stale.

## How to fix

Remove the unsafe code, or move it into a crate the map marks unsafe; drop `unsafe = true` from a crate that no longer needs it.
