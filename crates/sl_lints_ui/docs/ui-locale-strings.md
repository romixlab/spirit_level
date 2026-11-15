## Why

A project that translates its UI reads every piece of text a person sees from a translation catalogue, keyed by locale, never from a literal written into the call. Off by default: a project that does not translate yet would see this fire on every label it has.

The lint reports a string literal that holds a letter, passed to one of `calls`. A literal with no letter (a placeholder, a number, punctuation) needs no catalogue entry and is not reported.

```rust
ui.label("Connected");           // reported once the lint is on: move it to the catalogue
ui.label(t!("status.connected")); // passes: the literal is the catalogue key, not the text
ui.label(format!("{count}"));     // passes: no letter in the format string itself
```

## How to fix

Move the text to the project's translation catalogue and pass the catalogue key (or the call the catalogue gives) instead of the literal.
