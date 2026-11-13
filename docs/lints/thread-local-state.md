# thread-local-state

**Rule:** Thread_local! state has one owner, or a reason.

| | |
|---|---|
| Category | [state](index.md#state) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | off |  |
| [small-workspace](../tiers/small-workspace.md) | `warn` | thread-local state is a hidden side channel between functions |
| [medium](../tiers/medium.md) | `warn` | thread-local state is a hidden side channel between functions |
| [big](../tiers/big.md) | `warn` | thread-local state is a hidden side channel between functions |

## Parameters

| Name | Type | Default | Meaning |
|---|---|---|---|
| `types` | list of strings | `["Cell", "RefCell", "Mutex", "RwLock", "UnsafeCell", "OnceCell"]` | type names that make a thread-local mutable |

## Why

`thread_local!` with a `Cell` or `RefCell` is global state in disguise: it works on one thread and silently differs on another, and it hides a side channel between functions that do not pass it. tpm's records store kept drafts in a thread-local side channel that one command filled and another read.

## How to fix

Pass the state down the call tree (a context argument). A per-thread buffer that only its own thread writes and a rare reader dumps is a fine use: say so in a waiver.

## Configure

```toml
[lints.thread-local-state]
level = "deny"
types = ["Cell", "RefCell", "Mutex", "RwLock", "UnsafeCell", "OnceCell"]
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(thread-local-state) <reason>` ([waivers](../waivers.md)).
