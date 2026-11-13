## Why

In a workspace, crates are a build unit, not a published API. With `publish = false` they can be split, renamed and merged freely, and `cargo publish` cannot push one by accident. The few crates that are published (a stable API) are listed in the parameter `published`.

## How to fix

Add `publish = false` under `[package]`, or list the crate in `published` when it really is released.
