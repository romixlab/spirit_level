## Why

While a branch is in work, a `FIXME` is a useful note to oneself: "revisit this before it lands". It must not outlive the branch. A `TODO` without a tracker id is never fine ([stop-gap-comments](A0010.md)); a `FIXME` is fine until the landing.

In an ordinary run the lint reports each `FIXME` as a candidate, which shows it and fails nothing. In a landing run (`spl check --landing`, see [Commits and landing](../commits.md)) it reports it at the lint's level.

```rust
// FIXME: the length check is off by one for empty frames   <- candidate on the branch, a finding at landing
```

## How to fix

Do what the note says before landing and remove it. Work that stays for later becomes a `TODO(ID)` with a FEATURES.md id.
