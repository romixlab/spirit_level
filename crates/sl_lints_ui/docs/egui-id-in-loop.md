## Why

egui keeps a widget's state (scroll offset, open or closed, column widths) under its id, made from the parent's id and a salt. A widget made in a loop with a constant salt gets the same id on every iteration: the widgets share their state, and egui paints an id clash warning over them.

The lint reports, inside a `for`, `while` or `loop` body, or the closure of a `loop-methods` call (`for_each`):

- a call of `id-calls` whose first argument is a literal: `Grid::new("grid")`, `.id_salt("list")`, `ComboBox::from_label("Mode")`;
- an `unsalted` constructor (`ScrollArea::vertical()`) with no `salt-methods` call in its chain.

A `salt-methods` call later in the chain (`Window::new("").id(id)`) gives the widget its id and clears both.

A widget inside a closure of another call in the loop (`ui.horizontal(|ui| ..)`) sits in a child `Ui` with an id of its own, and is not reported. Findings are estimates: a loop that runs once is fine.

## How to fix

Salt the id with the item: `Grid::new(("grid", i))`, `.id_salt(("list", row.id))`, or wrap the body in `ui.push_id(i, |ui| ..)`.
