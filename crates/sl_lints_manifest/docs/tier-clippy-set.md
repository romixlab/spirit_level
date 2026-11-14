## Why

A tier is more than spirit_level's own lints: each one names the Clippy lints a project of its size is expected to run, in a `[clippy]` table of the tier file. Clippy reads them from the project's own `[workspace.lints.clippy]` (or `[lints.clippy]`) and `clippy.toml`, so `cargo clippy`, the IDE and CI all see the same set; spirit_level checks those files and never passes Clippy flags of its own.

An opt-out is fine when it says why. A bare `cast_lossless = "allow"` is a decision nobody can review; with `# 457 hits, widening casts are never lossy` it is one.

## What it checks

- Every lint or group the tiers set to `warn` or `deny` is on in the table at that level or higher. Groups are matched by name (`pedantic`); a lint is not counted as on through a group.
- Every `allow` in the table has a reason: a comment after it on its line, or a comment line right above it.
- Every `clippy.toml` setting the tiers name is there. An integer is a limit: the project's value may be lower. Anything else must be equal.
- A workspace member with its own `[lints]` table (not `workspace = true`) is checked on its own; a member without one is [workspace-lints](M0024.md)'s finding.

## How to fix

The help of a finding is the TOML to paste, each line with the tier's reason as its comment:

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 } # the stricter idioms; the opt-outs below are noise for most code
unwrap_used = "warn" # a panic says what failed (expect) or the error is returned
cast_lossless = "allow" # widening `as` casts cannot lose a value
```

To expect a different set, override entries in `spirit_level.toml` with a reason, as for any lint:

```toml
[clippy.lints]
print_stdout = { level = "allow", reason = "a CLI-only workspace: printing is its job" }

[clippy.config]
cognitive-complexity-threshold = { value = 20, reason = "this team splits early" }
```
