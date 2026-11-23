# unimplemented

**Clippy lint:** No stop-gaps: an unimplemented!() is a panic waiting in shipped code.

| | |
|---|---|
| Clippy's docs | [`unimplemented`](https://rust-lang.github.io/rust-clippy/master/index.html#unimplemented) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | stop-gaps |
| [small-workspace](../tiers/small-workspace.md) | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | stop-gaps |
| [medium](../tiers/medium.md) | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | stop-gaps |
| [big](../tiers/big.md) | `warn` | no stop-gaps: an unimplemented!() is a panic waiting in shipped code | stop-gaps |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
unimplemented = "warn" # no stop-gaps: an unimplemented!() is a panic waiting in shipped code
```
