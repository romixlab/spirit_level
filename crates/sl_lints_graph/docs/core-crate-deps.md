## Why

A `_core` crate holds the types others share: ids, data, errors, the vocabulary of a domain. To stay usable by a CLI, a server, a test or a wasm build it must not drag anything in: it depends only on other `_core` crates, on the layers named in `layers`, and on a short list of vocabulary crates (`serde`, `thiserror`).

## How to fix

Move what needs the dependency into the crate above the `_core` crate, or, when the dependency really is part of the types (a units crate, say), list it in `vocabulary` with an override reason.
