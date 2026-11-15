## Why

egui closes a `menu_button`'s menu on the first click inside it. A dropdown of checkboxes meant to tick several options needs the menu to stay open while ticking, which `vhrd_egui`'s checklist widget does; a plain `menu_button` closure full of `checkbox` calls closes after the first tick, hiding the rest of the list from the person mid-choice.

The lint reports, in [UI crates](../ui.md#which-crates-are-ui-crates), a `menu-calls` call (`menu_button`) whose last argument is a closure holding `min-checkboxes` or more calls of `checkbox-calls` (`checkbox`), anywhere in the closure's body. Findings are estimates: a single checkbox below `min-checkboxes`, or one meant to close the menu on its own tick, is not a checklist.

## How to fix

Replace the `menu_button` closure with the shared checklist widget (`widget`, `ve_basics::Checklist` by default), which keeps the menu open while the person ticks several boxes.
