# cargo_common_metadata

**Clippy lint:** Keywords and categories matter only for published crates; package-description covers the rest.

| | |
|---|---|
| Clippy's docs | [`cargo_common_metadata`](https://rust-lang.github.io/rust-clippy/master/index.html#cargo_common_metadata) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | keywords and categories matter only for published crates; package-description covers the rest | medium |
| [big](../tiers/big.md) | `allow` | keywords and categories matter only for published crates; package-description covers the rest | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cargo_common_metadata = "allow" # keywords and categories matter only for published crates; package-description covers the rest
```
