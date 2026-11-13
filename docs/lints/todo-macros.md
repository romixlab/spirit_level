# todo-macros

**Rule:** No todo!() or unimplemented!() outside tests.

| | |
|---|---|
| Category | [code](index.md#code) |
| Source | spirit_level |
| Fix | by hand |

## Level per tier

| Tier | Level | Reason |
|---|---|---|
| [small](../tiers/small.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code |
| [small-workspace](../tiers/small-workspace.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code |
| [medium](../tiers/medium.md) | `warn` | no stop-gaps: a todo!() is a panic waiting in shipped code |
| [big](../tiers/big.md) | `deny` | no stop-gaps: a todo!() is a panic waiting in shipped code |
| [ai](../tiers/ai.md) | `deny` | todo!() and unimplemented!() are stop-gaps that compile |

## Why

`todo!()` and `unimplemented!()` compile and then panic at run time: they are a promise that the design will be finished later. Under the no-stop-gaps rule a deferred design is a defect; planned work is an entry in FEATURES.md, not a panic waiting in shipped code. Test code is not checked. (Comment markers such as `TODO` and `FIXME` are checked by the prose lints.)

## How to fix

Write the real implementation, or return an error that names what is not supported.

## Configure

```toml
[lints.todo-macros]
level = "deny"
reason = "why this project wants it at this level"
```

Allow one finding in the code with `// spirit-level: allow(todo-macros) <reason>` ([waivers](../waivers.md)).
