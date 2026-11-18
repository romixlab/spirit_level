# let_underscore_must_use

**Clippy lint:** `let _ =` on a must_use value drops an error unseen.

| | |
|---|---|
| Clippy's docs | [`let_underscore_must_use`](https://rust-lang.github.io/rust-clippy/master/index.html#let_underscore_must_use) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| [medium](../tiers/medium.md) | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| [big](../tiers/big.md) | `warn` | `let _ =` on a must_use value drops an error unseen | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
let_underscore_must_use = "warn" # `let _ =` on a must_use value drops an error unseen
```
