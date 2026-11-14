## Why

A greyed-out control that does not say why leaves the person guessing what to do. The practice: never hide a control that cannot be used now, show it disabled, and say why and what would enable it, in a tooltip or, where there is no hover, a short line.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates), a call of `disable-calls` (egui: `add_enabled`, `add_enabled_ui`, `disable`) when no call of `reason-calls` (egui: `on_disabled_hover_text`) is in the same statement. A disable call that is a statement of its own (`ui.disable();`) may also take its reasons from the later statements of its block or an enclosing one. A disable call whose first argument is `true` is skipped. Findings are estimates: a reason given through a helper of the project is not seen.

In egui, `on_hover_text` does not show on a disabled widget; only `on_disabled_hover_text` does.

```rust
ui.add_enabled(connected, Button::new("Flash"));                      // reported
ui.add_enabled(connected, Button::new("Flash"))
    .on_disabled_hover_text("Not connected: plug the probe in first"); // passes
```

## How to fix

Attach the reason and what would enable the control. When the cause is on screen, hovering the control highlights it (vhrd_egui: `ve_basics::hover_link`).
