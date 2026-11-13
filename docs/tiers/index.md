# Tiers

A tier is a TOML file of lints shipped with spirit_level. Each size tier extends the one before it; a project picks one with `extends` and adds its own settings on top. Tiers that stand beside the size tiers (like `embedded` and `ai`) are combined with one: `extends = ["small", "embedded"]`.

| Tier | What it is for |
|---|---|
| [small](small.md) | One crate: metadata, docs, pure-Rust dependencies, build profiles, the repo's rule files, valid waivers |
| [small-workspace](small-workspace.md) | A small workspace: shared package keys, lints and dependencies declared once at the root and inherited |
| [medium](medium.md) | A medium workspace: internal crates unpublished, MSRV declared, toolchain pinned, dependency rules strict |
| [big](big.md) | A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity |
