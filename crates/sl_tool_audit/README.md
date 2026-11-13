# sl_tool_audit

The cargo-audit adapter of spirit_level (layer lints): runs `cargo audit --json` and reports vulnerabilities and the warnings of the RustSec database under `audit-vulnerabilities`, `audit-unmaintained`, `audit-unsound` and `audit-yanked`. It is run, not linked: the `rustsec` library fetches the database only with its default features, which bring C (aws-lc-sys) and 249 crates; without them it cannot update the database by itself. cargo-deny's `deny-advisories` covers the same database; enable one of the two.

It may depend on `sl_core`, `sl_model`, `sl_analysis` and `sl_tools`.
