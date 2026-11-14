## Why

A thread or task started from UI code has to do three things the UI depends on: carry a name that shows in a crash report and a profiler, report its failure to the person instead of dying silently, and ask the UI for a repaint when it has news. One spawn wrapper does all three for every call; each raw `std::thread::spawn` or `tokio::spawn` does them by hand or not at all. rerun bans its raw `spawn_local` for the same reason.

The lint looks at [UI crates](../ui.md#which-crates-are-ui-crates) and reports every call of a function in `functions`, resolved through the file's `use` items. With `ui-types` set (the `egui` tier: `Ui`, `Context`, `Frame`), only functions that take a parameter of one of these types count as UI code; a worker module of the same crate is left alone. The crate that implements the wrapper gets an exception in `spirit_level.toml`.

```rust
std::thread::spawn(move || load(path));              // reported
app.spawn("load project", move || load(path));       // the wrapper
```

## How to fix

Start the work through the project's spawn wrapper (the `wrapper` parameter names it in the help). Where the project has none yet, write one: it takes a name, runs the closure, catches its error and requests a repaint.
