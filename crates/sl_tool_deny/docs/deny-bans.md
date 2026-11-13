## Why

`deny.toml`'s `[bans]` names crates the project refuses (OpenSSL, a second async runtime) and how it treats a crate in two versions. Without a check the graph drifts back to them.

```toml
[bans]
multiple-versions = "deny"
deny = [{ name = "openssl", reason = "rustls only" }]
```

## How to fix

Remove the crate from the graph (`cargo tree -i NAME` shows who pulls it in), or change `[bans]` with a reason (`skip` for a duplicate that cannot be avoided yet).
