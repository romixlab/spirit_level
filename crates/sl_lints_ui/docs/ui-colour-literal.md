## Why

One theme makes apps look alike, and lets them follow the light and dark modes. A colour written as a value in UI code ignores both: it stays the same in the dark theme, drifts from the palette when the palette changes, and is chosen anew in every app. rerun bans `Color32::from_rgb` and friends outside its design tokens for the same reason.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates) outside the theme:

- calls of a function in `constructors` whose arguments are all literals (`Color32::from_rgb(190, 90, 0)`); a colour computed from data is not reported;
- named colour constants matching `consts` (`Color32::RED`), except the `allowed` ones (`Color32::TRANSPARENT`);
- the macros in `macros` (`hex_color!`);
- CSS colour strings (`"#1e1e2e"`), unless `hex-strings` is false.

The `ui` tier knows no toolkit, so its `constructors`, `consts` and `macros` are empty; the `egui` tier fills them. The theme is the crate the map marks `theme = true`, a crate matching `theme-crates`, or a file matching `theme-paths`.

## How to fix

Take the colour from the theme's tokens (in vhrd_egui: `ui.tokens().colors.warn`, `Tone::color`, `colors.cat[i]` for categories). A colour the theme does not have yet is added to the theme, with a name for what it means.
