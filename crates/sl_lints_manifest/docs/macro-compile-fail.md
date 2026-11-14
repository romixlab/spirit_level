## Why

A proc macro's job is half refusal: it must reject what it cannot expand, with an error that points at the user's code. Ordinary tests only show what compiles. A compile-fail crate holds the inputs the macro must refuse, each a `.rs` file with its expected `.stderr`, and fails when an error message changes or a bad input compiles. Bevy keeps one such crate for each of its macro crates.

## How to fix

Add a test crate beside the macro crate that depends on it (directly or through the crate that re-exports it) and on `trybuild` or `ui_test`:

```text
crates/widgets_macros/            the proc-macro crate
crates/widgets_compile_fail/      dev-dependency on trybuild
  tests/it/main.rs                trybuild::TestCases::new().compile_fail("tests/cases/*.rs")
  tests/cases/missing_field.rs
  tests/cases/missing_field.stderr
```

The lint looks for a crate that reaches the macro crate and depends on one of the harness crates (parameter `harnesses`); it does not run the cases.
