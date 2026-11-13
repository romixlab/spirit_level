# Output and exit codes

## Text

For people, findings are grouped by lint, denied groups first, then warnings, each by count:

```text
deny M0024 workspace-lints ×28: the root declares [workspace.lints] and every member inherits them
    --> Cargo.toml:15:1
     |
  13 | # under crates/ have their own
  14 | # (`app_base::build`).
  15 | [workspace]
     | ^^^^^^^^^^^ the root declares no [workspace.lints]
  16 | members = ["crates/*"]
  17 |
     |
  help: add `[lints]` with `workspace = true` to its Cargo.toml
  Cargo.toml  app does not inherit the workspace lints
  Cargo.toml:15  the root declares no [workspace.lints]
  crates/app_approve/Cargo.toml  app_approve does not inherit the workspace lints
  crates/app_base/Cargo.toml  app_base does not inherit the workspace lints
  crates/app_browse/Cargo.toml  app_browse does not inherit the workspace lints
  fix (--fix): add [lints] workspace = true (27 of 28)
  +23 more: `spl check --all` lists them (explain: `spl explain M0024`)
```

- **Header:** level, code, id, count and the lint's rule.
- **Excerpt:** the source of the first place that points at a line, with 2 lines of context and its exact range marked with the finding's message. `--excerpts N` shows N per group; `0` turns them off.
- **Places:** up to 5, then how many more and the command that lists them. `--all` lists every place. An exact fix is named once per group, with how many places it fixes.
- **`--group-by crate|file`** groups by crate or file instead of lint.
- **Summary** last: the result, how many lints ran, and the counts.

Colour is off when the output is not a terminal, with `NO_COLOR`, or with `--color never`. `--show-allowed` also lists findings allowed by waivers and exceptions; `--timings` shows the time of each analysis.

## JSON

`spl check --format json` prints one document:

```json
{
  "schema": "spirit_level.report/1",
  "root": "/path/to/project",
  "extends": ["big"],
  "diagnostics": [
    {
      "lint": "crate-lines",
      "code": "S0001",
      "level": "warn",
      "confidence": "certain",
      "message": "foo has 23412 code lines in src/",
      "help": "split it: ...",
      "spans": [{ "file": "crates/foo/src/lib.rs", "line": 0, "column": 0 }],
      "item": "crate:foo",
      "krate": "foo",
      "metric": { "name": "lines", "value": 23412.0, "limit": 20000.0 }
    }
  ],
  "ran": ["crate-lines", "..."],
  "not_run": [{ "lint": "deny-licenses", "code": "X0021", "level": "deny", "need": "tool:cargo-deny", "reason": "..." }],
  "timings": [{ "analysis": "size", "ms": 41 }]
}
```

JSON is complete and ungrouped: every finding, whatever `--all` and `--group-by` say. A span may carry `end_line` and `end_column` (the column after its last character); the text view reads the file for its excerpt, JSON carries no source. `rule` names an adapter's tool rule (`"clippy:needless_return"`). `allowed` (the reason) and `fix` (the edits) appear on a finding when they apply. Configuration errors print `{"schema": "spirit_level.config-errors/1", "errors": [{"file", "line", "message"}]}`.

## Confidence

`certain` findings rest only on the project's files. `estimate` findings rest on something read without type information; the message says what was assumed. `candidate` findings are places to look at and never fail a run.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | no denied finding (warnings allowed) |
| 1 | at least one denied finding |
| 2 | a configuration or usage error, or a project cargo cannot read |
| 3 | no denied finding, but a lint at `deny` could not run (a tool, the crate map or the resolved graph is missing) |
