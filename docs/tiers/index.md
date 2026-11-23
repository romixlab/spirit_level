# Tiers

A tier is a TOML file of lints shipped with spirit_level. Each size tier extends the one before it; a project picks one with `extends` and adds its own settings on top. Each size tier is built from [sets](sets.md), small groups of lints by what they guard, and a project may extend a set on its own. Tiers that stand beside the size tiers (like `embedded` and `ai`) are combined with one: `extends = ["small", "embedded"]`.

`spl tiers` shows the ladder and marks the tiers a project extends. `spl check --next` checks a project at the next size tier, `spl check --at TIER` at any, both with the project's own overrides and exceptions and without writing anything.

| Tier | What it is for |
|---|---|
| [small](small.md) | One crate: metadata, docs, pure-Rust dependencies, build profiles, the pinned toolchain, the repo's rule files, valid waivers |
| [small-workspace](small-workspace.md) | A small workspace: shared package keys, lints and dependencies declared once at the root and inherited |
| [medium](medium.md) | A medium workspace: internal crates unpublished, MSRV declared, dependency rules strict |
| [big](big.md) | A big workspace: a crate map with layers checked edge by edge, size limits with expiring exceptions, strict conformity |
| [ai](ai.md) | Code and docs written with AI agents: agent files, terse output, AI-isms, stop-gap smells, agent-config hygiene |
| [embedded](embedded.md) | Bare-metal firmware: executors, interrupts, DMA, stack, memory, size, faults, boot, power, flash, pins, clocks, errata, USB, host input, logging, counters, build configuration |
| [ui](ui.md) | User interfaces of any toolkit: theme colours and sizes, disabled controls with a reason, confirmed destructive actions, no blocking or raw spawns on the UI thread, wording |
| [egui](egui.md) | egui apps: the ui tier with egui's calls filled in, plus ids, repaints, the shared theme and status glyphs |
