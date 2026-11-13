# sl_lints_dead

The dead-code lint of spirit_level that rustc can't report itself, `dead-pub`: public items of a workspace library that no other crate of the workspace names (by name, a candidate list). `allow(dead_code)` without a reason is `allow-without-reason` in `sl_lints_code`.

Its documentation is in `docs/dead-pub.md`, its fixtures in `tests/fixtures/dead-pub/`. It depends on `sl_analysis` and `sl_core`, with syn and proc-macro2 for the syntax.
