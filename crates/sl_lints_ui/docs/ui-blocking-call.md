## Why

A UI draws a frame, waits for input and draws the next one, all on one thread. A call that waits for a disk, a peer, a process or a timer on that thread freezes the window until it returns: no repaint, no input, and on most desktops a "not responding" dialog. zed lints blocking I/O on its foreground thread for the same reason.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates):

- calls of the functions in `functions` (`std::fs::read_to_string`, `std::thread::sleep`, ...), resolved through the file's `use` items;
- the methods in `methods` (`blocking_read`, `block_on`, ...);
- a method of `command-methods` (`output`, `status`) on a chain that starts at a `command-paths` constructor: `Command::new("git").arg("log").output()`.

Code inside a closure or async block passed to one of `spawners` runs elsewhere and is skipped, as are the functions in `skip-fns` (`main`, before the UI runs). With `ui-types` set (the `egui` tier: `Ui`, `Context`, `Frame`), only functions that take a parameter of one of these types count: those draw or update the UI, while the rest of a one-crate app may be a worker. Without types, the lint cannot see that a function is only ever called from a worker: findings are estimates.

## How to fix

Run the work in a worker (through the spawn wrapper) and hand the result back through a channel; the UI polls it each frame and the worker asks for a repaint when it is done. A lock shared with workers is read with `try_read`, never `blocking_read`. Where a function only ever runs on a worker, waive the finding with that reason.
