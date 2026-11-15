## Why

Every crate linked into the project brings its licence. `deny.toml`'s `[licenses]` lists the ones the project accepts; a crate under another licence, or under none, is reported.

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0"]
```

Without `deny.toml`, cargo-deny's default policy allows no licence at all, so every crate would be rejected; this lint reports that once, asking for the file, instead of once per crate.

## How to fix

Add `deny.toml` with the `[licenses]` section above. With one in place, replace the crate or add its licence to `allow` when it is acceptable. A workspace crate states its own licence in `Cargo.toml`.
