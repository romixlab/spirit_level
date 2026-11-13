## Why

cargo-udeps builds the workspace on nightly and asks the compiler which dependencies it loaded; one never loaded is unused for certain in that build. It is slower than cargo-machete or cargo-shear but has no false positives from names in comments or macros (it does miss dependencies used only in doc tests).

## How to fix

Remove the dependency. One used only in doc tests is listed in the crate's manifest:

```toml
[package.metadata.cargo-udeps.ignore]
development = ["criterion"]
```
