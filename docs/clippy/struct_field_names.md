# struct_field_names

**Clippy lint:** A field named after its struct is often the clearest name.

| | |
|---|---|
| Clippy's docs | [`struct_field_names`](https://rust-lang.github.io/rust-clippy/master/index.html#struct_field_names) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | a field named after its struct is often the clearest name | medium |
| [big](../tiers/big.md) | `allow` | a field named after its struct is often the clearest name | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
struct_field_names = "allow" # a field named after its struct is often the clearest name
```
