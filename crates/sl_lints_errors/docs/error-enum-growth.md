## Why

An error enum starts with three variants. Each new failure adds one: `ConfigMissing`, `ConfigUnreadable`, `ConfigBadKey`, `SyncTimeout`, `SyncRefused`... After a year it has forty, every caller's `match` lists them all or gives up with `_`, and the message of the lower error (the `io::Error` with its path, the parser's line) was turned into a string on the way.

The fix is structure: a variant per *source*, wrapping that source's own error (`Config(ConfigError)`, `Io(std::io::Error)`), so each layer owns its failures and the chain stays intact for `source()`. An enum with many variants and no such structure is the sign that this has not happened.

## What is checked

- Error enums: an enum whose name ends in `Error`, that derives `Error` (thiserror), or that the crate implements `std::error::Error` for.
- Structure: at least one variant wraps another error (a field whose type names an `...Error` type, a `#[from]` or `#[source]` field, `#[error(transparent)]`) or groups variants (a field whose type is another enum of the crate).
- Reported: an error enum with more than `max-variants` variants and no structure. The metric is the variant count. Test code is left out.

## Example

```rust
pub enum SyncError {          // 14 flat variants: reported
    ConfigMissing,
    ConfigUnreadable(String),
    PeerTimeout,
    PeerRefused,
    // ...
}
```

## How to fix

```rust
pub enum SyncError {
    Config(ConfigError),        // the config layer's own enum
    Peer(PeerError),
    Io(std::io::Error),
}
```

Group the variants by where they come from, give each group its own enum in the module that fails that way, and wrap it. Callers match on the area and look deeper only where they act differently.
