## Why

A comment that says what the next line says doubles the text and drifts when the code changes. Comments are for what the code cannot say: why, the unit, the invariant, the source of a number.

The measure, per single-line `//` comment directly above a code line:

1. The comment's content words: words of two or more characters, stopwords (`the`, `a`, `to`, `of`, `this`, ...) dropped, `snake_case` and `CamelCase` split, lowercased, a plural or `-ed`/`-ing` ending removed.
2. The code's words: the identifiers of the next line split the same way, plus words for its operators and keywords (`+=` gives increment, increase, add; `new` gives create, new, make; `return`, `push`, `len`, `clone`, `for`, `if`).
3. The overlap: the share of the comment's content words found among the code's words.

A comment with between `min-words` and `max-words` content words and an overlap of at least `min-overlap` is reported, with the overlap as its metric.

```rust
// increment the counter          <- overlap 100%: increment (+=), counter
counter += 1;

// the PLL needs 2 ms to lock     <- overlap 25%: passes
delay_ms(2);
```

## How to fix

Delete the comment, or replace it with what the code does not show.
