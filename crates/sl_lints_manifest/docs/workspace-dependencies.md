## Why

Every dependency's version and features are written once, in the root's `[workspace.dependencies]`; members say `foo.workspace = true`. Then a workspace has one version of each crate, and an upgrade is one line (zed's `package-conformity` checks the same).

## How to fix

Move the version and features to `[workspace.dependencies]` (with `default-features = false`), and write `foo.workspace = true` in the member, adding member-specific features there if needed. Path dependencies on other members count too unless the parameter `members` is false.
