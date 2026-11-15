## Why

The crate map is the source of truth for a big workspace's structure: every crate's layer, what it may depend on, whether it may use `unsafe`. A crate missing from the map escapes every check that reads it; a map entry for a crate that is gone misleads every reader. (rerun's `lint.py` fails on a crate missing from its ARCHITECTURE.md; intent's map is checked the same way.)

A repository with a second workspace (its tools in `tools/`) lists those crates too, each with `workspace = "tools"`; the lint reads every workspace the map names ([Nested workspaces](../map.md#nested-workspaces)).

## How to fix

Add the new crate under `[map.crates]` with its layer (and `workspace` for a crate of a nested workspace), or remove the entry of a crate that no longer exists. A crate in the wrong workspace gets its entry's `workspace` key corrected.
