# sl_lints_graph

The lints that hold a workspace to its crate map (`[map]` in `spirit_level.toml`): every member is in the map and the map has no crate that is gone, every normal and build edge of the real cargo graph is allowed by the layers and the declared deps, binaries never reach their forbidden crates, external git crates have the declared form and role, heavy third-party crates stay in their crates, `_core` crates hold only types, only listed crates use `unsafe`, and the crate table in the README is the generated one. Without a map, it still finds cycles among members.

Depends on `sl_analysis`, `sl_config` (the map types), `sl_core`, `sl_model` and `syn`.
