# needless_raw_string_hashes

**Clippy lint:** R#"..."# keeps a string editable when a quote is added.

| | |
|---|---|
| Clippy's docs | [`needless_raw_string_hashes`](https://rust-lang.github.io/rust-clippy/master/index.html#needless_raw_string_hashes) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | r#"..."# keeps a string editable when a quote is added | medium |
| [big](../tiers/big.md) | `allow` | r#"..."# keeps a string editable when a quote is added | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
needless_raw_string_hashes = "allow" # r#"..."# keeps a string editable when a quote is added
```
