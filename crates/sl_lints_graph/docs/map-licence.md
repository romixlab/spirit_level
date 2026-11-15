## Why

A workspace often ships under more than one licence: the bottom layers (helpers, contracts, numerics) permissive so others can build on them, the product layers under the project's own terms. Written into 400 manifests by hand, one crate drifts and nobody notices until a release review. The crate map already says which layer each crate is in; a `licence` per layer, and on a crate entry where it differs, makes the map the one place where licences are decided, and this lint holds every manifest to it.

```toml
[map.layers.utils]
allow = ["utils"]
licence = "MIT OR Apache-2.0"

[map.layers.app]
allow = ["utils", "app"]
licence = "AGPL-3.0-only"

[map.crates]
foo_ids = { layer = "utils" }
foo_cli = { layer = "app" }
foo_fonts = { layer = "app", licence = "MIT OR Apache-2.0" }
```

The licences are compared as written: the SPDX expression in the map and the `license` cargo reports for the crate, after workspace inheritance. A crate whose layer and entry set no licence is not checked; `package-license` still asks it to state one.

`workspace-inheritance` accepts a crate whose `license` is the one the map sets, so `[workspace.package].license` holds the licence of most crates and the others write their own without an override of `keys`. When every crate's licence comes from the map, `[workspace.package]` need not declare one.

## How to fix

Write the licence the map sets: `license.workspace = true` when it is the workspace's, else `license = "..."`. `spl check --fix` writes it. A crate whose licence is right and whose map entry is wrong: change its layer's `licence`, or give its entry a `licence` of its own.
