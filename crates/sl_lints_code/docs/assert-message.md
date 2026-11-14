## Why

A failed `assert!(cond)` prints the expression, `assertion failed: self.taken.get() & bit == 0`. A message says what the expression means: `endpoint handed out twice`. In firmware the message is what a crash record carries back from the field; with defmt it costs no flash.

`assert_eq!` and `assert_ne!` print both values and need no message. Assertions in a `const` item or block are not checked: a failure there is a compile error. Test code is not checked.

## How to fix

Add a message that names the invariant in words: `assert!(n > 0, "a frame has at least one byte")`.
