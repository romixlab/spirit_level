# taplo-fmt

**Rule:** Every TOML file parses and is formatted as taplo writes it.

| | |
|---|---|
| Category | [taplo](index.md#taplo) |
| Source | taplo (run as an executable) |
| Fix | exact: `spirit-level check --fix` applies it |
| Needs | `taplo` installed |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change |
| [medium](../tiers/medium.md) | `warn` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change |
| [big](../tiers/big.md) | `deny` | manifests are edited by people and by tools; one TOML formatting keeps their diffs about the change |

## Why

Manifests and configuration files are edited by people and by tools (`cargo add`, `cargo hakari`); one formatting keeps their diffs about the change. taplo formats TOML by the project's `taplo.toml` (or its defaults) and reports files that do not parse.

```toml
# taplo.toml
[formatting]
align_entries = false
column_width = 120
```

## How to fix

`spirit-level check --fix` writes each file as `taplo fmt` formats it. A file that does not parse is fixed by hand at the place reported.

## Configure

```toml
[lints.taplo-fmt]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(taplo-fmt) <reason>` ([waivers](../waivers.md)).
