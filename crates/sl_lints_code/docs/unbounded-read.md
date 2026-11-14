## Why

`read_to_end` and `read_to_string` read until the other side stops. On a socket, a request body or a file from outside, the other side decides how much memory the process takes. Every input that grows from outside needs a limit.

The lint reports each call whose receiver chain has no `take(...)`. It cannot tell a socket from a file the program wrote itself, so each finding is a candidate: a place to look at, never a failed run.

```rust
stream.read_to_end(&mut buf)?;                       // reported
stream.take(MAX_FRAME_BYTES).read_to_end(&mut buf)?; // passes
```

## How to fix

Bound the read with `take(LIMIT)` and refuse input that reached the limit. Where the source is trusted (the program's own file), waive the line with that reason.
