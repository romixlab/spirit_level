# profile-defaults

**Rule:** The dev profile splits debug info and a deploy profile builds incrementally.

| | |
|---|---|
| Category | [repo](index.md#repo) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins |
| [small-workspace](../tiers/small-workspace.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins |
| [medium](../tiers/medium.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins |
| [big](../tiers/big.md) | `warn` | split debug info and an incremental deploy profile: measured large build-time wins |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `deploy-profile` | string | `"deploy"` | the profile `just install` builds with |

## Why

Measured on real projects: `split-debuginfo = "unpacked"` in the dev and test profiles took an edit-then-test build from 141 s to 11 s (test binaries stopped being I/O-bound), and an incremental deploy profile took a one-file deploy from 43 s to 15 s.

## How to fix

```toml
[profile.dev]
split-debuginfo = "unpacked"

[profile.test]
split-debuginfo = "unpacked"

[profile.deploy]
inherits = "release"
incremental = true
```

`just install` builds with `--profile deploy`. The deploy profile's name is the parameter `deploy-profile`.

## Configure

```toml
[lints.profile-defaults]
level = "deny"
deploy-profile = "deploy"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(profile-defaults) <reason>` ([waivers](../waivers.md)).
