## Why

A button whose label is a glyph or an icon carries no words a screen reader or a person scanning the toolbar can read; a tooltip is the only place left to say what it does. The check is a candidate: a short label can be a real word in another script, and a project may have its own way of attaching help text that the lint does not see.

The lint reports a call of `button-calls` whose label, trimmed, holds no letter or digit (an icon) or is one of `egui-status-glyph`'s `glyphs` (a status glyph), when no call of `hover-calls` wraps it.

```rust
ui.button("🗑");                       // reported: no tooltip says what it does
ui.button("🗑").on_hover_text("Delete"); // passes
ui.button("Delete");                    // passes: the label is words, not an icon
```

## How to fix

Attach a tooltip (one of `hover-calls`) naming the action.
