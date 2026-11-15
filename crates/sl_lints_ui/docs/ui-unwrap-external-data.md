## Why

`panic-in-library` counts bare `unwrap` in library code, but a UI app's binary crate holds no library for it to check. The same file, parse or connection failures reach it directly: a missing config file, a stale cache that no longer deserialises, a server gone. Unwrapping there turns a bad environment into a crash, where a library would have returned a typed error.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates), a `panic-methods` call (`unwrap`, `expect`) at the end of a chain rooted in one of `calls`: a file read or write, a `serde`-style parse (`serde_json::from_str`, `toml::from_str`, ...), or a network connection (`reqwest::get`, `TcpStream::connect`, ...), resolved through the file's `use` items. A `?`, a reference or a method in between (`.map_err(..)`) does not clear the chain. Findings are estimates: the value may be known good by construction (a file baked into the binary) or already checked a few lines above.

## How to fix

Return the error with `?` from a function whose signature allows it, or match it and show the person what happened and what to do, the way `ui-debug-text` and the UX guide's error wording expect. Where the value truly cannot fail (a fixture, a constant parsed once at compile time), waive the finding with that reason.
