## Why

Manifests and configuration files are edited by people and by tools (`cargo add`, `cargo hakari`); one formatting keeps their diffs about the change. taplo formats TOML by the project's `taplo.toml` (or its defaults) and reports files that do not parse.

```toml
# taplo.toml
[formatting]
align_entries = false
column_width = 120
```

## How to fix

`spirit-level check --fix` writes each file as `taplo fmt` formats it. A file that does not parse is fixed by hand at the place reported.
