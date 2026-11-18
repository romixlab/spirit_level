# unnecessary_safety_doc

**Clippy lint:** A # Safety section on a safe function sends readers looking for a contract that is not there.

| | |
|---|---|
| Clippy's docs | [`unnecessary_safety_doc`](https://rust-lang.github.io/rust-clippy/master/index.html#unnecessary_safety_doc) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| [medium](../tiers/medium.md) | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| [big](../tiers/big.md) | `warn` | a # Safety section on a safe function sends readers looking for a contract that is not there | small |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | `deny` | a # Safety section on a safe function sends readers looking for a contract that is not there | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
unnecessary_safety_doc = "warn" # a # Safety section on a safe function sends readers looking for a contract that is not there
```
