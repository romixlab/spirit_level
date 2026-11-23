# max-fn-params-bools

**`clippy.toml` setting:** Two bool parameters are a call site nobody can read; an enum or a struct says what each means.

| | |
|---|---|
| Clippy's docs | [`max-fn-params-bools`](https://doc.rust-lang.org/clippy/lint_configuration.html#max-fn-params-bools) |
| Written as | a key of `clippy.toml` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Value per tier

| Tier | Value | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `1` | two bool parameters are a call site nobody can read; an enum or a struct says what each means | clippy-pedantic |
| [big](../tiers/big.md) | `1` | two bool parameters are a call site nobody can read; an enum or a struct says what each means | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
# clippy.toml
max-fn-params-bools = 1 # two bool parameters are a call site nobody can read; an enum or a struct says what each means
```
