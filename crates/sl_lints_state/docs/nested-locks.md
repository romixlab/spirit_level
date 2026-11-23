## Why

Taking a second lock while holding one creates a lock order that every other path must follow; one path that takes them the other way deadlocks. tpm's turn engine split its state over 16 mutexes and took several at once.

The lint follows guards as `lock-held-across` does, lock helpers included: a call of a helper while a guard lives is a second lock, and so is a lock taken inside the closure a helper runs with its guard held. A decoder's `rd.read()?` or `rd.read::<T>()` is not a lock. Neither is `self.lock()` or `self.read()` in an impl whose type has its own method of that name returning a value (a file lock, a file read): the call resolves to that method.

## Example

```rust
let slots = locked(&self.slots);
locked(&self.wakers).push(None);   // reported: `slots` is still alive
```

## How to fix

Put both pieces of state under one owner, or take the locks one after the other with the first guard dropped in between.
