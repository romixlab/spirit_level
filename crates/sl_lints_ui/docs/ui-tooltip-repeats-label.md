## Why

A tooltip earns its place by saying what the label cannot: what a control does, a shortcut, a constraint, or what happens next. A tooltip that repeats the label word for word gives the person reading it nothing new, and costs the same hover they already paid for the label.

The lint reports a call of `hover-calls` whose text is exactly the text of the `label-calls` call it is attached to, reached through its receiver chain.

```rust
ui.button("Save").on_hover_text("Save");          // reported: the tooltip adds nothing
ui.button("Save").on_hover_text("Ctrl+S");         // passes: it adds the shortcut
ui.add(Button::new("Save")).on_hover_text("Save"); // reported: the label is found through `add`
```

## How to fix

Say what the label cannot, or remove the tooltip.
