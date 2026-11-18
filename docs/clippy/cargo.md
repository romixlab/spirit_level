# cargo

**Clippy group:** Manifest hygiene: feature names, wildcard versions.

| | |
|---|---|
| Clippy's docs | [`cargo`](https://doc.rust-lang.org/clippy/lints.html#cargo) |
| Written as | a group in `[workspace.lints.clippy]`, with `priority = -1` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | manifest hygiene: feature names, wildcard versions | medium |
| [big](../tiers/big.md) | `warn` | manifest hygiene: feature names, wildcard versions | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
cargo = { level = "warn", priority = -1 } # manifest hygiene: feature names, wildcard versions
```
