## Why

The same wording everywhere: a button that opens a dialog ends in "…", the one character, and a button label is an action, not a sentence, so it ends without a full stop. Three dots break differently, read differently to a screen reader and look different in proportional fonts.

The lint reports:

- "..." in text of the calls in `calls`;
- a full stop at the end of a label of `button-calls`.

The `ui` tier knows no toolkit, so both lists are empty; the `egui` tier fills them. Whether a button opens a dialog is for review.

## How to fix

`spl check --fix` writes "…" and drops the full stop.
