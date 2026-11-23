# allow-unwrap-in-tests

**`clippy.toml` setting:** A test that unwraps fails at the unwrap, which is what a test should do.

| | |
|---|---|
| Clippy's docs | [`allow-unwrap-in-tests`](https://doc.rust-lang.org/clippy/lint_configuration.html#allow-unwrap-in-tests) |
| Written as | a key of `clippy.toml` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Value per tier

| Tier | Value | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `true` | a test that unwraps fails at the unwrap, which is what a test should do | panics |
| [small-workspace](../tiers/small-workspace.md) | `true` | a test that unwraps fails at the unwrap, which is what a test should do | panics |
| [medium](../tiers/medium.md) | `true` | a test that unwraps fails at the unwrap, which is what a test should do | panics |
| [big](../tiers/big.md) | `true` | a test that unwraps fails at the unwrap, which is what a test should do | panics |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
# clippy.toml
allow-unwrap-in-tests = true # a test that unwraps fails at the unwrap, which is what a test should do
```
