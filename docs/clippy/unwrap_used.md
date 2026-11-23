# unwrap_used

**Clippy lint:** A panic says what failed (expect with a message), or the error is returned.

| | |
|---|---|
| Clippy's docs | [`unwrap_used`](https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | a panic says what failed (expect with a message), or the error is returned | panics |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a panic says what failed (expect with a message), or the error is returned | panics |
| [medium](../tiers/medium.md) | `warn` | a panic says what failed (expect with a message), or the error is returned | panics |
| [big](../tiers/big.md) | `warn` | a panic says what failed (expect with a message), or the error is returned | panics |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
unwrap_used = "warn" # a panic says what failed (expect with a message), or the error is returned
```
