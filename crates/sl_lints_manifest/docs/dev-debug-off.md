## Why

`cargo`'s default for `profile.dev` is full debug info, which it writes to every object file the edit-build-test
loop produces. A repo that turns debug info on only for the run that needs it (`CARGO_PROFILE_DEV_DEBUG=2` or
`line-tables-only`) builds and links faster the rest of the time, and a missing `debug` key quietly keeps the
slower default.

## How to fix

```toml
[profile.dev]
debug = false
```

A run that needs line numbers or full debug info sets the environment variable for that run instead of changing
the manifest: `CARGO_PROFILE_DEV_DEBUG=line-tables-only cargo test`.

A project that wants another value by default names it in `allow`, with the reason on the line above:

```toml
[lints.dev-debug-off]
level = "warn"
# this crate's debugger workflow needs file and line info on every build
allow = ["line-tables-only"]
```

With `debug` off, `profile-defaults`'s `split-debuginfo` check stops firing: there is no debug info to split.
