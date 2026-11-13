## Why

`HashMap` and `HashSet` with the default hasher are seeded randomly per process, so iteration order changes from run to run. In a crate that promises deterministic output, any loop over such a map that reaches an output (a file, a log, a hash) breaks it.

The lint applies to crates the map marks `deterministic = true` and to those listed in `also`, and reports `HashMap<K, V>` / `HashSet<T>` written without an explicit hasher type.

## How to fix

Use `BTreeMap` / `BTreeSet`, or a map with a fixed-seed hasher (`HashMap<K, V, FixedState>`), where the order can reach an output.
