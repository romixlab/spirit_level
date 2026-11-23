# large_include_file

**Clippy lint:** An include_bytes! of a large file fills the flash unseen.

| | |
|---|---|
| Clippy's docs | [`large_include_file`](https://rust-lang.github.io/rust-clippy/master/index.html#large_include_file) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | not in the set |  |  |
| [big](../tiers/big.md) | not in the set |  |  |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | `warn` | an include_bytes! of a large file fills the flash unseen | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
large_include_file = "warn" # an include_bytes! of a large file fills the flash unseen
```
