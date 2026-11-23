# must_use_candidate

**Clippy lint:** Thousands of hits; must_use goes where ignoring the value is a bug.

| | |
|---|---|
| Clippy's docs | [`must_use_candidate`](https://rust-lang.github.io/rust-clippy/master/index.html#must_use_candidate) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `allow` | thousands of hits; must_use goes where ignoring the value is a bug | clippy-pedantic |
| [big](../tiers/big.md) | `allow` | thousands of hits; must_use goes where ignoring the value is a bug | clippy-pedantic |
| [ai](../tiers/ai.md) | not in the set |  |  |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
must_use_candidate = "allow" # thousands of hits; must_use goes where ignoring the value is a bug
```
