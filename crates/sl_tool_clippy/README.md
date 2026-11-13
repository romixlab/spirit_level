# sl_tool_clippy

The clippy adapter of spirit_level (layer lints): runs `cargo clippy --message-format json` and reports each finding under the lint of its clippy group (`clippy-style`, `clippy-pedantic`, ...); compiler warnings and errors go to `clippy-build`.

Clippy needs `rustc_private`, so it is never linked: the executable of the rustup component runs. The group of each clippy lint is read from the installed clippy (`clippy-driver -W help`), so the table follows the toolchain.

It may depend on `sl_core`, `sl_analysis` and `sl_tools`.
