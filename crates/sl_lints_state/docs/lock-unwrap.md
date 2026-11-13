## Why

Each `.lock().unwrap()` is a lock reached from more than one place, and a panic when another thread panicked while holding it (poison). In tpm's crates there were about 1,200 of them. A count per crate shows where shared state concentrates and whether a refactor is making it shrink.

The lint counts `.lock()`, `.read()` and `.write()` with no arguments followed by `.unwrap()` or `.expect(..)`, outside test code, and reports a crate whose count passes `max`, with the first places.

## How to fix

Give the state one owner (see [static-interior-mutability](static-interior-mutability.md)). Locks that stay are taken through one small helper that decides what poison means (recover with `into_inner`, or fail with a message), so the count goes to zero.
