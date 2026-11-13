## Why

Every crate of a project is on the same, current edition, so code moves between crates without surprises (closure captures, `impl Trait` capture rules, `unsafe extern`, let chains).

## How to fix

Run `cargo fix --edition`, then set `edition` (or inherit it from `[workspace.package]`). The parameter `edition` names the edition expected.
