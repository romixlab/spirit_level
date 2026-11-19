## Why

A binary's name turns up as a systemd unit, a log tag, a shell command and a crate name on crates.io; a dash in one place and an underscore in another makes these inconsistent for no reason. This lint checks every binary of the checked crates against a set of allowed characters, lowercase ASCII letters and digits by default: an explicit `[[bin]]`'s `name`, `src/main.rs` (which takes the package name), and `src/bin/*.rs` or `src/bin/*/main.rs` (which take the file or folder name).

## Example

```toml
[[bin]]
name = "cs-noded"
path = "src/bin/main.rs"
```

fails with the default `chars` (`a-z0-9`); `csnoded` passes.

## How to fix

Rename it: an explicit `[[bin]]` takes a `name = "..."`; `src/main.rs` takes the package name, so rename the package; `src/bin/*.rs` and `src/bin/*/main.rs` take the file or folder name, so rename that file or folder. A project that wants a wider set (say, dashes) sets the `chars` parameter.
