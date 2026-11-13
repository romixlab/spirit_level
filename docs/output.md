# Output and exit codes

## Text

For people: one block per finding with its level, lint id, place, message, measurement, help and fix; then the lints that did not run and why; then a summary line. Colour is off when the output is not a terminal or `NO_COLOR` is set. `--show-allowed` also lists findings allowed by waivers and exceptions; `--timings` shows the time of each analysis.

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
  "not_run": [{ "lint": "deny-licenses", "level": "deny", "need": "tool:cargo-deny", "reason": "..." }],
  "timings": [{ "analysis": "size", "ms": 41 }]
}
```

`allowed` (the reason) and `fix` (the edits) appear on a finding when they apply. Configuration errors print `{"schema": "spirit_level.config-errors/1", "errors": [{"file", "line", "message"}]}`.

## Confidence

`certain` findings rest only on the project's files. `estimate` findings rest on something read without type information; the message says what was assumed. `candidate` findings are places to look at and never fail a run.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | no denied finding (warnings allowed) |
| 1 | at least one denied finding |
| 2 | a configuration or usage error, or a project cargo cannot read |
| 3 | no denied finding, but a lint at `deny` could not run (a tool, the crate map or the resolved graph is missing) |
