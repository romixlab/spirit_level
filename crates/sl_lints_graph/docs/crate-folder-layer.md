## Why

A workspace past about a hundred crates is hard to browse as one flat `crates/` folder: the listing stops telling what the parts are. Folders by layer make the map visible in the tree, and a layer of about a hundred crates or more splits once more by discipline. The map decides the path, so the tree and the map never disagree:

| Workspace | Path |
|---|---|
| small | `crates/<crate>` |
| large | `crates/<layer>/<crate>` |
| a large layer | `crates/<layer>/<discipline>/<crate>` |

Never deeper: a third folder level is a sign the layers are cut wrong.

A flat workspace passes. Once any crate sits in a layer folder, or the map gives any crate a `discipline`, every mapped crate under the root sits in its layer folder.

```toml
[map.crates]
it_fem = { layer = "kernel" }                      # crates/kernel/it_fem
it_antenna = { layer = "engines", discipline = "rf" } # crates/engines/rf/it_antenna
```

## How to fix

Move the crate to the folder its map entry names, and update the `members` globs (`crates/*/*`, `crates/*/*/*`). When the folder is right and the map is wrong, fix the map. [crate-folder-size](S0006.md) says when a folder has grown too big.
