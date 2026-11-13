## Why

`static mut` is global mutable state with no synchronisation at all: every access is `unsafe`, and since edition 2024 taking a reference to one is an error. There is always a better owner.

## How to fix

A counter or a flag is an atomic. Anything else gets one owner (see [static-interior-mutability](static-interior-mutability.md)).
