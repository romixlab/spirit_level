# as_ptr_cast_mut

**Clippy lint:** `as_ptr() as *mut` writes through a shared borrow: undefined behaviour.

| | |
|---|---|
| Clippy's docs | [`as_ptr_cast_mut`](https://rust-lang.github.io/rust-clippy/master/index.html#as_ptr_cast_mut) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | unsafe |
| [small-workspace](../tiers/small-workspace.md) | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | unsafe |
| [medium](../tiers/medium.md) | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | unsafe |
| [big](../tiers/big.md) | `warn` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | unsafe |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | `deny` | `as_ptr() as *mut` writes through a shared borrow: undefined behaviour | embedded |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
as_ptr_cast_mut = "warn" # `as_ptr() as *mut` writes through a shared borrow: undefined behaviour
```
