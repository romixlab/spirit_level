# too_long_first_doc_paragraph

**Clippy lint:** The first doc paragraph is the summary rustdoc lists; one short sentence.

| | |
|---|---|
| Clippy's docs | [`too_long_first_doc_paragraph`](https://rust-lang.github.io/rust-clippy/master/index.html#too_long_first_doc_paragraph) |
| Written as | a lint in `[workspace.lints.clippy]` |
| Checked by | [M0025 tier-clippy-set](../lints/M0025.md) |

## Level per tier

| Tier | Level | Reason | Set in |
|---|---|---|---|
| [small](../tiers/small.md) | not in the set |  |  |
| [small-workspace](../tiers/small-workspace.md) | not in the set |  |  |
| [medium](../tiers/medium.md) | `warn` | the first doc paragraph is the summary rustdoc lists; one short sentence | medium |
| [big](../tiers/big.md) | `warn` | the first doc paragraph is the summary rustdoc lists; one short sentence | medium |
| [ai](../tiers/ai.md) | `warn` | agents write long doc summaries; the first paragraph is one short sentence | ai |
| [embedded](../tiers/embedded.md) | not in the set |  |  |
| [ui](../tiers/ui.md) | not in the set |  |  |
| [egui](../tiers/egui.md) | not in the set |  |  |
| [units-strict](../tiers/units-strict.md) | not in the set |  |  |
| [units-paranoid](../tiers/units-paranoid.md) | not in the set |  |  |

## Configure

```toml
[workspace.lints.clippy]
too_long_first_doc_paragraph = "warn" # the first doc paragraph is the summary rustdoc lists; one short sentence
```
