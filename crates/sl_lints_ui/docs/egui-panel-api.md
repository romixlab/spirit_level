## Why

egui's panel and app entry-point API moves: `SidePanel` and `TopBottomPanel` are replaced by a single `Panel`, showing a panel directly on a `Context` (`.show`) is replaced by `.show_inside` on a `Ui`, and `eframe::App::update(&mut self, ctx, frame)` is replaced by `App::ui`, which takes a `&mut Ui`. Code written against an older egui keeps compiling for a while under a deprecation warning, then breaks on the next bump; this lint catches it at review time, project by project.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates):

- a constructor matching `constructors` (`SidePanel::*`, `TopBottomPanel::*`), resolved through the file's `use` items, and any method later in its chain: the whole type is replaced, not one method of it;
- a `context-method` call (`show`) whose chain is rooted in a `context-types` constructor (`CentralPanel::*`): showing it directly on a `Context` instead of `.show_inside` on a `Ui`;
- an `impl` of one of `app-traits` (`App`, `eframe::App`) whose method is named `app-method` (`update`).

Because `constructors`, `context-types`, `app-traits` and `app-method` are parameters, a later egui release that replaces another piece of the panel or app API needs a tier change, not a new lint. Findings are estimates: the lint has no type information, so a chain stored in a variable before its `.show` call is not seen.

## How to fix

Follow egui's own migration notes for the version in use: build panels from `Panel`, show them with `.show_inside` on the `Ui` that holds them, and move an app's per-frame code from `App::update` to `App::ui`. Where a dependency still pins an egui version that lacks the replacement, waive the finding with that reason.
