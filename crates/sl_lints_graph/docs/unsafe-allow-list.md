## Why

`unsafe` code is where memory bugs live. The workspace forbids it (`unsafe_code = "forbid"` in `[workspace.lints.rust]`), and only the crates the map marks `unsafe = true` override that, each a small crate with a safe API around its unsafe core. The lint holds both sides: the forbid is set, every crate with unsafe code is on the list, and every crate on the list still has unsafe code, so the list never grows stale.

Unsafe code is what rustc's `unsafe_code` lint fires on: an `unsafe` block, fn, impl or trait, the attributes `no_mangle`, `export_name` and `link_section` (bare or inside `unsafe(...)`), a linkme `distributed_slice(NAME)` element, and an `allow(unsafe_code)` or `expect(unsafe_code)`.

## Commands that register themselves

A linkme element does not compile under `forbid(unsafe_code)`: `#[distributed_slice(COMMANDS)]` expands to a `link_section` attribute in the crate that writes it, and rustc refuses it ("usage of the unsafe `link_section` attribute", rustc 1.99, linkme 0.3.38). Declaring the slice compiles; adding to it does not. `inventory::submit!` compiles under the forbid, because its unsafe parts expand from inside the `inventory` crate.

Pick one:

- **inventory**, when the registering crates should stay under the forbid. Each crate submits its entries; nothing in the map changes.
- **linkme in crates marked unsafe**, when registration should not run code before `main`. Mark each registering crate `unsafe = true` in the map, give it its own `[lints.rust]` with `unsafe_code = "deny"` (a crate cannot loosen an inherited forbid), and put `#[expect(unsafe_code, reason = "linkme places the entry with link_section")]` on each element. Keep those crates small: the registry and the command modules, not the logic behind them.

```toml
# spirit_level.toml
[map.crates]
app_commands = { layer = "app", unsafe = true }
```

```toml
# crates/app_commands/Cargo.toml
[lints.rust]
unsafe_code = "deny"
```

## How to fix

Remove the unsafe code, or move it into a crate the map marks unsafe; drop `unsafe = true` from a crate that no longer needs it.
