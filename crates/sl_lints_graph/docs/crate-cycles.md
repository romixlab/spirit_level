## Why

Cargo refuses a cycle of normal dependencies, but allows one through dev-dependencies: `a` depends on `b`, and `b`'s tests depend on `a`. Then neither can be built, tested or moved apart, and `-p` test runs rebuild both. The lint reports every strongly connected component of the member graph, dev edges included.

## How to fix

Move the shared test helpers into a test-only crate both use as a dev-dependency.
