## Why

Crates of other projects come in by git; Cargo.lock fixes the commit. A different URL brings a second copy, another branch than the listed one brings another line of work; a crate used from a layer that may not see its role leaks a dependency upward. Every git dependency is listed in `[map.external]`, so the set of outside projects is visible in one place.

## How to fix

Take it by git with the URL the map lists (and its branch, when the entry names one), from a crate whose layer may see the crate's `role`. A new outside crate gets an entry first.

Whether it may name a `rev`, `tag` or `branch` is the project's pin policy, checked by `git-dependency-pins`.
