## Why

Some crates must never end up in a binary, directly or through others: a UI process that must not link a 3D renderer, a root-run tool that must stay small, a library that must not pull a CLI framework. Edge checks alone cannot see this, because the forbidden crate arrives three hops away. The lint walks what the crate builds, features resolved for it alone ([what each crate builds](../configuration.md#what-each-crate-builds)), over normal and build edges plus its own dev-dependencies, and reports each forbidden crate it reaches, with the path.

## How to fix

Cut the edge on the path that brings it in, or move the part that needs it into a crate the binary does not link.
