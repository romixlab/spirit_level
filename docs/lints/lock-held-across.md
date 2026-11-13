# lock-held-across

**Rule:** No lock guard is held across an .await or an I/O call.

| | |
|---|---|
| Category | [state](index.md#state) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer |
| [small-workspace](../tiers/small-workspace.md) | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer |
| [medium](../tiers/medium.md) | `warn` | a guard across .await or I/O stalls every other caller for as long as the slowest peer |
| [big](../tiers/big.md) | `deny` | a guard across .await or I/O stalls every other caller for as long as the slowest peer |
| [ai](../tiers/ai.md) | off |  |
| [embedded](../tiers/embedded.md) | off |  |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `io` | list of strings | `["File", "fs", "Command", "TcpStream", "UdpSocket", "UnixStream", "write_all", "read_to_end", "read_to_string", "flush", "sync_all", "output", "spawn", "send", "recv", "block_on", "sleep"]` | path segments and method names counted as I/O |

## Why

A guard held across an `.await` blocks every other task that wants the lock for as long as the awaited future takes, and with a std mutex it can deadlock the executor thread. A guard held across I/O (a file write, a socket, a subprocess, an HTTP call) turns a fast critical section into one that lasts as long as the slowest disk or peer. tpm's mail server held a lock across IMAP and database work for minutes.

The lint follows a guard bound with `let` from a zero-argument `lock()`, `read()`, `write()`, `try_lock()`, `borrow()` or `borrow_mut()` call (after `.unwrap()`, `.expect()`, `?`, `.await`) to the end of its block or a `drop(guard)`, and reports the first `.await` or I/O call while it lives. Without type information the method name is the clue, so findings are estimates. I/O is recognised by the path segments and method names of the parameter `io`. Code inside a closure or an async block defined there is not counted (it runs later).

## Example

```rust
let mut state = self.state.lock().unwrap();
state.pending += 1;
self.client.send(msg).await?;   // reported: the guard is still alive
```

## How to fix

Copy what is needed out and drop the guard first (`drop(state)` or a smaller scope), or let one owner task hold the state and do the I/O itself.

## Configure

```toml
[lints.lock-held-across]
level = "deny"
io = ["File", "fs", "Command", "TcpStream", "UdpSocket", "UnixStream", "write_all", "read_to_end", "read_to_string", "flush", "sync_all", "output", "spawn", "send", "recv", "block_on", "sleep"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(lock-held-across) <reason>` ([waivers](../waivers.md)).
