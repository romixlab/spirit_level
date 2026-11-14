## Why

egui redraws when input arrives or when something asks for it. A `request_repaint()` that runs on every frame asks for the next frame each time: the app draws at full rate while idle, burns a core and drains a battery. A `request_repaint_after` with a delay of a few milliseconds is the same loop.

The lint reports, in a function that takes a `ui-types` parameter (`Ui`, `Frame`: it draws a frame, so it runs every frame; a function that takes only the `Context` is often called once, on an event), a statement of the function body itself (not inside a condition or a closure) that calls one of `calls` without arguments, or one of `after-calls` with a `Duration::from_*` literal below `interval-ms-min`. Findings are estimates.

## How to fix

Ask for a repaint where something changed: the worker that has news calls `ctx.request_repaint()` (vhrd_egui: `ve_app::Repainter`), an animation asks while it runs, a clock asks with `request_repaint_after` for its next tick.
