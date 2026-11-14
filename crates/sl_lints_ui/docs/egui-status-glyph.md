## Why

A status glyph (done, blocked, failing) carries meaning. Written as a character inside a label, it takes the text colour and has no tooltip; the theme paints it in its tone with a tooltip that says what it means, so colour never carries the meaning alone.

The lint reports text of the calls in `calls` that holds one of `glyphs`.

```rust
ui.label(format!("{DONE} {name}"));                    // reported: DONE is the check-mark glyph
ve_theme::status_icon(ui, "done", "Built and tested"); // then the name beside it
```

## How to fix

Paint the glyph with `replacement` (`ve_theme::status_icon`) beside the text.
