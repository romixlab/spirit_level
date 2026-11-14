# sl_lints_ui

UI lints read with `syn`, without type information, over the crates that depend on a UI toolkit or that the crate map marks `ui = true`:

- `ui-*`: practices of any UI. The calls they look for are parameters; the `ui` tier leaves the toolkit-specific ones empty and the `egui` tier fills them.
- `egui-*`: practices only egui has: the shared theme, status glyphs, ids in loops, repaints on every frame.

The catalogue of practices, with what no lint can check, is the book's "UI practices" page (`docs/ui.md`).

Depends on `sl_analysis`, `sl_core`, `sl_model`, `syn` and `proc-macro2`.
