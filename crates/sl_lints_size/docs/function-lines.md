## Why

A function longer than a screen is read in parts and changed without seeing all of it. Long functions are where state machines hide and where edits collide. Test functions are not counted.

## Tightening the limit

`target-lines` sets a lower limit to move toward. A function above the target but within `max-lines` is reported as a warning, even where the lint denies, so the limit can come down one step at a time without a flood of failures:

```toml
[lints.function-lines]
max-lines = 150
target-lines = 120
```

When the warnings are gone, `max-lines` takes the target's value and the target moves on.

## How to fix

Name the steps and make each a function; a long `match` often wants one function per arm.
