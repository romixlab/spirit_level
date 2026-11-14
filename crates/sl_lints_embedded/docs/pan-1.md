## Why

A panic in firmware resets the whole MCU: embassy has no per-task restart. A release should not ship a panic nobody looked at. The ELF says what can still panic after the optimizer has run: every call into the standard library's panic entry points (`core::panicking::*`, `unwrap_failed`, `panic_bounds_check`, the overflow and division checks, `RefCell`'s borrow failures), placed in the source by the DWARF of the call. At the strict level each of those sites is acknowledged in a file in the repo, with a reason: a review sees a new panic as a new line in that file.

`spl panics --elf target/thumbv7em-none-eabihf/release/app` lists the sites.

## Example

```toml
# panics.toml, beside the firmware's Cargo.toml
[[site]]
function = "app::power::duty"
kind = "panic"
index = 0
reason = "a zero period is a broken config table; the stage is off until the reset"

[[crate]]
name = "embassy_stm32"
reason = "the HAL's own invariants; its configuration errors fire at init, before any output"
```

A site is named by the path of the function it is written in, without generic arguments, its kind (`unwrap`, `expect`, `panic`, `assert`, `bounds`, `slice`, `overflow`, `division`, `refcell`, `alloc`) and its index among the sites of that kind in that function, in source order. The key survives edits in other functions and new lines above it; a new site of the same kind earlier in the same function shifts the indices after it, and the review sees that.

- **Inlining**: a site inlined into several callers is one site, placed where it is written, with several call addresses.
- **Generics**: each monomorphization is a copy of the same site: one key.
- **Folded sites**: where the optimizer merged several sites into one call, the DWARF has no line (line 0). The site is listed under the function that holds the call and is acknowledged like any other.
- **Dependencies**: a `[[crate]]` entry acknowledges every site of a crate outside the project at once. The project's own crates are acknowledged site by site.
- **Service modules**: a site in a module of a class in `zero-in` (`service`) is reported whatever the file says: such a module never reaches the panic handler in release.

An entry that matches no site any more is reported as stale.

## What the ELF cannot tell

The lint does not run, and says why, when the ELF:

- has no DWARF: set `debug = 2` in the release profile, debug info does not reach the flash;
- has compressed debug sections;
- has no panic handler and no `core::panicking` function: built with `panic-immediate-abort`, where every panic is a trap that no call names, or no panic path is left;
- is not Arm Thumb code.

A check the optimizer proved never fails is not in the binary: it cannot panic, and it is not listed. A call reached through a literal pool the linear Thumb decoder misreads, or a conditional tail branch into a panic entry, is not seen.

## How to fix

Remove the panic (a `Result`, `.get(i)`, `checked_*`), or add the site to the file with a reason: why it cannot fire, or why a reset is the right answer there.

## How it is checked

`elf`: the call sites of the Thumb code into the panic entry points, the DWARF inline chain of each, the acknowledgement file (`file`, relative to the firmware crate directory).
