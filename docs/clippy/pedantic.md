# pedantic

**Clippy group:** The stricter idioms; the opt-outs below are the ones that are noise for most code.

| | |
|---|---|
| Clippy's docs | [`pedantic`](https://doc.rust-lang.org/clippy/lints.html#pedantic) |
| Written as | a group in `[workspace.lints.clippy]`, with `priority = -1` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | the stricter idioms; the opt-outs below are the ones that are noise for most code | medium |
| [big](../tiers/big.md) | `warn` | the stricter idioms; the opt-outs below are the ones that are noise for most code | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 } # the stricter idioms; the opt-outs below are the ones that are noise for most code
```
