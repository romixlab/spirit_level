# External tools

spirit_level reuses established Rust lint tools instead of reimplementing them. Each tool sits behind an adapter: an analysis like any other, whose lints carry the tool's prefix (`clippy-`, `deny-`, `machete-`, ...) and are configured, overridden and waived in `spirit_level.toml` like native ones.

- **Library or executable.** A tool with a usable library API on stable Rust is linked; one without (clippy needs `rustc_private` on nightly) runs as an executable and its machine-readable output is read.
- **Optional.** Each adapter is a cargo feature of `spirit_level_cli`, so the default build stays small.
- **Missing tools.** A lint whose tool is not installed is reported as not run with the reason; at `deny` the run exits with code 3, never passes green.

The adapters and the table of decisions are in [FEATURES.md](https://github.com/romixlab/spirit_level/blob/main/FEATURES.md) area `TOOL`; this page lists them as they land.
