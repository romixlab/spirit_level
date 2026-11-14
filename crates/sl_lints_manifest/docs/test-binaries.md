## Why

Every file directly under `tests/` is an integration-test binary of its own: each links the library and its dependencies again and instantiates their generics again. Three files mean three links per test run. One binary with a module per file links once.

## How to fix

```text
tests/it/main.rs     mod parse; mod render;
tests/it/parse.rs
tests/it/render.rs
```

Cargo builds `tests/it/main.rs` as one binary named `it`; files in subdirectories of `tests/` are not binaries of their own. Tests that assume a process to themselves (a global, the working directory, environment variables) need that assumption removed first. A crate that needs several binaries, such as a harness with `harness = false` beside ordinary tests, raises the parameter `max-binaries` in an exception for that crate.
