# pub-surface

**Rule:** A library's public surface stays small for its size.

| | |
|---|---|
| Category | [api](index.md#api) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | off |  |
| [medium](../tiers/medium.md) | off |  |
| [big](../tiers/big.md) | `warn` | in a big workspace pub is the easy way across crates; a library that is all surface keeps nothing apart |
| [ai](../tiers/ai.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `max-per-kloc` | number | `50` | public items per 1000 code lines above which a library is reported |
| `min-lines` | integer | `2000` | libraries with fewer code lines are not checked |
| `skip` | list of strings | `[]` | libraries whose wide surface is their purpose (a types crate, bindings) |

## Why

Every public item of a library is a promise to its users: they may call it, so it can't change without looking at all of them. In a workspace, `pub` is often the easy way to let one crate reach into another, and the surface grows with every such reach. A library whose surface is large for its size has no inside left: everything is API, nothing can be changed alone, and the crate boundary does not keep anything apart.

The number is relative: public items per 1000 code lines. Small libraries are left out (`min-lines`), because a 300-line types crate is all surface by design.

Measured on tpm_cli's crates (November 2026): most sit between 15 and 45 per 1000 lines; tpm_base (60) and tpm_spec (54), crates made by moving modules out of the binary with everything left `pub`, stand out.

## What is counted

- Public items reachable from the crate root: `pub` items of modules reached through `pub mod`, `pub use` of a module and `pub use m::*`, items re-exported by name, exported `macro_rules!`.
- The `pub` methods and associated constants of the public types among them (inherent impls only; trait impls follow the trait).
- Code lines of the library's modules, test code left out.

The finding names the three modules with the most public items.

## Example

```text
warn[pub-surface] tpm_base offers 690 public items in 11475 code lines, 60 per 1000 (limit 50);
  most in core (167), ui (89), nodes (55)
```

## How to fix

- Make what only the crate uses `pub(crate)`: [dead-pub](dead-pub.md) lists the public items no other crate names.
- Keep modules private and re-export from the root only the items users need.
- Move a group of items that other crates do use into a crate of its own, with a surface made for them.

## Configure

```toml
[lints.pub-surface]
level = "deny"
max-per-kloc = 50
min-lines = 2000
skip = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(pub-surface) <reason>` ([waivers](../waivers.md)).
