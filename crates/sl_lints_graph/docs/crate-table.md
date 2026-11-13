## Why

A big workspace keeps a table of its crates (name, layer, one-line description) where readers look first. Written by hand it drifts; generated, it is always the truth. Each crate's description is written once, in its `Cargo.toml`, and its layer in the map (rerun generates its ARCHITECTURE.md tables from `cargo metadata` the same way).

The table sits in the file named by `file`, between two marker lines:

```markdown
<!-- spirit-level:crates -->
| Crate | Layer | Description |
|---|---|---|
| [`foo_core`](crates/foo_core) | model | Types of foo: ids, units, errors |
<!-- /spirit-level:crates -->
```

Crates are ordered by their layer's place in the map, then by name.

## How to fix

`spl check --fix` regenerates the table. To change a row, edit the crate's `description` or its map entry.
