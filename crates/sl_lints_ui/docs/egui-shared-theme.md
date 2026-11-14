## Why

Apps look and behave alike when they share one theme: colours, spacing, radii and type from one set of tokens. In egui that is `ve_theme::setup` at start-up; an app that skips it draws egui's default look, and its own colours and sizes creep in.

The lint reports a call of `entry-calls` (`eframe::run_native`) in a crate that never calls one of `setup-calls` (`ve_theme::setup`). Findings are estimates: a crate that starts its app through a shell that installs the theme for it (vhrd_egui's `ve_app`) does not call `run_native` itself.

## How to fix

Call `ve_theme::setup(&cc.egui_ctx)` in the app's creation closure, before the first frame.
