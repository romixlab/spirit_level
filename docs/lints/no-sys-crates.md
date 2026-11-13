# no-sys-crates

**Rule:** The dependency graph has no -sys crates and no C build steps.

| | |
|---|---|
| Category | [dependencies](index.md#dependencies) |
| Source | spirit_level |
| Fix | by hand |
| Needs | the resolved dependency graph (full `cargo metadata`) |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason |
| [small-workspace](../tiers/small-workspace.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason |
| [medium](../tiers/medium.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason |
| [big](../tiers/big.md) | `deny` | pure-Rust crates only: no bundled C, no -sys bindings without a written reason |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `allow` | list of strings | `[]` | crates allowed anyway, each named with its reason in AGENTS.md |

## Why

C libraries bring memory bugs Rust cannot see and builds that need a C toolchain, system libraries and cross-compilers (static musl, Windows, embedded). The house rule is pure-Rust crates: rustls not OpenSSL, gix not git2.

The lint reads the resolved graph (normal, build and dev edges of every member) and reports crates that are a C build step (`cc`, `bindgen`, `cmake`, `pkg-config`, `vcpkg`, `autotools`) or link a native library (`links` in their manifest). Pure-Rust binding crates such as `windows-sys` declare neither and are not reported.

## How to fix

Find who pulls it with `cargo tree -i NAME -e normal,build`, and pick a pure-Rust crate or turn off the feature that pulls it. When it must stay, list it in the parameter `allow` and write the reason in AGENTS.md.

## Configure

```toml
[lints.no-sys-crates]
level = "deny"
allow = []
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(no-sys-crates) <reason>` ([waivers](../waivers.md)).
