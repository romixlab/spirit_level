# sl_lints_code

Source lints read with `syn`, without type information:

- `todo-macros`: `todo!()` and `unimplemented!()` (the no-stop-gaps rule; comment markers are the prose lints' job).
- `panic-in-library`: `unwrap()`, `expect()`, `panic!`, `unreachable!` in non-test library code.
- `print-in-library`: `println!`, `eprintln!`, `print!`, `eprint!`, `dbg!` in library code.
- `allow-without-reason`: `#[allow(..)]` and `#[expect(..)]` without `reason = "..."`.
- `float-determinism`: float transcendentals in crates the map marks `deterministic`.
- `random-state-hash`: `HashMap` / `HashSet` with the default random hasher in such crates.
- `cli-subprocess`: `Command::new` of a binary of the same workspace.

Depends on `sl_analysis`, `sl_core`, `sl_model` and `syn`.
