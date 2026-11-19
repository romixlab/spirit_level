# Tiers

A tier is a TOML file of lints shipped with spirit_level. Each size tier extends the one before it; a project picks one with `extends` and adds its own settings on top. Tiers that stand beside the size tiers (like `embedded` and `ai`) are combined with one: `extends = ["small", "embedded"]`.

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
| [units-strict](units-strict.md) | Units everywhere: indices, counts, sequence numbers and time spans in exact_units types, besides the quantities every tier checks |
| [units-paranoid](units-paranoid.md) | Units inside function bodies: locals, literal arguments, casts, float maths, bare comparisons, raw arithmetic, scale constants and conversions by hand |
