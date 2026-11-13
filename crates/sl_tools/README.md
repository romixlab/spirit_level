# sl_tools

What spirit_level's tool adapters share (layer model):

- `tools.toml`: the pinned version of every external tool, the one table `spl tools install` reads.
- The tool cache: `cargo install --locked` builds keyed on the crate, its version and `rustc -vV`.
- `find`: a tool's executable, the cache first, then PATH.
- `run`: a tool started in the project with the outer cargo's environment removed.
- `cargo_json`: the compiler messages of `cargo ... --message-format json` as spans and codes.

It may depend on `sl_core` and `sl_model`.
