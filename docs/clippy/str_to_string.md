# str_to_string

**Clippy lint:** To_owned says what happens: a copy of a borrowed str.

| | |
|---|---|
| Clippy's docs | [`str_to_string`](https://rust-lang.github.io/rust-clippy/master/index.html#str_to_string) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | to_owned says what happens: a copy of a borrowed str | clippy-pedantic |
| [big](../tiers/big.md) | `warn` | to_owned says what happens: a copy of a borrowed str | clippy-pedantic |
| [ai](../tiers/ai.md) | `warn` | to_owned says what happens: a copy of a borrowed str | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
str_to_string = "warn" # to_owned says what happens: a copy of a borrowed str
```
