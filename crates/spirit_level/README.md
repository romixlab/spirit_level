# spirit_level

The library behind the `spirit-level` command, and the one place that knows every analysis.

- `analyses()`: every registered analysis (manifest, graph, size, code, state, prose, and the tool adapters enabled by cargo features).
- `TIERS`: the shipped tier files (`tiers/*.toml`), embedded at build time; `tier(name)` returns one.
- `check(dir, &Options)`: load the project and its `spirit_level.toml`, run, return the report.

A new analysis crate is added here with one line in `analyses()` and one dependency. The CLI (`spirit_level_cli`) and the book generator (`sl_book`) read everything through this crate.
