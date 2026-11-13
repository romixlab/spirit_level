## Why

A cargo feature that enables nothing and that no code tests is a switch connected to nothing: left over from a removed experiment, or added "for later". It shows up in docs and `cargo add` and does nothing.

A feature is reported when its list is empty (`x = []`) and no source file of its crate holds `feature = "x"` (in `cfg`, `cfg_attr` or `cfg!`), and no other feature of the crate enables it. Features that enable dependencies or other features are aliases and pass.

```toml
[features]
fast-path = []      <- reported when no #[cfg(feature = "fast-path")] exists
```

## How to fix

Remove the feature, or gate the code it was meant for.
