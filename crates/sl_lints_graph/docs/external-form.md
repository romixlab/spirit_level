## Why

Crates of other projects come in by git, following a branch; Cargo.lock fixes the commit and a nightly update finds breaks early. A `rev` or `tag` pin hides a break; a different URL brings a second copy; a crate used from a layer that may not see its role leaks a dependency upward. Every git dependency is listed in `[map.external]`, so the set of outside projects is visible in one place.

## How to fix

Take it by git with the URL and branch the map lists, no `rev` or `tag`, from a crate whose layer may see the crate's `role`. A new outside crate gets an entry first.
