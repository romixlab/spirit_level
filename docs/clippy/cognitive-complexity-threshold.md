# cognitive-complexity-threshold

**`clippy.toml` setting:** Set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths.

| | |
|---|---|
| Clippy's docs | [`cognitive-complexity-threshold`](https://doc.rust-lang.org/clippy/lint_configuration.html#cognitive-complexity-threshold) |
| Written as | a key of `clippy.toml` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Value per tier

| Tier | Value | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `20` | set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths | medium |
| [big](../tiers/big.md) | `20` | set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths | medium |
| [ai](../tiers/ai.md) | `20` | set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
# clippy.toml
cognitive-complexity-threshold = 20 # set on samples: 17 read fine, 21 and 25 were too long; spl's function-complexity (cyclomatic, 15) counts paths
```
