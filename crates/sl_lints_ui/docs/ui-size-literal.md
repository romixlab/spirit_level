## Why

Spacing, radii, stroke widths and type sizes written as numbers in every screen drift apart: one panel uses `6.0`, the next `8.0`, and a change of the theme reaches none of them. Fixed pixel widths also break the layout when the text, the language or the font changes.

The lint reports number literals, in [UI crates](../ui.md#which-crates-are-ui-crates) outside the theme:

- in the arguments of the calls in `calls`: a method by its name (`add_space`), a function by `Type::name` (`Margin::same`); an array, a tuple or a `vec2(..)` counts number by number;
- assigned to a style field in `fields` (`spacing_mut().item_spacing.x = 6.0`).

Numbers in `allowed` (`0`) are not reported. The `ui` tier knows no toolkit, so `calls` and `fields` are empty; the `egui` tier fills them.

## How to fix

Take the size from the theme's tokens (in vhrd_egui: `t.space.m`, `t.radius.s`, `t.stroke.thin`, `t.type_scale.title.font_id()`). A width that must not jump is learned from the content (`ve_dash::steady_width`), not written down.
