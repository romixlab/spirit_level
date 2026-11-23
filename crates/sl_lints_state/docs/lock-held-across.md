## Why

A guard held across an `.await` blocks every other task that wants the lock for as long as the awaited future takes, and with a std mutex it can deadlock the executor thread. A guard held across I/O (a file write, a socket, a subprocess, an HTTP call) turns a fast critical section into one that lasts as long as the slowest disk or peer. tpm's mail server held a lock across IMAP and database work for minutes.

The lint follows a guard bound with `let` from a zero-argument `lock()`, `read()`, `write()`, `try_lock()`, `borrow()` or `borrow_mut()` call (after `.unwrap()`, `.expect()`, `?`, `.await`) to the end of its block or a `drop(guard)`, and reports the first `.await` or I/O call while it lives. Without type information the method name is the clue, so findings are estimates. I/O is recognised by the path segments and method names of the parameter `io`. A channel `send` is not I/O: it is in memory, and an async channel's send is an `.await`. Code inside a closure or an async block defined there is not counted (it runs later). A `read()` or `write()` under a `?` or with a turbofish (`rd.read()?`, `rd.read::<T>()`) is a decoder's, not a lock's. So is `self.read()` in an impl whose type has its own `fn read(&self)` returning a value.

A crate's lock helper takes a guard too. A function anywhere in the workspace whose body ends in a lock call and whose return type is a guard (a type named `...Guard`, or a `Result` or `Option` of one) returns the guard: `let g = m.locked();` is followed like `m.lock()`. A function that ends in `self.read()` and returns a value is not a helper. A method helper is matched by `x.locked()` or a path through a type (`Self::locked(x)`, `Locked::locked(x)`), never by a free function's call of the same name (`keep::state(x)`). One that binds the guard and ends in a call of one of its closure parameters runs that closure with the guard held: the closure's body is checked as the guard's lifetime.

## Example

```rust
let mut state = self.state.lock().unwrap();
state.pending += 1;
self.client.send(msg).await?;   // reported: the guard is still alive
```

## How to fix

Copy what is needed out and drop the guard first (`drop(state)` or a smaller scope), or let one owner task hold the state and do the I/O itself.
