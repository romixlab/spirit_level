## Why

Every crate linked into the project brings its licence. `deny.toml`'s `[licenses]` lists the ones the project accepts; a crate under another licence, or under none, is reported.

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0"]
```

## How to fix

Replace the crate, or add its licence to `allow` when it is acceptable. A workspace crate states its own licence in `Cargo.toml`.
