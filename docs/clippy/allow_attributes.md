# allow_attributes

**Clippy lint:** #[expect] fails once the lint stops firing, so an exception cannot outlive its cause.

| | |
|---|---|
| Clippy's docs | [`allow_attributes`](https://rust-lang.github.io/rust-clippy/master/index.html#allow_attributes) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | #[expect] fails once the lint stops firing, so an exception cannot outlive its cause | medium |
| [big](../tiers/big.md) | `warn` | #[expect] fails once the lint stops firing, so an exception cannot outlive its cause | medium |
| [ai](../tiers/ai.md) | `warn` | #[expect] fails once the lint stops firing, so an exception cannot outlive its cause | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
allow_attributes = "warn" # #[expect] fails once the lint stops firing, so an exception cannot outlive its cause
```
