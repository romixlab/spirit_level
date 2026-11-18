# exit

**Clippy lint:** Process::exit skips every destructor; main returns instead.

| | |
|---|---|
| Clippy's docs | [`exit`](https://rust-lang.github.io/rust-clippy/master/index.html#exit) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | process::exit skips every destructor; main returns instead | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | process::exit skips every destructor; main returns instead | small |
| [medium](../tiers/medium.md) | `warn` | process::exit skips every destructor; main returns instead | small |
| [big](../tiers/big.md) | `warn` | process::exit skips every destructor; main returns instead | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
exit = "warn" # process::exit skips every destructor; main returns instead
```
