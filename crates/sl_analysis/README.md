# sl_analysis

The plug-in point of spirit_level. Every analysis, native or an adapter of an external tool, is a type implementing `Analysis`: it names the lints it reports (their `LintMeta`) and, given a `Context` (the project, the resolved configuration, today's date), emits diagnostics. The engine decides which analyses run, applies levels, exceptions and waivers, and reports.

A new analysis is a small crate that depends on this one, `sl_core` and `sl_model`, exports a `static` implementing `Analysis`, and is listed once in the `spirit_level` facade. `syntax` holds the helpers the source analyses share (test-code detection, line numbers of syn spans, item paths).
