## Why

A crate without a `license` (or `license-file`) cannot be reused, vendored or published, and licence checks such as `cargo deny check licenses` cannot judge it.

## How to fix

Add `license = "MIT OR Apache-2.0"` (or the project's licence) to `[package]`, or `license.workspace = true` in a workspace.
