# too_many_lines

**Clippy lint:** Spl's function-lines measures function length, with a limit per crate and expiring exceptions.

| | |
|---|---|
| Clippy's docs | [`too_many_lines`](https://rust-lang.github.io/rust-clippy/master/index.html#too_many_lines) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | spl's function-lines measures function length, with a limit per crate and expiring exceptions | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | spl's function-lines measures function length, with a limit per crate and expiring exceptions | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
too_many_lines = "allow" # spl's function-lines measures function length, with a limit per crate and expiring exceptions
```
