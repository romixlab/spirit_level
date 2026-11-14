# ui

User interfaces of any toolkit: theme colours and sizes, disabled controls with a reason, confirmed destructive actions, no blocking or raw spawns on the UI thread, wording.

Use it with `extends = "ui"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [U0001](../lints/U0001.md) | ui-raw-spawn | `warn` | work started outside the spawn wrapper has no name, no crash report and no repaint; warn while toolkits ship no wrapper | ui |
| [U0002](../lints/U0002.md) | ui-blocking-call | `warn` | a blocking call on the UI thread freezes the window; an estimate, since a function may run on a worker that the lint does not see | ui |
| [U0003](../lints/U0003.md) | ui-colour-literal | `warn` | a colour written as a value ignores the theme and the light and dark modes | ui |
| [U0004](../lints/U0004.md) | ui-size-literal | `warn` | sizes written as numbers drift apart between screens and apps | ui |
| [U0005](../lints/U0005.md) | ui-disabled-without-reason | `warn` | a greyed-out control that does not say why leaves the person guessing | ui |
| [U0006](../lints/U0006.md) | ui-destructive-unconfirmed | `warn` | one tap must not throw work away; an estimate, since the confirmation may live elsewhere | ui |
| [U0007](../lints/U0007.md) | ui-more-count | `warn` | "N more" hides items the person may need; a fold shows them on demand | ui |
| [U0008](../lints/U0008.md) | ui-label-punctuation | `deny` | one way to write the ellipsis and button labels; certain, rare, and `spl check --fix` applies it | ui |
| [U0009](../lints/U0009.md) | ui-debug-text | `warn` | Rust's debug form is for developers; a person reads what happened and what to do | ui |

## The file

```toml
# Beside the size tiers: an app with a user interface extends ["medium", "ui"]. The practices of any UI, as lints
# that know no toolkit: the calls each looks for are parameters, empty here where they name a toolkit's calls. The
# egui tier fills them. Most lints start at warn: they fire on code written before the practice was written down
# (colours and sizes by the dozen per app), and most are estimates (they cannot see types).
[tier]
description = "User interfaces of any toolkit: theme colours and sizes, disabled controls with a reason, confirmed destructive actions, no blocking or raw spawns on the UI thread, wording"

[lints.ui-raw-spawn]
level = "warn"
reason = "work started outside the spawn wrapper has no name, no crash report and no repaint; warn while toolkits ship no wrapper"

[lints.ui-blocking-call]
level = "warn"
reason = "a blocking call on the UI thread freezes the window; an estimate, since a function may run on a worker that the lint does not see"

[lints.ui-colour-literal]
level = "warn"
reason = "a colour written as a value ignores the theme and the light and dark modes"

[lints.ui-size-literal]
level = "warn"
reason = "sizes written as numbers drift apart between screens and apps"

[lints.ui-disabled-without-reason]
level = "warn"
reason = "a greyed-out control that does not say why leaves the person guessing"

[lints.ui-destructive-unconfirmed]
level = "warn"
reason = "one tap must not throw work away; an estimate, since the confirmation may live elsewhere"

[lints.ui-more-count]
level = "warn"
reason = "\"N more\" hides items the person may need; a fold shows them on demand"

[lints.ui-label-punctuation]
level = "deny"
reason = "one way to write the ellipsis and button labels; certain, rare, and `spl check --fix` applies it"

[lints.ui-debug-text]
level = "warn"
reason = "Rust's debug form is for developers; a person reads what happened and what to do"
```
