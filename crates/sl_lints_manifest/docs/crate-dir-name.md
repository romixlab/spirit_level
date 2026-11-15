## Why

The crate's own folder is the crate: search, completion and the crate table all assume the two names agree (matklad's rule for large workspaces). It holds at every depth: a flat `crates/<crate>`, a layer folder `crates/<layer>/<crate>` and a discipline folder `crates/<layer>/<discipline>/<crate>`. Which of those a crate sits in is [crate-folder-layer](G0011.md)'s check.

## How to fix

Rename the directory (and its `members` entry) to the package name.
