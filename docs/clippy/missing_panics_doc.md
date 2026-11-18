# missing_panics_doc

**Clippy lint:** A public function that can panic says when in a # Panics section; the docs keep up with the asserts.

| | |
|---|---|
| Clippy's docs | [`missing_panics_doc`](https://rust-lang.github.io/rust-clippy/master/index.html#missing_panics_doc) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | a public function that can panic says when in a # Panics section; the docs keep up with the asserts | medium |
| [big](../tiers/big.md) | `warn` | a public function that can panic says when in a # Panics section; the docs keep up with the asserts | medium |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
missing_panics_doc = "warn" # a public function that can panic says when in a # Panics section; the docs keep up with the asserts
```
