## Why

In a flat `crates/<name>` layout the folder is the crate: search, completion and the crate table all assume the two names agree (matklad's rule for large workspaces).

## How to fix

Rename the directory (and its `members` entry) to the package name.
