## Why

rustc's front end runs one crate on one thread, so a big crate is one long serial step of every build, and every change in it rebuilds all of it. Past about 20,000 lines a crate also stops having one subject: tpm_cli's root crate reached 316,000 lines with 85% of them in one tangle. Split before it grows on.

Code lines are lines that are not blank and not only a comment, in `src/` (tests apart, see [test-lines](test-lines.md)). Crates the [crate map](../map.md) marks `generated` are not counted.

## Exceptions with expiry

A crate that must stay big for a while gets a limit of its own, a reason and an expiry, so every exception is visible and temporary:

```toml
[[exceptions]]
lint = "crate-lines"
crates = ["foo_editor"]
max-lines = 45000
level = "deny"
reason = "split into foo_editor_core and foo_editor_ui is FEATURES SPLIT-3"
expires = "2027-03-31"
```

`spl config` lists every exception with its reason and expiry.

## How to fix

Split along the seams: types into a `_core` crate, a heavy dependency into a crate of its own, parts that change at their own pace apart.
