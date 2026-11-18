# allow-print-in-tests

**`clippy.toml` setting:** A test may print what it saw.

| | |
|---|---|
| Clippy's docs | [`allow-print-in-tests`](https://doc.rust-lang.org/clippy/lint_configuration.html#allow-print-in-tests) |
| Written as | a key of `clippy.toml` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Value per tier

| Tier | Value | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `true` | a test may print what it saw | medium |
| [big](../tiers/big.md) | `true` | a test may print what it saw | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
# clippy.toml
allow-print-in-tests = true # a test may print what it saw
```
