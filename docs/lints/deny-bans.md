# deny-bans

**Rule:** The dependency graph holds no banned crate and no duplicate deny.toml refuses.

| | |
|---|---|
| Category | [deny](index.md#deny) |
| Source | cargo-deny (run as an executable) |
| Fix | by hand |
| Needs | `cargo-deny` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `deny` | banned crates (OpenSSL, a second runtime) and duplicate versions stay out of a big graph |

## Why

`deny.toml`'s `[bans]` names crates the project refuses (OpenSSL, a second async runtime) and how it treats a crate in two versions. Without a check the graph drifts back to them.

```toml
[bans]
multiple-versions = "deny"
deny = [{ name = "openssl", reason = "rustls only" }]
```

## How to fix

Remove the crate from the graph (`cargo tree -i NAME` shows who pulls it in), or change `[bans]` with a reason (`skip` for a duplicate that cannot be avoided yet).

## Configure

```toml
[lints.deny-bans]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(deny-bans) <reason>` ([waivers](../waivers.md)).
