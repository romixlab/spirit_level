# Configuration

A project's configuration is `spirit_level.toml` at the workspace root. It extends tiers and changes lints, each change with a reason.

```toml
extends = "big"                      # a shipped tier, a list, or a path to a shared .toml file
map-file = "crates.toml"             # the crate map, or inline as [map]

[lints.crate-lines]                  # an override
level = "deny"                       # allow, warn or deny
max-lines = 15000                    # a parameter of the lint
reason = "this repo splits early"    # required
expires = "2027-06-30"               # optional: a date, or a release such as "v0.5.0"

[[exceptions]]                       # an override for some crates or paths only
lint = "crate-lines"
crates = ["foo_sdk_types"]           # and/or paths = ["crates/foo/src/generated/**"]
max-lines = 120000
reason = "generated protocol types"
expires = "v0.4.0"
```

## Rules

- **extends** is a tier name (`spirit-level tiers` lists them), a list (`["small", "embedded"]`, later ones win), or a path to a `.toml` file of the same form, relative to the file that names it.
- **Every entry needs a `reason`**, in tier files too. An entry without one is an error.
- **Levels:** `allow` (off), `warn` (reported), `deny` (reported, fails the run). A lint that no extended tier enables needs a `level` when a project first sets it.
- **Parameters** are the lint's own keys, listed on its page with their type and default. An unknown parameter or a value of the wrong type is an error.
- **Expiries:** a date (expired from that day on) or a release (expired once the project's version, from `[workspace.package]` or `[package]`, reaches it). An expired override or exception is an error: fix what it allowed and remove it, or renew it with a new reason and expiry.
- **Exceptions** apply to findings about the listed crates or whose file matches a path pattern (`*` within a segment, `**` across segments). They change the level, parameters, or both. Their parameters apply to the crates they name.
- **Unknown keys fail.** A typo in a key, a lint id or a tier name stops the run with the file and line, before any lint runs.

## Seeing the result

`spirit-level config` prints every lint that is on, the level, where it was set and the reason, then every exception with its scope, reason and expiry.
