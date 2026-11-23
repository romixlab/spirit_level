## Why

A number named `timeout` may be seconds, milliseconds or ticks; the reader guesses, and a wrong guess is an error by a factor of 1000 that no type catches. With the unit last in the name (`timeout_ms`, `frame_size_bytes`, `latency_ms_max`) the call site shows it, and related names sort together.

The lint runs only in a project that turned off [exact-units-quantity](C0033.md). That lint puts the unit into the type, where the compiler checks it, and a unit in the name is then redundant.

The lint looks at numeric struct fields, constants and statics whose name holds a word of `words` (time and size words) but no word of `units`. It reads names only, so the finding is an estimate: a size counted in items passes with the item named (`queue_size_items`) or with the word added to `units`. A name in `exempt` passes whole: a protocol header field named as its standard names it (IPv4's `ttl` counts hops, not seconds).

```rust
const READ_TIMEOUT: u64 = 500;        // reported
const READ_TIMEOUT_MS: u64 = 500;     // passes
pub struct Limits { pub frame_size_bytes: u32 }   // passes
```

## How to fix

Put the unit last in the name, or use a type that carries it (`Duration`). A project with its own units extends `units`. Turning [exact-units-quantity](C0033.md) back on replaces this lint with the typed form.
