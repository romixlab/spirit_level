## Why

Test code builds too: a crate with a huge `tests/` folder relinks its library into every test binary and slows the edit-test loop as much as a big `src/`. Counted apart from `src/` so a well-tested crate is not pushed to split its library.

## How to fix

Move integration tests into a test-only crate (a `tests` layer), or split the crate under test.
