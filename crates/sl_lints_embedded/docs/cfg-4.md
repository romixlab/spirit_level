## Why

When one dependency (WireWeaver, embassy) is reachable both as a `git` source and as a `[patch]`-redirected `path` source at the same time — because a sibling crate's own `Cargo.toml` still points at git while the workspace root patches it to the local checkout — Cargo ends up compiling and linking two copies. Types that look identical (`ww_indication::Indication`, `shrink_wrap`'s traits) become two distinct types to the compiler, and every call between the two copies is a type error.

## Example

Typec_dongle FW-51: the workspace stopped building on 24 Oct 2026 because `bedrock_crash` came in through embedded_bedrock's own `git` dependency (which itself pulled a templated `bedrock_template/Cargo.toml`), while `bedrock_led` and `bedrock_sync` brought WireWeaver a *second* time from git on top of the workspace's `[patch]` to the sibling `../wire_weaver` checkout — thirteen `E0308` type mismatches in `ww/leds.rs` and `ww/clock.rs`, all "expected `shrink_wrap::X`, found `shrink_wrap::X`" from the two copies. The fix made `bedrock_crash` a path dependency on `../embedded_bedrock`, like the other bedrock crates, and patched `fw/portwright/Cargo.toml`'s WireWeaver source to the same sibling checkout that `bedrock_led`'s own `Cargo.toml` already asked for.

## How to fix

Patch every crate in the tree to exactly one source (git xor the sibling path) in the workspace root's `[patch]`, and check every crate that depends on it transitively agrees — a sibling crate's own `Cargo.toml` dependency line can reintroduce the git copy even with a correct root patch.

## How it is checked

Check: cfg easy — `cargo tree -d` (duplicate dependencies) catches this directly; the lint flags any dependency present at two different git/path sources in the resolved tree.

## Source

Ported from embedded_bedrock's firmware check catalogue, `CFG-4` (`book/checks.md`, detail in `book/checks/cfg.md`). Found on: typec_dongle FW-51.
