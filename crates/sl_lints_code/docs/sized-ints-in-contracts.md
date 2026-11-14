## Why

A contract crate holds the serde types that cross a process or node boundary: every party links it and reads the same bytes. `usize` and `isize` are 32 bits on one party and 64 on another (firmware and host, wasm and server), so a value that fits on one side overflows or fails to decode on the other.

The lint checks the fields of every struct and enum that derives `Serialize` or `Deserialize` in crates the [crate map](../map.md) marks `contract = true`, and in those listed in `also`.

```rust
#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub len: usize,   // reported
    pub seq: u32,     // passes
}
```

## How to fix

Use a sized integer that holds every value the contract allows: `u32`, `u64`, `i64`. Convert to `usize` where the value indexes memory.
