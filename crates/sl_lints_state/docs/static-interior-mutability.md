## Why

A `static` with interior mutability is shared mutable state that every caller in the process reaches. Each one brings a lock order, a poison story, a risk of being held across I/O, and contention on a hot path. In tpm's crates about 1,700 hits of `Mutex|RwLock|OnceLock|...` held some 200 real pieces of global state; most of the CLI's were a `Mutex` that only existed to satisfy `static` in a single-threaded run.

The rule: every piece of mutable state has one owner. Pick the first shape that fits:

| Shape | When |
|---|---|
| V, by value | state of one request, run or connection: a local or a context passed by `&mut` |
| A, owner task | outlives a request, several tasks change it: one task owns it, others send it messages |
| S, swapped snapshot | read far more than written: `ArcSwap` / `watch`, readers never block |
| E, the event log | shared across machines: a fold over the stream |
| O, set once | `OnceLock` / `LazyLock` of something without interior mutability: not reported |
| K, a lock that is right | a tiny section with no I/O, a callback API that demands `Sync`, a per-thread buffer: with the reason |

## Example

```rust
// reported
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// kept on purpose, with the reason
// spirit-level: allow(static-interior-mutability) K: the panic hook's ring; one writer per thread, dumped on exit
static RING: Mutex<Ring> = Mutex::new(Ring::new());
```

## How to fix

Move the state to its owner (see the table). When a global lock is the right tool, write the reason in a waiver on the item. The parameter `types` lists the type names that count; `atomics = true` also reports atomic statics.
