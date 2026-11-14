## Why

A tap that throws work away (delete, reset, wipe) must ask first, saying what will be lost, or be undoable for a few seconds. A button that does it at once turns a slip of the mouse into lost work.

The lint reads the labels of `calls` (egui: `button`, `Button::new`, `danger_button`). A label whose first word is one of `verbs` is reported unless the function that makes it names a confirmation or an undo (an identifier containing one of `confirm-words`: `confirm`, `undo`, `modal`, `dialog`), or the label ends in "…", which says it opens a dialog. Findings are estimates: a confirmation built in another function is not seen.

```rust
if ui.button("Reset settings").clicked() { self.settings = Settings::default(); } // reported
if ui.button("Reset settings…").clicked() { self.confirm_reset = true; }          // passes
```

## How to fix

Open a confirmation that says what will be lost, with the safe choice as the default (egui: an `egui::Modal`), and end the label in "…"; or keep the old state for an undo. Where the action loses nothing (a filter, a selection), waive the finding with that reason.
