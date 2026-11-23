# large_stack_frames

**Clippy lint:** A large frame overflows a small stack with no guard page.

| | |
|---|---|
| Clippy's docs | [`large_stack_frames`](https://rust-lang.github.io/rust-clippy/master/index.html#large_stack_frames) |
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
| [embedded](../tiers/embedded.md) | `warn` | a large frame overflows a small stack with no guard page | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
large_stack_frames = "warn" # a large frame overflows a small stack with no guard page
```
