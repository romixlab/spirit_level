# Getting started

## Install

```sh
cargo install --locked --git https://github.com/romixlab/spirit_level spirit_level_cli
```

## Pick a tier

| Project | Tier |
|---|---|
| one crate | `small` |
| a few crates in a workspace | `small-workspace` |
| tens of crates, internal crates, an MSRV | `medium` |
| a big workspace with declared layers | `big` (needs a [crate map](map.md)) |

```sh
spirit-level init --tier small-workspace   # writes spirit_level.toml
spirit-level check                         # runs it
spirit-level check --tier big              # tries another tier without editing the file
```

## Read a finding

```text
deny workspace-lints crates/foo/Cargo.toml
  foo does not inherit the workspace lints
  help: add `[lints]` with `workspace = true` to its Cargo.toml
  fix (--fix): add [lints] workspace = true
```

`spirit-level explain workspace-lints` prints the lint's page. `spirit-level check --fix` applies the exact fixes and checks again.

## Change what does not fit

Every change says why ([configuration](configuration.md)):

```toml
extends = "small-workspace"

[lints.duplicate-versions]
level = "allow"
reason = "syn 2 and 3 until clap moves; checked monthly"
expires = "2027-01-31"
```

A single finding is allowed where it is, with a [waiver](waivers.md).
