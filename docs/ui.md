# UI practices

Apps built by one team should look and behave alike: one theme, no colours written as values, disabled controls that say why, no blocking work on the UI thread, the same spacing and wording everywhere. The UI lints check the part of these practices that the source shows. The rest stays a review item, listed below so that a reviewer, or an agent, knows what is left.

## Two layers

| Tier | What it holds | Combine it with |
|---|---|---|
| [ui](tiers/ui.md) | The practices of any user interface, as lints that know no toolkit. Each lint names the calls it looks for in parameters; the `ui` tier leaves the toolkit-specific ones empty. | a size tier: `extends = ["medium", "ui"]` |
| [egui](tiers/egui.md) | Extends `ui`: fills those parameters with egui's calls and types and the [vhrd_egui](https://github.com/vhrdtech/vhrd_egui) helpers that replace them, and adds lints that only egui has. | a size tier: `extends = ["medium", "egui"]` |

A project on another toolkit extends `ui` and sets the call parameters for its toolkit in `spirit_level.toml`, the way the `egui` tier does.

## Which crates are UI crates

The UI lints look only at UI crates, and never at test code.

- A crate that depends directly on a UI toolkit is a UI crate. The toolkits are the `toolkits` parameter of each lint (egui, eframe, iced, slint, gtk4 and others by default).
- The [crate map](map.md) says it outright: `ui = true` makes a crate a UI crate, `ui = false` exempts one that depends on a toolkit only for its types.
- The theme crate is where colours and sizes are written as values. The map marks it `theme = true`; without a map, crates named like `*_theme` and files named `theme.rs` or under `theme/` count (`theme-crates` and `theme-paths`). `ui-colour-literal` and `ui-size-literal` skip it.

```toml
[map.crates]
foo_ui = { layer = "app", ui = true }
foo_theme = { layer = "app", ui = true, theme = true }
foo_model = { layer = "model" }
```

## The catalogue

One row per practice. Sources: the vhrd_egui UX guide (`docs/ux-guide.md`, by section) and its app skill, the workspace guideline's lint policy, and how rerun and zed keep their UI code in check. "By review" means no lint can check it honestly; a person or an agent reads the code or the screen.

### Any UI

| Practice | Source | Check | Confidence |
|---|---|---|---|
| A cut list ends in a fold that opens to every item, never "N more" alone | UX guide §2 | [`ui-more-count`](lints/U0007.md) | estimate |
| A control that cannot be used now stays visible, greyed out | UX guide §2, §6 | by review: a hidden control is code that is not there | - |
| A disabled control says why and what would enable it | UX guide §2 | [`ui-disabled-without-reason`](lints/U0005.md) | estimate |
| Hovering a disabled control highlights its cause on screen | UX guide §2 | by review | - |
| Every control, number and badge has a tooltip that adds to its label | UX guide §4 | [`ui-tooltip-repeats-label`](lints/U0017.md) flags a tooltip that only repeats the label; whether it adds enough is by review | certain |
| An icon-only or glyph button says what it does in a tooltip | UX guide §4 | [`ui-hover-text`](lints/U0019.md) | candidate |
| Related items light up together on hover | UX guide §5 | by review | - |
| Colours come from the theme | UX guide §3; rerun's colour bans | [`ui-colour-literal`](lints/U0003.md) | certain |
| Spacing, sizes, radii and type come from the theme; widths are learned, never fixed | UX guide §3, §6 | [`ui-size-literal`](lints/U0004.md) | certain |
| Status colour never carries meaning alone; categorical colours avoid red and orange | UX guide §3 | by review (vhrd_egui tests its palette) | - |
| Button labels are actions, starting with a verb | UX guide §7 | by review: no word list tells a verb from a noun ("Export", "Status") | - |
| A button that opens a dialog ends in "…", the one character; a button label has no closing full stop | UX guide §7 | [`ui-label-punctuation`](lints/U0008.md), with an exact fix; whether a button opens a dialog is by review | certain |
| The likely action stands out; a destructive one never does | UX guide §7 | by review | - |
| A tap that throws work away asks first or can be undone | UX guide §7 | [`ui-destructive-unconfirmed`](lints/U0006.md) | estimate |
| Undo covers every edit and names it; interruptions match their weight; a warning sits by its field | UX guide §7 | by review | - |
| The UI thread never blocks: no file, network, process or sleep calls in UI code | UX guide §9; the rust-repo rules; zed's `blocking_io_on_foreground` | [`ui-blocking-call`](lints/U0002.md) | estimate |
| UI code spawns work through one wrapper that names it, reports a crash and repaints | the rust-repo rules; workspace guideline §5; rerun's `spawn_local` ban | [`ui-raw-spawn`](lints/U0001.md) | certain |
| Slow work shows what runs and how far | UX guide §9 | by review | - |
| Errors say what happened and what to do, in plain words | UX guide §9 | [`ui-debug-text`](lints/U0009.md) finds Rust's debug form shown to a person; the wording is by review | estimate |
| No unwrap or expect on data from disk, the network or the person | UX guide §9 | [`panic-in-library`](lints/C0005.md) for library crates; [`ui-unwrap-external-data`](lints/U0014.md) for a file, parse or connection call in a UI crate | estimate |
| Settings and layout survive a restart; transient state is never saved | UX guide §9 | by review | - |
| Everything works from the keyboard | UX guide §8 (planned there) | by review | - |
| Text a person reads comes from locale files | UX guide §2 | [`ui-locale-strings`](lints/U0018.md), opt-in: off until a project translates its UI | certain |
| The UI keeps its shape; big data stays fast; nothing updates itself unasked | UX guide §6, §7 | by review | - |
| Say why when doing something non-obvious; measure an input, then confirm | UX guide §2 | by review | - |

### egui and vhrd_egui

The same practices, specialised by the `egui` tier's parameters, and the practices only egui has.

| Practice | Source | Check | Confidence |
|---|---|---|---|
| Colours from `ve_theme::Tokens` (`ui.tokens().colors`), never `Color32::from_rgb`, `Color32::RED` or `hex_color!` in app code | UX guide §3 | `ui-colour-literal` with egui's constructors | certain |
| Sizes from `Tokens` (`space`, `radius`, `stroke`, `type_scale`), widths learned with `ve_dash::steady_width`, never `add_space(6.0)` or `.size(11.0)` | UX guide §3, §6 | `ui-size-literal` with egui's sizing calls and `Spacing` fields | certain |
| `add_enabled`, `add_enabled_ui`, `disable` come with `on_disabled_hover_text` (`on_hover_text` does not show on a disabled widget) | UX guide §2 | `ui-disabled-without-reason` with egui's calls | estimate |
| Every app installs the shared theme: `ve_theme::setup` | UX guide §3 | [`egui-shared-theme`](lints/U0010.md) | estimate |
| A status glyph is painted in its tone with `ve_theme::status_icon`, never a character in a label | UX guide §3 | [`egui-status-glyph`](lints/U0011.md) | certain |
| A widget's id is unique within its parent: no constant id given in a loop | egui's id rules; vhrd_egui `FoldRest` docs | [`egui-id-in-loop`](lints/U0012.md) | estimate |
| `request_repaint` is called by the worker that has news, not by every frame | UX guide §9; egui's repaint model | [`egui-repaint-every-frame`](lints/U0013.md) | estimate |
| `on_hover_text` and `on_disabled_hover_text` add to the label, never repeat it | UX guide §4 | `ui-tooltip-repeats-label` with egui's label and hover calls | certain |
| Literals passed to `label`, `button`, `heading` and the like come from the catalogue | UX guide §2 | `ui-locale-strings` with egui's calls, opt-in | certain |
| A button with no words in its label (an emoji, a status glyph) has `on_hover_text` | UX guide §4 | `ui-hover-text` with egui's button and hover calls | candidate |
| One egui version in the whole graph | UX guide §12 | [`duplicate-versions`](lints/M0002.md) | certain |
| Every widget sits in `ve_frame::WidgetFrame`; panels use the egui API the version in use ships | UX guide §6; app skill "Panels" | [`egui-panel-api`](lints/U0015.md) for the API egui's own deprecations name (`SidePanel`, `TopBottomPanel`, `.show` on a `Context`, `App::update`); `WidgetFrame` itself is by review | estimate |
| A dropdown of checkboxes is a `ve_basics::Checklist`, open while ticked | UX guide §7 | [`egui-checklist-menu`](lints/U0016.md) | estimate |
| Labels are not selectable (`ve_basics::setup_labels`); ids and paths are copyable | UX guide §5 | by review | - |
| A crash at start is recovered with `ve_basics::StartupGuard`; build info and the app menu sit in the top bar | UX guide §4, §9 | by review | - |
| Web crates never call the OS clock, threads, files or processes | vhrd_egui AGENTS.md "Web crates" | each crate's `clippy.toml` (`disallowed-methods`), run by `clippy-build` | certain |
| UI behaviour is tested headless with egui_kittest | UX guide §12 | by review | - |

## Web UIs and bot buttons

Nothing is built for them yet. These general lints carry over once their parameters name the calls of a web or bot framework:

- `ui-colour-literal`: CSS colour strings in Rust are found already (`hex-strings`); a web crate names its style builders in `constructors`.
- `ui-disabled-without-reason` and `ui-destructive-unconfirmed`: a bot keyboard builder (`InlineKeyboardButton::callback`) is a label call like a button.
- `ui-more-count`, `ui-label-punctuation`, `ui-debug-text`, `ui-locale-strings`: the text of a page or a bot reply is text a person reads.
- `ui-hover-text`: a bot button with an emoji label (no text) needs its tooltip or help text named in `hover-calls`.
- `ui-tooltip-repeats-label`: once `label-calls` and `hover-calls` name a framework's own calls, the same check applies.

Web-only and bot-only practices (`textContent` over `innerHTML` with data; button text from known values, never model output) would need lints of their own over JavaScript or the bot's handlers.
