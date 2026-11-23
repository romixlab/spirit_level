# egui

Egui apps: the ui tier with egui's calls filled in, plus ids, repaints, the shared theme and status glyphs.

Extends [ui](ui.md): everything below is on top of it.

Use it with `extends = "egui"` in `spirit_level.toml`.

## Lints enabled (with what it extends)

| Code | Lint | Level | Reason | Set in |
|---|---|---|---|---|
| [U0001](../lints/U0001.md) | ui-raw-spawn | `warn` | work started outside the spawn wrapper has no name, no crash report and no repaint; vhrd_egui has no wrapper yet, so warn | [egui](egui.md) |
| [U0002](../lints/U0002.md) | ui-blocking-call | `warn` | a blocking call in a function that draws or updates the UI freezes the window; an estimate, without types | [egui](egui.md) |
| [U0003](../lints/U0003.md) | ui-colour-literal | `warn` | colours come from ve_theme's tokens (`ui.tokens().colors`), which follow the light and dark theme | [egui](egui.md) |
| [U0004](../lints/U0004.md) | ui-size-literal | `warn` | spacing, radii, strokes and type come from ve_theme's tokens (`space`, `radius`, `stroke`, `type_scale`); widths are learned with ve_dash's steady_width | [egui](egui.md) |
| [U0005](../lints/U0005.md) | ui-disabled-without-reason | `warn` | a disabled egui widget shows only on_disabled_hover_text; on_hover_text stays silent on it | [egui](egui.md) |
| [U0006](../lints/U0006.md) | ui-destructive-unconfirmed | `warn` | one tap must not throw work away: an egui::Modal asks first, or the edit can be undone | [egui](egui.md) |
| [U0007](../lints/U0007.md) | ui-more-count | `warn` | "N more" hides items the person may need; a fold (ve_basics::fold_rest) shows them on demand | [egui](egui.md) |
| [U0008](../lints/U0008.md) | ui-label-punctuation | `deny` | one way to write the ellipsis and button labels; certain, rare, and `spl check --fix` applies it | [egui](egui.md) |
| [U0009](../lints/U0009.md) | ui-debug-text | `warn` | Rust's debug form is for developers; a person reads what happened and what to do | [egui](egui.md) |
| [U0010](../lints/U0010.md) | egui-shared-theme | `warn` | every app looks alike when it installs ve_theme; an app that does not is an app of its own | [egui](egui.md) |
| [U0011](../lints/U0011.md) | egui-status-glyph | `deny` | a glyph in a label takes the text colour; ve_theme::status_icon paints it in its tone with a tooltip | [egui](egui.md) |
| [U0012](../lints/U0012.md) | egui-id-in-loop | `warn` | widgets that share an id share their state, and egui reports the clash on screen | [egui](egui.md) |
| [U0013](../lints/U0013.md) | egui-repaint-every-frame | `warn` | an unconditional repaint request keeps the app drawing at full rate and burns a core while idle | [egui](egui.md) |
| [U0014](../lints/U0014.md) | ui-unwrap-external-data | `warn` | panic-in-library skips binary crates; a UI app's bad file, parse or connection still deserves an error, not a crash; an estimate, since the chain may be known good or already checked | [ui](ui.md) |
| [U0015](../lints/U0015.md) | egui-panel-api | `warn` | panel and App code written for an older egui keeps compiling under a deprecation warning, then breaks on the next bump; an estimate, without type information | [egui](egui.md) |
| [U0016](../lints/U0016.md) | egui-checklist-menu | `warn` | a menu_button closes on the first tick inside it; ve_basics::Checklist keeps it open for several | [egui](egui.md) |
| [U0017](../lints/U0017.md) | ui-tooltip-repeats-label | `warn` | a tooltip that only repeats the label cost the person a hover for nothing; certain, since the match is exact | [egui](egui.md) |
| [U0019](../lints/U0019.md) | ui-hover-text | `warn` | an icon-only button with no tooltip has no words at all; a candidate, since a short label may be a real word or get help another way | [egui](egui.md) |

## The file

```toml
# Beside the size tiers: an egui app extends ["medium", "egui"]. The ui tier with egui's calls and types filled in,
# the replacements vhrd_egui gives named in the help, and the lints only egui has.
extends = "ui"

[tier]
description = "egui apps: the ui tier with egui's calls filled in, plus ids, repaints, the shared theme and status glyphs"

[lints.ui-raw-spawn]
ui-types = ["Ui", "Context", "Frame"]
reason = "work started outside the spawn wrapper has no name, no crash report and no repaint; vhrd_egui has no wrapper yet, so warn"

[lints.ui-blocking-call]
ui-types = ["Ui", "Context", "Frame"]
reason = "a blocking call in a function that draws or updates the UI freezes the window; an estimate, without types"

[lints.ui-colour-literal]
constructors = ["Color32::from_*", "Rgba::from_*", "Hsva::new"]
consts = ["Color32::*", "Rgba::*"]
allowed = ["Color32::TRANSPARENT", "Color32::PLACEHOLDER", "Rgba::TRANSPARENT"]
macros = ["hex_color"]
reason = "colours come from ve_theme's tokens (`ui.tokens().colors`), which follow the light and dark theme"

[lints.ui-size-literal]
calls = [
  "add_space",
  "set_width",
  "set_min_width",
  "set_max_width",
  "set_height",
  "set_min_height",
  "set_max_height",
  "min_col_width",
  "max_col_width",
  "spacing",
  "add_sized",
  "min_size",
  "max_size",
  "set_min_size",
  "set_max_size",
  "inner_margin",
  "outer_margin",
  "corner_radius",
  "rounding",
  "size",
  "default_width",
  "default_height",
  "min_width",
  "max_width",
  "min_height",
  "max_height",
  "Margin::same",
  "Margin::symmetric",
  "CornerRadius::same",
  "Rounding::same",
  "Stroke::new",
  "FontId::new",
  "FontId::proportional",
  "FontId::monospace",
]
fields = [
  "item_spacing",
  "button_padding",
  "indent",
  "interact_size",
  "window_margin",
  "menu_margin",
  "icon_width",
  "icon_spacing",
  "slider_width",
  "combo_width",
  "text_edit_width",
  "tooltip_width",
]
reason = "spacing, radii, strokes and type come from ve_theme's tokens (`space`, `radius`, `stroke`, `type_scale`); widths are learned with ve_dash's steady_width"

[lints.ui-disabled-without-reason]
disable-calls = ["add_enabled", "add_enabled_ui", "set_enabled", "disable"]
reason-calls = ["on_disabled_hover_text", "on_disabled_hover_ui"]
reason = "a disabled egui widget shows only on_disabled_hover_text; on_hover_text stays silent on it"

[lints.ui-destructive-unconfirmed]
calls = ["button", "small_button", "Button::new", "primary_button", "danger_button"]
reason = "one tap must not throw work away: an egui::Modal asks first, or the edit can be undone"

[lints.ui-more-count]
fold-calls = ["collapsing", "CollapsingHeader::new", "ve_basics::fold_rest", "header"]
reason = "\"N more\" hides items the person may need; a fold (ve_basics::fold_rest) shows them on demand"

[lints.ui-label-punctuation]
calls = [
  "label",
  "button",
  "small_button",
  "heading",
  "selectable_label",
  "checkbox",
  "radio",
  "radio_value",
  "selectable_value",
  "menu_button",
  "collapsing",
  "on_hover_text",
  "on_disabled_hover_text",
  "Label::new",
  "Button::new",
  "RichText::new",
  "CollapsingHeader::new",
  "Window::new",
  "primary_button",
  "danger_button",
  "section_header",
  "muted_label",
  "badge",
]
button-calls = ["button", "small_button", "menu_button", "Button::new", "primary_button", "danger_button"]
reason = "one way to write the ellipsis and button labels; certain, rare, and `spl check --fix` applies it"

[lints.ui-debug-text]
calls = [
  "label",
  "button",
  "small_button",
  "heading",
  "monospace",
  "selectable_label",
  "on_hover_text",
  "on_disabled_hover_text",
  "Label::new",
  "Button::new",
  "RichText::new",
  "muted_label",
  "badge",
]
reason = "Rust's debug form is for developers; a person reads what happened and what to do"

[lints.egui-shared-theme]
level = "warn"
reason = "every app looks alike when it installs ve_theme; an app that does not is an app of its own"

[lints.egui-status-glyph]
level = "deny"
reason = "a glyph in a label takes the text colour; ve_theme::status_icon paints it in its tone with a tooltip"

[lints.egui-id-in-loop]
level = "warn"
reason = "widgets that share an id share their state, and egui reports the clash on screen"

[lints.egui-repaint-every-frame]
level = "warn"
reason = "an unconditional repaint request keeps the app drawing at full rate and burns a core while idle"

[lints.egui-panel-api]
level = "warn"
reason = "panel and App code written for an older egui keeps compiling under a deprecation warning, then breaks on the next bump; an estimate, without type information"

[lints.egui-checklist-menu]
level = "warn"
reason = "a menu_button closes on the first tick inside it; ve_basics::Checklist keeps it open for several"

[lints.ui-tooltip-repeats-label]
label-calls = [
  "label",
  "button",
  "small_button",
  "heading",
  "selectable_label",
  "checkbox",
  "radio",
  "radio_value",
  "selectable_value",
  "menu_button",
  "Label::new",
  "Button::new",
  "RichText::new",
  "primary_button",
  "danger_button",
]
hover-calls = ["on_hover_text", "on_disabled_hover_text"]
reason = "a tooltip that only repeats the label cost the person a hover for nothing; certain, since the match is exact"

[lints.ui-locale-strings]
calls = [
  "label",
  "button",
  "small_button",
  "heading",
  "selectable_label",
  "checkbox",
  "Label::new",
  "Button::new",
  "RichText::new",
]
level = "allow"
reason = "opt-in: a project that has not started translating its UI would see this fire on every label; a project that translates turns it on"

[lints.ui-hover-text]
button-calls = [
  "button",
  "small_button",
  "selectable_label",
  "Button::new",
  "menu_button",
  "primary_button",
  "danger_button",
]
hover-calls = ["on_hover_text", "on_disabled_hover_text"]
reason = "an icon-only button with no tooltip has no words at all; a candidate, since a short label may be a real word or get help another way"
```
